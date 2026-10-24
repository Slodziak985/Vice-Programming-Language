use crate::ast::{Expr, ExprKind, Program, Span, Stmt, StmtKind};
use crate::error::{ViceError, ViceErrorType};
use crate::value::Value;
use crate::vm::opcodes::Opcode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Opcode>,
    pub constants: Vec<Value>,
    pub locations: Vec<Span>,
    pub source_id: usize,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            locations: Vec::new(),
            source_id: 0,
        }
    }

    pub fn write(&mut self, opcode: Opcode, line: usize) {
        self.code.push(opcode);
        self.locations
            .push(Span::new(line).in_source(self.source_id));
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        if let Some(index) = self.constants.iter().position(|constant| *constant == value) {
            return index;
        }
        self.constants.push(value);
        self.constants.len() - 1
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub chunk: Chunk,
    pub arity: usize,
    pub local_count: usize,
    pub capture_count: usize,
    pub is_memoized: bool,
    pub budget: Option<u64>,
    pub is_nonreentrant: bool,
    pub max_depth: Option<usize>,
    pub is_checked: bool,
    pub is_saturating: bool,
    pub is_wrapping: bool,
    pub is_leaf: bool,
    pub is_unreachable: bool,
    pub is_profiled: bool,
    pub retry_limit: Option<usize>,
    pub rate_limit: Option<usize>,
    pub timeout_ms: Option<u64>,
    pub is_audit: bool,
}

#[derive(Debug, Clone)]
pub struct CompiledProgram {
    pub functions: HashMap<String, CompiledFunction>,
    pub entry: String,
    pub sources: Vec<String>,
    pub data_declarations: HashMap<String, HashMap<String, crate::types::Type>>,
    pub singleton_types: std::collections::HashSet<String>,
}

impl CompiledProgram {
    pub fn verify(&self) -> Result<(), ViceError> {
        crate::vm::verifier::verify(self)
    }
}

#[derive(Clone)]
pub struct Local {
    name: String,
    depth: usize,
    ty: crate::types::Type,
    mutable: bool,
}

pub struct LoopInfo {
    pub start_pos: usize,
    pub break_jumps: Vec<usize>,
    pub continue_jumps: Vec<usize>,
}

pub struct Compiler {
    pub chunks: HashMap<String, CompiledFunction>,
    pub current_chunk: Chunk,
    pub locals: Vec<Local>,
    pub scope_depth: usize,
    pub loop_stack: Vec<LoopInfo>,
    pub data_declarations: HashMap<String, Vec<crate::ast::DataField>>,
    pub enum_defs: HashMap<String, Vec<String>>,
    pub current_mode: crate::types::HybridMode,
    pub max_locals: usize,
    pub lambda_counter: usize,
    pub sources: Vec<String>,
    pub singleton_types: std::collections::HashSet<String>,
}

fn builtin_bool_field(name: &str) -> crate::ast::DataField {
    crate::ast::DataField {
        name: name.to_string(),
        type_annotation: crate::types::Type::Bool,
        default_value: crate::ast::Expr::new(crate::ast::ExprKind::Bool(false), 0),
        span: crate::ast::Span::new(0),
    }
}

fn builtin_int_field(name: &str) -> crate::ast::DataField {
    crate::ast::DataField {
        name: name.to_string(),
        type_annotation: crate::types::Type::Int,
        default_value: crate::ast::Expr::new(crate::ast::ExprKind::Number("0".to_string()), 0),
        span: crate::ast::Span::new(0),
    }
}

fn builtin_string_field(name: &str) -> crate::ast::DataField {
    crate::ast::DataField {
        name: name.to_string(),
        type_annotation: crate::types::Type::String,
        default_value: crate::ast::Expr::new(crate::ast::ExprKind::String("".to_string()), 0),
        span: crate::ast::Span::new(0),
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mut data_declarations = HashMap::new();
        data_declarations.insert(
            "Result".to_string(),
            vec![
                crate::ast::DataField {
                    name: "is_ok".to_string(),
                    type_annotation: crate::types::Type::Bool,
                    default_value: crate::ast::Expr::new(crate::ast::ExprKind::Bool(false), 0),
                    span: crate::ast::Span::new(0),
                },
                crate::ast::DataField {
                    name: "value".to_string(),
                    type_annotation: crate::types::Type::Unknown,
                    default_value: crate::ast::Expr::new(crate::ast::ExprKind::Bool(false), 0),
                    span: crate::ast::Span::new(0),
                },
                crate::ast::DataField {
                    name: "error".to_string(),
                    type_annotation: crate::types::Type::Unknown,
                    default_value: crate::ast::Expr::new(crate::ast::ExprKind::Bool(false), 0),
                    span: crate::ast::Span::new(0),
                },
            ],
        );
        data_declarations.insert(
            "Option".to_string(),
            vec![
                crate::ast::DataField {
                    name: "is_some".to_string(),
                    type_annotation: crate::types::Type::Bool,
                    default_value: crate::ast::Expr::new(crate::ast::ExprKind::Bool(false), 0),
                    span: crate::ast::Span::new(0),
                },
                crate::ast::DataField {
                    name: "value".to_string(),
                    type_annotation: crate::types::Type::Unknown,
                    default_value: crate::ast::Expr::new(crate::ast::ExprKind::Bool(false), 0),
                    span: crate::ast::Span::new(0),
                },
            ],
        );
        data_declarations.insert(
            "Error".to_string(),
            vec![
                crate::ast::DataField {
                    name: "message".to_string(),
                    type_annotation: crate::types::Type::String,
                    default_value: crate::ast::Expr::new(
                        crate::ast::ExprKind::String("".to_string()),
                        0,
                    ),
                    span: crate::ast::Span::new(0),
                },
                crate::ast::DataField {
                    name: "code".to_string(),
                    type_annotation: crate::types::Type::Int,
                    default_value: crate::ast::Expr::new(
                        crate::ast::ExprKind::Number("0".to_string()),
                        0,
                    ),
                    span: crate::ast::Span::new(0),
                },
            ],
        );
        data_declarations.insert(
            "Window".to_string(),
            vec![
                crate::ast::DataField {
                    name: "is_open".to_string(),
                    type_annotation: crate::types::Type::Bool,
                    default_value: crate::ast::Expr::new(crate::ast::ExprKind::Bool(false), 0),
                    span: crate::ast::Span::new(0),
                },
                crate::ast::DataField {
                    name: "width".to_string(),
                    type_annotation: crate::types::Type::Int,
                    default_value: crate::ast::Expr::new(
                        crate::ast::ExprKind::Number("0".to_string()),
                        0,
                    ),
                    span: crate::ast::Span::new(0),
                },
                crate::ast::DataField {
                    name: "height".to_string(),
                    type_annotation: crate::types::Type::Int,
                    default_value: crate::ast::Expr::new(
                        crate::ast::ExprKind::Number("0".to_string()),
                        0,
                    ),
                    span: crate::ast::Span::new(0),
                },
                crate::ast::DataField {
                    name: "dpi".to_string(),
                    type_annotation: crate::types::Type::Float,
                    default_value: crate::ast::Expr::new(
                        crate::ast::ExprKind::Number("1.0".to_string()),
                        0,
                    ),
                    span: crate::ast::Span::new(0),
                },
            ],
        );
        data_declarations.insert(
            "UiResponse".to_string(),
            vec![
                builtin_bool_field("clicked"),
                builtin_bool_field("hovered"),
                builtin_bool_field("active"),
                builtin_bool_field("focused"),
                builtin_bool_field("changed"),
            ],
        );
        data_declarations.insert(
            "TextInputResponse".to_string(),
            vec![
                builtin_string_field("value"),
                builtin_bool_field("changed"),
                builtin_bool_field("submitted"),
                builtin_bool_field("focused"),
            ],
        );
        data_declarations.insert(
            "Canvas".to_string(),
            vec![
                builtin_string_field("id"),
                builtin_int_field("width"),
                builtin_int_field("height"),
            ],
        );
        data_declarations.insert("Theme".to_string(), vec![builtin_string_field("name")]);
        Self {
            chunks: HashMap::new(),
            current_chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loop_stack: Vec::new(),
            data_declarations,
            enum_defs: HashMap::new(),
            current_mode: crate::types::HybridMode::Default,
            max_locals: 0,
            lambda_counter: 0,
            sources: Vec::new(),
            singleton_types: std::collections::HashSet::new(),
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.locals.pop();
            } else {
                break;
            }
        }
    }

    fn add_local(
        &mut self,
        name: String,
        depth: usize,
        ty: crate::types::Type,
        mutable: bool,
    ) -> usize {
        let index = self.locals.len();
        self.locals.push(Local {
            name,
            depth,
            ty,
            mutable,
        });
        self.max_locals = self.max_locals.max(self.locals.len());
        index
    }

    fn error(&self, span: Span, message: impl Into<String>) -> ViceError {
        ViceError::at(
            ViceErrorType::Compile,
            &message.into(),
            self.sources.get(span.source_id).cloned(),
            span.line,
            span.column,
            span.length,
        )
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    fn mode_from_attributes(
        attributes: &[crate::ast::Attribute],
        fallback: crate::types::HybridMode,
    ) -> crate::types::HybridMode {
        if attributes.iter().any(|attr| attr.name == "lowlevel") {
            crate::types::HybridMode::LowLevel
        } else if attributes.iter().any(|attr| attr.name == "highlevel") {
            crate::types::HybridMode::HighLevel
        } else {
            fallback
        }
    }

    pub fn compile(self, program: &Program) -> Result<CompiledProgram, ViceError> {
        self.compile_with_entry(program, None)
    }

    /// Compiles a program while selecting a specific top-level function as its entry point.
    ///
    /// The regular `@entry` validation is still performed. This is primarily used by tooling
    /// such as the test runner, which needs to execute each `@test` function in isolation.
    pub fn compile_with_entry(
        mut self,
        program: &Program,
        entry_override: Option<&str>,
    ) -> Result<CompiledProgram, ViceError> {
        self.sources = program.sources.clone();
        let mut module_mode = crate::types::HybridMode::Default;
        for top in &program.top_level {
            match top {
                crate::ast::TopLevel::Function(function) => {
                    module_mode = Self::mode_from_attributes(
                        &function.attributes,
                        crate::types::HybridMode::Default,
                    );
                    break;
                }
                crate::ast::TopLevel::Data(statement) => {
                    if let crate::ast::StmtKind::DataDeclaration { attributes, .. } =
                        &statement.kind
                    {
                        module_mode = Self::mode_from_attributes(
                            attributes,
                            crate::types::HybridMode::Default,
                        );
                        break;
                    }
                }
                _ => {}
            }
        }

        for top in &program.top_level {
            if let crate::ast::TopLevel::EnumDef(stmt) = top {
                if let crate::ast::StmtKind::EnumDef { name, variants, .. } = &stmt.kind {
                    self.enum_defs.insert(name.clone(), variants.clone());
                }
            } else if let crate::ast::TopLevel::Data(stmt) = top {
                if let crate::ast::StmtKind::DataDeclaration { name, fields, attributes, .. } = &stmt.kind {
                    if attributes.iter().any(|a| a.name == "singleton") {
                        self.singleton_types.insert(name.clone());
                    }
                    self.data_declarations.insert(name.clone(), fields.clone());
                }
            }
        }
        for data_stmt in &program.data_declarations {
            if let crate::ast::StmtKind::DataDeclaration { name, fields, attributes, .. } = &data_stmt.kind {
                if attributes.iter().any(|a| a.name == "singleton") {
                    self.singleton_types.insert(name.clone());
                }
                self.data_declarations.insert(name.clone(), fields.clone());
            }
        }
        let entries = program
            .functions
            .iter()
            .filter(|function| {
                function
                    .attributes
                    .iter()
                    .any(|attribute| attribute.name == "entry")
            })
            .collect::<Vec<_>>();
        if entries.len() > 1 {
            return Err(self.error(entries[1].span, "Multiple @entry functions are not allowed"));
        }
        let entry = if let Some(name) = entry_override {
            let function = program
                .functions
                .iter()
                .find(|function| function.name == name)
                .ok_or_else(|| {
                    self.error(
                        Span::new(0),
                        format!("Selected entry function '{}' does not exist", name),
                    )
                })?;
            if !function.params.is_empty() {
                return Err(self.error(
                    function.span,
                    format!(
                        "Selected entry function '{}' cannot accept parameters",
                        name
                    ),
                ));
            }
            name.to_string()
        } else if let Some(function) = entries.first() {
            function.name.clone()
        } else if program
            .functions
            .iter()
            .any(|function| function.name == "main")
        {
            "main".to_string()
        } else {
            String::new()
        };

        for func in &program.functions {
            self.compile_function(func, func.name.clone(), module_mode)?;
        }

        for impl_block in &program.impls {
            let target_name = match &impl_block.target {
                crate::types::Type::Data(name, _) => name.clone(),
                crate::types::Type::Enum(name) => name.clone(),
                _ => return Err(self.error(impl_block.span, "Impl target must be a nominal type")),
            };
            for func in &impl_block.methods {
                let full_name = format!("{}::{}", target_name, func.name);
                self.compile_function(func, full_name, module_mode)?;
            }
        }

        let data_declarations = self
            .data_declarations
            .iter()
            .map(|(name, fields)| {
                (
                    name.clone(),
                    fields
                        .iter()
                        .map(|field| (field.name.clone(), field.type_annotation.clone()))
                        .collect(),
                )
            })
            .collect();
        let compiled = CompiledProgram {
            functions: self.chunks,
            entry,
            sources: self.sources,
            data_declarations,
            singleton_types: self.singleton_types,
        };
        compiled.verify()?;
        Ok(compiled)
    }

    fn compile_function(
        &mut self,
        function: &crate::ast::Function,
        name: String,
        module_mode: crate::types::HybridMode,
    ) -> Result<(), ViceError> {
        self.current_mode = Self::mode_from_attributes(&function.attributes, module_mode);
        self.current_chunk = Chunk::new();
        self.current_chunk.source_id = function.span.source_id;
        self.locals.clear();
        self.scope_depth = 0;
        self.max_locals = 0;

        for param in &function.params {
            self.add_local(
                param.name.clone(),
                self.scope_depth,
                param
                    .type_annotation
                    .clone()
                    .unwrap_or(crate::types::Type::Unknown),
                param.mutable,
            );
        }
        for statement in &function.body {
            self.compile_statement(statement)
                .map_err(|message| self.error(statement.span, message))?;
        }
        let unit = self.current_chunk.add_constant(Value::Unit);
        self.current_chunk
            .write(Opcode::LoadConst(unit), function.span.line);
        self.current_chunk.write(Opcode::Return, function.span.line);

        optimize_chunk(&mut self.current_chunk);

        let is_memoized = function.attributes.iter().any(|a| a.name == "memoize");
        let budget = function.attributes.iter().find(|a| a.name == "budget").and_then(|attr| {
            for arg in &attr.args {
                if arg.name.as_deref() == Some("instructions") || arg.name.as_deref() == Some("limit") || arg.name.is_none() {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        return n.parse::<u64>().ok();
                    }
                }
            }
            None
        });
        let max_depth = function.attributes.iter().find(|a| a.name == "max_depth").and_then(|attr| {
            for arg in &attr.args {
                if arg.name.as_deref() == Some("limit") || arg.name.as_deref() == Some("depth") || arg.name.is_none() {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        return n.parse::<usize>().ok();
                    }
                }
            }
            None
        });
        let is_checked = function.attributes.iter().any(|a| a.name == "checked");
        let is_saturating = function.attributes.iter().any(|a| a.name == "saturating");
        let is_wrapping = function.attributes.iter().any(|a| a.name == "wrapping");
        let is_leaf = function.attributes.iter().any(|a| a.name == "leaf");
        let is_unreachable = function.attributes.iter().any(|a| a.name == "unreachable");
        let is_profiled = function.attributes.iter().any(|a| a.name == "profiled");
        let retry_limit = function.attributes.iter().find(|a| a.name == "retry").and_then(|attr| {
            for arg in &attr.args {
                if arg.name.as_deref() == Some("count") || arg.name.as_deref() == Some("attempts") || arg.name.is_none() {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        return n.parse::<usize>().ok();
                    }
                }
            }
            None
        });
        let rate_limit = function.attributes.iter().find(|a| a.name == "rate_limit").and_then(|attr| {
            for arg in &attr.args {
                if arg.name.as_deref() == Some("calls") || arg.name.as_deref() == Some("limit") || arg.name.as_deref() == Some("max") || arg.name.is_none() {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        return n.parse::<usize>().ok();
                    }
                }
            }
            None
        });
        let timeout_ms = function.attributes.iter().find(|a| a.name == "timeout").and_then(|attr| {
            for arg in &attr.args {
                if arg.name.as_deref() == Some("ms") || arg.name.as_deref() == Some("limit") || arg.name.as_deref() == Some("millis") || arg.name.is_none() {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        return n.parse::<u64>().ok();
                    }
                }
            }
            None
        });
        let is_audit = function.attributes.iter().any(|a| a.name == "audit");
        self.chunks.insert(
            name.clone(),
            CompiledFunction {
                name,
                chunk: self.current_chunk.clone(),
                arity: function.params.len(),
                local_count: self.max_locals,
                capture_count: 0,
                is_memoized,
                budget,
                is_nonreentrant: function.attributes.iter().any(|a| a.name == "nonreentrant"),
                max_depth,
                is_checked,
                is_saturating,
                is_wrapping,
                is_leaf,
                is_unreachable,
                is_profiled,
                retry_limit,
                rate_limit,
                timeout_ms,
                is_audit,
            },
        );
        Ok(())
    }

    fn compile_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match &stmt.kind {
            StmtKind::DataDeclaration { name, fields, attributes, .. } => {
                if attributes.iter().any(|a| a.name == "singleton") {
                    self.singleton_types.insert(name.clone());
                }
                self.data_declarations.insert(name.clone(), fields.clone());
            }
            StmtKind::Expression(expr) => {
                self.compile_expression(expr)?;
                self.current_chunk.write(Opcode::Pop, stmt.span.line);
            }
            StmtKind::Variable {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                self.compile_expression(value)?;
                let ty = type_annotation
                    .clone()
                    .unwrap_or(crate::types::Type::Unknown);
                if self.current_mode == crate::types::HybridMode::LowLevel
                    && ty == crate::types::Type::Int
                {
                    self.current_chunk
                        .write(Opcode::TruncateToInt, stmt.span.line);
                }
                let index = self.add_local(name.clone(), self.scope_depth, ty, *mutable);
                self.current_chunk
                    .write(Opcode::StoreLocal(index), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);
            }
            StmtKind::Assign { name, value } => {
                self.compile_expression(value)?;
                if let Some(idx) = self.resolve_local(name) {
                    let local_ty = self.locals[idx].ty.clone();
                    if self.current_mode == crate::types::HybridMode::LowLevel
                        && local_ty == crate::types::Type::Int
                    {
                        self.current_chunk
                            .write(Opcode::TruncateToInt, stmt.span.line);
                    }
                    self.current_chunk
                        .write(Opcode::StoreLocal(idx), stmt.span.line);
                } else {
                    let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                    self.current_chunk
                        .write(Opcode::StoreGlobal(idx), stmt.span.line);
                }
                self.current_chunk.write(Opcode::Pop, stmt.span.line);
            }
            StmtKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.compile_expression(expr)?;
                } else {
                    let idx = self.current_chunk.add_constant(Value::Unit);
                    self.current_chunk
                        .write(Opcode::LoadConst(idx), stmt.span.line);
                }
                self.current_chunk.write(Opcode::Return, stmt.span.line);
            }

            StmtKind::Break => {
                if let Some(loop_info) = self.loop_stack.last_mut() {
                    let jmp = self.current_chunk.code.len();
                    self.current_chunk.write(Opcode::Jump(0), stmt.span.line);
                    loop_info.break_jumps.push(jmp);
                } else {
                    return Err("Break outside of loop".to_string());
                }
            }
            StmtKind::Continue => {
                if let Some(loop_info) = self.loop_stack.last_mut() {
                    let jmp = self.current_chunk.code.len();
                    self.current_chunk.write(Opcode::Jump(0), stmt.span.line);
                    loop_info.continue_jumps.push(jmp);
                } else {
                    return Err("Continue outside of loop".to_string());
                }
            }
            StmtKind::While { condition, body } => {
                if let Some(Value::Bool(false)) = try_fold_constant(condition, self.current_mode) {
                    return Ok(());
                }
                let loop_start = self.current_chunk.code.len();
                self.loop_stack.push(LoopInfo {
                    start_pos: loop_start,
                    break_jumps: Vec::new(),
                    continue_jumps: Vec::new(),
                });
                self.compile_expression(condition)?;
                let exit_jmp = self.current_chunk.code.len();
                self.current_chunk
                    .write(Opcode::JumpIfFalse(0), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);
                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                let continue_pos = loop_start;
                self.current_chunk
                    .write(Opcode::Jump(loop_start), stmt.span.line);
                let exit_pos = self.current_chunk.code.len();
                if let Opcode::JumpIfFalse(ref mut offset) = self.current_chunk.code[exit_jmp] {
                    *offset = exit_pos;
                }
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                let break_pos = self.current_chunk.code.len();
                let loop_info = self.loop_stack.pop().ok_or_else(|| {
                    "Internal compiler error: missing while-loop context".to_string()
                })?;
                for break_jmp in loop_info.break_jumps {
                    if let Opcode::Jump(ref mut offset) = self.current_chunk.code[break_jmp] {
                        *offset = break_pos;
                    }
                }
                for cont_jmp in loop_info.continue_jumps {
                    if let Opcode::Jump(ref mut offset) = self.current_chunk.code[cont_jmp] {
                        *offset = continue_pos;
                    }
                }
            }
            StmtKind::Output(exprs) | StmtKind::Emit(exprs) => {
                for expr in exprs {
                    self.compile_expression(expr)?;
                }
                self.current_chunk
                    .write(Opcode::Output(exprs.len()), stmt.span.line);
            }
            StmtKind::Input(name) => {
                self.current_chunk.write(Opcode::Input, stmt.span.line);
                if let Some(idx) = self.resolve_local(name) {
                    self.current_chunk
                        .write(Opcode::StoreLocal(idx), stmt.span.line);
                } else {
                    let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                    self.current_chunk
                        .write(Opcode::StoreGlobal(idx), stmt.span.line);
                }
                self.current_chunk.write(Opcode::Pop, stmt.span.line);
            }
            StmtKind::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.compile_statement(stmt)?;
                }
                self.end_scope();
            }
            // Add if/else stub
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(Value::Bool(cond)) = try_fold_constant(condition, self.current_mode) {
                    if cond {
                        self.begin_scope();
                        for s in then_branch {
                            self.compile_statement(s)?;
                        }
                        self.end_scope();
                    } else if let Some(e_branch) = else_branch {
                        self.begin_scope();
                        for s in e_branch {
                            self.compile_statement(s)?;
                        }
                        self.end_scope();
                    }
                    return Ok(());
                }
                self.compile_expression(condition)?;
                let then_jmp = self.current_chunk.code.len();
                self.current_chunk
                    .write(Opcode::JumpIfFalse(0), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);
                self.begin_scope();
                for s in then_branch {
                    self.compile_statement(s)?;
                }
                self.end_scope();
                let else_jmp = self.current_chunk.code.len();
                self.current_chunk.write(Opcode::Jump(0), stmt.span.line);

                let then_pos = self.current_chunk.code.len();
                if let Opcode::JumpIfFalse(ref mut offset) = self.current_chunk.code[then_jmp] {
                    *offset = then_pos;
                }
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                if let Some(e_branch) = else_branch {
                    self.begin_scope();
                    for s in e_branch {
                        self.compile_statement(s)?;
                    }
                    self.end_scope();
                }

                let else_pos = self.current_chunk.code.len();
                if let Opcode::Jump(ref mut offset) = self.current_chunk.code[else_jmp] {
                    *offset = else_pos;
                }
            }

            StmtKind::IndexAssign {
                entity,
                index,
                value,
            } => {
                self.compile_expression(entity)?;
                self.compile_expression(index)?;
                self.compile_expression(value)?;
                self.current_chunk.write(Opcode::SetIndex, stmt.span.line);
                self.compile_write_back(entity)?;
            }

            StmtKind::MemberAssign {
                entity,
                name,
                value,
            } => {
                self.compile_expression(value)?;
                self.compile_expression(entity)?;
                let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                self.current_chunk
                    .write(Opcode::SetProperty(idx), stmt.span.line);
                self.compile_write_back(entity)?;
            }

            StmtKind::ForLoop {
                iterator,
                start,
                end,
                body,
            } => {
                self.begin_scope();
                self.compile_expression(start)?;
                let iter_idx = self.add_local(
                    iterator.clone(),
                    self.scope_depth,
                    crate::types::Type::Int,
                    true,
                );
                self.current_chunk
                    .write(Opcode::StoreLocal(iter_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);
                self.compile_expression(end)?;
                let end_idx = self.add_local(
                    format!("__end_{}", self.scope_depth),
                    self.scope_depth,
                    crate::types::Type::Int,
                    false,
                );
                self.current_chunk
                    .write(Opcode::StoreLocal(end_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                let loop_start = self.current_chunk.code.len();
                self.loop_stack.push(LoopInfo {
                    start_pos: loop_start,
                    break_jumps: Vec::new(),
                    continue_jumps: Vec::new(),
                });

                self.current_chunk
                    .write(Opcode::LoadLocal(iter_idx), stmt.span.line);
                self.current_chunk
                    .write(Opcode::LoadLocal(end_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Lte, stmt.span.line);

                let exit_jmp = self.current_chunk.code.len();
                self.current_chunk
                    .write(Opcode::JumpIfFalse(0), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                let continue_pos = self.current_chunk.code.len();

                self.current_chunk
                    .write(Opcode::LoadLocal(iter_idx), stmt.span.line);
                let one_idx = self.current_chunk.add_constant(Value::Int(1));
                self.current_chunk
                    .write(Opcode::LoadConst(one_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Add, stmt.span.line);
                self.current_chunk
                    .write(Opcode::StoreLocal(iter_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                self.current_chunk
                    .write(Opcode::Jump(loop_start), stmt.span.line);

                let exit_pos = self.current_chunk.code.len();
                if let Opcode::JumpIfFalse(ref mut offset) = self.current_chunk.code[exit_jmp] {
                    *offset = exit_pos;
                }
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                let break_pos = self.current_chunk.code.len();

                let loop_info = self.loop_stack.pop().ok_or_else(|| {
                    "Internal compiler error: missing for-loop context".to_string()
                })?;
                for break_jmp in loop_info.break_jumps {
                    if let Opcode::Jump(ref mut offset) = self.current_chunk.code[break_jmp] {
                        *offset = break_pos;
                    }
                }
                for cont_jmp in loop_info.continue_jumps {
                    if let Opcode::Jump(ref mut offset) = self.current_chunk.code[cont_jmp] {
                        *offset = continue_pos;
                    }
                }
                self.end_scope();
            }
            StmtKind::ForEach {
                iterator,
                collection,
                body,
            } => {
                self.begin_scope();
                self.compile_expression(collection)?;
                let col_idx = self.add_local(
                    format!("__col_{}", self.scope_depth),
                    self.scope_depth,
                    crate::types::Type::Unknown,
                    false,
                );
                self.current_chunk
                    .write(Opcode::StoreLocal(col_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                let zero_idx = self.current_chunk.add_constant(Value::Int(0));
                self.current_chunk
                    .write(Opcode::LoadConst(zero_idx), stmt.span.line);
                let idx_idx = self.add_local(
                    format!("__idx_{}", self.scope_depth),
                    self.scope_depth,
                    crate::types::Type::Int,
                    true,
                );
                self.current_chunk
                    .write(Opcode::StoreLocal(idx_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                let null_idx = self.current_chunk.add_constant(Value::Unit);
                self.current_chunk
                    .write(Opcode::LoadConst(null_idx), stmt.span.line);
                let iter_idx = self.add_local(
                    iterator.clone(),
                    self.scope_depth,
                    crate::types::Type::Unknown,
                    true,
                );
                self.current_chunk
                    .write(Opcode::StoreLocal(iter_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                let loop_start = self.current_chunk.code.len();
                self.loop_stack.push(LoopInfo {
                    start_pos: loop_start,
                    break_jumps: Vec::new(),
                    continue_jumps: Vec::new(),
                });

                // Condition: idx < len(col)
                self.current_chunk
                    .write(Opcode::LoadLocal(idx_idx), stmt.span.line);
                self.current_chunk
                    .write(Opcode::LoadLocal(col_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Len, stmt.span.line);
                self.current_chunk.write(Opcode::Lt, stmt.span.line);

                let exit_jmp = self.current_chunk.code.len();
                self.current_chunk
                    .write(Opcode::JumpIfFalse(0), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                // Assign iterator = col[idx]
                self.current_chunk
                    .write(Opcode::LoadLocal(col_idx), stmt.span.line);
                self.current_chunk
                    .write(Opcode::LoadLocal(idx_idx), stmt.span.line);
                self.current_chunk.write(Opcode::GetIndex, stmt.span.line);
                self.current_chunk
                    .write(Opcode::StoreLocal(iter_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                let continue_pos = self.current_chunk.code.len();

                // Increment index
                self.current_chunk
                    .write(Opcode::LoadLocal(idx_idx), stmt.span.line);
                let one_idx = self.current_chunk.add_constant(Value::Int(1));
                self.current_chunk
                    .write(Opcode::LoadConst(one_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Add, stmt.span.line);
                self.current_chunk
                    .write(Opcode::StoreLocal(idx_idx), stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                self.current_chunk
                    .write(Opcode::Jump(loop_start), stmt.span.line);

                let exit_pos = self.current_chunk.code.len();
                if let Opcode::JumpIfFalse(ref mut offset) = self.current_chunk.code[exit_jmp] {
                    *offset = exit_pos;
                }
                self.current_chunk.write(Opcode::Pop, stmt.span.line);

                let break_pos = self.current_chunk.code.len();

                let loop_info = self.loop_stack.pop().ok_or_else(|| {
                    "Internal compiler error: missing foreach-loop context".to_string()
                })?;
                for break_jmp in loop_info.break_jumps {
                    if let Opcode::Jump(ref mut offset) = self.current_chunk.code[break_jmp] {
                        *offset = break_pos;
                    }
                }
                for cont_jmp in loop_info.continue_jumps {
                    if let Opcode::Jump(ref mut offset) = self.current_chunk.code[cont_jmp] {
                        *offset = continue_pos;
                    }
                }
                self.end_scope();
            }

            StmtKind::Match { condition, arms } => {
                // 1. Compile condition
                self.compile_expression(condition)?; // stack: [cond]

                let mut end_jumps = Vec::new();

                for arm in arms {
                    let mut arm_body_jump_idxs = Vec::new();

                    for (i, pat) in arm.patterns.iter().enumerate() {
                        if i > 0 {
                            // Pop the `false` from the previous check
                            self.current_chunk.write(Opcode::Pop, stmt.span.line);
                        }

                        if let ExprKind::Identifier(name) = &pat.kind {
                            if name == "_" {
                                // Wildcard pattern
                                let c_true = self.current_chunk.add_constant(Value::Bool(true));
                                self.current_chunk
                                    .write(Opcode::LoadConst(c_true), stmt.span.line);
                                let jump_idx = self.current_chunk.code.len();
                                self.current_chunk.write(Opcode::Jump(0), stmt.span.line);
                                arm_body_jump_idxs.push((jump_idx, false));
                                continue;
                            }
                        }

                        if let ExprKind::Range { start, end } = &pat.kind {
                            self.current_chunk.write(Opcode::Dup, stmt.span.line);
                            self.compile_expression(start)?;
                            self.current_chunk.write(Opcode::Gte, stmt.span.line);
                            self.current_chunk.write(Opcode::Swap, stmt.span.line);
                            self.current_chunk.write(Opcode::Dup, stmt.span.line);
                            self.compile_expression(end)?;
                            self.current_chunk.write(Opcode::Lte, stmt.span.line);
                            self.current_chunk.write(Opcode::Rotate3, stmt.span.line);
                            self.current_chunk.write(Opcode::And, stmt.span.line);
                            let jump_idx = self.current_chunk.code.len();
                            self.current_chunk
                                .write(Opcode::JumpIfTrue(0), stmt.span.line);
                            arm_body_jump_idxs.push((jump_idx, true));
                        } else {
                            // Normal pattern
                            self.current_chunk.write(Opcode::Dup, stmt.span.line);
                            self.compile_expression(pat)?;
                            self.current_chunk.write(Opcode::Eq, stmt.span.line);
                            let jump_idx = self.current_chunk.code.len();
                            self.current_chunk
                                .write(Opcode::JumpIfTrue(0), stmt.span.line);
                            arm_body_jump_idxs.push((jump_idx, true));
                        }
                    }

                    let has_wildcard = arm.patterns.iter().any(|pat| {
                        if let ExprKind::Identifier(name) = &pat.kind {
                            name == "_"
                        } else {
                            false
                        }
                    });

                    let next_arm_jump_idx = if !has_wildcard {
                        self.current_chunk.write(Opcode::Pop, stmt.span.line);
                        let idx = self.current_chunk.code.len();
                        self.current_chunk.write(Opcode::Jump(0), stmt.span.line);
                        Some(idx)
                    } else {
                        None
                    };

                    // Compile arm body
                    let body_start = self.current_chunk.code.len();
                    for (jump_idx, is_conditional) in arm_body_jump_idxs {
                        if is_conditional {
                            self.current_chunk.code[jump_idx] = Opcode::JumpIfTrue(body_start);
                        } else {
                            self.current_chunk.code[jump_idx] = Opcode::Jump(body_start);
                        }
                    }

                    self.current_chunk.write(Opcode::Pop, stmt.span.line); // pop true
                    self.current_chunk.write(Opcode::Pop, stmt.span.line); // pop cond

                    self.compile_statement(&arm.body)?;

                    let end_jump_idx = self.current_chunk.code.len();
                    self.current_chunk.write(Opcode::Jump(0), stmt.span.line);
                    end_jumps.push(end_jump_idx);

                    if let Some(idx) = next_arm_jump_idx {
                        let next_arm_start = self.current_chunk.code.len();
                        self.current_chunk.code[idx] = Opcode::Jump(next_arm_start);
                    }
                }

                // If we fell through all arms, pop cond
                self.current_chunk.write(Opcode::Pop, stmt.span.line); // pop cond

                let match_end = self.current_chunk.code.len();
                for jump_idx in end_jumps {
                    self.current_chunk.code[jump_idx] = Opcode::Jump(match_end);
                }
            }

            StmtKind::Error { args, message } => {
                for arg in args {
                    self.compile_expression(&arg.value)?;
                }
                self.compile_expression(message)?;
                self.current_chunk
                    .write(Opcode::Raise(args.len()), stmt.span.line);
            }
            StmtKind::Fail { message } => {
                self.compile_expression(message)?;
                self.current_chunk.write(Opcode::Raise(0), stmt.span.line);
            }
            StmtKind::DerefAssign { pointer, value } => {
                self.compile_expression(pointer)?;
                self.compile_expression(value)?;
                self.current_chunk
                    .write(Opcode::AssignReference, stmt.span.line);
                self.current_chunk.write(Opcode::Pop, stmt.span.line);
            }

            _ => {
                return Err(format!(
                    "Unsupported statement in compiler: {:?}",
                    stmt.kind
                ))
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expr) -> Result<(), String> {
        if is_constant_candidate(expr, 0) {
            if let Some(folded) = try_fold_constant(expr, self.current_mode) {
                let idx = self.current_chunk.add_constant(folded);
                self.current_chunk
                    .write(Opcode::LoadConst(idx), expr.span.line);
                return Ok(());
            }
        }
        match &expr.kind {
            ExprKind::Number(val) => {
                if let Ok(i) = val.parse::<i64>() {
                    let idx = self.current_chunk.add_constant(Value::Int(i));
                    self.current_chunk
                        .write(Opcode::LoadConst(idx), expr.span.line);
                } else if let Ok(f) = val.parse::<f64>() {
                    let idx = self.current_chunk.add_constant(Value::Float(f));
                    self.current_chunk
                        .write(Opcode::LoadConst(idx), expr.span.line);
                }
            }
            ExprKind::String(val) => {
                let idx = self.current_chunk.add_constant(Value::String(val.clone()));
                self.current_chunk
                    .write(Opcode::LoadConst(idx), expr.span.line);
            }
            ExprKind::Identifier(name) => {
                if let Some(idx) = self.resolve_local(name) {
                    self.current_chunk
                        .write(Opcode::LoadLocal(idx), expr.span.line);
                } else {
                    let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                    self.current_chunk
                        .write(Opcode::LoadGlobal(idx), expr.span.line);
                }
            }
            ExprKind::Bool(val) => {
                let idx = self.current_chunk.add_constant(Value::Bool(*val));
                self.current_chunk
                    .write(Opcode::LoadConst(idx), expr.span.line);
            }
            ExprKind::Call { name, args, .. } => {
                if let Some(fields) = self.data_declarations.get(name).cloned() {
                    for (i, field) in fields.iter().enumerate() {
                        let field_idx = self
                            .current_chunk
                            .add_constant(Value::String(field.name.clone()));
                        self.current_chunk
                            .write(Opcode::LoadConst(field_idx), expr.span.line);

                        let matched_arg = args
                            .iter()
                            .find(|a| a.name.as_deref() == Some(field.name.as_str()))
                            .or_else(|| args.get(i).filter(|a| a.name.is_none()));

                        if let Some(arg) = matched_arg {
                            self.compile_expression(&arg.value)?;
                        } else {
                            self.compile_expression(&field.default_value)?;
                        }
                    }
                    let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                    self.current_chunk
                        .write(Opcode::MakeInstance(idx, fields.len()), expr.span.line);
                } else if name == "len" {
                    if args.len() != 1 {
                        return Err("len expects exactly 1 argument".into());
                    }
                    self.compile_expression(&args[0].value)?;
                    self.current_chunk.write(Opcode::Len, expr.span.line);
                } else {
                    for arg in args {
                        self.compile_expression(&arg.value)?;
                    }
                    if let Some(local) = self.resolve_local(name) {
                        self.current_chunk
                            .write(Opcode::LoadLocal(local), expr.span.line);
                    } else {
                        let idx = self
                            .current_chunk
                            .add_constant(Value::Callable(name.clone()));
                        self.current_chunk
                            .write(Opcode::LoadConst(idx), expr.span.line);
                    }
                    self.current_chunk
                        .write(Opcode::Call(args.len()), expr.span.line);
                }
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                if !matches!(left.kind, ExprKind::String(_) | ExprKind::InterpolatedString(_))
                    && !matches!(right.kind, ExprKind::String(_) | ExprKind::InterpolatedString(_))
                {
                    match operator {
                        crate::ast::BinaryOperator::Add => {
                            if matches!(&right.kind, ExprKind::Number(n) if n == "0" || n == "0.0") {
                                return self.compile_expression(left);
                            }
                            if matches!(&left.kind, ExprKind::Number(n) if n == "0" || n == "0.0") {
                                return self.compile_expression(right);
                            }
                        }
                        crate::ast::BinaryOperator::Subtract => {
                            if matches!(&right.kind, ExprKind::Number(n) if n == "0" || n == "0.0") {
                                return self.compile_expression(left);
                            }
                        }
                        crate::ast::BinaryOperator::Multiply => {
                            if matches!(&right.kind, ExprKind::Number(n) if n == "1" || n == "1.0") {
                                return self.compile_expression(left);
                            }
                            if matches!(&left.kind, ExprKind::Number(n) if n == "1" || n == "1.0") {
                                return self.compile_expression(right);
                            }
                        }
                        _ => {}
                    }
                }
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                let opcode = match operator {
                    crate::ast::BinaryOperator::Add => Opcode::Add,
                    crate::ast::BinaryOperator::Subtract => Opcode::Sub,
                    crate::ast::BinaryOperator::Multiply => Opcode::Mul,
                    crate::ast::BinaryOperator::Divide => {
                        if self.current_mode == crate::types::HybridMode::HighLevel {
                            Opcode::HighLevelDiv
                        } else {
                            Opcode::Div
                        }
                    }
                    crate::ast::BinaryOperator::Modulo => Opcode::Mod,
                    crate::ast::BinaryOperator::Equal => Opcode::Eq,
                    crate::ast::BinaryOperator::NotEqual => Opcode::Neq,
                    crate::ast::BinaryOperator::Less => Opcode::Lt,
                    crate::ast::BinaryOperator::LessEqual => Opcode::Lte,
                    crate::ast::BinaryOperator::Greater => Opcode::Gt,
                    crate::ast::BinaryOperator::GreaterEqual => Opcode::Gte,
                };
                self.current_chunk.write(opcode, expr.span.line);
            }

            ExprKind::Array(elements) => {
                for el in elements {
                    self.compile_expression(el)?;
                }
                self.current_chunk
                    .write(Opcode::MakeArray(elements.len()), expr.span.line);
            }
            ExprKind::Map(pairs) => {
                for (k, v) in pairs {
                    self.compile_expression(k)?;
                    self.compile_expression(v)?;
                }
                self.current_chunk
                    .write(Opcode::MakeMap(pairs.len()), expr.span.line);
            }
            ExprKind::Index { entity, index } => {
                self.compile_expression(entity)?;
                self.compile_expression(index)?;
                self.current_chunk.write(Opcode::GetIndex, expr.span.line);
            }

            ExprKind::Get { entity, name } => {
                if let ExprKind::Identifier(enum_name) = &entity.kind {
                    if self.enum_defs.contains_key(enum_name) {
                        let s = format!("{}.{}", enum_name, name);
                        let idx = self
                            .current_chunk
                            .add_constant(crate::value::Value::String(s));
                        self.current_chunk
                            .write(Opcode::LoadConst(idx), expr.span.line);
                        return Ok(());
                    }
                }
                self.compile_expression(entity)?;
                let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                self.current_chunk
                    .write(Opcode::GetProperty(idx), expr.span.line);
            }
            ExprKind::MethodCall {
                entity,
                method,
                args,
                ..
            } => {
                if let ExprKind::Identifier(module) = &entity.kind {
                    if crate::stdlib::BUILTIN_MODULES.contains(&module.as_str()) {
                        for arg in args {
                            self.compile_expression(&arg.value)?;
                        }
                        let module_idx = self
                            .current_chunk
                            .add_constant(Value::String(module.clone()));
                        let method_idx = self
                            .current_chunk
                            .add_constant(Value::String(method.clone()));
                        self.current_chunk.write(
                            Opcode::NativeCall(module_idx, method_idx, args.len()),
                            expr.span.line,
                        );
                        return Ok(());
                    }
                }
                if !self.compile_address(entity)? {
                    self.compile_expression(entity)?;
                }
                for arg in args {
                    self.compile_expression(&arg.value)?;
                }
                let name_idx = self
                    .current_chunk
                    .add_constant(Value::String(method.clone()));
                self.current_chunk
                    .write(Opcode::MethodCall(name_idx, args.len()), expr.span.line);
            }

            ExprKind::InterpolatedString(parts) => {
                if parts.is_empty() {
                    let idx = self
                        .current_chunk
                        .add_constant(crate::value::Value::String("".to_string()));
                    self.current_chunk
                        .write(Opcode::LoadConst(idx), expr.span.line);
                } else {
                    self.compile_expression(&parts[0])?;
                    if !matches!(parts[0].kind, ExprKind::String(_)) {
                        let to_str_idx = self
                            .current_chunk
                            .add_constant(crate::value::Value::Callable("to_str".to_string()));
                        self.current_chunk
                            .write(Opcode::LoadConst(to_str_idx), expr.span.line);
                        self.current_chunk.write(Opcode::Call(1), expr.span.line);
                    }
                    for part in parts.iter().skip(1) {
                        self.compile_expression(part)?;
                        if !matches!(part.kind, ExprKind::String(_)) {
                            let to_str_idx = self
                                .current_chunk
                                .add_constant(crate::value::Value::Callable("to_str".to_string()));
                            self.current_chunk
                                .write(Opcode::LoadConst(to_str_idx), expr.span.line);
                            self.current_chunk.write(Opcode::Call(1), expr.span.line);
                        }
                        self.current_chunk.write(Opcode::Add, expr.span.line);
                    }
                }
            }
            ExprKind::Lambda { params, body, .. } => {
                self.compile_lambda(params, body, expr.span)?;
            }
            ExprKind::Logical {
                left,
                operator,
                right,
            } => {
                match operator {
                    crate::ast::LogicalOperator::And => {
                        if matches!(left.kind, ExprKind::Bool(true)) {
                            return self.compile_expression(right);
                        }
                        if matches!(right.kind, ExprKind::Bool(true)) {
                            return self.compile_expression(left);
                        }
                    }
                    crate::ast::LogicalOperator::Or => {
                        if matches!(left.kind, ExprKind::Bool(false)) {
                            return self.compile_expression(right);
                        }
                        if matches!(right.kind, ExprKind::Bool(false)) {
                            return self.compile_expression(left);
                        }
                    }
                }
                self.compile_expression(left)?;
                match operator {
                    crate::ast::LogicalOperator::And => {
                        let jump_idx = self.current_chunk.code.len();
                        self.current_chunk
                            .write(Opcode::JumpIfFalse(0), expr.span.line);
                        self.current_chunk.write(Opcode::Pop, expr.span.line);
                        self.compile_expression(right)?;
                        let end_jump_idx = self.current_chunk.code.len();
                        self.current_chunk.write(Opcode::Jump(0), expr.span.line);

                        let false_start = self.current_chunk.code.len();
                        self.current_chunk.code[jump_idx] = Opcode::JumpIfFalse(false_start);

                        let end_start = self.current_chunk.code.len();
                        self.current_chunk.code[end_jump_idx] = Opcode::Jump(end_start);
                    }
                    crate::ast::LogicalOperator::Or => {
                        let jump_idx = self.current_chunk.code.len();
                        self.current_chunk
                            .write(Opcode::JumpIfTrue(0), expr.span.line);
                        self.current_chunk.write(Opcode::Pop, expr.span.line);
                        self.compile_expression(right)?;
                        let end_jump_idx = self.current_chunk.code.len();
                        self.current_chunk.write(Opcode::Jump(0), expr.span.line);

                        let true_start = self.current_chunk.code.len();
                        self.current_chunk.code[jump_idx] = Opcode::JumpIfTrue(true_start);

                        let end_start = self.current_chunk.code.len();
                        self.current_chunk.code[end_jump_idx] = Opcode::Jump(end_start);
                    }
                }
            }
            ExprKind::Unary { operator, right } => match operator {
                crate::ast::UnaryOperator::Reference => {
                    if !self.compile_address(right)? {
                        return Err("References require a variable, field, or index".into());
                    }
                }
                crate::ast::UnaryOperator::Dereference => {
                    self.compile_expression(right)?;
                    self.current_chunk
                        .write(Opcode::Dereference, expr.span.line);
                }
                crate::ast::UnaryOperator::Not => {
                    self.compile_expression(right)?;
                    self.current_chunk.write(Opcode::Not, expr.span.line);
                }
                crate::ast::UnaryOperator::Negate => {
                    self.compile_expression(right)?;
                    self.current_chunk.write(Opcode::Neg, expr.span.line);
                }
            },
            ExprKind::Match { condition, arms } => {
                // 1. Compile condition
                self.compile_expression(condition)?; // stack: [cond]

                let mut end_jumps = Vec::new();

                for arm in arms {
                    let mut arm_body_jump_idxs = Vec::new();

                    for (i, pat) in arm.patterns.iter().enumerate() {
                        if i > 0 {
                            // Pop the `false` from the previous check
                            self.current_chunk.write(Opcode::Pop, expr.span.line);
                        }

                        if let ExprKind::Identifier(name) = &pat.kind {
                            if name == "_" {
                                // Wildcard pattern
                                let c_true = self.current_chunk.add_constant(Value::Bool(true));
                                self.current_chunk
                                    .write(Opcode::LoadConst(c_true), expr.span.line);
                                let jump_idx = self.current_chunk.code.len();
                                self.current_chunk.write(Opcode::Jump(0), expr.span.line);
                                arm_body_jump_idxs.push((jump_idx, false));
                                continue;
                            }
                        }

                        if let ExprKind::Range { start, end } = &pat.kind {
                            self.current_chunk.write(Opcode::Dup, expr.span.line);
                            self.compile_expression(start)?;
                            self.current_chunk.write(Opcode::Gte, expr.span.line);
                            self.current_chunk.write(Opcode::Swap, expr.span.line);
                            self.current_chunk.write(Opcode::Dup, expr.span.line);
                            self.compile_expression(end)?;
                            self.current_chunk.write(Opcode::Lte, expr.span.line);
                            self.current_chunk.write(Opcode::Rotate3, expr.span.line);
                            self.current_chunk.write(Opcode::And, expr.span.line);
                            let jump_idx = self.current_chunk.code.len();
                            self.current_chunk
                                .write(Opcode::JumpIfTrue(0), expr.span.line);
                            arm_body_jump_idxs.push((jump_idx, true));
                        } else {
                            // Normal pattern
                            self.current_chunk.write(Opcode::Dup, expr.span.line);
                            self.compile_expression(pat)?;
                            self.current_chunk.write(Opcode::Eq, expr.span.line);
                            let jump_idx = self.current_chunk.code.len();
                            self.current_chunk
                                .write(Opcode::JumpIfTrue(0), expr.span.line);
                            arm_body_jump_idxs.push((jump_idx, true));
                        }
                    }

                    let has_wildcard = arm.patterns.iter().any(|pat| {
                        if let ExprKind::Identifier(name) = &pat.kind {
                            name == "_"
                        } else {
                            false
                        }
                    });

                    let next_arm_jump_idx = if !has_wildcard {
                        self.current_chunk.write(Opcode::Pop, expr.span.line);
                        let idx = self.current_chunk.code.len();
                        self.current_chunk.write(Opcode::Jump(0), expr.span.line);
                        Some(idx)
                    } else {
                        None
                    };

                    // Compile arm body
                    let body_start = self.current_chunk.code.len();
                    for (jump_idx, is_conditional) in arm_body_jump_idxs {
                        if is_conditional {
                            self.current_chunk.code[jump_idx] = Opcode::JumpIfTrue(body_start);
                        } else {
                            self.current_chunk.code[jump_idx] = Opcode::Jump(body_start);
                        }
                    }

                    self.current_chunk.write(Opcode::Pop, expr.span.line); // pop true
                    self.current_chunk.write(Opcode::Pop, expr.span.line); // pop cond

                    self.compile_match_arm_body(&arm.body)?;

                    let end_jump_idx = self.current_chunk.code.len();
                    self.current_chunk.write(Opcode::Jump(0), expr.span.line);
                    end_jumps.push(end_jump_idx);

                    if let Some(idx) = next_arm_jump_idx {
                        let next_arm_start = self.current_chunk.code.len();
                        self.current_chunk.code[idx] = Opcode::Jump(next_arm_start);
                    }
                }

                // A non-exhaustive match expression must fail explicitly instead
                // of silently manufacturing a value of the wrong type.
                self.current_chunk.write(Opcode::Pop, expr.span.line); // pop cond
                let message = self
                    .current_chunk
                    .add_constant(Value::String("Non-exhaustive match expression".to_string()));
                self.current_chunk
                    .write(Opcode::LoadConst(message), expr.span.line);
                self.current_chunk.write(Opcode::Raise(0), expr.span.line);

                let match_end = self.current_chunk.code.len();
                for jump_idx in end_jumps {
                    self.current_chunk.code[jump_idx] = Opcode::Jump(match_end);
                }
            }
            ExprKind::EnumVariant {
                enum_name,
                variant_name,
            } => {
                let s = format!("{}.{}", enum_name, variant_name);
                let idx = self
                    .current_chunk
                    .add_constant(crate::value::Value::String(s));
                self.current_chunk
                    .write(Opcode::LoadConst(idx), expr.span.line);
            }
            ExprKind::Try { expr: inner } => {
                self.compile_expression(inner)?;
                self.current_chunk.write(Opcode::Try, expr.span.line);
            }
            ExprKind::Unwrap { expr: inner } => {
                self.compile_expression(inner)?;
                self.current_chunk.write(Opcode::Unwrap, expr.span.line);
            }
            ExprKind::PropertyCheck { entity, name } => {
                self.compile_expression(entity)?;
                let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                self.current_chunk
                    .write(Opcode::GetProperty(idx), expr.span.line);
            }
            _ => {
                return Err(format!(
                    "Unsupported expression in compiler: {:?}",
                    expr.kind
                ))
            }
        }
        Ok(())
    }

    fn compile_address(&mut self, expression: &Expr) -> Result<bool, String> {
        match &expression.kind {
            ExprKind::Identifier(name) => {
                if let Some(index) = self.resolve_local(name) {
                    let mutable = self.locals[index].mutable;
                    self.current_chunk
                        .write(Opcode::ReferenceLocal(index, mutable), expression.span.line);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ExprKind::Get { entity, name } => {
                if !self.compile_address(entity)? {
                    return Ok(false);
                }
                let name = self.current_chunk.add_constant(Value::String(name.clone()));
                self.current_chunk
                    .write(Opcode::ReferenceProperty(name), expression.span.line);
                Ok(true)
            }
            ExprKind::Index { entity, index } => {
                if !self.compile_address(entity)? {
                    return Ok(false);
                }
                self.compile_expression(index)?;
                self.current_chunk
                    .write(Opcode::ReferenceIndex, expression.span.line);
                Ok(true)
            }
            ExprKind::Unary {
                operator: crate::ast::UnaryOperator::Dereference,
                right,
            } => {
                self.compile_expression(right)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn compile_lambda(
        &mut self,
        params: &[crate::ast::Param],
        body: &Expr,
        span: Span,
    ) -> Result<(), String> {
        let name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        let outer_locals = self.locals.clone();
        let parameter_names = params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<std::collections::HashSet<_>>();
        let captures = outer_locals
            .iter()
            .enumerate()
            .filter(|(_, local)| !parameter_names.contains(local.name.as_str()))
            .map(|(index, local)| (index, local.clone()))
            .collect::<Vec<_>>();

        let saved_chunk = std::mem::take(&mut self.current_chunk);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scope = self.scope_depth;
        let saved_max_locals = self.max_locals;
        let saved_loops = std::mem::take(&mut self.loop_stack);

        self.current_chunk.source_id = span.source_id;
        self.scope_depth = 0;
        self.max_locals = 0;
        for param in params {
            self.add_local(
                param.name.clone(),
                0,
                param
                    .type_annotation
                    .clone()
                    .unwrap_or(crate::types::Type::Unknown),
                param.mutable,
            );
        }
        for (_, local) in &captures {
            self.add_local(local.name.clone(), 0, local.ty.clone(), local.mutable);
        }
        self.compile_expression(body)?;
        self.current_chunk.write(Opcode::Return, span.line);

        optimize_chunk(&mut self.current_chunk);

        let function = CompiledFunction {
            name: name.clone(),
            chunk: self.current_chunk.clone(),
            arity: params.len(),
            local_count: self.max_locals,
            capture_count: captures.len(),
            is_memoized: false,
            budget: None,
            is_nonreentrant: false,
            max_depth: None,
            is_checked: false,
            is_saturating: false,
            is_wrapping: false,
            is_leaf: false,
            is_unreachable: false,
            is_profiled: false,
            retry_limit: None,
            rate_limit: None,
            timeout_ms: None,
            is_audit: false,
        };

        self.current_chunk = saved_chunk;
        self.locals = saved_locals;
        self.scope_depth = saved_scope;
        self.max_locals = saved_max_locals;
        self.loop_stack = saved_loops;
        self.chunks.insert(name.clone(), function);

        let function_name = self.current_chunk.add_constant(Value::String(name));
        self.current_chunk.write(
            Opcode::MakeClosure(
                function_name,
                captures.iter().map(|(index, _)| *index).collect(),
            ),
            span.line,
        );
        Ok(())
    }

    fn compile_write_back(&mut self, entity: &crate::ast::Expr) -> Result<(), String> {
        match &entity.kind {
            crate::ast::ExprKind::Identifier(name) => {
                if let Some(idx) = self.resolve_local(name) {
                    if matches!(self.locals[idx].ty, crate::types::Type::Reference { .. }) {
                        self.current_chunk.write(Opcode::Pop, entity.span.line);
                        return Ok(());
                    }
                    self.current_chunk
                        .write(Opcode::StoreLocal(idx), entity.span.line);
                } else {
                    let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                    self.current_chunk
                        .write(Opcode::StoreGlobal(idx), entity.span.line);
                }
                self.current_chunk.write(Opcode::Pop, entity.span.line);
            }
            crate::ast::ExprKind::Get {
                entity: inner,
                name,
            } => {
                self.compile_expression(inner)?;
                let idx = self.current_chunk.add_constant(Value::String(name.clone()));
                self.current_chunk
                    .write(Opcode::SetProperty(idx), entity.span.line);
                self.compile_write_back(inner)?;
            }
            crate::ast::ExprKind::Index {
                entity: inner,
                index,
            } => {
                self.compile_expression(inner)?;
                self.compile_expression(index)?;
                self.current_chunk.write(Opcode::Rotate3, entity.span.line);
                self.current_chunk.write(Opcode::SetIndex, entity.span.line);
                self.compile_write_back(inner)?;
            }
            _ => return Err(format!("Invalid assignment target: {:?}", entity.kind)),
        }
        Ok(())
    }

    fn compile_match_arm_body(&mut self, stmt: &crate::ast::Stmt) -> Result<(), String> {
        match &stmt.kind {
            crate::ast::StmtKind::Expression(expr) => {
                self.compile_expression(expr)?;
            }
            crate::ast::StmtKind::Block(stmts) => {
                self.begin_scope();
                if !stmts.is_empty() {
                    for s in &stmts[..stmts.len() - 1] {
                        self.compile_statement(s)?;
                    }
                    self.compile_match_arm_body(&stmts[stmts.len() - 1])?;
                } else {
                    let idx = self.current_chunk.add_constant(Value::Int(0));
                    self.current_chunk
                        .write(Opcode::LoadConst(idx), stmt.span.line);
                }
                self.end_scope();
            }
            _ => {
                self.compile_statement(stmt)?;
            }
        }
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

fn is_constant_candidate(expr: &Expr, depth: usize) -> bool {
    if depth > 10 {
        return false;
    }
    match &expr.kind {
        ExprKind::Number(_) | ExprKind::Bool(_) | ExprKind::String(_) => true,
        ExprKind::Binary { left, right, .. } | ExprKind::Logical { left, right, .. } => {
            is_constant_candidate(left, depth + 1) && is_constant_candidate(right, depth + 1)
        }
        ExprKind::Unary { right, .. } => is_constant_candidate(right, depth + 1),
        ExprKind::InterpolatedString(parts) => {
            parts.iter().all(|p| is_constant_candidate(p, depth + 1))
        }
        ExprKind::Array(elems) => elems.iter().all(|e| is_constant_candidate(e, depth + 1)),
        ExprKind::MethodCall { entity, args, .. } => {
            is_constant_candidate(entity, depth + 1)
                && args.iter().all(|a| is_constant_candidate(&a.value, depth + 1))
        }
        ExprKind::Call { args, .. } => {
            args.iter().all(|a| is_constant_candidate(&a.value, depth + 1))
        }
        _ => false,
    }
}

fn try_fold_constant(expr: &Expr, mode: crate::types::HybridMode) -> Option<Value> {
    if !is_constant_candidate(expr, 0) {
        return None;
    }
    match &expr.kind {
        ExprKind::Number(val) => {
            if val.contains('.') || val.contains('e') || val.contains('E') {
                val.parse::<f64>().ok().map(Value::Float)
            } else {
                val.parse::<i64>().ok().map(Value::Int)
            }
        }
        ExprKind::Bool(b) => Some(Value::Bool(*b)),
        ExprKind::String(s) => Some(Value::String(s.clone())),
        ExprKind::Binary { left, operator, right } => {
            let left_val = try_fold_constant(left, mode)?;
            let right_val = try_fold_constant(right, mode)?;
            fold_binary_values(&left_val, operator, &right_val, mode)
        }
        ExprKind::Logical { left, operator, right } => {
            let left_val = try_fold_constant(left, mode)?;
            let right_val = try_fold_constant(right, mode)?;
            match (left_val, operator, right_val) {
                (Value::Bool(a), crate::ast::LogicalOperator::And, Value::Bool(b)) => Some(Value::Bool(a && b)),
                (Value::Bool(a), crate::ast::LogicalOperator::Or, Value::Bool(b)) => Some(Value::Bool(a || b)),
                _ => None,
            }
        }
        ExprKind::Unary { operator, right } => {
            let val = try_fold_constant(right, mode)?;
            match operator {
                crate::ast::UnaryOperator::Negate => match val {
                    Value::Int(i) => Some(Value::Int(i.wrapping_neg())),
                    Value::Float(f) => Some(Value::Float(-f)),
                    _ => None,
                },
                crate::ast::UnaryOperator::Not => match val {
                    Value::Bool(b) => Some(Value::Bool(!b)),
                    _ => None,
                },
                _ => None,
            }
        }
        ExprKind::InterpolatedString(parts) => {
            let mut folded_string = String::new();
            for part in parts {
                if let Some(val) = try_fold_constant(part, mode) {
                    folded_string.push_str(&crate::stdlib::value_to_string(&val));
                } else {
                    return None;
                }
            }
            Some(Value::String(folded_string))
        }
        ExprKind::Array(elements) => {
            let mut folded = Vec::with_capacity(elements.len());
            for elem in elements {
                folded.push(try_fold_constant(elem, mode)?);
            }
            Some(Value::Array(folded))
        }
        ExprKind::MethodCall { entity, method, args, .. } => {
            let ent_val = try_fold_constant(entity, mode)?;
            match (&ent_val, method.as_str()) {
                (Value::String(s), "len") if args.is_empty() => Some(Value::Int(s.chars().count() as i64)),
                (Value::String(s), "is_empty") if args.is_empty() => Some(Value::Bool(s.is_empty())),
                (Value::String(s), "trim") if args.is_empty() => Some(Value::String(s.trim().to_string())),
                (Value::String(s), "upper") if args.is_empty() => Some(Value::String(s.to_uppercase())),
                (Value::String(s), "lower") if args.is_empty() => Some(Value::String(s.to_lowercase())),
                (Value::String(s), "contains") if args.len() == 1 => {
                    let arg = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(needle) = arg {
                        Some(Value::Bool(s.contains(&needle)))
                    } else {
                        None
                    }
                }
                (Value::String(s), "starts_with") if args.len() == 1 => {
                    let arg = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(needle) = arg {
                        Some(Value::Bool(s.starts_with(&needle)))
                    } else {
                        None
                    }
                }
                (Value::String(s), "ends_with") if args.len() == 1 => {
                    let arg = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(needle) = arg {
                        Some(Value::Bool(s.ends_with(&needle)))
                    } else {
                        None
                    }
                }
                (Value::String(s), "char_at") if args.len() == 1 => {
                    let arg = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(idx) = arg {
                        let chars: Vec<char> = s.chars().collect();
                        if idx >= 0 && (idx as usize) < chars.len() {
                            Some(Value::String(chars[idx as usize].to_string()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                (Value::String(s), "reverse") if args.is_empty() => {
                    Some(Value::String(s.chars().rev().collect()))
                }
                (Value::String(s), "count") if args.len() == 1 => {
                    let needle = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(n) = needle {
                        if n.is_empty() {
                            Some(Value::Int(0))
                        } else {
                            Some(Value::Int(s.matches(&n).count() as i64))
                        }
                    } else {
                        None
                    }
                }
                (Value::String(s), "capitalize") if args.is_empty() => {
                    let mut chars = s.chars();
                    let res = match chars.next() {
                        None => String::new(),
                        Some(first) => {
                            let mut cap = first.to_uppercase().to_string();
                            cap.push_str(chars.as_str());
                            cap
                        }
                    };
                    Some(Value::String(res))
                }
                (Value::String(s), "replace") if args.len() == 2 => {
                    let from = try_fold_constant(&args[0].value, mode)?;
                    let to = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(f), Value::String(t)) = (from, to) {
                        Some(Value::String(s.replace(&f, &t)))
                    } else {
                        None
                    }
                }
                (Value::String(s), "split") if args.len() == 1 => {
                    let delim = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(d) = delim {
                        let parts = s.split(&d).map(|p| Value::String(p.to_string())).collect();
                        Some(Value::Array(parts))
                    } else {
                        None
                    }
                }
                (Value::String(s), "repeat") if args.len() == 1 => {
                    let arg = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(cnt) = arg {
                        if cnt >= 0 && (cnt as usize) <= 100_000 {
                            Some(Value::String(s.repeat(cnt as usize)))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                (Value::String(s), "char_at") if args.len() == 1 => {
                    let arg = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(idx) = arg {
                        if idx >= 0 {
                            s.chars().nth(idx as usize).map(|c| Value::String(c.to_string()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                (Value::String(s), "lines") if args.is_empty() => {
                    let lines = s.lines().map(|l| Value::String(l.to_string())).collect();
                    Some(Value::Array(lines))
                }
                (Value::String(s), "pad_start") if args.len() == 2 => {
                    let len_val = try_fold_constant(&args[0].value, mode)?;
                    let pad_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Int(target_len), Value::String(pad)) = (len_val, pad_val) {
                        if target_len >= 0 && (target_len as usize) <= 100_000 && !pad.is_empty() {
                            let char_count = s.chars().count();
                            let target = target_len as usize;
                            if char_count >= target {
                                Some(Value::String(s.clone()))
                            } else {
                                let diff = target - char_count;
                                let repeated_pad = pad.repeat((diff / pad.chars().count()) + 1);
                                let prefix: String = repeated_pad.chars().take(diff).collect();
                                Some(Value::String(format!("{}{}", prefix, s)))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                (Value::String(s), "pad_end") if args.len() == 2 => {
                    let len_val = try_fold_constant(&args[0].value, mode)?;
                    let pad_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Int(target_len), Value::String(pad)) = (len_val, pad_val) {
                        if target_len >= 0 && (target_len as usize) <= 100_000 && !pad.is_empty() {
                            let char_count = s.chars().count();
                            let target = target_len as usize;
                            if char_count >= target {
                                Some(Value::String(s.clone()))
                            } else {
                                let diff = target - char_count;
                                let repeated_pad = pad.repeat((diff / pad.chars().count()) + 1);
                                let suffix: String = repeated_pad.chars().take(diff).collect();
                                Some(Value::String(format!("{}{}", s, suffix)))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                (Value::String(s), "levenshtein") if args.len() == 1 => {
                    let other = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(o) = other {
                        Some(Value::Int(crate::stdlib::levenshtein_distance(s, &o) as i64))
                    } else {
                        None
                    }
                }
                (Value::String(s), "hamming") if args.len() == 1 => {
                    let other = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(o) = other {
                        Some(Value::Int(crate::stdlib::hamming_distance(s, &o)))
                    } else {
                        None
                    }
                }
                (Value::String(s), "slugify") if args.is_empty() => {
                    Some(Value::String(crate::stdlib::slugify(s)))
                }
                (Value::String(s), "truncate") if args.len() >= 1 && args.len() <= 2 => {
                    let len_val = try_fold_constant(&args[0].value, mode)?;
                    let ellipsis = if args.len() == 2 {
                        match try_fold_constant(&args[1].value, mode)? {
                            Value::String(e) => e,
                            _ => return None,
                        }
                    } else {
                        "...".to_string()
                    };
                    if let Value::Int(max_len) = len_val {
                        Some(Value::String(crate::stdlib::truncate_string(s, max_len.max(0) as usize, &ellipsis)))
                    } else {
                        None
                    }
                }
                (Value::String(s), "hex_encode") if args.is_empty() => {
                    Some(Value::String(crate::stdlib::hex_encode(s)))
                }
                (Value::String(s), "hex_decode") if args.is_empty() => {
                    Some(Value::String(crate::stdlib::hex_decode(s)))
                }
                (Value::String(s), "lines_count") if args.is_empty() => {
                    Some(Value::Int(crate::stdlib::string_lines_count(s)))
                }
                (Value::String(s), "words_count") if args.is_empty() => {
                    Some(Value::Int(crate::stdlib::string_words_count(s)))
                }
                (Value::Array(arr), "compact") if args.is_empty() => {
                    let compacted: Vec<Value> = arr.iter().filter(|v| !matches!(v, Value::Unit)).cloned().collect();
                    Some(Value::Array(compacted))
                }
                (Value::Array(arr), "diff") if args.len() == 1 => {
                    let target = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(other) = target {
                        let diff: Vec<Value> = arr.iter().filter(|v| !other.contains(v)).cloned().collect();
                        Some(Value::Array(diff))
                    } else {
                        None
                    }
                }
                (Value::Array(arr), "intersect") if args.len() == 1 => {
                    let target = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(other) = target {
                        let mut res: Vec<Value> = Vec::new();
                        for v in arr {
                            if other.contains(v) && !res.contains(v) {
                                res.push(v.clone());
                            }
                        }
                        Some(Value::Array(res))
                    } else {
                        None
                    }
                }
                (Value::Array(arr), "is_sorted") if args.is_empty() => {
                    Some(Value::Bool(crate::stdlib::is_array_sorted(arr)))
                }
                (Value::Array(arr), "all_unique") if args.is_empty() => {
                    Some(Value::Bool(crate::stdlib::are_array_elements_unique(arr)))
                }
                (Value::Array(arr), "len") if args.is_empty() => Some(Value::Int(arr.len() as i64)),
                (Value::Array(arr), "is_empty") if args.is_empty() => Some(Value::Bool(arr.is_empty())),
                (Value::Array(arr), "sort") if args.is_empty() => {
                    let mut sorted = arr.clone();
                    sorted.sort_by(|a, b| a.compare_order(b));
                    Some(Value::Array(sorted))
                }
                (Value::Array(arr), "reverse") if args.is_empty() => {
                    let mut rev = arr.clone();
                    rev.reverse();
                    Some(Value::Array(rev))
                }
                (Value::Array(arr), "first") if args.is_empty() => arr.first().cloned(),
                (Value::Array(arr), "last") if args.is_empty() => arr.last().cloned(),
                (Value::Array(arr), "contains") if args.len() == 1 => {
                    let target = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Bool(arr.contains(&target)))
                }
                (Value::Array(arr), "index_of") if args.len() == 1 => {
                    let target = try_fold_constant(&args[0].value, mode)?;
                    let idx = arr.iter().position(|v| v == &target).map(|i| i as i64).unwrap_or(-1);
                    Some(Value::Int(idx))
                }
                (Value::Array(arr), "concat") if args.len() == 1 => {
                    let target = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(other) = target {
                        let mut combined = arr.clone();
                        combined.extend(other);
                        Some(Value::Array(combined))
                    } else {
                        None
                    }
                }
                (Value::Array(arr), "swap") if args.len() == 2 => {
                    let i_val = try_fold_constant(&args[0].value, mode)?;
                    let j_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Int(i), Value::Int(j)) = (i_val, j_val) {
                        let len = arr.len() as i64;
                        if i >= 0 && i < len && j >= 0 && j < len {
                            let mut swapped = arr.clone();
                            swapped.swap(i as usize, j as usize);
                            Some(Value::Array(swapped))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                (Value::Array(arr), "count") if args.len() == 1 => {
                    let target = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Int(arr.iter().filter(|v| **v == target).count() as i64))
                }
                (Value::Array(arr), "take") if args.len() == 1 => {
                    let n_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(n) = n_val {
                        let count = n.max(0) as usize;
                        Some(Value::Array(arr.iter().take(count).cloned().collect()))
                    } else {
                        None
                    }
                }
                (Value::Array(arr), "drop") if args.len() == 1 => {
                    let n_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(n) = n_val {
                        let count = n.max(0) as usize;
                        Some(Value::Array(arr.iter().skip(count).cloned().collect()))
                    } else {
                        None
                    }
                }
                (Value::Array(arr), "join") if args.len() == 1 => {
                    let sep = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = sep {
                        let parts: Option<Vec<String>> = arr.iter().map(|item| match item {
                            Value::String(st) => Some(st.clone()),
                            Value::Int(n) => Some(n.to_string()),
                            Value::Float(f) => Some(f.to_string()),
                            Value::Bool(b) => Some(b.to_string()),
                            _ => None,
                        }).collect();
                        parts.map(|p| Value::String(p.join(&s)))
                    } else {
                        None
                    }
                }
                (Value::Array(arr), "slice") if args.len() == 2 => {
                    let start_val = try_fold_constant(&args[0].value, mode)?;
                    let end_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Int(start), Value::Int(end)) = (start_val, end_val) {
                        let len = arr.len() as i64;
                        let s = start.clamp(0, len) as usize;
                        let e = end.clamp(s as i64, len) as usize;
                        Some(Value::Array(arr[s..e].to_vec()))
                    } else {
                        None
                    }
                }
                (Value::Map(map), "has_key" | "contains" | "contains_key") if args.len() == 1 => {
                    let key = try_fold_constant(&args[0].value, mode)?;
                    let map_key = crate::value::MapKey::from_value(&key)?;
                    Some(Value::Bool(map.contains_key(&map_key)))
                }
                (Value::Map(map), "count" | "len") if args.is_empty() => {
                    Some(Value::Int(map.len() as i64))
                }
                (Value::Array(arr), "sum") if args.is_empty() => {
                    let mut total_int = 0i64;
                    let mut total_float = 0.0f64;
                    let mut is_float = false;
                    for v in arr {
                        match v {
                            Value::Int(i) => {
                                total_int = total_int.wrapping_add(*i);
                                total_float += *i as f64;
                            }
                            Value::Float(f) => {
                                is_float = true;
                                total_float += *f;
                            }
                            _ => return None,
                        }
                    }
                    if is_float {
                        Some(Value::Float(total_float))
                    } else {
                        Some(Value::Int(total_int))
                    }
                }
                (Value::Array(arr), "product") if args.is_empty() => {
                    let mut total_int = 1i64;
                    let mut total_float = 1.0f64;
                    let mut is_float = false;
                    for v in arr {
                        match v {
                            Value::Int(i) => {
                                total_int = total_int.wrapping_mul(*i);
                                total_float *= *i as f64;
                            }
                            Value::Float(f) => {
                                is_float = true;
                                total_float *= *f;
                            }
                            _ => return None,
                        }
                    }
                    if is_float {
                        Some(Value::Float(total_float))
                    } else {
                        Some(Value::Int(total_int))
                    }
                }
                (Value::Array(arr), "dedup") if args.is_empty() => {
                    let mut deduped: Vec<Value> = Vec::new();
                    for v in arr {
                        if deduped.last() != Some(v) {
                            deduped.push(v.clone());
                        }
                    }
                    Some(Value::Array(deduped))
                }
                (Value::Array(arr), "reverse") if args.is_empty() => {
                    let mut reversed = arr.clone();
                    reversed.reverse();
                    Some(Value::Array(reversed))
                }
                (Value::Array(arr), "len") if args.is_empty() => Some(Value::Int(arr.len() as i64)),
                (Value::String(s), "rot13") if args.is_empty() => {
                    let rotated: String = s.chars().map(|c| match c {
                        'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
                        'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
                        _ => c,
                    }).collect();
                    Some(Value::String(rotated))
                }
                (Value::String(s), "reverse") if args.is_empty() => {
                    Some(Value::String(s.chars().rev().collect()))
                }
                (Value::String(s), "len") if args.is_empty() => {
                    Some(Value::Int(s.chars().count() as i64))
                }
                _ => None,
            }
        }
        ExprKind::Call { name, args, .. } => {
            match name.as_str() {
                "to_str" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::String(crate::stdlib::value_to_string(&val)))
                }
                "math.abs" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Int(i) => Some(Value::Int(i.wrapping_abs())),
                        Value::Float(f) => Some(Value::Float(f.abs())),
                        _ => None,
                    }
                }
                "math.min" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => Some(Value::Int(x.min(y))),
                        (Value::Float(x), Value::Float(y)) => Some(Value::Float(x.min(y))),
                        (Value::Int(x), Value::Float(y)) => Some(Value::Float((x as f64).min(y))),
                        (Value::Float(x), Value::Int(y)) => Some(Value::Float(x.min(y as f64))),
                        _ => None,
                    }
                }
                "math.max" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => Some(Value::Int(x.max(y))),
                        (Value::Float(x), Value::Float(y)) => Some(Value::Float(x.max(y))),
                        (Value::Int(x), Value::Float(y)) => Some(Value::Float((x as f64).max(y))),
                        (Value::Float(x), Value::Int(y)) => Some(Value::Float(x.max(y as f64))),
                        _ => None,
                    }
                }
                "math.clamp" if args.len() == 3 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    let min = try_fold_constant(&args[1].value, mode)?;
                    let max = try_fold_constant(&args[2].value, mode)?;
                    match (val, min, max) {
                        (Value::Int(v), Value::Int(mi), Value::Int(ma)) if mi <= ma => {
                            Some(Value::Int(v.max(mi).min(ma)))
                        }
                        (Value::Float(v), Value::Float(mi), Value::Float(ma)) if mi <= ma => {
                            Some(Value::Float(v.max(mi).min(ma)))
                        }
                        _ => None,
                    }
                }
                "math.sign" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Int(i) => Some(Value::Int(if i > 0 { 1 } else if i < 0 { -1 } else { 0 })),
                        Value::Float(f) => Some(Value::Int(if f > 0.0 { 1 } else if f < 0.0 { -1 } else { 0 })),
                        _ => None,
                    }
                }
                "math.floor" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) => Some(Value::Int(f.floor() as i64)),
                        Value::Int(i) => Some(Value::Int(i)),
                        _ => None,
                    }
                }
                "math.ceil" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) => Some(Value::Int(f.ceil() as i64)),
                        Value::Int(i) => Some(Value::Int(i)),
                        _ => None,
                    }
                }
                "math.round" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) => Some(Value::Int(f.round() as i64)),
                        Value::Int(i) => Some(Value::Int(i)),
                        _ => None,
                    }
                }
                "math.sqrt" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) if f >= 0.0 => Some(Value::Float(f.sqrt())),
                        Value::Int(i) if i >= 0 => Some(Value::Float((i as f64).sqrt())),
                        _ => None,
                    }
                }
                "math.pi" if args.is_empty() => Some(Value::Float(std::f64::consts::PI)),
                "math.e" if args.is_empty() => Some(Value::Float(std::f64::consts::E)),
                "math.hypot" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    let x = crate::stdlib::as_float(&a);
                    let y = crate::stdlib::as_float(&b);
                    Some(Value::Float(x.hypot(y)))
                }
                "math.atan2" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    let y = crate::stdlib::as_float(&a);
                    let x = crate::stdlib::as_float(&b);
                    Some(Value::Float(y.atan2(x)))
                }
                "math.deg" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).to_degrees()))
                }
                "math.rad" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).to_radians()))
                }
                "math.signum" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    match a {
                        Value::Int(i) => Some(Value::Int(if i > 0 { 1 } else if i < 0 { -1 } else { 0 })),
                        Value::Float(f) => Some(Value::Float(if f > 0.0 { 1.0 } else if f < 0.0 { -1.0 } else { 0.0 })),
                        _ => None,
                    }
                }
                "math.trunc" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).trunc()))
                }
                "math.fract" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).fract()))
                }
                "math.cbrt" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).cbrt()))
                }
                "math.gcd" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    Some(Value::Int(gcd_i64(crate::stdlib::as_int(&a), crate::stdlib::as_int(&b))))
                }
                "math.lcm" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    Some(Value::Int(lcm_i64(crate::stdlib::as_int(&a), crate::stdlib::as_int(&b))))
                }
                "math.pow" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    match (a, b) {
                        (Value::Int(base), Value::Int(exp)) if exp >= 0 => {
                            Some(Value::Int(base.wrapping_pow(exp as u32)))
                        }
                        (Value::Float(base), Value::Float(exp)) => Some(Value::Float(base.powf(exp))),
                        (Value::Int(base), Value::Float(exp)) => Some(Value::Float((base as f64).powf(exp))),
                        (Value::Float(base), Value::Int(exp)) => Some(Value::Float(base.powi(exp as i32))),
                        _ => None,
                    }
                }
                "math.sin" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) => Some(Value::Float(f.sin())),
                        Value::Int(i) => Some(Value::Float((i as f64).sin())),
                        _ => None,
                    }
                }
                "math.cos" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) => Some(Value::Float(f.cos())),
                        Value::Int(i) => Some(Value::Float((i as f64).cos())),
                        _ => None,
                    }
                }
                "math.tan" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) => Some(Value::Float(f.tan())),
                        Value::Int(i) => Some(Value::Float((i as f64).tan())),
                        _ => None,
                    }
                }
                "math.asin" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) if (-1.0..=1.0).contains(&f) => Some(Value::Float(f.asin())),
                        Value::Int(i) if (-1..=1).contains(&i) => Some(Value::Float((i as f64).asin())),
                        _ => None,
                    }
                }
                "math.acos" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) if (-1.0..=1.0).contains(&f) => Some(Value::Float(f.acos())),
                        Value::Int(i) if (-1..=1).contains(&i) => Some(Value::Float((i as f64).acos())),
                        _ => None,
                    }
                }
                "math.atan" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) => Some(Value::Float(f.atan())),
                        Value::Int(i) => Some(Value::Float((i as f64).atan())),
                        _ => None,
                    }
                }
                "math.log" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) if f > 0.0 => Some(Value::Float(f.ln())),
                        Value::Int(i) if i > 0 => Some(Value::Float((i as f64).ln())),
                        _ => None,
                    }
                }
                "math.log2" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) if f > 0.0 => Some(Value::Float(f.log2())),
                        Value::Int(i) if i > 0 => Some(Value::Float((i as f64).log2())),
                        _ => None,
                    }
                }
                "math.log10" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    match val {
                        Value::Float(f) if f > 0.0 => Some(Value::Float(f.log10())),
                        Value::Int(i) if i > 0 => Some(Value::Float((i as f64).log10())),
                        _ => None,
                    }
                }
                "string.len" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = val {
                        Some(Value::Int(s.chars().count() as i64))
                    } else {
                        None
                    }
                }
                "string.reverse" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = val {
                        Some(Value::String(s.chars().rev().collect()))
                    } else {
                        None
                    }
                }
                "array.len" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = val {
                        Some(Value::Int(arr.len() as i64))
                    } else {
                        None
                    }
                }
                "array.sort" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = val {
                        let mut sorted = arr;
                        sorted.sort_by(|x, y| x.compare_order(y));
                        Some(Value::Array(sorted))
                    } else {
                        None
                    }
                }
                "array.reverse" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = val {
                        let mut rev = arr;
                        rev.reverse();
                        Some(Value::Array(rev))
                    } else {
                        None
                    }
                }
                "math.is_nan" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Bool(crate::stdlib::as_float(&a).is_nan()))
                }
                "math.is_finite" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Bool(crate::stdlib::as_float(&a).is_finite()))
                }
                "math.factorial" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let n = crate::stdlib::as_int(&a);
                    if !(0..=20).contains(&n) {
                        None
                    } else {
                        let mut res: i64 = 1;
                        for i in 2..=n {
                            res = res.saturating_mul(i);
                        }
                        Some(Value::Int(res))
                    }
                }
                "math.round_to" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    let val = crate::stdlib::as_float(&a);
                    let dec = crate::stdlib::as_int(&b).clamp(0, 10) as u32;
                    let factor = 10_f64.powi(dec as i32);
                    Some(Value::Float((val * factor).round() / factor))
                }
                "math.lerp" if args.len() == 3 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    let t = try_fold_constant(&args[2].value, mode)?;
                    let a_f = crate::stdlib::as_float(&a);
                    let b_f = crate::stdlib::as_float(&b);
                    let t_f = crate::stdlib::as_float(&t);
                    Some(Value::Float(a_f + (b_f - a_f) * t_f))
                }
                "math.exp" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).exp()))
                }
                "math.cosh" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).cosh()))
                }
                "math.sinh" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).sinh()))
                }
                "math.tanh" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).tanh()))
                }
                "math.copysign" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    Some(Value::Float(crate::stdlib::as_float(&a).copysign(crate::stdlib::as_float(&b))))
                }
                "math.dist" if args.len() == 4 => {
                    let x1 = try_fold_constant(&args[0].value, mode)?;
                    let y1 = try_fold_constant(&args[1].value, mode)?;
                    let x2 = try_fold_constant(&args[2].value, mode)?;
                    let y2 = try_fold_constant(&args[3].value, mode)?;
                    let dx = crate::stdlib::as_float(&x2) - crate::stdlib::as_float(&x1);
                    let dy = crate::stdlib::as_float(&y2) - crate::stdlib::as_float(&y1);
                    Some(Value::Float(dx.hypot(dy)))
                }
                "math.is_even" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Bool(crate::stdlib::as_int(&a) % 2 == 0))
                }
                "math.is_odd" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Bool(crate::stdlib::as_int(&a) % 2 != 0))
                }
                "math.bitwise_and" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    Some(Value::Int(crate::stdlib::as_int(&a) & crate::stdlib::as_int(&b)))
                }
                "math.bitwise_or" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    Some(Value::Int(crate::stdlib::as_int(&a) | crate::stdlib::as_int(&b)))
                }
                "math.bitwise_xor" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    Some(Value::Int(crate::stdlib::as_int(&a) ^ crate::stdlib::as_int(&b)))
                }
                "math.bitwise_not" if args.len() == 1 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    Some(Value::Int(!crate::stdlib::as_int(&a)))
                }
                "math.bitwise_shl" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    let shift = (crate::stdlib::as_int(&b) & 63) as u32;
                    Some(Value::Int(crate::stdlib::as_int(&a).wrapping_shl(shift)))
                }
                "math.bitwise_shr" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    let shift = (crate::stdlib::as_int(&b) & 63) as u32;
                    Some(Value::Int(crate::stdlib::as_int(&a).wrapping_shr(shift)))
                }
                "string.count" if args.len() == 2 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let p_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s), Value::String(p)) = (s_val, p_val) {
                        if p.is_empty() {
                            Some(Value::Int(0))
                        } else {
                            Some(Value::Int(s.matches(&p).count() as i64))
                        }
                    } else {
                        None
                    }
                }
                "string.capitalize" if args.len() == 1 => {
                    let val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = val {
                        let mut chars = s.chars();
                        let res = match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                let mut cap = first.to_uppercase().to_string();
                                cap.push_str(chars.as_str());
                                cap
                            }
                        };
                        Some(Value::String(res))
                    } else {
                        None
                    }
                }
                "string.replace" if args.len() == 3 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let f_val = try_fold_constant(&args[1].value, mode)?;
                    let t_val = try_fold_constant(&args[2].value, mode)?;
                    if let (Value::String(s), Value::String(f), Value::String(t)) = (s_val, f_val, t_val) {
                        Some(Value::String(s.replace(&f, &t)))
                    } else {
                        None
                    }
                }
                "string.split" if args.len() == 2 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let d_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s), Value::String(d)) = (s_val, d_val) {
                        let parts = s.split(&d).map(|p| Value::String(p.to_string())).collect();
                        Some(Value::Array(parts))
                    } else {
                        None
                    }
                }
                "string.contains" if args.len() == 2 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let needle = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s), Value::String(n)) = (s_val, needle) {
                        Some(Value::Bool(s.contains(&n)))
                    } else {
                        None
                    }
                }
                "string.starts_with" if args.len() == 2 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let p_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s), Value::String(p)) = (s_val, p_val) {
                        Some(Value::Bool(s.starts_with(&p)))
                    } else {
                        None
                    }
                }
                "string.ends_with" if args.len() == 2 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let p_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s), Value::String(p)) = (s_val, p_val) {
                        Some(Value::Bool(s.ends_with(&p)))
                    } else {
                        None
                    }
                }
                "string.trim" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::String(s.trim().to_string()))
                    } else {
                        None
                    }
                }
                "string.trim_start" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::String(s.trim_start().to_string()))
                    } else {
                        None
                    }
                }
                "string.trim_end" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::String(s.trim_end().to_string()))
                    } else {
                        None
                    }
                }
                "string.lines" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        let lines = s.lines().map(|line| Value::String(line.to_string())).collect();
                        Some(Value::Array(lines))
                    } else {
                        None
                    }
                }
                "string.lower" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::String(s.to_lowercase()))
                    } else {
                        None
                    }
                }
                "string.upper" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::String(s.to_uppercase()))
                    } else {
                        None
                    }
                }
                "string.is_empty" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::Bool(s.is_empty()))
                    } else {
                        None
                    }
                }
                "string.repeat" if args.len() == 2 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let n_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s), Value::Int(n)) = (s_val, n_val) {
                        if n >= 0 && (n as usize) <= 100_000 {
                            Some(Value::String(s.repeat(n as usize)))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "string.index_of" if args.len() == 2 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let needle = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s), Value::String(n)) = (s_val, needle) {
                        Some(Value::Int(s.find(&n).map(|i| i as i64).unwrap_or(-1)))
                    } else {
                        None
                    }
                }
                "array.is_empty" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        Some(Value::Bool(arr.is_empty()))
                    } else {
                        None
                    }
                }
                "array.contains" if args.len() == 2 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    let target = try_fold_constant(&args[1].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        Some(Value::Bool(arr.contains(&target)))
                    } else {
                        None
                    }
                }
                "array.first" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        arr.first().cloned()
                    } else {
                        None
                    }
                }
                "array.last" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        arr.last().cloned()
                    } else {
                        None
                    }
                }
                "array.concat" if args.len() == 2 => {
                    let a_val = try_fold_constant(&args[0].value, mode)?;
                    let b_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Array(mut a), Value::Array(b)) = (a_val, b_val) {
                        a.extend(b);
                        Some(Value::Array(a))
                    } else {
                        None
                    }
                }
                "array.join" if args.len() == 2 => {
                    let a_val = try_fold_constant(&args[0].value, mode)?;
                    let d_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Array(arr), Value::String(sep)) = (a_val, d_val) {
                        let strings: Vec<String> = arr.iter().map(crate::stdlib::value_to_string).collect();
                        Some(Value::String(strings.join(&sep)))
                    } else {
                        None
                    }
                }
                "array.swap" if args.len() == 3 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    let i_val = try_fold_constant(&args[1].value, mode)?;
                    let j_val = try_fold_constant(&args[2].value, mode)?;
                    if let (Value::Array(mut arr), Value::Int(i), Value::Int(j)) = (arr_val, i_val, j_val) {
                        let len = arr.len() as i64;
                        if i >= 0 && i < len && j >= 0 && j < len {
                            arr.swap(i as usize, j as usize);
                            Some(Value::Array(arr))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "array.count" if args.len() == 2 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    let target = try_fold_constant(&args[1].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        let cnt = arr.iter().filter(|v| **v == target).count() as i64;
                        Some(Value::Int(cnt))
                    } else {
                        None
                    }
                }
                "array.take" if args.len() == 2 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    let n_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Array(arr), Value::Int(n)) = (arr_val, n_val) {
                        let count = n.max(0) as usize;
                        Some(Value::Array(arr.into_iter().take(count).collect()))
                    } else {
                        None
                    }
                }
                "array.drop" if args.len() == 2 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    let n_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Array(arr), Value::Int(n)) = (arr_val, n_val) {
                        let count = n.max(0) as usize;
                        Some(Value::Array(arr.into_iter().skip(count).collect()))
                    } else {
                        None
                    }
                }
                "string.slice" if args.len() == 3 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let start = try_fold_constant(&args[1].value, mode)?;
                    let end = try_fold_constant(&args[2].value, mode)?;
                    if let (Value::String(s), Value::Int(start_idx), Value::Int(end_idx)) = (s_val, start, end) {
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len() as i64;
                        let clamped_start = start_idx.max(0).min(len) as usize;
                        let clamped_end = end_idx.max(clamped_start as i64).min(len) as usize;
                        Some(Value::String(chars[clamped_start..clamped_end].iter().collect()))
                    } else {
                        None
                    }
                }
                "string.replace" if args.len() == 3 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let from_val = try_fold_constant(&args[1].value, mode)?;
                    let to_val = try_fold_constant(&args[2].value, mode)?;
                    if let (Value::String(s), Value::String(from), Value::String(to)) = (s_val, from_val, to_val) {
                        Some(Value::String(s.replace(&from, &to)))
                    } else {
                        None
                    }
                }
                "array.slice" if args.len() == 3 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    let start = try_fold_constant(&args[1].value, mode)?;
                    let end = try_fold_constant(&args[2].value, mode)?;
                    if let (Value::Array(arr), Value::Int(start_idx), Value::Int(end_idx)) = (arr_val, start, end) {
                        let len = arr.len() as i64;
                        let clamped_start = start_idx.max(0).min(len) as usize;
                        let clamped_end = end_idx.max(clamped_start as i64).min(len) as usize;
                        Some(Value::Array(arr[clamped_start..clamped_end].to_vec()))
                    } else {
                        None
                    }
                }
                "array.repeat" if args.len() == 2 => {
                    let elem = try_fold_constant(&args[0].value, mode)?;
                    let count_val = try_fold_constant(&args[1].value, mode)?;
                    if let Value::Int(count) = count_val {
                        if (0..=1000).contains(&count) {
                            Some(Value::Array(vec![elem; count as usize]))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "array.fill" if args.len() == 2 => {
                    let a = try_fold_constant(&args[0].value, mode)?;
                    let b = try_fold_constant(&args[1].value, mode)?;
                    let (count, value) = match (&a, &b) {
                        (Value::Int(c), val) => (*c, val.clone()),
                        (val, Value::Int(c)) => (*c, val.clone()),
                        _ => return None,
                    };
                    if (0..=1000).contains(&count) {
                        Some(Value::Array(vec![value; count as usize]))
                    } else {
                        None
                    }
                }
                "hash.fnv1a" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        let mut hash: u64 = 0xcbf29ce484222325;
                        for byte in s.as_bytes() {
                            hash ^= *byte as u64;
                            hash = hash.wrapping_mul(0x100000001b3);
                        }
                        Some(Value::Int(hash as i64))
                    } else {
                        None
                    }
                }
                "hash.djb2" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        let mut hash: u64 = 5381;
                        for byte in s.as_bytes() {
                            hash = hash.wrapping_mul(33).wrapping_add(*byte as u64);
                        }
                        Some(Value::Int(hash as i64))
                    } else {
                        None
                    }
                }
                "hash.adler32" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        let mut a: u32 = 1;
                        let mut b: u32 = 0;
                        for byte in s.as_bytes() {
                            a = (a + *byte as u32) % 65521;
                            b = (b + a) % 65521;
                        }
                        Some(Value::Int(((b << 16) | a) as i64))
                    } else {
                        None
                    }
                }
                "hash.crc32" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        let mut crc = 0xffffffffu32;
                        for byte in s.as_bytes() {
                            crc ^= *byte as u32;
                            for _ in 0..8 {
                                if (crc & 1) != 0 {
                                    crc = (crc >> 1) ^ 0xedb88320;
                                } else {
                                    crc >>= 1;
                                }
                            }
                        }
                        Some(Value::Int((!crc) as i64))
                    } else {
                        None
                    }
                }
                "hash.murmur3" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::Int(crate::stdlib::murmur3_32(s.as_bytes(), 0) as i64))
                    } else {
                        None
                    }
                }
                "hash.xxhash32" if !args.is_empty() => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let seed = if args.len() > 1 {
                        if let Some(Value::Int(sd)) = try_fold_constant(&args[1].value, mode) {
                            sd as u32
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    if let Value::String(s) = s_val {
                        Some(Value::Int(crate::stdlib::xxhash32(s.as_bytes(), seed) as i64))
                    } else {
                        None
                    }
                }
                "hash.rot13" | "string.rot13" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        let rotated: String = s.chars().map(|c| match c {
                            'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
                            'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
                            _ => c,
                        }).collect();
                        Some(Value::String(rotated))
                    } else {
                        None
                    }
                }
                "math.is_prime" if args.len() == 1 => {
                    let n_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(n) = n_val {
                        if n <= 1 {
                            Some(Value::Bool(false))
                        } else if n <= 3 {
                            Some(Value::Bool(true))
                        } else if n % 2 == 0 || n % 3 == 0 {
                            Some(Value::Bool(false))
                        } else {
                            let mut i = 5i64;
                            let mut is_p = true;
                            while i * i <= n {
                                if n % i == 0 || n % (i + 2) == 0 {
                                    is_p = false;
                                    break;
                                }
                                i += 6;
                            }
                            Some(Value::Bool(is_p))
                        }
                    } else {
                        None
                    }
                }
                "math.count_ones" if args.len() == 1 => {
                    let n_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(n) = n_val {
                        Some(Value::Int(n.count_ones() as i64))
                    } else {
                        None
                    }
                }
                "math.is_power_of_two" if args.len() == 1 => {
                    let n_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(n) = n_val {
                        Some(Value::Bool(n > 0 && (n & (n - 1)) == 0))
                    } else {
                        None
                    }
                }
                "math.next_power_of_two" if args.len() == 1 => {
                    let n_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(n) = n_val {
                        if n <= 1 {
                            Some(Value::Int(1))
                        } else {
                            Some(Value::Int((n as u64).next_power_of_two() as i64))
                        }
                    } else {
                        None
                    }
                }
                "math.clamp01" if args.len() == 1 => {
                    let n_val = try_fold_constant(&args[0].value, mode)?;
                    match n_val {
                        Value::Int(i) => Some(Value::Int(i.clamp(0, 1))),
                        Value::Float(f) => Some(Value::Float(crate::stdlib::clamp01_f64(f))),
                        _ => None,
                    }
                }
                "math.midpoint" if args.len() == 2 => {
                    let a_val = try_fold_constant(&args[0].value, mode)?;
                    let b_val = try_fold_constant(&args[1].value, mode)?;
                    match (a_val, b_val) {
                        (Value::Int(a), Value::Int(b)) => Some(Value::Int(crate::stdlib::midpoint_i64(a, b))),
                        (Value::Float(a), Value::Float(b)) => Some(Value::Float((a + b) / 2.0)),
                        _ => None,
                    }
                }
                "string.levenshtein" if args.len() == 2 => {
                    let s1_val = try_fold_constant(&args[0].value, mode)?;
                    let s2_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s1), Value::String(s2)) = (s1_val, s2_val) {
                        Some(Value::Int(crate::stdlib::levenshtein_distance(&s1, &s2) as i64))
                    } else {
                        None
                    }
                }
                "string.hamming" if args.len() == 2 => {
                    let s1_val = try_fold_constant(&args[0].value, mode)?;
                    let s2_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::String(s1), Value::String(s2)) = (s1_val, s2_val) {
                        Some(Value::Int(crate::stdlib::hamming_distance(&s1, &s2)))
                    } else {
                        None
                    }
                }
                "string.slugify" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::String(crate::stdlib::slugify(&s)))
                    } else {
                        None
                    }
                }
                "string.truncate" if args.len() >= 2 && args.len() <= 3 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    let len_val = try_fold_constant(&args[1].value, mode)?;
                    let ellipsis = if args.len() == 3 {
                        match try_fold_constant(&args[2].value, mode)? {
                            Value::String(e) => e,
                            _ => return None,
                        }
                    } else {
                        "...".to_string()
                    };
                    if let (Value::String(s), Value::Int(len)) = (s_val, len_val) {
                        Some(Value::String(crate::stdlib::truncate_string(&s, len.max(0) as usize, &ellipsis)))
                    } else {
                        None
                    }
                }
                "hash.hex_encode" | "string.hex_encode" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::String(crate::stdlib::hex_encode(&s)))
                    } else {
                        None
                    }
                }
                "hash.hex_decode" | "string.hex_decode" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::String(crate::stdlib::hex_decode(&s)))
                    } else {
                        None
                    }
                }
                "array.sum" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        let mut total_int = 0i64;
                        let mut total_float = 0.0f64;
                        let mut is_float = false;
                        for v in arr {
                            match v {
                                Value::Int(i) => {
                                    total_int = total_int.wrapping_add(i);
                                    total_float += i as f64;
                                }
                                Value::Float(f) => {
                                    is_float = true;
                                    total_float += f;
                                }
                                _ => return None,
                            }
                        }
                        if is_float {
                            Some(Value::Float(total_float))
                        } else {
                            Some(Value::Int(total_int))
                        }
                    } else {
                        None
                    }
                }
                "array.product" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        let mut total_int = 1i64;
                        let mut total_float = 1.0f64;
                        let mut is_float = false;
                        for v in arr {
                            match v {
                                Value::Int(i) => {
                                    total_int = total_int.wrapping_mul(i);
                                    total_float *= i as f64;
                                }
                                Value::Float(f) => {
                                    is_float = true;
                                    total_float *= f;
                                }
                                _ => return None,
                            }
                        }
                        if is_float {
                            Some(Value::Float(total_float))
                        } else {
                            Some(Value::Int(total_int))
                        }
                    } else {
                        None
                    }
                }
                "array.dedup" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        let mut deduped: Vec<Value> = Vec::new();
                        for v in arr {
                            if deduped.last() != Some(&v) {
                                deduped.push(v);
                            }
                        }
                        Some(Value::Array(deduped))
                    } else {
                        None
                    }
                }
                "array.reverse" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(mut arr) = arr_val {
                        arr.reverse();
                        Some(Value::Array(arr))
                    } else {
                        None
                    }
                }
                "math.is_square" if args.len() == 1 => {
                    let n_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Int(n) = n_val {
                        Some(Value::Bool(crate::stdlib::is_integer_square(n)))
                    } else {
                        None
                    }
                }
                "string.lines_count" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::Int(crate::stdlib::string_lines_count(&s)))
                    } else {
                        None
                    }
                }
                "string.words_count" if args.len() == 1 => {
                    let s_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::String(s) = s_val {
                        Some(Value::Int(crate::stdlib::string_words_count(&s)))
                    } else {
                        None
                    }
                }
                "array.compact" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        let compacted: Vec<Value> = arr.iter().filter(|v| !matches!(v, Value::Unit)).cloned().collect();
                        Some(Value::Array(compacted))
                    } else {
                        None
                    }
                }
                "array.diff" if args.len() == 2 => {
                    let a_val = try_fold_constant(&args[0].value, mode)?;
                    let b_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Array(a), Value::Array(b)) = (a_val, b_val) {
                        let diff: Vec<Value> = a.iter().filter(|v| !b.contains(v)).cloned().collect();
                        Some(Value::Array(diff))
                    } else {
                        None
                    }
                }
                "array.intersect" if args.len() == 2 => {
                    let a_val = try_fold_constant(&args[0].value, mode)?;
                    let b_val = try_fold_constant(&args[1].value, mode)?;
                    if let (Value::Array(a), Value::Array(b)) = (a_val, b_val) {
                        let mut res: Vec<Value> = Vec::new();
                        for v in a {
                            if b.contains(&v) && !res.contains(&v) {
                                res.push(v);
                            }
                        }
                        Some(Value::Array(res))
                    } else {
                        None
                    }
                }
                "array.is_sorted" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        Some(Value::Bool(crate::stdlib::is_array_sorted(&arr)))
                    } else {
                        None
                    }
                }
                "array.all_unique" if args.len() == 1 => {
                    let arr_val = try_fold_constant(&args[0].value, mode)?;
                    if let Value::Array(arr) = arr_val {
                        Some(Value::Bool(crate::stdlib::are_array_elements_unique(&arr)))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn fold_binary_values(
    left: &Value,
    op: &crate::ast::BinaryOperator,
    right: &Value,
    mode: crate::types::HybridMode,
) -> Option<Value> {
    match (left, op, right) {
        (Value::Int(a), crate::ast::BinaryOperator::Add, Value::Int(b)) => Some(Value::Int(a.wrapping_add(*b))),
        (Value::Int(a), crate::ast::BinaryOperator::Subtract, Value::Int(b)) => Some(Value::Int(a.wrapping_sub(*b))),
        (Value::Int(a), crate::ast::BinaryOperator::Multiply, Value::Int(b)) => Some(Value::Int(a.wrapping_mul(*b))),
        (Value::Int(a), crate::ast::BinaryOperator::Divide, Value::Int(b)) => {
            if *b == 0 {
                None // Leave to runtime division-by-zero error
            } else if mode == crate::types::HybridMode::HighLevel {
                Some(Value::Float(*a as f64 / *b as f64))
            } else {
                Some(Value::Int(a.wrapping_div(*b)))
            }
        }
        (Value::Int(a), crate::ast::BinaryOperator::Modulo, Value::Int(b)) => {
            if *b == 0 {
                None
            } else {
                Some(Value::Int(a.wrapping_rem(*b)))
            }
        }
        (Value::Int(a), crate::ast::BinaryOperator::Equal, Value::Int(b)) => Some(Value::Bool(a == b)),
        (Value::Int(a), crate::ast::BinaryOperator::NotEqual, Value::Int(b)) => Some(Value::Bool(a != b)),
        (Value::Int(a), crate::ast::BinaryOperator::Less, Value::Int(b)) => Some(Value::Bool(a < b)),
        (Value::Int(a), crate::ast::BinaryOperator::LessEqual, Value::Int(b)) => Some(Value::Bool(a <= b)),
        (Value::Int(a), crate::ast::BinaryOperator::Greater, Value::Int(b)) => Some(Value::Bool(a > b)),
        (Value::Int(a), crate::ast::BinaryOperator::GreaterEqual, Value::Int(b)) => Some(Value::Bool(a >= b)),

        // Mixed Int and Float
        (Value::Int(a), crate::ast::BinaryOperator::Add, Value::Float(b)) => Some(Value::Float(*a as f64 + b)),
        (Value::Float(a), crate::ast::BinaryOperator::Add, Value::Int(b)) => Some(Value::Float(a + *b as f64)),
        (Value::Int(a), crate::ast::BinaryOperator::Subtract, Value::Float(b)) => Some(Value::Float(*a as f64 - b)),
        (Value::Float(a), crate::ast::BinaryOperator::Subtract, Value::Int(b)) => Some(Value::Float(a - *b as f64)),
        (Value::Int(a), crate::ast::BinaryOperator::Multiply, Value::Float(b)) => Some(Value::Float(*a as f64 * b)),
        (Value::Float(a), crate::ast::BinaryOperator::Multiply, Value::Int(b)) => Some(Value::Float(a * *b as f64)),
        (Value::Int(a), crate::ast::BinaryOperator::Divide, Value::Float(b)) => {
            if *b == 0.0 { None } else { Some(Value::Float(*a as f64 / b)) }
        }
        (Value::Float(a), crate::ast::BinaryOperator::Divide, Value::Int(b)) => {
            if *b == 0 { None } else { Some(Value::Float(a / *b as f64)) }
        }
        (Value::Int(a), crate::ast::BinaryOperator::Equal, Value::Float(b)) => Some(Value::Bool((*a as f64) == *b)),
        (Value::Float(a), crate::ast::BinaryOperator::Equal, Value::Int(b)) => Some(Value::Bool(*a == (*b as f64))),
        (Value::Int(a), crate::ast::BinaryOperator::NotEqual, Value::Float(b)) => Some(Value::Bool((*a as f64) != *b)),
        (Value::Float(a), crate::ast::BinaryOperator::NotEqual, Value::Int(b)) => Some(Value::Bool(*a != (*b as f64))),
        (Value::Int(a), crate::ast::BinaryOperator::Less, Value::Float(b)) => Some(Value::Bool((*a as f64) < *b)),
        (Value::Float(a), crate::ast::BinaryOperator::Less, Value::Int(b)) => Some(Value::Bool(*a < (*b as f64))),
        (Value::Int(a), crate::ast::BinaryOperator::LessEqual, Value::Float(b)) => Some(Value::Bool((*a as f64) <= *b)),
        (Value::Float(a), crate::ast::BinaryOperator::LessEqual, Value::Int(b)) => Some(Value::Bool(*a <= (*b as f64))),
        (Value::Int(a), crate::ast::BinaryOperator::Greater, Value::Float(b)) => Some(Value::Bool((*a as f64) > *b)),
        (Value::Float(a), crate::ast::BinaryOperator::Greater, Value::Int(b)) => Some(Value::Bool(*a > (*b as f64))),
        (Value::Int(a), crate::ast::BinaryOperator::GreaterEqual, Value::Float(b)) => Some(Value::Bool((*a as f64) >= *b)),
        (Value::Float(a), crate::ast::BinaryOperator::GreaterEqual, Value::Int(b)) => Some(Value::Bool(*a >= (*b as f64))),

        (Value::Float(a), crate::ast::BinaryOperator::Add, Value::Float(b)) => Some(Value::Float(a + b)),
        (Value::Float(a), crate::ast::BinaryOperator::Subtract, Value::Float(b)) => Some(Value::Float(a - b)),
        (Value::Float(a), crate::ast::BinaryOperator::Multiply, Value::Float(b)) => Some(Value::Float(a * b)),
        (Value::Float(a), crate::ast::BinaryOperator::Divide, Value::Float(b)) => {
            if *b == 0.0 {
                None
            } else {
                Some(Value::Float(a / b))
            }
        }
        (Value::Float(a), crate::ast::BinaryOperator::Equal, Value::Float(b)) => Some(Value::Bool(a == b)),
        (Value::Float(a), crate::ast::BinaryOperator::NotEqual, Value::Float(b)) => Some(Value::Bool(a != b)),
        (Value::Float(a), crate::ast::BinaryOperator::Less, Value::Float(b)) => Some(Value::Bool(a < b)),
        (Value::Float(a), crate::ast::BinaryOperator::LessEqual, Value::Float(b)) => Some(Value::Bool(a <= b)),
        (Value::Float(a), crate::ast::BinaryOperator::Greater, Value::Float(b)) => Some(Value::Bool(a > b)),
        (Value::Float(a), crate::ast::BinaryOperator::GreaterEqual, Value::Float(b)) => Some(Value::Bool(a >= b)),

        (Value::String(a), crate::ast::BinaryOperator::Add, right) => {
            Some(Value::String(format!("{}{}", a, crate::stdlib::value_to_string(right))))
        }
        (Value::Int(a), crate::ast::BinaryOperator::Add, Value::String(b)) => {
            Some(Value::String(format!("{}{}", a, b)))
        }
        (Value::Float(a), crate::ast::BinaryOperator::Add, Value::String(b)) => {
            Some(Value::String(format!("{}{}", a, b)))
        }
        (Value::Bool(a), crate::ast::BinaryOperator::Add, Value::String(b)) => {
            Some(Value::String(format!("{}{}", a, b)))
        }
        (Value::String(a), crate::ast::BinaryOperator::Equal, Value::String(b)) => Some(Value::Bool(a == b)),
        (Value::String(a), crate::ast::BinaryOperator::NotEqual, Value::String(b)) => Some(Value::Bool(a != b)),
        (Value::String(a), crate::ast::BinaryOperator::Less, Value::String(b)) => Some(Value::Bool(a < b)),
        (Value::String(a), crate::ast::BinaryOperator::LessEqual, Value::String(b)) => Some(Value::Bool(a <= b)),
        (Value::String(a), crate::ast::BinaryOperator::Greater, Value::String(b)) => Some(Value::Bool(a > b)),
        (Value::String(a), crate::ast::BinaryOperator::GreaterEqual, Value::String(b)) => Some(Value::Bool(a >= b)),

        (Value::Bool(a), crate::ast::BinaryOperator::Equal, Value::Bool(b)) => Some(Value::Bool(a == b)),
        (Value::Bool(a), crate::ast::BinaryOperator::NotEqual, Value::Bool(b)) => Some(Value::Bool(a != b)),
        _ => None,
    }
}

pub fn optimize_chunk(chunk: &mut Chunk) {
    if chunk.code.is_empty() {
        return;
    }
    for _ in 0..8 {
        let changed = run_optimization_pass(chunk);
        if !changed {
            break;
        }
    }
}

fn run_optimization_pass(chunk: &mut Chunk) -> bool {
    let mut changed = false;

    // Pass 1: Jump threading and Jump-to-Return forwarding
    for i in 0..chunk.code.len() {
        match chunk.code[i] {
            Opcode::Jump(mut target) => {
                let mut visited = std::collections::HashSet::new();
                visited.insert(target);
                while target < chunk.code.len() {
                    if let Opcode::Jump(next) = chunk.code[target] {
                        if visited.insert(next) {
                            target = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if target < chunk.code.len() && chunk.code[target] == Opcode::Return {
                    chunk.code[i] = Opcode::Return;
                    changed = true;
                } else if Opcode::Jump(target) != chunk.code[i] {
                    chunk.code[i] = Opcode::Jump(target);
                    changed = true;
                }
            }
            Opcode::JumpIfFalse(mut target) => {
                let mut visited = std::collections::HashSet::new();
                visited.insert(target);
                while target < chunk.code.len() {
                    if let Opcode::Jump(next) = chunk.code[target] {
                        if visited.insert(next) {
                            target = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if target == i + 1 {
                    chunk.code[i] = Opcode::Pop;
                    changed = true;
                } else if Opcode::JumpIfFalse(target) != chunk.code[i] {
                    chunk.code[i] = Opcode::JumpIfFalse(target);
                    changed = true;
                }
            }
            Opcode::JumpIfTrue(mut target) => {
                let mut visited = std::collections::HashSet::new();
                visited.insert(target);
                while target < chunk.code.len() {
                    if let Opcode::Jump(next) = chunk.code[target] {
                        if visited.insert(next) {
                            target = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if target == i + 1 {
                    chunk.code[i] = Opcode::Pop;
                    changed = true;
                } else if Opcode::JumpIfTrue(target) != chunk.code[i] {
                    chunk.code[i] = Opcode::JumpIfTrue(target);
                    changed = true;
                }
            }
            Opcode::Loop(mut target) => {
                let mut visited = std::collections::HashSet::new();
                visited.insert(target);
                while target < chunk.code.len() {
                    if let Opcode::Jump(next) = chunk.code[target] {
                        if visited.insert(next) {
                            target = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if Opcode::Loop(target) != chunk.code[i] {
                    chunk.code[i] = Opcode::Loop(target);
                    changed = true;
                }
            }
            _ => {}
        }
    }

    // Pass 2: Unreachable instruction elimination (Dead Code Elimination)
    let code_len = chunk.code.len();
    if code_len > 0 {
        let mut reachable = vec![false; code_len];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(0);

        while let Some(ip) = queue.pop_front() {
            if ip >= code_len || reachable[ip] {
                continue;
            }
            reachable[ip] = true;
            match &chunk.code[ip] {
                Opcode::Return | Opcode::Raise(_) => {}
                Opcode::Jump(target) | Opcode::Loop(target) => {
                    if *target < code_len {
                        queue.push_back(*target);
                    }
                }
                Opcode::JumpIfFalse(target) | Opcode::JumpIfTrue(target) => {
                    if *target < code_len {
                        queue.push_back(*target);
                    }
                    if ip + 1 < code_len {
                        queue.push_back(ip + 1);
                    }
                }
                _ => {
                    if ip + 1 < code_len {
                        queue.push_back(ip + 1);
                    }
                }
            }
        }

        if !reachable.iter().all(|&r| r) {
            let mut remap = vec![0; code_len + 1];
            let mut new_idx = 0;
            for (old_idx, &is_reach) in reachable.iter().enumerate() {
                remap[old_idx] = new_idx;
                if is_reach {
                    new_idx += 1;
                }
            }
            remap[code_len] = new_idx;

            let mut new_code = Vec::with_capacity(new_idx);
            let mut new_locs = Vec::with_capacity(new_idx);
            for (i, &is_reach) in reachable.iter().enumerate().take(code_len) {
                if is_reach {
                    let op = match &chunk.code[i] {
                        Opcode::Jump(t) => Opcode::Jump(remap[*t.min(&code_len)]),
                        Opcode::JumpIfFalse(t) => Opcode::JumpIfFalse(remap[*t.min(&code_len)]),
                        Opcode::JumpIfTrue(t) => Opcode::JumpIfTrue(remap[*t.min(&code_len)]),
                        Opcode::Loop(t) => Opcode::Loop(remap[*t.min(&code_len)]),
                        other => other.clone(),
                    };
                    new_code.push(op);
                    new_locs.push(chunk.locations[i]);
                }
            }
            chunk.code = new_code;
            chunk.locations = new_locs;
            changed = true;
        }
    }

    // Pass 3: Consecutive redundant operation pruning
    if chunk.code.len() >= 2 {
        let mut jump_targets = std::collections::HashSet::new();
        for op in &chunk.code {
            match op {
                Opcode::Jump(t) | Opcode::JumpIfFalse(t) | Opcode::JumpIfTrue(t) | Opcode::Loop(t) => {
                    jump_targets.insert(*t);
                }
                _ => {}
            }
        }

        let mut keep = vec![true; chunk.code.len()];
        let mut pair_removed = false;
        let mut i = 0;
        while i < chunk.code.len() {
            // Elimination of redundant jump to immediate next instruction
            if !jump_targets.contains(&i) {
                if let Opcode::Jump(target) = chunk.code[i] {
                    if target == i + 1 {
                        keep[i] = false;
                        pair_removed = true;
                        i += 1;
                        continue;
                    }
                }
            }

            // Store forwarding: [StoreLocal(a), Pop, LoadLocal(a)] => [StoreLocal(a)]
            if i + 2 < chunk.code.len() && !jump_targets.contains(&(i + 1)) && !jump_targets.contains(&(i + 2)) {
                if let (Opcode::StoreLocal(a), Opcode::Pop, Opcode::LoadLocal(b)) = (&chunk.code[i], &chunk.code[i + 1], &chunk.code[i + 2]) {
                    if *a == *b {
                        keep[i + 1] = false;
                        keep[i + 2] = false;
                        pair_removed = true;
                        changed = true;
                        i += 3;
                        continue;
                    }
                }
                if let (Opcode::StoreGlobal(a), Opcode::Pop, Opcode::LoadGlobal(b)) = (&chunk.code[i], &chunk.code[i + 1], &chunk.code[i + 2]) {
                    if *a == *b {
                        keep[i + 1] = false;
                        keep[i + 2] = false;
                        pair_removed = true;
                        changed = true;
                        i += 3;
                        continue;
                    }
                }
            }

            // Triple: LoadLocal(a), StoreLocal(a), Pop => all redundant
            if i + 2 < chunk.code.len() && !jump_targets.contains(&i) && !jump_targets.contains(&(i + 1)) && !jump_targets.contains(&(i + 2)) {
                if let (Opcode::LoadLocal(a), Opcode::StoreLocal(b), Opcode::Pop) = (&chunk.code[i], &chunk.code[i + 1], &chunk.code[i + 2]) {
                    if *a == *b {
                        keep[i] = false;
                        keep[i + 1] = false;
                        keep[i + 2] = false;
                        pair_removed = true;
                        i += 3;
                        continue;
                    }
                }
            }

            // StoreLocal(a) followed by LoadLocal(a) => Dup, StoreLocal(a)
            if i + 1 < chunk.code.len() && !jump_targets.contains(&i) && !jump_targets.contains(&(i + 1)) {
                let opt_pair = match (&chunk.code[i], &chunk.code[i + 1]) {
                    (Opcode::StoreLocal(a), Opcode::LoadLocal(b)) if a == b => Some(*a),
                    _ => None,
                };
                if let Some(local_idx) = opt_pair {
                    chunk.code[i] = Opcode::Dup;
                    chunk.code[i + 1] = Opcode::StoreLocal(local_idx);
                    changed = true;
                    i += 2;
                    continue;
                }
            }

            // Constant accumulation for Add: [LoadConst(a), Add, LoadConst(b), Add] => [LoadConst(a + b), Add]
            if i + 3 < chunk.code.len()
                && !jump_targets.contains(&(i + 1))
                && !jump_targets.contains(&(i + 2))
                && !jump_targets.contains(&(i + 3))
            {
                if let (Opcode::LoadConst(idx_a), Opcode::Add, Opcode::LoadConst(idx_b), Opcode::Add) =
                    (&chunk.code[i], &chunk.code[i + 1], &chunk.code[i + 2], &chunk.code[i + 3])
                {
                    if let (Some(val_a), Some(val_b)) = (chunk.constants.get(*idx_a), chunk.constants.get(*idx_b)) {
                        let combined = match (val_a, val_b) {
                            (crate::value::Value::Int(a), crate::value::Value::Int(b)) => a.checked_add(*b).map(crate::value::Value::Int),
                            (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Float(a + b)),
                            _ => None,
                        };
                        if let Some(c) = combined {
                            let new_idx = chunk.add_constant(c);
                            chunk.code[i] = Opcode::LoadConst(new_idx);
                            chunk.code[i + 1] = Opcode::Add;
                            keep[i + 2] = false;
                            keep[i + 3] = false;
                            pair_removed = true;
                            changed = true;
                            i += 4;
                            continue;
                        }
                    }
                }
                // Constant accumulation for Mul: [LoadConst(a), Mul, LoadConst(b), Mul] => [LoadConst(a * b), Mul]
                if let (Opcode::LoadConst(idx_a), Opcode::Mul, Opcode::LoadConst(idx_b), Opcode::Mul) =
                    (&chunk.code[i], &chunk.code[i + 1], &chunk.code[i + 2], &chunk.code[i + 3])
                {
                    if let (Some(val_a), Some(val_b)) = (chunk.constants.get(*idx_a), chunk.constants.get(*idx_b)) {
                        let combined = match (val_a, val_b) {
                            (crate::value::Value::Int(a), crate::value::Value::Int(b)) => a.checked_mul(*b).map(crate::value::Value::Int),
                            (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Float(a * b)),
                            _ => None,
                        };
                        if let Some(c) = combined {
                            let new_idx = chunk.add_constant(c);
                            chunk.code[i] = Opcode::LoadConst(new_idx);
                            chunk.code[i + 1] = Opcode::Mul;
                            keep[i + 2] = false;
                            keep[i + 3] = false;
                            pair_removed = true;
                            changed = true;
                            i += 4;
                            continue;
                        }
                    }
                }
            }

            // Constant boolean branch folding:
            // [LoadConst(false), JumpIfFalse(t)] => [LoadConst(false), Jump(t)]
            // [LoadConst(true), JumpIfTrue(t)] => [LoadConst(true), Jump(t)]
            if i + 1 < chunk.code.len() && !jump_targets.contains(&(i + 1)) {
                if let (Opcode::LoadConst(idx), Opcode::JumpIfFalse(target)) = (&chunk.code[i], &chunk.code[i + 1]) {
                    if matches!(chunk.constants.get(*idx), Some(crate::value::Value::Bool(false))) {
                        let t = *target;
                        chunk.code[i + 1] = Opcode::Jump(t);
                        changed = true;
                        i += 2;
                        continue;
                    }
                }
                if let (Opcode::LoadConst(idx), Opcode::JumpIfTrue(target)) = (&chunk.code[i], &chunk.code[i + 1]) {
                    if matches!(chunk.constants.get(*idx), Some(crate::value::Value::Bool(true))) {
                        let t = *target;
                        chunk.code[i + 1] = Opcode::Jump(t);
                        changed = true;
                        i += 2;
                        continue;
                    }
                }
            }

            // Identity arithmetic: [LoadConst(0), Add] or [LoadConst(0), Sub] or [LoadConst(1), Mul/Div]
            if i + 1 < chunk.code.len() && !jump_targets.contains(&i) && !jump_targets.contains(&(i + 1)) {
                let is_identity_arith = match (&chunk.code[i], &chunk.code[i + 1]) {
                    (Opcode::LoadConst(idx), Opcode::Add) | (Opcode::LoadConst(idx), Opcode::Sub) => {
                        match chunk.constants.get(*idx) {
                            Some(crate::value::Value::Int(0)) => true,
                            Some(crate::value::Value::Float(f)) if *f == 0.0 => true,
                            _ => false,
                        }
                    }
                    (Opcode::LoadConst(idx), Opcode::Mul)
                    | (Opcode::LoadConst(idx), Opcode::Div)
                    | (Opcode::LoadConst(idx), Opcode::HighLevelDiv)
                    | (Opcode::LoadConst(idx), Opcode::Pow) => {
                        match chunk.constants.get(*idx) {
                            Some(crate::value::Value::Int(1)) => true,
                            Some(crate::value::Value::Float(f)) if *f == 1.0 => true,
                            _ => false,
                        }
                    }
                    (Opcode::LoadConst(idx), Opcode::And) => {
                        matches!(chunk.constants.get(*idx), Some(crate::value::Value::Bool(true)))
                    }
                    (Opcode::LoadConst(idx), Opcode::Or) => {
                        matches!(chunk.constants.get(*idx), Some(crate::value::Value::Bool(false)))
                    }
                    _ => false,
                };
                if is_identity_arith {
                    keep[i] = false;
                    keep[i + 1] = false;
                    pair_removed = true;
                    i += 2;
                    continue;
                }
            }

            // Three-instruction constant binary fold: [LoadConst(a), LoadConst(b), BinOp] => [LoadConst(folded)]
            if i + 2 < chunk.code.len()
                && !jump_targets.contains(&i)
                && !jump_targets.contains(&(i + 1))
                && !jump_targets.contains(&(i + 2))
            {
                if let (Opcode::LoadConst(idx_a), Opcode::LoadConst(idx_b)) = (&chunk.code[i], &chunk.code[i + 1]) {
                    if let (Some(val_a), Some(val_b)) = (chunk.constants.get(*idx_a), chunk.constants.get(*idx_b)) {
                        let folded = match &chunk.code[i + 2] {
                            Opcode::Add => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) => a.checked_add(*b).map(crate::value::Value::Int),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Float(a + b)),
                                (crate::value::Value::String(a), crate::value::Value::String(b)) => Some(crate::value::Value::String(format!("{}{}", a, b))),
                                _ => None,
                            },
                            Opcode::Sub => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) => a.checked_sub(*b).map(crate::value::Value::Int),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Float(a - b)),
                                _ => None,
                            },
                            Opcode::Mul => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) => a.checked_mul(*b).map(crate::value::Value::Int),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Float(a * b)),
                                _ => None,
                            },
                            Opcode::Div => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) if *b != 0 => a.checked_div(*b).map(crate::value::Value::Int),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) if *b != 0.0 => Some(crate::value::Value::Float(a / b)),
                                _ => None,
                            },
                            Opcode::Mod => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) if *b != 0 => a.checked_rem(*b).map(crate::value::Value::Int),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) if *b != 0.0 => Some(crate::value::Value::Float(a % b)),
                                _ => None,
                            },
                            Opcode::Pow => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) if *b >= 0 && *b <= u32::MAX as i64 => {
                                    a.checked_pow(*b as u32).map(crate::value::Value::Int)
                                }
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Float(a.powf(*b))),
                                _ => None,
                            },
                            Opcode::And => match (val_a, val_b) {
                                (crate::value::Value::Bool(a), crate::value::Value::Bool(b)) => Some(crate::value::Value::Bool(*a && *b)),
                                _ => None,
                            },
                            Opcode::Or => match (val_a, val_b) {
                                (crate::value::Value::Bool(a), crate::value::Value::Bool(b)) => Some(crate::value::Value::Bool(*a || *b)),
                                _ => None,
                            },
                            Opcode::Eq => Some(crate::value::Value::Bool(val_a == val_b)),
                            Opcode::Neq => Some(crate::value::Value::Bool(val_a != val_b)),
                            Opcode::Lt => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) => Some(crate::value::Value::Bool(a < b)),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Bool(a < b)),
                                _ => None,
                            },
                            Opcode::Lte => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) => Some(crate::value::Value::Bool(a <= b)),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Bool(a <= b)),
                                _ => None,
                            },
                            Opcode::Gt => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) => Some(crate::value::Value::Bool(a > b)),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Bool(a > b)),
                                _ => None,
                            },
                            Opcode::Gte => match (val_a, val_b) {
                                (crate::value::Value::Int(a), crate::value::Value::Int(b)) => Some(crate::value::Value::Bool(a >= b)),
                                (crate::value::Value::Float(a), crate::value::Value::Float(b)) => Some(crate::value::Value::Bool(a >= b)),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some(res) = folded {
                            let new_idx = chunk.add_constant(res);
                            chunk.code[i] = Opcode::LoadConst(new_idx);
                            keep[i + 1] = false;
                            keep[i + 2] = false;
                            pair_removed = true;
                            changed = true;
                            i += 3;
                            continue;
                        }
                    }
                }
            }

            // Constant unary folding: [LoadConst(bool), Not] or [LoadConst(num), Neg]
            if i + 1 < chunk.code.len() && !jump_targets.contains(&i) && !jump_targets.contains(&(i + 1)) {
                if let (Opcode::LoadConst(idx), Opcode::Not) = (&chunk.code[i], &chunk.code[i + 1]) {
                    if let Some(crate::value::Value::Bool(b)) = chunk.constants.get(*idx) {
                        let new_idx = chunk.add_constant(crate::value::Value::Bool(!*b));
                        chunk.code[i] = Opcode::LoadConst(new_idx);
                        keep[i + 1] = false;
                        pair_removed = true;
                        changed = true;
                        i += 2;
                        continue;
                    }
                }
                if let (Opcode::LoadConst(idx), Opcode::Neg) = (&chunk.code[i], &chunk.code[i + 1]) {
                    let folded = match chunk.constants.get(*idx) {
                        Some(crate::value::Value::Int(n)) if *n != i64::MIN => {
                            Some(crate::value::Value::Int(-n))
                        }
                        Some(crate::value::Value::Float(f)) => {
                            Some(crate::value::Value::Float(-f))
                        }
                        _ => None,
                    };
                    if let Some(c) = folded {
                        let new_idx = chunk.add_constant(c);
                        chunk.code[i] = Opcode::LoadConst(new_idx);
                        keep[i + 1] = false;
                        pair_removed = true;
                        changed = true;
                        i += 2;
                        continue;
                    }
                }
            }

            // Constant collection/string length folding: [LoadConst(string/array/map), Len] => [LoadConst(len)]
            if i + 1 < chunk.code.len() && !jump_targets.contains(&i) && !jump_targets.contains(&(i + 1)) {
                if let (Opcode::LoadConst(idx), Opcode::Len) = (&chunk.code[i], &chunk.code[i + 1]) {
                    let len_opt = match chunk.constants.get(*idx) {
                        Some(crate::value::Value::String(s)) => Some(s.chars().count() as i64),
                        Some(crate::value::Value::Array(arr)) => Some(arr.len() as i64),
                        Some(crate::value::Value::Map(m)) => Some(m.len() as i64),
                        _ => None,
                    };
                    if let Some(len_val) = len_opt {
                        let new_idx = chunk.add_constant(crate::value::Value::Int(len_val));
                        chunk.code[i] = Opcode::LoadConst(new_idx);
                        keep[i + 1] = false;
                        pair_removed = true;
                        changed = true;
                        i += 2;
                        continue;
                    }
                }
                // Empty collection length folding: [MakeArray(0)/MakeMap(0), Len] => [LoadConst(0)]
                if (chunk.code[i] == Opcode::MakeArray(0) || chunk.code[i] == Opcode::MakeMap(0))
                    && chunk.code[i + 1] == Opcode::Len
                {
                    let new_idx = chunk.add_constant(crate::value::Value::Int(0));
                    chunk.code[i] = Opcode::LoadConst(new_idx);
                    keep[i + 1] = false;
                    pair_removed = true;
                    changed = true;
                    i += 2;
                    continue;
                }
                // Zero power reduction: [LoadConst(0), Pow] => [Pop, LoadConst(1)]
                if let (Opcode::LoadConst(idx), Opcode::Pow) = (&chunk.code[i], &chunk.code[i + 1]) {
                    let is_zero = match chunk.constants.get(*idx) {
                        Some(crate::value::Value::Int(0)) => true,
                        Some(crate::value::Value::Float(f)) if *f == 0.0 => true,
                        _ => false,
                    };
                    if is_zero {
                        let one_idx = chunk.add_constant(crate::value::Value::Int(1));
                        chunk.code[i] = Opcode::Pop;
                        chunk.code[i + 1] = Opcode::LoadConst(one_idx);
                        changed = true;
                        i += 2;
                        continue;
                    }
                }
            }

            // Commutative operation with swap: [Swap, Add/Mul/Eq/Neq] => [Add/Mul/Eq/Neq]
            if i + 1 < chunk.code.len()
                && !jump_targets.contains(&i)
                && !jump_targets.contains(&(i + 1))
                && chunk.code[i] == Opcode::Swap
                && matches!(
                    chunk.code[i + 1],
                    Opcode::Add | Opcode::Mul | Opcode::Eq | Opcode::Neq
                )
            {
                keep[i] = false;
                pair_removed = true;
                changed = true;
                i += 2;
                continue;
            }

            // Swap with ordering comparator: [Swap, Lt] => [Gt], [Swap, Lte] => [Gte], [Swap, Gt] => [Lt], [Swap, Gte] => [Lte]
            if i + 1 < chunk.code.len()
                && !jump_targets.contains(&i)
                && !jump_targets.contains(&(i + 1))
                && chunk.code[i] == Opcode::Swap
            {
                let inverted_cmp = match chunk.code[i + 1] {
                    Opcode::Lt => Some(Opcode::Gt),
                    Opcode::Lte => Some(Opcode::Gte),
                    Opcode::Gt => Some(Opcode::Lt),
                    Opcode::Gte => Some(Opcode::Lte),
                    _ => None,
                };
                if let Some(inv) = inverted_cmp {
                    chunk.code[i + 1] = inv;
                    keep[i] = false;
                    pair_removed = true;
                    changed = true;
                    i += 2;
                    continue;
                }
            }

            // Triple: Rotate3 x 3 is a complete 3-element identity rotation
            if i + 2 < chunk.code.len()
                && !jump_targets.contains(&i)
                && !jump_targets.contains(&(i + 1))
                && !jump_targets.contains(&(i + 2))
                && chunk.code[i] == Opcode::Rotate3
                && chunk.code[i + 1] == Opcode::Rotate3
                && chunk.code[i + 2] == Opcode::Rotate3
            {
                keep[i] = false;
                keep[i + 1] = false;
                keep[i + 2] = false;
                pair_removed = true;
                changed = true;
                i += 3;
                continue;
            }

            // LoadConst(a), LoadConst(b), Swap => LoadConst(b), LoadConst(a)
            if i + 2 < chunk.code.len()
                && !jump_targets.contains(&(i + 1))
                && !jump_targets.contains(&(i + 2))
            {
                if let (Opcode::LoadConst(idx_a), Opcode::LoadConst(idx_b), Opcode::Swap) =
                    (&chunk.code[i], &chunk.code[i + 1], &chunk.code[i + 2])
                {
                    let b = *idx_b;
                    let a = *idx_a;
                    chunk.code[i] = Opcode::LoadConst(b);
                    chunk.code[i + 1] = Opcode::LoadConst(a);
                    keep[i + 2] = false;
                    pair_removed = true;
                    changed = true;
                    i += 3;
                    continue;
                }
            }

            // Pairs
            if i + 1 < chunk.code.len() && !jump_targets.contains(&i) && !jump_targets.contains(&(i + 1)) {
                // Dup + Swap => Dup (swapping two identical items on top of stack is a no-op)
                if chunk.code[i] == Opcode::Dup && chunk.code[i + 1] == Opcode::Swap {
                    keep[i + 1] = false;
                    pair_removed = true;
                    changed = true;
                    i += 2;
                    continue;
                }

                // Comparator + Not => Inverted comparator
                let inverted_cmp = match chunk.code[i] {
                    Opcode::Eq => Some(Opcode::Neq),
                    Opcode::Neq => Some(Opcode::Eq),
                    Opcode::Lt => Some(Opcode::Gte),
                    Opcode::Lte => Some(Opcode::Gt),
                    Opcode::Gt => Some(Opcode::Lte),
                    Opcode::Gte => Some(Opcode::Lt),
                    _ => None,
                };
                if let Some(inv_op) = inverted_cmp {
                    if chunk.code[i + 1] == Opcode::Not {
                        chunk.code[i] = inv_op;
                        keep[i + 1] = false;
                        pair_removed = true;
                        changed = true;
                        i += 2;
                        continue;
                    }
                }


                let is_redundant = matches!(
                    (&chunk.code[i], &chunk.code[i + 1]),
                    (Opcode::LoadConst(_), Opcode::Pop)
                        | (Opcode::LoadGlobal(_), Opcode::Pop)
                        | (Opcode::Dup, Opcode::Pop)
                        | (Opcode::Not, Opcode::Not)
                        | (Opcode::Neg, Opcode::Neg)
                        | (Opcode::Swap, Opcode::Swap)
                        | (Opcode::LoadLocal(_), Opcode::Pop)
                        | (Opcode::MakeArray(0), Opcode::Pop)
                        | (Opcode::MakeMap(0), Opcode::Pop)
                        | (Opcode::TruncateToInt, Opcode::TruncateToInt)
                );
                if is_redundant {
                    keep[i] = false;
                    keep[i + 1] = false;
                    pair_removed = true;
                    i += 2;
                    continue;
                }
            }
            i += 1;
        }

        if pair_removed {
            let old_len = chunk.code.len();
            let mut remap = vec![0; old_len + 1];
            let mut new_idx = 0;
            for (old_idx, &is_kept) in keep.iter().enumerate() {
                remap[old_idx] = new_idx;
                if is_kept {
                    new_idx += 1;
                }
            }
            remap[old_len] = new_idx;

            let mut new_code = Vec::with_capacity(new_idx);
            let mut new_locs = Vec::with_capacity(new_idx);
            for (idx, &is_kept) in keep.iter().enumerate().take(old_len) {
                if is_kept {
                    let op = match &chunk.code[idx] {
                        Opcode::Jump(t) => Opcode::Jump(remap[*t.min(&old_len)]),
                        Opcode::JumpIfFalse(t) => Opcode::JumpIfFalse(remap[*t.min(&old_len)]),
                        Opcode::JumpIfTrue(t) => Opcode::JumpIfTrue(remap[*t.min(&old_len)]),
                        Opcode::Loop(t) => Opcode::Loop(remap[*t.min(&old_len)]),
                        other => other.clone(),
                    };
                    new_code.push(op);
                    new_locs.push(chunk.locations[idx]);
                }
            }
            chunk.code = new_code;
            chunk.locations = new_locs;
            changed = true;
        }
    }

    changed
}

pub fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

pub fn lcm_i64(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        let g = gcd_i64(a, b);
        if g == 0 {
            0
        } else {
            (a.abs() / g).saturating_mul(b.abs())
        }
    }
}


