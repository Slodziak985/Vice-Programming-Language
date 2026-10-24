use crate::ast::*;
use crate::error::{ViceError, ViceErrorType};
use crate::token::{Token, TokenKind};
use crate::types::Type;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    in_map_literal: u32,
    allow_struct_literal: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            in_map_literal: 0,
            allow_struct_literal: true,
        }
    }

    pub fn parse(&mut self) -> Result<Program, ViceError> {
        let mut imports = Vec::new();
        let mut functions = Vec::new();
        let mut data_declarations = Vec::new();
        let mut impls = Vec::new();
        let mut top_level = Vec::new();

        while !self.is_at_end() {
            if self.match_kind(TokenKind::Use) {
                let use_token = self.previous().clone();
                let path_token = self.advance();
                if path_token.kind != TokenKind::String {
                    return Err(self.syntax_error("Expected string path after 'use'", &path_token));
                }
                self.consume(TokenKind::Semicolon, "Expected ';' after use import")?;
                let import = Import {
                    path: path_token.lexeme.clone(),
                    span: Self::token_span(&use_token),
                };
                imports.push(import.clone());
                top_level.push(TopLevel::Import(import));
                continue;
            }

            if self.check(TokenKind::Attribute) {
                if let Some(next) = self.tokens.get(self.current) {
                    if next.lexeme == "mount" {
                        let mount_token = self.advance(); // consume @mount
                        let path_token = self.advance();
                        if path_token.kind != TokenKind::String {
                            return Err(self
                                .syntax_error("Expected string path after '@mount'", &path_token));
                        }
                        // Optional semicolon for backwards compatibility or clean look
                        self.match_kind(TokenKind::Semicolon);

                        let import = Import {
                            path: path_token.lexeme.clone(),
                            span: Self::token_span(&mount_token),
                        };
                        imports.push(import.clone());
                        top_level.push(TopLevel::Import(import));
                        continue;
                    }
                }
            }

            let attributes = self.attributes()?;
            if self.check(TokenKind::Data) {
                let data = self.data_declaration(attributes)?;
                data_declarations.push(data.clone());
                top_level.push(TopLevel::Data(data));
            } else if self.check(TokenKind::Impl) {
                let impl_block = self.impl_declaration()?;
                impls.push(impl_block.clone());
                top_level.push(TopLevel::Impl(impl_block));
            } else if self.check(TokenKind::Enum) {
                let enum_def = self.enum_declaration(attributes)?;
                data_declarations.push(enum_def.clone()); // We can keep enums in data_declarations or create a separate list
                top_level.push(TopLevel::EnumDef(enum_def));
            } else if self.check(TokenKind::Trait) {
                let trait_def = self.trait_declaration(attributes)?;
                data_declarations.push(trait_def.clone());
                top_level.push(TopLevel::TraitDef(trait_def));
            } else {
                let function = self.function(attributes)?;
                functions.push(function.clone());
                top_level.push(TopLevel::Function(function));
            }
        }

        Ok(Program {
            imports,
            functions,
            data_declarations,
            impls,
            top_level,
            sources: Vec::new(),
        })
    }

    fn attributes(&mut self) -> Result<Vec<Attribute>, ViceError> {
        let mut attributes = Vec::new();
        while self.check(TokenKind::Attribute) {
            let token = self.advance();
            let span = Self::token_span(&token);
            let name = token.lexeme.clone();
            let mut args = Vec::new();

            if self.match_kind(TokenKind::LParen) {
                args = self.call_args()?;
                self.consume(TokenKind::RParen, "Expected ')' after attribute arguments")?;
            }

            attributes.push(Attribute { name, args, span });
            self.match_kind(TokenKind::TypeArrow);
        }
        Ok(attributes)
    }

    fn function(&mut self, attributes: Vec<Attribute>) -> Result<Function, ViceError> {
        let span = Self::token_span(self.peek());
        self.consume(TokenKind::Func, "Expected 'fn'")?;
        let name = self.consume_ident("Expected function name")?;
        let type_params = self.parse_type_params()?;
        self.consume(TokenKind::LParen, "Expected '('")?;

        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                let param_span = Self::token_span(self.peek());
                let mutable = self.match_kind(TokenKind::Mut);
                let name = self.consume_ident("Expected parameter name")?;
                let type_annotation = if self.match_kind(TokenKind::Colon) {
                    Some(self.type_annotation()?)
                } else {
                    None
                };
                params.push(Param {
                    name,
                    mutable,
                    type_annotation,
                    span: param_span,
                });
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RParen, "Expected ')'")?;

        let return_type = if self.match_kind(TokenKind::TypeArrow) {
            Some(self.type_annotation()?)
        } else {
            None
        };

        let body = self.block()?;

        Ok(Function {
            name,
            type_params,
            attributes,
            params,
            return_type,
            body,
            span,
        })
    }

    fn enum_declaration(&mut self, attributes: Vec<Attribute>) -> Result<Stmt, ViceError> {
        let span = Self::token_span(self.peek());
        self.consume(TokenKind::Enum, "Expected 'enum'")?;
        let name = self.consume_ident("Expected enum name")?;
        self.consume(TokenKind::LBrace, "Expected '{' before enum variants")?;

        let mut variants = Vec::new();
        if !self.check(TokenKind::RBrace) {
            loop {
                variants.push(self.consume_ident("Expected variant name")?);
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RBrace, "Expected '}' after enum variants")?;

        Ok(Stmt::at(
            StmtKind::EnumDef {
                name,
                attributes,
                variants,
            },
            span,
        ))
    }

    fn impl_declaration(&mut self) -> Result<ImplBlock, ViceError> {
        let span = Self::token_span(self.peek());
        self.consume(TokenKind::Impl, "Expected 'impl'")?;
        let type_params = self.parse_type_params()?;

        let mut trait_name = None;
        let mut target = self.type_annotation()?;

        if self.match_kind(TokenKind::For) {
            trait_name = Some(target);
            target = self.type_annotation()?;
        }

        self.consume(TokenKind::LBrace, "Expected '{' after impl target")?;

        let mut methods = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let attributes = self.attributes()?;
            let function = self.function(attributes)?;
            methods.push(function);
        }

        self.consume(TokenKind::RBrace, "Expected '}' after impl block methods")?;

        Ok(ImplBlock {
            target,
            trait_name,
            type_params,
            methods,
            span,
        })
    }

    fn trait_declaration(&mut self, attributes: Vec<Attribute>) -> Result<Stmt, ViceError> {
        let span = Self::token_span(self.peek());
        self.consume(TokenKind::Trait, "Expected 'trait'")?;
        let name = self.consume_ident("Expected trait name")?;
        let type_params = self.parse_type_params()?;
        self.consume(TokenKind::LBrace, "Expected '{' after trait name")?;

        let mut methods = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let method_span = Self::token_span(self.peek());
            self.consume(TokenKind::Func, "Expected 'fn' in trait method")?;
            let method_name = self.consume_ident("Expected method name")?;
            self.consume(TokenKind::LParen, "Expected '(' after method name")?;
            let mut params = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    let param_span = Self::token_span(self.peek());
                    let mutable = self.match_kind(TokenKind::Mut);
                    let name = self.consume_ident("Expected parameter name")?;
                    let type_annotation = if self.match_kind(TokenKind::Colon) {
                        Some(self.type_annotation()?)
                    } else {
                        None
                    };
                    params.push(Param {
                        name,
                        mutable,
                        type_annotation,
                        span: param_span,
                    });
                    if !self.match_kind(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume(TokenKind::RParen, "Expected ')'")?;
            let return_type = if self.match_kind(TokenKind::TypeArrow) {
                Some(self.type_annotation()?)
            } else {
                None
            };
            self.consume(
                TokenKind::Semicolon,
                "Expected ';' after trait method declaration",
            )?;

            methods.push(TraitMethodDef {
                name: method_name,
                params,
                return_type,
                span: method_span,
            });
        }

        self.consume(TokenKind::RBrace, "Expected '}' after trait methods")?;

        Ok(Stmt::at(
            StmtKind::TraitDef {
                name,
                type_params,
                attributes,
                methods,
            },
            span,
        ))
    }

    fn parse_type_params(&mut self) -> Result<Vec<String>, ViceError> {
        let mut type_params = Vec::new();
        if self.match_kind(TokenKind::Less) {
            if !self.check(TokenKind::Greater) {
                loop {
                    type_params.push(self.consume_ident("Expected type parameter name")?);
                    if !self.match_kind(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume(TokenKind::Greater, "Expected '>' after type parameters")?;
        }
        Ok(type_params)
    }

    fn type_annotation(&mut self) -> Result<Type, ViceError> {
        if self.match_kind(TokenKind::Ampersand) {
            let mutable = self.match_kind(TokenKind::Mut);
            let inner = self.type_annotation()?;
            return Ok(Type::Reference {
                mutable,
                inner: Box::new(inner),
            });
        }
        let name = self.consume_ident("Expected type name")?;
        let mut ty = if name == "map" && self.match_kind(TokenKind::LBracket) {
            let key_type = self.type_annotation()?;
            self.consume(
                TokenKind::Comma,
                "Expected ',' between map key and value types",
            )?;
            let value_type = self.type_annotation()?;
            self.consume(TokenKind::RBracket, "Expected ']' after map types")?;
            Type::Map(Box::new(key_type), Box::new(value_type))
        } else {
            let mut base_type = Type::from_name(&name);
            if self.match_kind(TokenKind::Less) {
                let mut type_args = Vec::new();
                if !self.check(TokenKind::Greater) {
                    loop {
                        type_args.push(self.type_annotation()?);
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::Greater, "Expected '>' after type arguments")?;
                if let Type::Data(n, _) = base_type {
                    base_type = Type::Data(n, type_args);
                } else {
                    return Err(self.syntax_error(
                        &format!("Type '{}' does not take type arguments", name),
                        self.previous(),
                    ));
                }
            }
            base_type
        };
        while self.match_kind(TokenKind::LBracket) {
            self.consume(TokenKind::RBracket, "Expected ']' in array type")?;
            ty = Type::Array(Box::new(ty));
        }
        Ok(ty)
    }

    fn statement(&mut self) -> Result<Stmt, ViceError> {
        let span = Self::token_span(self.peek());
        if self.check(TokenKind::LBrace) {
            return Ok(Stmt::at(StmtKind::Block(self.block()?), span));
        }
        if self.check(TokenKind::While) {
            return self.parse_while_statement();
        }
        if self.check(TokenKind::For) {
            return self.parse_for_statement();
        }
        if self.match_kind(TokenKind::If) {
            return self.if_statement(span);
        }
        if self.match_kind(TokenKind::Break) {
            self.consume(TokenKind::Semicolon, "Expected ';' after break")?;
            return Ok(Stmt::at(StmtKind::Break, span));
        }
        if self.match_kind(TokenKind::Continue) {
            self.consume(TokenKind::Semicolon, "Expected ';' after continue")?;
            return Ok(Stmt::at(StmtKind::Continue, span));
        }
        if self.check(TokenKind::Match) {
            return self.match_statement();
        }
        if self.match_kind(TokenKind::Let) {
            let is_mut = self.match_kind(TokenKind::Mut);
            return self.variable(is_mut, span);
        }
        if self.match_kind(TokenKind::Mut) {
            return self.variable(true, span);
        }
        if self.match_kind(TokenKind::Return) {
            let value = if !self.check(TokenKind::Semicolon) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume(TokenKind::Semicolon, "Expected ';' after return")?;
            return Ok(Stmt::at(StmtKind::Return(value), span));
        }
        if self.check(TokenKind::Data) {
            return self.data_declaration(Vec::new());
        }
        if self.check(TokenKind::Trait) {
            return self.trait_declaration(Vec::new());
        }
        if self.check(TokenKind::Identifier) && self.peek().lexeme == "error" {
            return self.error_statement();
        }
        if self.match_kind(TokenKind::Fail) {
            return self.fail_statement(span);
        }
        if self.match_kind(TokenKind::Input) {
            return self.input_statement(span);
        }
        if self.match_kind(TokenKind::Output) {
            return self.output(span);
        }
        if self.match_kind(TokenKind::Emit) {
            return self.emit(span);
        }

        let expr = self.expression()?;
        if self.check(TokenKind::Equal)
            || self.check(TokenKind::PlusEqual)
            || self.check(TokenKind::MinusEqual)
        {
            return self.finish_assignment(expr);
        }

        self.consume(TokenKind::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::at(StmtKind::Expression(expr), span))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ViceError> {
        self.consume(TokenKind::LBrace, "Expected '{'")?;
        let mut body = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            body.push(self.statement()?);
        }
        self.consume(TokenKind::RBrace, "Expected '}'")?;
        Ok(body)
    }

    fn match_statement(&mut self) -> Result<Stmt, ViceError> {
        let span = Self::token_span(&self.advance()); // Konsumuje 'match'
        let (condition, arms) = self.parse_match_body()?;
        Ok(Stmt::at(StmtKind::Match { condition, arms }, span))
    }

    fn parse_match_body(&mut self) -> Result<(Expr, Vec<MatchArm>), ViceError> {
        let condition = self.expression()?;
        self.consume(TokenKind::LBrace, "Expected '{' after match condition")?;

        let mut arms = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let mut patterns = Vec::new();
            loop {
                let expr = self.expression()?;
                if self.match_kind(TokenKind::Range) {
                    let end = self.expression()?;
                    let span = expr.span;
                    patterns.push(Expr::at(
                        ExprKind::Range {
                            start: Box::new(expr),
                            end: Box::new(end),
                        },
                        span,
                    ));
                } else {
                    patterns.push(expr);
                }
                if !self.match_kind(TokenKind::Pipe) {
                    break;
                }
            }
            self.consume(TokenKind::AssignArrow, "Expected '=>' after match pattern")?;

            // Ciało ramienia może być blokiem {} lub pojedynczym wyrażeniem
            let body = if self.check(TokenKind::LBrace) {
                let body_span = Self::token_span(self.peek());
                Stmt::at(StmtKind::Block(self.block()?), body_span)
            } else {
                let expr = self.expression()?;
                Stmt::at(
                    StmtKind::Expression(expr),
                    Self::token_span(self.previous()),
                )
            };

            arms.push(MatchArm { patterns, body });
            // Opcjonalny przecinek między ramionami
            self.match_kind(TokenKind::Comma);
        }
        self.consume(TokenKind::RBrace, "Expected '}' after match arms")?;
        Ok((condition, arms))
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, ViceError> {
        let span = Self::token_span(self.peek());
        self.consume(TokenKind::While, "Expected 'while'")?;
        let condition = if self.match_kind(TokenKind::LParen) {
            let expr = self.expression()?;
            self.consume(TokenKind::RParen, "Expected ')' after condition")?;
            expr
        } else {
            let prev = self.allow_struct_literal;
            self.allow_struct_literal = false;
            let expr = self.expression();
            self.allow_struct_literal = prev;
            expr?
        };
        let body = self.block()?;
        Ok(Stmt::at(StmtKind::While { condition, body }, span))
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, ViceError> {
        let span = Self::token_span(self.peek());
        self.consume(TokenKind::For, "Expected 'for'")?;
        let iterator = self.consume_ident("Expected iterator name")?;
        self.consume(TokenKind::In, "Expected 'in' after iterator")?;
        let expr = self.expression()?;

        if self.match_kind(TokenKind::Range) {
            let end = self.expression()?;
            let body = self.block()?;
            Ok(Stmt::at(
                StmtKind::ForLoop {
                    iterator,
                    start: expr,
                    end,
                    body,
                },
                span,
            ))
        } else {
            let body = self.block()?;
            Ok(Stmt::at(
                StmtKind::ForEach {
                    iterator,
                    collection: expr,
                    body,
                },
                span,
            ))
        }
    }

    fn input_statement(&mut self, span: Span) -> Result<Stmt, ViceError> {
        self.consume(TokenKind::TypeArrow, "Expected '->' after input")?;
        let name = self.consume_ident("Expected variable name for input")?;
        self.consume(TokenKind::Semicolon, "Expected ';'")?;
        Ok(Stmt::at(StmtKind::Input(name), span))
    }

    fn finish_assignment(&mut self, expr: Expr) -> Result<Stmt, ViceError> {
        let op_token = self.advance();
        let op_span = Self::token_span(&op_token);
        let op_kind = op_token.kind.clone();
        let mut value = self.expression()?;

        if op_kind == TokenKind::PlusEqual {
            value = Expr::at(
                ExprKind::Binary {
                    left: Box::new(expr.clone()),
                    operator: BinaryOperator::Add,
                    right: Box::new(value),
                },
                op_span,
            );
        } else if op_kind == TokenKind::MinusEqual {
            value = Expr::at(
                ExprKind::Binary {
                    left: Box::new(expr.clone()),
                    operator: BinaryOperator::Subtract,
                    right: Box::new(value),
                },
                op_span,
            );
        }

        self.consume(TokenKind::Semicolon, "Expected ';'")?;

        match expr.kind {
            ExprKind::Identifier(name) => Ok(Stmt::at(StmtKind::Assign { name, value }, op_span)),
            ExprKind::Get { entity, name } => Ok(Stmt::at(
                StmtKind::MemberAssign {
                    entity: *entity,
                    name,
                    value,
                },
                op_span,
            )),
            ExprKind::Index { entity, index } => Ok(Stmt::at(
                StmtKind::IndexAssign {
                    entity: *entity,
                    index: *index,
                    value,
                },
                op_span,
            )),
            ExprKind::Unary {
                operator: UnaryOperator::Dereference,
                right,
            } => Ok(Stmt::at(
                StmtKind::DerefAssign {
                    pointer: *right,
                    value,
                },
                op_span,
            )),
            _ => Err(self.syntax_error("Invalid assignment target", &op_token)),
        }
    }

    fn if_statement(&mut self, span: Span) -> Result<Stmt, ViceError> {
        let condition = if self.match_kind(TokenKind::LParen) {
            let expr = self.expression()?;
            self.consume(TokenKind::RParen, "Expected ')' after if condition")?;
            expr
        } else {
            let prev = self.allow_struct_literal;
            self.allow_struct_literal = false;
            let expr = self.expression();
            self.allow_struct_literal = prev;
            expr?
        };
        let body = self.block()?;
        let else_branch = if self.match_kind(TokenKind::Else) {
            if self.match_kind(TokenKind::If) {
                // Check for 'else if'
                Some(vec![self.if_statement(span)?]) // Parse another if statement as a single statement block
            } else {
                Some(self.block()?) // Regular 'else' block
            }
        } else {
            None
        };
        Ok(Stmt::at(
            StmtKind::If {
                condition,
                then_branch: body,
                else_branch,
            },
            span,
        ))
    }

    fn variable(&mut self, mutable: bool, span: Span) -> Result<Stmt, ViceError> {
        let name = self.consume_ident("Expected variable name")?;
        let type_annotation =
            if self.match_kind(TokenKind::TypeArrow) || self.match_kind(TokenKind::Colon) {
                Some(self.type_annotation()?)
            } else {
                None
            };
        if !self.match_kind(TokenKind::AssignArrow) && !self.match_kind(TokenKind::Equal) {
            return Err(self.syntax_error("Expected '=>' or '='", self.peek()));
        }
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';'")?;
        Ok(Stmt::at(
            StmtKind::Variable {
                name,
                mutable,
                type_annotation,
                value,
            },
            span,
        ))
    }

    fn default_value_for_type(ty: &Type, span: Span) -> Expr {
        match ty {
            Type::Int => Expr::at(ExprKind::Number("0".to_string()), span),
            Type::Float => Expr::at(ExprKind::Number("0.0".to_string()), span),
            Type::Bool => Expr::at(ExprKind::Bool(false), span),
            Type::String => Expr::at(ExprKind::String(String::new()), span),
            Type::Array(_) => Expr::at(ExprKind::Array(Vec::new()), span),
            Type::Map(_, _) => Expr::at(ExprKind::Map(Vec::new()), span),
            Type::Data(name, _) => Expr::at(
                ExprKind::Call {
                    name: name.clone(),
                    type_args: Vec::new(),
                    args: Vec::new(),
                },
                span,
            ),
            _ => Expr::at(ExprKind::Bool(false), span),
        }
    }

    fn data_declaration(&mut self, attributes: Vec<Attribute>) -> Result<Stmt, ViceError> {
        let span = Self::token_span(self.peek());
        self.consume(TokenKind::Data, "Expected 'data'")?;
        let name = self.consume_ident("Expected data name")?;
        let type_params = self.parse_type_params()?;
        self.consume(TokenKind::LBrace, "Expected '{' after data name")?;

        let mut fields = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            let field_span = Self::token_span(self.peek());
            // Pozwalamy na opcjonalne słowo 'mut' przed nazwą pola dla czytelności
            self.match_kind(TokenKind::Mut);
            let field_name = self.consume_ident("Expected field name")?;
            self.consume(TokenKind::Colon, "Expected ':' after field name")?;
            let type_annotation = self.type_annotation()?;
            let default_value = if self.match_kind(TokenKind::AssignArrow) || self.match_kind(TokenKind::Equal) {
                let default_expr = self.expression()?;
                self.match_kind(TokenKind::Semicolon);
                self.match_kind(TokenKind::Comma);
                default_expr
            } else {
                self.match_kind(TokenKind::Semicolon);
                self.match_kind(TokenKind::Comma);
                Self::default_value_for_type(&type_annotation, field_span)
            };
            fields.push(DataField {
                name: field_name,
                type_annotation,
                default_value,
                span: field_span,
            });
        }

        self.consume(TokenKind::RBrace, "Expected '}' after data fields")?;
        Ok(Stmt::at(
            StmtKind::DataDeclaration {
                name,
                type_params,
                attributes,
                fields,
            },
            span,
        ))
    }

    fn error_statement(&mut self) -> Result<Stmt, ViceError> {
        let span = Self::token_span(self.peek());
        self.consume_ident("Expected 'error'")?;
        self.consume(TokenKind::LParen, "Expected '(' after error")?;
        let args = self.call_args()?;
        self.consume(TokenKind::RParen, "Expected ')' after error arguments")?;
        self.consume(TokenKind::OutputArrow, "Expected '<-' after error(...)")?;
        let message = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after error statement")?;
        Ok(Stmt::at(StmtKind::Error { args, message }, span))
    }

    fn fail_statement(&mut self, span: Span) -> Result<Stmt, ViceError> {
        self.consume(TokenKind::OutputArrow, "Expected '<-' after fail")?;
        let message = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after fail statement")?;
        Ok(Stmt::at(StmtKind::Fail { message }, span))
    }

    fn output(&mut self, span: Span) -> Result<Stmt, ViceError> {
        self.consume(TokenKind::OutputArrow, "Expected '<-'")?;
        let mut expressions = Vec::new();
        loop {
            expressions.push(self.expression()?);
            if !self.match_kind(TokenKind::Comma) {
                break;
            }
        }
        self.consume(TokenKind::Semicolon, "Expected ';'")?;
        Ok(Stmt::at(StmtKind::Output(expressions), span))
    }

    fn emit(&mut self, span: Span) -> Result<Stmt, ViceError> {
        self.consume(TokenKind::OutputArrow, "Expected '<-' after emit")?;
        let mut expressions = Vec::new();
        loop {
            expressions.push(self.expression()?);
            if !self.match_kind(TokenKind::Comma) {
                break;
            }
        }
        self.consume(TokenKind::Semicolon, "Expected ';' after emit statement")?;
        Ok(Stmt::at(StmtKind::Emit(expressions), span))
    }

    fn expression(&mut self) -> Result<Expr, ViceError> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<Expr, ViceError> {
        let mut expr = self.logical_and()?;
        while self.match_kind(TokenKind::Or) {
            let token = self.previous().clone();
            let right = self.logical_and()?;
            expr = Expr::at(
                ExprKind::Logical {
                    left: Box::new(expr),
                    operator: LogicalOperator::Or,
                    right: Box::new(right),
                },
                Self::token_span(&token),
            );
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, ViceError> {
        let mut expr = self.equality()?;
        while self.match_kind(TokenKind::And) {
            let token = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::at(
                ExprKind::Logical {
                    left: Box::new(expr),
                    operator: LogicalOperator::And,
                    right: Box::new(right),
                },
                Self::token_span(&token),
            );
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ViceError> {
        let mut expr = self.comparison()?;
        while self.match_kind(TokenKind::EqualEqual) || self.match_kind(TokenKind::BangEqual) {
            let token = self.previous().clone();
            let operator = match token.kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::BangEqual => BinaryOperator::NotEqual,
                _ => return Err(self.syntax_error("Invalid equality operator", &token)),
            };
            let right = self.comparison()?;
            expr = Expr::at(
                ExprKind::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                Self::token_span(&token),
            );
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ViceError> {
        let mut expr = self.term()?;
        while self.match_kind(TokenKind::Greater)
            || self.match_kind(TokenKind::GreaterEqual)
            || self.match_kind(TokenKind::Less)
            || self.match_kind(TokenKind::LessEqual)
        {
            let token = self.previous().clone();
            let operator = match token.kind {
                TokenKind::Greater => BinaryOperator::Greater,
                TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenKind::Less => BinaryOperator::Less,
                TokenKind::LessEqual => BinaryOperator::LessEqual,
                _ => return Err(self.syntax_error("Invalid comparison operator", &token)),
            };
            let right = self.term()?;
            expr = Expr::at(
                ExprKind::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                Self::token_span(&token),
            );
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ViceError> {
        let mut expr = self.factor()?;
        while self.match_kind(TokenKind::Plus) || self.match_kind(TokenKind::Minus) {
            let token = self.previous().clone();
            let operator = match token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => return Err(self.syntax_error("Invalid additive operator", &token)),
            };
            let right = self.factor()?;
            expr = Expr::at(
                ExprKind::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                Self::token_span(&token),
            );
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ViceError> {
        let mut expr = self.unary()?;
        while self.match_kind(TokenKind::Star)
            || self.match_kind(TokenKind::Slash)
            || self.match_kind(TokenKind::Percent)
        {
            let token = self.previous().clone();
            let operator = match token.kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Modulo,
                _ => return Err(self.syntax_error("Invalid multiplicative operator", &token)),
            };
            let right = self.unary()?;
            expr = Expr::at(
                ExprKind::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                Self::token_span(&token),
            );
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ViceError> {
        if self.match_kind(TokenKind::Bang)
            || self.match_kind(TokenKind::Minus)
            || self.match_kind(TokenKind::Ampersand)
            || self.match_kind(TokenKind::Star)
        {
            let token = self.previous().clone();
            let operator = match token.kind {
                TokenKind::Bang => UnaryOperator::Not,
                TokenKind::Minus => UnaryOperator::Negate,
                TokenKind::Ampersand => UnaryOperator::Reference,
                TokenKind::Star => UnaryOperator::Dereference,
                _ => return Err(self.syntax_error("Invalid unary operator", &token)),
            };
            let right = self.unary()?;
            return Ok(Expr::at(
                ExprKind::Unary {
                    operator,
                    right: Box::new(right),
                },
                Self::token_span(&token),
            ));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ViceError> {
        let token = self.advance();
        let span = Self::token_span(&token);
        let mut expr = match token.kind {
            TokenKind::String => {
                let value = token.lexeme;
                if value.contains("${") {
                    self.parse_interpolated_string(&value, span)?
                } else {
                    Expr::at(ExprKind::String(value), span)
                }
            }
            TokenKind::Number => Expr::at(ExprKind::Number(token.lexeme), span),
            TokenKind::True => Expr::at(ExprKind::Bool(true), span),
            TokenKind::False => Expr::at(ExprKind::Bool(false), span),
            TokenKind::Identifier => Expr::at(ExprKind::Identifier(token.lexeme), span),
            TokenKind::LParen => {
                let expr = self.expression()?;
                self.consume(TokenKind::RParen, "Expected ')'")?;
                expr
            }
            TokenKind::LBracket => {
                let mut elements = Vec::new();
                if !self.check(TokenKind::RBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RBracket, "Expected ']' after array elements")?;
                Expr::at(ExprKind::Array(elements), span)
            }
            TokenKind::LBrace => {
                let mut elements = Vec::new();
                if !self.check(TokenKind::RBrace) {
                    self.in_map_literal += 1;
                    loop {
                        let key = self.expression()?;
                        self.consume(TokenKind::Colon, "Expected ':' after map key")?;
                        let value = self.expression()?;
                        elements.push((key, value));
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.in_map_literal -= 1;
                }
                self.consume(TokenKind::RBrace, "Expected '}' after map elements")?;
                Expr::at(ExprKind::Map(elements), span)
            }
            TokenKind::Match => {
                let (condition, arms) = self.parse_match_body()?;
                Expr::at(
                    ExprKind::Match {
                        condition: Box::new(condition),
                        arms,
                    },
                    span,
                )
            }
            TokenKind::Pipe => {
                let mut params = Vec::new();
                if !self.check(TokenKind::Pipe) {
                    loop {
                        let param_span = Self::token_span(self.peek());
                        let mutable = self.match_kind(TokenKind::Mut);
                        let name = self.consume_ident("Expected parameter name in lambda")?;
                        let type_annotation = if self.match_kind(TokenKind::Colon) {
                            Some(self.type_annotation()?)
                        } else {
                            None
                        };
                        params.push(Param {
                            name,
                            mutable,
                            type_annotation,
                            span: param_span,
                        });
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::Pipe, "Expected '|' after lambda parameters")?;

                let return_type = if self.match_kind(TokenKind::TypeArrow) {
                    Some(self.type_annotation()?)
                } else {
                    None
                };

                self.consume(
                    TokenKind::AssignArrow,
                    "Expected '=>' after lambda signature",
                )?;
                let body = self.expression()?;

                Expr::at(
                    ExprKind::Lambda {
                        params,
                        return_type,
                        body: Box::new(body),
                    },
                    span,
                )
            }
            _ => return Err(self.syntax_error("Expected expression", &token)),
        };

        loop {
            let mut type_args = Vec::new();
            if self.match_kind(TokenKind::DoubleColon) {
                self.consume(TokenKind::Less, "Expected '<' after '::'")?;
                if !self.check(TokenKind::Greater) {
                    loop {
                        type_args.push(self.type_annotation()?);
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::Greater, "Expected '>' after type arguments")?;
            }

            if self.match_kind(TokenKind::LParen) {
                let call_token = self.previous().clone();
                let name = match expr.kind {
                    ExprKind::Identifier(name) => name,
                    _ => {
                        return Err(self.syntax_error("Only identifiers can be called", self.peek()))
                    }
                };
                let args = self.call_args()?;
                self.consume(TokenKind::RParen, "Expected ')' after arguments")?;
                expr = Expr::at(
                    ExprKind::Call {
                        name,
                        type_args,
                        args,
                    },
                    Self::token_span(&call_token),
                );
            } else if !type_args.is_empty() {
                return Err(self.syntax_error("Expected '(' after type arguments", self.peek()));
            } else if self.allow_struct_literal
                && self.check(TokenKind::LBrace)
                && matches!(&expr.kind, ExprKind::Identifier(name) if name.chars().next().is_some_and(|c| c.is_uppercase()))
            {
                let brace_token = self.advance();
                let mut args = Vec::new();
                if !self.check(TokenKind::RBrace) {
                    loop {
                        let field_span = Self::token_span(self.peek());
                        let field_name = self.consume_ident("Expected field name in struct initialization")?;
                        self.consume(TokenKind::Colon, "Expected ':' after field name")?;
                        let val = self.expression()?;
                        args.push(CallArg::new(Some(field_name), val, field_span));
                        if !self.match_kind(TokenKind::Comma) || self.check(TokenKind::RBrace) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RBrace, "Expected '}' after struct fields")?;
                let name = match expr.kind {
                    ExprKind::Identifier(n) => n,
                    _ => unreachable!(),
                };
                expr = Expr::at(
                    ExprKind::Call {
                        name,
                        type_args,
                        args,
                    },
                    Self::token_span(&brace_token),
                );
            } else if self.match_kind(TokenKind::LBracket) {
                let index_token = self.previous().clone();
                let index = self.expression()?;
                self.consume(TokenKind::RBracket, "Expected ']' after index")?;
                expr = Expr::at(
                    ExprKind::Index {
                        entity: Box::new(expr),
                        index: Box::new(index),
                    },
                    Self::token_span(&index_token),
                );
            } else if self.match_kind(TokenKind::Dot) {
                let dot_token = self.previous().clone();
                let name = self.consume_ident("Expected field name after '.'")?;

                let mut method_type_args = Vec::new();
                if self.match_kind(TokenKind::DoubleColon) {
                    self.consume(TokenKind::Less, "Expected '<' after '::'")?;
                    if !self.check(TokenKind::Greater) {
                        loop {
                            method_type_args.push(self.type_annotation()?);
                            if !self.match_kind(TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::Greater, "Expected '>' after type arguments")?;
                }

                if self.match_kind(TokenKind::LParen) {
                    let args = self.call_args()?;
                    self.consume(TokenKind::RParen, "Expected ')'")?;
                    expr = Expr::at(
                        ExprKind::MethodCall {
                            entity: Box::new(expr),
                            method: name,
                            type_args: method_type_args,
                            args,
                        },
                        Self::token_span(&dot_token),
                    );
                } else if !method_type_args.is_empty() {
                    return Err(self.syntax_error("Expected '(' after type arguments", self.peek()));
                } else {
                    expr = Expr::at(
                        ExprKind::Get {
                            entity: Box::new(expr),
                            name,
                        },
                        Self::token_span(&dot_token),
                    );
                }
            } else if self.check(TokenKind::Colon) && self.in_map_literal == 0 {
                self.advance();
                let token = self.previous().clone();
                let name = self.consume_ident("Expected property name after ':'")?;
                expr = Expr::at(
                    ExprKind::PropertyCheck {
                        entity: Box::new(expr),
                        name,
                    },
                    Self::token_span(&token),
                );
            } else if self.match_kind(TokenKind::Question) {
                let token = self.previous().clone();
                expr = Expr::at(
                    ExprKind::Try {
                        expr: Box::new(expr),
                    },
                    Self::token_span(&token),
                );
            } else if self.match_kind(TokenKind::Bang) {
                let token = self.previous().clone();
                expr = Expr::at(
                    ExprKind::Unwrap {
                        expr: Box::new(expr),
                    },
                    Self::token_span(&token),
                );
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn call_args(&mut self) -> Result<Vec<CallArg>, ViceError> {
        let mut args = Vec::new();
        if self.check(TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            if self.check(TokenKind::Identifier)
                && (self.check_next(TokenKind::AssignArrow) || self.check_next(TokenKind::Colon))
            {
                let name_token = self.advance();
                let span = Self::token_span(&name_token);
                if self.check(TokenKind::AssignArrow) {
                    self.advance();
                } else {
                    self.consume(TokenKind::Colon, "Expected '=>' or ':' after named argument")?;
                }
                let value = self.expression()?;
                args.push(CallArg::new(Some(name_token.lexeme), value, span));
            } else {
                let value = self.expression()?;
                let span = value.span;
                args.push(CallArg::new(None, value, span));
            }

            if !self.match_kind(TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<(), ViceError> {
        if self.check(kind) {
            self.advance();
            return Ok(());
        }
        Err(self.syntax_error(msg, self.peek()))
    }

    fn consume_ident(&mut self, msg: &str) -> Result<String, ViceError> {
        if self.check(TokenKind::Identifier) {
            return Ok(self.advance().lexeme);
        }
        Err(self.syntax_error(msg, self.peek()))
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn parse_interpolated_string(&mut self, value: &str, span: Span) -> Result<Expr, ViceError> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut in_expr = false;
        let mut brace_depth = 0;

        // The lexeme includes the outer quotes, e.g. "Masz ${hp} życia"
        // Let's strip the first and last quote if they exist.
        let inner_value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            &value[1..value.len() - 1]
        } else {
            value
        };

        let mut chars = inner_value.chars().peekable();

        while let Some(c) = chars.next() {
            if !in_expr {
                if c == '$' && chars.peek() == Some(&'{') {
                    chars.next(); // consume '{'
                    if !current_part.is_empty() {
                        parts.push(Expr::at(ExprKind::String(current_part.clone()), span));
                        current_part.clear();
                    }
                    in_expr = true;
                    brace_depth = 1;
                } else {
                    current_part.push(c);
                }
            } else {
                if c == '{' {
                    brace_depth += 1;
                    current_part.push(c);
                } else if c == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        // We extracted an expression string.
                        let mut lexer = crate::lexer::Lexer::new(current_part.clone());
                        let tokens = lexer.tokenize()?;
                        let mut sub_parser = Parser::new(tokens);

                        // Parse it as an expression!
                        if !sub_parser.is_at_end() {
                            let expr = sub_parser.expression()?;
                            parts.push(expr);
                        }

                        current_part.clear();
                        in_expr = false;
                    } else {
                        current_part.push(c);
                    }
                } else {
                    current_part.push(c);
                }
            }
        }

        if in_expr {
            return Err(self.syntax_error("Unclosed interpolation brace '}'", self.peek()));
        }

        if !current_part.is_empty() {
            parts.push(Expr::at(ExprKind::String(current_part), span));
        }

        Ok(Expr::at(ExprKind::InterpolatedString(parts), span))
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind == kind
    }

    fn check_next(&self, kind: TokenKind) -> bool {
        self.tokens
            .get(self.current + 1)
            .map(|token| token.kind == kind)
            .unwrap_or(false)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EOF
    }

    fn token_span(token: &Token) -> Span {
        Span::at(token.line, token.column, token.length)
    }

    fn syntax_error(&self, msg: &str, token: &Token) -> ViceError {
        // Provide the found token in the message to make errors clearer
        let found = if token.kind == TokenKind::EOF {
            "end of file".to_string()
        } else if token.lexeme.trim().is_empty() {
            format!("{:?}", token.kind)
        } else {
            format!("'{}'", token.lexeme)
        };
        let full_msg = format!("{}; found {}", msg, found);

        ViceError::at(
            ViceErrorType::Syntax,
            &full_msg,
            None,
            token.line,
            token.column,
            token.length,
        )
    }
}
