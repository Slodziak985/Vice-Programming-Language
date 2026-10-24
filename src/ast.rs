use crate::types::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub source_id: usize,
}

impl Span {
    pub fn new(line: usize) -> Self {
        Self {
            line,
            column: 1,
            length: 1,
            source_id: 0,
        }
    }

    pub fn at(line: usize, column: usize, length: usize) -> Self {
        Self {
            line,
            column: column.max(1),
            length: length.max(1),
            source_id: 0,
        }
    }

    pub fn in_source(mut self, source_id: usize) -> Self {
        self.source_id = source_id;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
    pub data_declarations: Vec<Stmt>,
    pub impls: Vec<ImplBlock>,
    pub top_level: Vec<TopLevel>,
    pub sources: Vec<String>,
}

impl Program {
    pub fn assign_source(&mut self, source_id: usize) {
        for import in &mut self.imports {
            import.span.source_id = source_id;
        }
        for function in &mut self.functions {
            assign_function_source(function, source_id);
        }
        for statement in &mut self.data_declarations {
            assign_stmt_source(statement, source_id);
        }
        for implementation in &mut self.impls {
            assign_impl_source(implementation, source_id);
        }
        for top in &mut self.top_level {
            match top {
                TopLevel::Import(import) => import.span.source_id = source_id,
                TopLevel::Function(function) => assign_function_source(function, source_id),
                TopLevel::Data(statement)
                | TopLevel::EnumDef(statement)
                | TopLevel::TraitDef(statement) => assign_stmt_source(statement, source_id),
                TopLevel::Impl(implementation) => assign_impl_source(implementation, source_id),
            }
        }
    }
}

fn assign_impl_source(implementation: &mut ImplBlock, source_id: usize) {
    implementation.span.source_id = source_id;
    for method in &mut implementation.methods {
        assign_function_source(method, source_id);
    }
}

fn assign_function_source(function: &mut Function, source_id: usize) {
    function.span.source_id = source_id;
    for attribute in &mut function.attributes {
        assign_attribute_source(attribute, source_id);
    }
    for param in &mut function.params {
        param.span.source_id = source_id;
    }
    for statement in &mut function.body {
        assign_stmt_source(statement, source_id);
    }
}

fn assign_attribute_source(attribute: &mut Attribute, source_id: usize) {
    attribute.span.source_id = source_id;
    for arg in &mut attribute.args {
        assign_call_arg_source(arg, source_id);
    }
}

fn assign_call_arg_source(arg: &mut CallArg, source_id: usize) {
    arg.span.source_id = source_id;
    assign_expr_source(&mut arg.value, source_id);
}

fn assign_stmt_source(statement: &mut Stmt, source_id: usize) {
    statement.span.source_id = source_id;
    match &mut statement.kind {
        StmtKind::Output(expressions) | StmtKind::Emit(expressions) => {
            for expression in expressions {
                assign_expr_source(expression, source_id);
            }
        }
        StmtKind::Variable { value, .. }
        | StmtKind::Assign { value, .. }
        | StmtKind::Return(Some(value)) => assign_expr_source(value, source_id),
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            assign_expr_source(condition, source_id);
            for item in then_branch {
                assign_stmt_source(item, source_id);
            }
            if let Some(items) = else_branch {
                for item in items {
                    assign_stmt_source(item, source_id);
                }
            }
        }
        StmtKind::While { condition, body } => {
            assign_expr_source(condition, source_id);
            for item in body {
                assign_stmt_source(item, source_id);
            }
        }
        StmtKind::ForLoop {
            start, end, body, ..
        } => {
            assign_expr_source(start, source_id);
            assign_expr_source(end, source_id);
            for item in body {
                assign_stmt_source(item, source_id);
            }
        }
        StmtKind::ForEach {
            collection, body, ..
        } => {
            assign_expr_source(collection, source_id);
            for item in body {
                assign_stmt_source(item, source_id);
            }
        }
        StmtKind::Block(items) => {
            for item in items {
                assign_stmt_source(item, source_id);
            }
        }
        StmtKind::Expression(expression) => assign_expr_source(expression, source_id),
        StmtKind::Match { condition, arms } => {
            assign_expr_source(condition, source_id);
            for arm in arms {
                assign_match_arm_source(arm, source_id);
            }
        }
        StmtKind::MemberAssign { entity, value, .. } => {
            assign_expr_source(entity, source_id);
            assign_expr_source(value, source_id);
        }
        StmtKind::IndexAssign {
            entity,
            index,
            value,
        } => {
            assign_expr_source(entity, source_id);
            assign_expr_source(index, source_id);
            assign_expr_source(value, source_id);
        }
        StmtKind::DataDeclaration {
            attributes, fields, ..
        } => {
            for attribute in attributes {
                assign_attribute_source(attribute, source_id);
            }
            for field in fields {
                field.span.source_id = source_id;
                assign_expr_source(&mut field.default_value, source_id);
            }
        }
        StmtKind::EnumDef { attributes, .. } => {
            for attribute in attributes {
                assign_attribute_source(attribute, source_id);
            }
        }
        StmtKind::TraitDef {
            attributes,
            methods,
            ..
        } => {
            for attribute in attributes {
                assign_attribute_source(attribute, source_id);
            }
            for method in methods {
                method.span.source_id = source_id;
                for param in &mut method.params {
                    param.span.source_id = source_id;
                }
            }
        }
        StmtKind::Error { args, message } => {
            for arg in args {
                assign_call_arg_source(arg, source_id);
            }
            assign_expr_source(message, source_id);
        }
        StmtKind::Fail { message } => {
            assign_expr_source(message, source_id);
        }
        StmtKind::DerefAssign { pointer, value } => {
            assign_expr_source(pointer, source_id);
            assign_expr_source(value, source_id);
        }
        StmtKind::Input(_) | StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {}
    }
}

fn assign_match_arm_source(arm: &mut MatchArm, source_id: usize) {
    for pattern in &mut arm.patterns {
        assign_expr_source(pattern, source_id);
    }
    assign_stmt_source(&mut arm.body, source_id);
}

fn assign_expr_source(expression: &mut Expr, source_id: usize) {
    expression.span.source_id = source_id;
    match &mut expression.kind {
        ExprKind::InterpolatedString(parts) | ExprKind::Array(parts) => {
            for part in parts {
                assign_expr_source(part, source_id);
            }
        }
        ExprKind::Map(entries) => {
            for (key, value) in entries {
                assign_expr_source(key, source_id);
                assign_expr_source(value, source_id);
            }
        }
        ExprKind::Unary { right, .. }
        | ExprKind::Try { expr: right }
        | ExprKind::Unwrap { expr: right } => assign_expr_source(right, source_id),
        ExprKind::Binary { left, right, .. } | ExprKind::Logical { left, right, .. } => {
            assign_expr_source(left, source_id);
            assign_expr_source(right, source_id);
        }
        ExprKind::Index { entity, index } => {
            assign_expr_source(entity, source_id);
            assign_expr_source(index, source_id);
        }
        ExprKind::Call { args, .. } => {
            for arg in args {
                assign_call_arg_source(arg, source_id);
            }
        }
        ExprKind::MethodCall { entity, args, .. } => {
            assign_expr_source(entity, source_id);
            for arg in args {
                assign_call_arg_source(arg, source_id);
            }
        }
        ExprKind::Get { entity, .. } | ExprKind::PropertyCheck { entity, .. } => {
            assign_expr_source(entity, source_id);
        }
        ExprKind::Match { condition, arms } => {
            assign_expr_source(condition, source_id);
            for arm in arms {
                assign_match_arm_source(arm, source_id);
            }
        }
        ExprKind::Lambda { params, body, .. } => {
            for param in params {
                param.span.source_id = source_id;
            }
            assign_expr_source(body, source_id);
        }
        ExprKind::Range { start, end } => {
            assign_expr_source(start, source_id);
            assign_expr_source(end, source_id);
        }
        ExprKind::String(_)
        | ExprKind::Number(_)
        | ExprKind::Bool(_)
        | ExprKind::Identifier(_)
        | ExprKind::EnumVariant { .. } => {}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevel {
    Import(Import),
    Function(Function),
    Data(Stmt),
    EnumDef(Stmt),
    TraitDef(Stmt),
    Impl(ImplBlock),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub target: Type,
    pub trait_name: Option<Type>,
    pub type_params: Vec<String>,
    pub methods: Vec<Function>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub type_params: Vec<String>,
    pub attributes: Vec<Attribute>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub mutable: bool,
    pub type_annotation: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethodDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataField {
    pub name: String,
    pub type_annotation: Type,
    pub default_value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, line: usize) -> Self {
        Self {
            kind,
            span: Span::new(line),
        }
    }

    pub fn at(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Output(Vec<Expr>),
    Emit(Vec<Expr>),
    Variable {
        name: String,
        mutable: bool,
        type_annotation: Option<Type>,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    Assign {
        name: String,
        value: Expr,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    ForLoop {
        iterator: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
    },
    ForEach {
        iterator: String,
        collection: Expr,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    Input(String),
    Expression(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    Match {
        condition: Expr,
        arms: Vec<MatchArm>,
    },
    MemberAssign {
        entity: Expr,
        name: String,
        value: Expr,
    },
    IndexAssign {
        entity: Expr,
        index: Expr,
        value: Expr,
    },
    DataDeclaration {
        name: String,
        type_params: Vec<String>,
        attributes: Vec<Attribute>,
        fields: Vec<DataField>,
    },
    EnumDef {
        name: String,
        attributes: Vec<Attribute>,
        variants: Vec<String>,
    },
    TraitDef {
        name: String,
        type_params: Vec<String>,
        attributes: Vec<Attribute>,
        methods: Vec<TraitMethodDef>,
    },
    Error {
        args: Vec<CallArg>,
        message: Expr,
    },
    Fail {
        message: Expr,
    },
    DerefAssign {
        pointer: Expr,
        value: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: Expr,
    pub span: Span,
}

impl CallArg {
    pub fn new(name: Option<String>, value: Expr, span: Span) -> Self {
        Self { name, value, span }
    }
}

impl Expr {
    pub fn new(kind: ExprKind, line: usize) -> Self {
        Self {
            kind,
            span: Span::new(line),
        }
    }

    pub fn at(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub patterns: Vec<Expr>,
    pub body: Stmt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    String(String),
    InterpolatedString(Vec<Expr>),
    Number(String),
    Bool(bool),
    Array(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Identifier(String),
    Unary {
        operator: UnaryOperator,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: LogicalOperator,
        right: Box<Expr>,
    },
    Index {
        entity: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        name: String,
        type_args: Vec<Type>,
        args: Vec<CallArg>,
    },
    MethodCall {
        entity: Box<Expr>,
        method: String,
        type_args: Vec<Type>,
        args: Vec<CallArg>,
    },
    Get {
        entity: Box<Expr>,
        name: String,
    },
    EnumVariant {
        enum_name: String,
        variant_name: String,
    },
    PropertyCheck {
        entity: Box<Expr>,
        name: String,
    },
    Match {
        condition: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Lambda {
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Box<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
    Try {
        expr: Box<Expr>,
    },
    Unwrap {
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Not,
    Reference,
    Dereference,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}
