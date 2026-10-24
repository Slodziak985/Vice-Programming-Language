use crate::error::{StackFrame, ViceError, ViceErrorType};
use crate::value::{MapKey, ReferenceStep, ReferenceValue, Slot, Value};
use crate::vm::compiler::{CompiledFunction, CompiledProgram};
use crate::vm::opcodes::Opcode;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct VmConfig {
    pub max_call_depth: usize,
    pub max_stack_size: usize,
    pub instruction_budget: Option<u64>,
    pub profile_opcodes: bool,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            max_call_depth: 64,
            max_stack_size: 1_000_000,
            instruction_budget: None,
            profile_opcodes: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub value: Value,
    pub output: Vec<String>,
    pub instructions: u64,
    pub opcode_counts: Vec<(String, u64)>,
    pub audit_trail: Vec<String>,
}

pub const MAX_MEMO_ENTRIES: usize = 20_000;
pub const MAX_STRING_SIZE: usize = 67_108_864; // 64 MB
pub const MAX_ARRAY_SIZE: usize = 10_000_000;
pub const MAX_MAP_SIZE: usize = 10_000_000;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: String,
    pub ip: usize,
    pub locals: Vec<Slot>,
    pub memo_key: Option<(String, Vec<MapKey>)>,
    pub instruction_budget: Option<u64>,
    pub instructions_executed: u64,
    pub is_nonreentrant: bool,
    pub is_checked: bool,
    pub is_saturating: bool,
    pub is_wrapping: bool,
    pub timeout_ms: Option<u64>,
    pub start_time: Option<std::time::Instant>,
    pub is_audit: bool,
}

pub struct Vm {
    program: Option<Rc<CompiledProgram>>,
    pub frames: Vec<CallFrame>,
    pub stack: Vec<Value>,
    pub output: Vec<String>,
    globals: HashMap<String, Value>,
    pub input: Vec<String>,
    pub input_idx: usize,
    pub last_line: usize,
    pub data_declarations: HashMap<String, HashMap<String, crate::types::Type>>,
    pub config: VmConfig,
    executed_instructions: u64,
    opcode_counts: HashMap<&'static str, u64>,
    pub memo_cache: HashMap<(String, Vec<MapKey>), Value>,
    pub active_nonreentrant: std::collections::HashSet<String>,
    pub instantiated_singletons: std::collections::HashSet<String>,
    pub profile_call_counts: HashMap<String, u64>,
    pub function_call_counts: std::cell::RefCell<HashMap<String, usize>>,
    pub audit_trail: std::cell::RefCell<Vec<String>>,
}

impl Vm {
    pub fn new_buffered(input: String) -> Self {
        Self::new_with_input(vec![input])
    }

    pub fn new_with_input(input: Vec<String>) -> Self {
        Self::with_config_and_input(VmConfig::default(), input)
    }

    pub fn with_config(config: VmConfig) -> Self {
        Self::with_config_and_input(config, Vec::new())
    }

    fn with_config_and_input(config: VmConfig, input: Vec<String>) -> Self {
        let mut globals = HashMap::new();
        for module in crate::stdlib::BUILTIN_MODULES {
            globals.insert(module.to_string(), Value::Module(module.to_string()));
        }
        Self {
            program: None,
            frames: Vec::new(),
            stack: Vec::new(),
            output: Vec::new(),
            globals,
            input,
            input_idx: 0,
            last_line: 0,
            data_declarations: HashMap::new(),
            config,
            executed_instructions: 0,
            opcode_counts: HashMap::new(),
            memo_cache: HashMap::new(),
            active_nonreentrant: std::collections::HashSet::new(),
            instantiated_singletons: std::collections::HashSet::new(),
            profile_call_counts: HashMap::new(),
            function_call_counts: std::cell::RefCell::new(HashMap::new()),
            audit_trail: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn new() -> Self {
        Self::new_with_input(Vec::new())
    }

    pub fn interpret(&mut self, program: CompiledProgram) -> Result<ExecutionResult, ViceError> {
        program.verify()?;
        self.frames.clear();
        self.stack.clear();
        self.output.clear();
        self.memo_cache.clear();
        self.active_nonreentrant.clear();
        self.instantiated_singletons.clear();
        self.profile_call_counts.clear();
        self.function_call_counts.borrow_mut().clear();
        self.audit_trail.borrow_mut().clear();
        self.executed_instructions = 0;
        self.opcode_counts.clear();
        self.last_line = 0;
        self.data_declarations = program.data_declarations.clone();
        self.globals
            .retain(|name, _| crate::stdlib::BUILTIN_MODULES.contains(&name.as_str()));
        for name in program.functions.keys() {
            self.globals
                .insert(name.clone(), Value::Callable(name.clone()));
        }
        let entry = program.entry.clone();
        self.program = Some(Rc::new(program));
        if entry.is_empty() {
            return Err(ViceError::new(
                ViceErrorType::Runtime,
                "No @entry or main function found",
                0,
            ));
        }
        let is_memo = self
            .program
            .as_ref()
            .and_then(|p| p.functions.get(&entry))
            .map(|f| f.is_memoized)
            .unwrap_or(false);
        let memo_key = if is_memo {
            Some((entry.clone(), Vec::new()))
        } else {
            None
        };
        if let Some(p) = &self.program {
            if let Some(f) = p.functions.get(&entry) {
                if f.is_profiled {
                    *self.profile_call_counts.entry(entry.clone()).or_insert(0) += 1;
                }
            }
        }
        let frame = self.make_frame(&entry, Vec::new(), Vec::new(), None, memo_key)?;
        let value = self.run(frame)?;
        Ok(ExecutionResult {
            value,
            output: self.output.clone(),
            instructions: self.executed_instructions,
            opcode_counts: if self.config.profile_opcodes {
                sorted_opcode_counts(&self.opcode_counts)
            } else {
                Vec::new()
            },
            audit_trail: self.audit_trail.borrow().clone(),
        })
    }

    fn run(&mut self, mut frame: CallFrame) -> Result<Value, ViceError> {
        let program = self.program.as_ref().cloned().expect("Program must be set");
        if frame.is_nonreentrant {
            self.active_nonreentrant.insert(frame.function.clone());
        }
        'frames: loop {
            let function = program.functions.get(&frame.function).ok_or_else(|| {
                self.runtime_error(&frame, format!("Undefined function: {}", frame.function))
            })?;
            let returned;
            loop {
                if frame.ip >= function.chunk.code.len() {
                    returned = Value::Unit;
                    break;
                }
                self.tick(&mut frame)?;
                let ip = frame.ip;
                let instruction = function.chunk.code.get(ip).ok_or_else(|| {
                    self.runtime_error(&frame, "Instruction pointer is out of bounds")
                })?;
                if self.config.profile_opcodes {
                    *self
                        .opcode_counts
                        .entry(opcode_name(instruction))
                        .or_insert(0) += 1;
                }
                if let Some(location) = function.chunk.locations.get(ip) {
                    self.last_line = location.line;
                }
                frame.ip += 1;

                match instruction {
                    Opcode::LoadConst(index) => {
                        let value = function
                            .chunk
                            .constants
                            .get(*index)
                            .cloned()
                            .ok_or_else(|| self.runtime_error(&frame, "Constant pool index is out of bounds"))?;
                        self.push(&frame, value)?;
                    }
                    Opcode::LoadLocal(index) => {
                        let value = self
                            .local(&frame, *index)?
                            .read()
                            .map_err(|message| self.runtime_error(&frame, message))?;
                        self.push(&frame, value)?;
                    }
                    Opcode::StoreLocal(index) => {
                        let value = self.peek(&frame)?.clone();
                        self.local(&frame, *index)?
                            .write(value)
                            .map_err(|message| self.runtime_error(&frame, message))?;
                    }
                    Opcode::LoadGlobal(index) => {
                        let name = self.string_constant(&frame, *index)?;
                        let value = self.globals.get(&name).cloned().ok_or_else(|| {
                            self.runtime_error(
                                &frame,
                                format!("Undefined global variable: {}", name),
                            )
                        })?;
                        self.push(&frame, value)?;
                    }
                    Opcode::StoreGlobal(index) => {
                        let name = self.string_constant(&frame, *index)?;
                        let value = self.peek(&frame)?.clone();
                        self.globals.insert(name, value);
                    }
                    Opcode::ReferenceLocal(index, mutable) => {
                        let slot = self.local(&frame, *index)?.clone();
                        let value = match slot.read() {
                            Ok(Value::Reference(reference)) => Value::Reference(reference),
                            Ok(_) => Value::Reference(ReferenceValue {
                                root: slot,
                                path: Vec::new(),
                                mutable: *mutable,
                            }),
                            Err(message) => return Err(self.runtime_error(&frame, message)),
                        };
                        self.push(&frame, value)?;
                    }
                    Opcode::ReferenceProperty(index) => {
                        let name = self.string_constant(&frame, *index)?;
                        let mut reference = self.pop_reference(&frame)?;
                        reference.path.push(ReferenceStep::Property(name));
                        self.push(&frame, Value::Reference(reference))?;
                    }
                    Opcode::ReferenceIndex => {
                        let index = self.pop_materialized(&frame)?;
                        let mut reference = self.pop_reference(&frame)?;
                        reference.path.push(ReferenceStep::Index(index));
                        self.push(&frame, Value::Reference(reference))?;
                    }
                    Opcode::Dereference => {
                        let reference = self.pop_reference(&frame)?;
                        let value = self.read_reference(&reference, &frame)?;
                        self.push(&frame, value)?;
                    }
                    Opcode::AssignReference => {
                        let value = self.pop(&frame)?;
                        let reference = self.pop_reference(&frame)?;
                        self.write_reference(&reference, value.clone(), &frame)?;
                        self.push(&frame, value)?;
                    }
                    Opcode::Jump(target) | Opcode::Loop(target) => frame.ip = *target,
                    Opcode::JumpIfFalse(target) => {
                        let value = self.materialize(self.peek(&frame)?.clone(), &frame)?;
                        if !truthy(&value) {
                            frame.ip = *target;
                        }
                    }
                    Opcode::JumpIfTrue(target) => {
                        let value = self.materialize(self.peek(&frame)?.clone(), &frame)?;
                        if truthy(&value) {
                            frame.ip = *target;
                        }
                    }
                    Opcode::Neg => {
                        let value = self.pop_materialized(&frame)?;
                        let result = match value {
                            Value::Int(value) => Value::Int(value.wrapping_neg()),
                            Value::Float(value) => Value::Float(-value),
                            other => {
                                return Err(self.runtime_error(
                                    &frame,
                                    format!("Cannot negate {}", other.type_name()),
                                ))
                            }
                        };
                        self.push(&frame, result)?;
                    }
                    Opcode::Add
                    | Opcode::Sub
                    | Opcode::Mul
                    | Opcode::Div
                    | Opcode::Mod
                    | Opcode::Pow
                    | Opcode::HighLevelDiv => {
                        let right = self.pop_materialized(&frame)?;
                        let left = self.pop_materialized(&frame)?;
                        let result = self.arithmetic(&frame, instruction.clone(), left, right)?;
                        self.push(&frame, result)?;
                    }
                    Opcode::Eq | Opcode::Neq => {
                        let right = self.pop_materialized(&frame)?;
                        let left = self.pop_materialized(&frame)?;
                        let equal = left == right;
                        self.push(
                            &frame,
                            Value::Bool(if *instruction == Opcode::Eq {
                                equal
                            } else {
                                !equal
                            }),
                        )?;
                    }
                    Opcode::Lt | Opcode::Lte | Opcode::Gt | Opcode::Gte => {
                        let right = self.pop_materialized(&frame)?;
                        let left = self.pop_materialized(&frame)?;
                        let result = compare(instruction, &left, &right).ok_or_else(|| {
                            self.runtime_error(&frame, "Comparison requires numeric operands")
                        })?;
                        self.push(&frame, Value::Bool(result))?;
                    }
                    Opcode::And | Opcode::Or => {
                        let right = self.pop_materialized(&frame)?;
                        let left = self.pop_materialized(&frame)?;
                        let (Value::Bool(left), Value::Bool(right)) = (left, right) else {
                            return Err(self.runtime_error(&frame, "Logical operands must be bool"));
                        };
                        self.push(
                            &frame,
                            Value::Bool(if *instruction == Opcode::And {
                                left && right
                            } else {
                                left || right
                            }),
                        )?;
                    }
                    Opcode::Not => {
                        let value = self.pop_materialized(&frame)?;
                        let Value::Bool(value) = value else {
                            return Err(
                                self.runtime_error(&frame, "Logical negation requires bool")
                            );
                        };
                        self.push(&frame, Value::Bool(!value))?;
                    }
                    Opcode::Call(argument_count) => {
                        let callable = self.pop(&frame)?;
                        let arguments = self.pop_many(&frame, *argument_count)?;
                        let called_has_max_depth = match &callable {
                            Value::Callable(name) | Value::String(name) => {
                                self.program
                                    .as_ref()
                                    .and_then(|p| p.functions.get(name))
                                    .and_then(|f| f.max_depth)
                                    .is_some()
                            }
                            _ => false,
                        };
                        let is_tail_call = !frame.is_nonreentrant
                            && !called_has_max_depth
                            && function.max_depth.is_none()
                            && frame.ip < function.chunk.code.len()
                            && matches!(function.chunk.code[frame.ip], Opcode::Return);
                        match callable {
                            Value::Callable(name) | Value::String(name) => {
                                if let Some(value) = self.call_builtin(&frame, &name, &arguments)? {
                                    self.push(&frame, value)?;
                                } else {
                                    let is_memo = self
                                        .program
                                        .as_ref()
                                        .and_then(|p| p.functions.get(&name))
                                        .map(|f| f.is_memoized)
                                        .unwrap_or(false);
                                    let memo_key = if is_memo {
                                        let mut keys = Vec::with_capacity(arguments.len());
                                        let mut ok = true;
                                        for arg in &arguments {
                                            if let Some(k) = MapKey::from_value(arg) {
                                                keys.push(k);
                                            } else {
                                                ok = false;
                                                break;
                                            }
                                        }
                                        if ok {
                                            Some((name.clone(), keys))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    if let Some(ref key) = memo_key {
                                        if let Some(cached_val) = self.memo_cache.get(key) {
                                            self.push(&frame, cached_val.clone())?;
                                            continue;
                                        }
                                    }

                                    let new_frame = self.make_frame(
                                        &name,
                                        arguments,
                                        Vec::new(),
                                        if is_tail_call { self.frames.last() } else { Some(&frame) },
                                        memo_key,
                                    )?;
                                    if let Some(p) = &self.program {
                                        if let Some(f) = p.functions.get(&name) {
                                            if f.is_profiled {
                                                *self.profile_call_counts.entry(name.clone()).or_insert(0) += 1;
                                            }
                                        }
                                    }
                                    if is_tail_call && frame.is_nonreentrant {
                                        self.active_nonreentrant.remove(&frame.function);
                                    }
                                    if new_frame.is_nonreentrant {
                                        self.active_nonreentrant.insert(new_frame.function.clone());
                                    }
                                    if !is_tail_call {
                                        self.frames.push(frame);
                                    }
                                    frame = new_frame;
                                    continue 'frames;
                                }
                            }
                            Value::Closure { function: func_name, captures } => {
                                let is_memo = self
                                    .program
                                    .as_ref()
                                    .and_then(|p| p.functions.get(&func_name))
                                    .map(|f| f.is_memoized)
                                    .unwrap_or(false);
                                let memo_key = if is_memo {
                                    let mut keys = Vec::with_capacity(arguments.len());
                                    let mut ok = true;
                                    for arg in &arguments {
                                        if let Some(k) = MapKey::from_value(arg) {
                                            keys.push(k);
                                        } else {
                                            ok = false;
                                            break;
                                        }
                                    }
                                    if ok {
                                        Some((func_name.clone(), keys))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                if let Some(ref key) = memo_key {
                                    if let Some(cached_val) = self.memo_cache.get(key) {
                                        self.push(&frame, cached_val.clone())?;
                                        continue;
                                    }
                                }

                                let new_frame = self.make_frame(
                                    &func_name,
                                    arguments,
                                    captures,
                                    if is_tail_call { self.frames.last() } else { Some(&frame) },
                                    memo_key,
                                 )?;
                                if let Some(p) = &self.program {
                                    if let Some(f) = p.functions.get(&func_name) {
                                        if f.is_profiled {
                                            *self.profile_call_counts.entry(func_name.clone()).or_insert(0) += 1;
                                        }
                                    }
                                }
                                if is_tail_call && frame.is_nonreentrant {
                                    self.active_nonreentrant.remove(&frame.function);
                                }
                                if new_frame.is_nonreentrant {
                                    self.active_nonreentrant.insert(new_frame.function.clone());
                                }
                                if !is_tail_call {
                                    self.frames.push(frame);
                                }
                                frame = new_frame;
                                continue 'frames;
                            }
                            other => {
                                return Err(self.runtime_error(
                                    &frame,
                                    format!("Attempted to call {}", other.type_name()),
                                ))
                            }
                        }
                    }
                    Opcode::MakeClosure(index, captures) => {
                        let func_name = self.string_constant(&frame, *index)?;
                        let mut slots = Vec::with_capacity(captures.len());
                        for capture in captures {
                            slots.push(self.local(&frame, *capture)?.clone());
                        }
                        self.push(
                            &frame,
                            Value::Closure {
                                function: func_name,
                                captures: slots,
                            },
                        )?;
                    }
                    Opcode::Return => {
                        returned = self.pop(&frame)?;
                        break;
                    }
                    Opcode::MakeArray(count) => {
                        if *count > MAX_ARRAY_SIZE {
                            return Err(self.runtime_error(&frame, "Array size exceeds maximum limit of 10,000,000"));
                        }
                        let values = self.pop_many(&frame, *count)?;
                        self.push(&frame, Value::Array(values))?;
                    }
                    Opcode::MakeMap(count) => {
                        if *count > MAX_MAP_SIZE {
                            return Err(self.runtime_error(&frame, "Map size exceeds maximum limit of 10,000,000"));
                        }
                        let entries = self.pop_many(&frame, count.saturating_mul(2))?;
                        let mut map = HashMap::new();
                        for pair in entries.chunks_exact(2) {
                            map.insert(
                                MapKey::from_value(&pair[0]).ok_or_else(|| {
                                    self.runtime_error(&frame, "Invalid map key type")
                                })?,
                                pair[1].clone(),
                            );
                        }
                        self.push(&frame, Value::Map(map))?;
                    }
                    Opcode::MakeInstance(type_index, count) => {
                        let data_name = self.string_constant(&frame, *type_index)?;
                        if let Some(p) = &self.program {
                            if p.singleton_types.contains(&data_name) {
                                if self.instantiated_singletons.contains(&data_name) {
                                    return Err(self.runtime_error(&frame, format!("Singleton struct '{}' cannot be instantiated more than once", data_name)));
                                }
                                self.instantiated_singletons.insert(data_name.clone());
                            }
                        }
                        let entries = self.pop_many(&frame, count.saturating_mul(2))?;
                        let field_types = self
                            .data_declarations
                            .get(&data_name)
                            .cloned()
                            .unwrap_or_default();
                        let mut fields = HashMap::new();
                        for pair in entries.chunks_exact(2) {
                            let Value::String(name) = &pair[0] else {
                                return Err(self
                                    .runtime_error(&frame, "Instance field name must be string"));
                            };
                            fields.insert(
                                name.clone(),
                                coerce_field(pair[1].clone(), field_types.get(name)),
                            );
                        }
                        self.push(
                            &frame,
                            Value::Instance {
                                data_name,
                                field_types,
                                fields: std::rc::Rc::new(std::cell::RefCell::new(fields)),
                            },
                        )?;
                    }
                    Opcode::GetIndex => {
                        let index = self.pop_materialized(&frame)?;
                        let entity = self.pop_materialized(&frame)?;
                        let value = get_index(&entity, &index)
                            .map_err(|message| self.runtime_error(&frame, message))?;
                        self.push(&frame, value)?;
                    }
                    Opcode::SetIndex => {
                        let value = self.pop(&frame)?;
                        let index = self.pop_materialized(&frame)?;
                        let entity = self.pop(&frame)?;
                        let updated = if let Value::Reference(mut reference) = entity {
                            reference.path.push(ReferenceStep::Index(index));
                            self.write_reference(&reference, value, &frame)?;
                            self.read_reference_root(&reference, &frame)?
                        } else {
                            let mut entity = self.materialize(entity, &frame)?;
                            set_index(&mut entity, &index, value)
                                .map_err(|message| self.runtime_error(&frame, message))?;
                            entity
                        };
                        self.push(&frame, updated)?;
                    }
                    Opcode::GetProperty(index) => {
                        let name = self.string_constant(&frame, *index)?;
                        let entity = self.pop_materialized(&frame)?;
                        let value = self.get_property(&frame, entity, &name)?;
                        self.push(&frame, value)?;
                    }
                    Opcode::SetProperty(index) => {
                        let name = self.string_constant(&frame, *index)?;
                        let entity = self.pop(&frame)?;
                        let value = self.pop(&frame)?;
                        let updated = if let Value::Reference(mut reference) = entity {
                            reference.path.push(ReferenceStep::Property(name));
                            self.write_reference(&reference, value, &frame)?;
                            self.read_reference_root(&reference, &frame)?
                        } else {
                            let entity = self.materialize(entity, &frame)?;
                            set_property(&entity, &name, value)
                                .map_err(|message| self.runtime_error(&frame, message))?;
                            entity
                        };
                        self.push(&frame, updated)?;
                    }
                    Opcode::Unwrap => {
                        let value = self.pop_materialized(&frame)?;
                        let unwrapped = self.unwrap_value(&frame, &value)?;
                        self.push(&frame, unwrapped)?;
                    }
                    Opcode::Try => {
                        let value = self.pop_materialized(&frame)?;
                        match self.try_value(&frame, &value)? {
                            Some(unwrapped) => self.push(&frame, unwrapped)?,
                            None => {
                                returned = value;
                                break;
                            }
                        }
                    }
                    Opcode::MethodCall(index, argument_count) => {
                        let method = self.string_constant(&frame, *index)?;
                        let arguments = self.pop_many(&frame, *argument_count)?;
                        let original = self.pop(&frame)?;
                        let reference = match &original {
                            Value::Reference(reference) => Some(reference.clone()),
                            _ => None,
                        };
                        let entity = self.materialize(original, &frame)?;
                        match entity {
                            Value::Module(module) => {
                                let value =
                                    crate::stdlib::call_native_vm(&module, &method, arguments)
                                        .map_err(|message| self.runtime_error(&frame, message))?;
                                self.push(&frame, value)?;
                            }
                            Value::Array(mut values) => match method.as_str() {
                                "push" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "push expects one argument")
                                        );
                                    }
                                    if values.len() >= MAX_ARRAY_SIZE {
                                        return Err(self.runtime_error(
                                            &frame,
                                            "Array size exceeds maximum limit of 10,000,000",
                                        ));
                                    }
                                    values.push(arguments[0].clone());
                                    let result = Value::Array(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "pop" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "pop expects no arguments")
                                        );
                                    }
                                    let popped = values.pop().ok_or_else(|| {
                                        self.runtime_error(&frame, "Cannot pop from an empty array")
                                    })?;
                                    if let Some(reference) = reference {
                                        self.write_reference(
                                            &reference,
                                            Value::Array(values),
                                            &frame,
                                        )?;
                                    }
                                    self.push(&frame, popped)?;
                                }
                                "len" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "len expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::Int(values.len() as i64))?;
                                }
                                "is_empty" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "is_empty expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::Bool(values.is_empty()))?;
                                }
                                "first" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "first expects no arguments")
                                        );
                                    }
                                    let val = values.first().cloned().ok_or_else(|| {
                                        self.runtime_error(&frame, "Cannot get first element of an empty array")
                                    })?;
                                    self.push(&frame, val)?;
                                }
                                "last" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "last expects no arguments")
                                        );
                                    }
                                    let val = values.last().cloned().ok_or_else(|| {
                                        self.runtime_error(&frame, "Cannot get last element of an empty array")
                                    })?;
                                    self.push(&frame, val)?;
                                }
                                "contains" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "contains expects one argument")
                                        );
                                    }
                                    self.push(&frame, Value::Bool(values.contains(&arguments[0])))?;
                                }
                                "reverse" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "reverse expects no arguments")
                                        );
                                    }
                                    values.reverse();
                                    let result = Value::Array(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "clear" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "clear expects no arguments")
                                        );
                                    }
                                    values.clear();
                                    let result = Value::Array(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "join" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "join expects one argument")
                                        );
                                    }
                                    let separator = crate::stdlib::value_to_string(&arguments[0]);
                                    let text = values
                                        .iter()
                                        .map(crate::stdlib::value_to_string)
                                        .collect::<Vec<_>>()
                                        .join(&separator);
                                    if text.len() > MAX_STRING_SIZE {
                                        return Err(self.runtime_error(&frame, "String size exceeds maximum limit of 64MB"));
                                    }
                                    self.push(&frame, Value::String(text))?;
                                }
                                "slice" => {
                                    if arguments.len() != 2 {
                                        return Err(self.runtime_error(&frame, "slice expects two arguments (start, end)"));
                                    }
                                    let start = crate::stdlib::as_int(&arguments[0]);
                                    let end = crate::stdlib::as_int(&arguments[1]);
                                    let len = values.len() as i64;
                                    let s = start.max(0).min(len) as usize;
                                    let e = end.max(0).min(len) as usize;
                                    let sliced = if s <= e { values[s..e].to_vec() } else { Vec::new() };
                                    self.push(&frame, Value::Array(sliced))?;
                                }
                                "index_of" | "find" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "index_of expects one argument"));
                                    }
                                    let target = &arguments[0];
                                    let idx = values.iter().position(|v| v == target).map(|i| i as i64).unwrap_or(-1);
                                    self.push(&frame, Value::Int(idx))?;
                                }
                                "insert" => {
                                    if arguments.len() != 2 {
                                        return Err(self.runtime_error(&frame, "insert expects two arguments (index, value)"));
                                    }
                                    let idx = crate::stdlib::as_int(&arguments[0]);
                                    if idx < 0 || (idx as usize) > values.len() {
                                        return Err(self.runtime_error(&frame, format!("insert index out of bounds: index {}, length {}", idx, values.len())));
                                    }
                                    if values.len() >= MAX_ARRAY_SIZE {
                                        return Err(self.runtime_error(&frame, "Array size exceeds maximum limit of 10,000,000"));
                                    }
                                    values.insert(idx as usize, arguments[1].clone());
                                    let result = Value::Array(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "remove_at" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "remove_at expects one argument (index)"));
                                    }
                                    let idx = crate::stdlib::as_int(&arguments[0]);
                                    if idx < 0 || (idx as usize) >= values.len() {
                                        return Err(self.runtime_error(&frame, format!("remove_at index out of bounds: index {}, length {}", idx, values.len())));
                                    }
                                    let removed = values.remove(idx as usize);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, Value::Array(values), &frame)?;
                                    }
                                    self.push(&frame, removed)?;
                                }
                                "clone" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "clone expects no arguments"));
                                    }
                                    self.push(&frame, Value::Array(values.clone()))?;
                                }
                                "sort" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "sort expects no arguments"));
                                    }
                                    values.sort_by(|a, b| a.compare_order(b));
                                    let result = Value::Array(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "concat" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "concat expects one argument (other_array)"));
                                    }
                                    let Value::Array(other) = &arguments[0] else {
                                        return Err(self.runtime_error(&frame, "concat argument must be an array"));
                                    };
                                    if values.len().saturating_add(other.len()) > MAX_ARRAY_SIZE {
                                        return Err(self.runtime_error(&frame, "Array size exceeds maximum limit of 10,000,000"));
                                    }
                                    values.extend(other.clone());
                                    let result = Value::Array(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "swap" => {
                                    if arguments.len() != 2 {
                                        return Err(self.runtime_error(&frame, "swap expects two arguments (index_a, index_b)"));
                                    }
                                    let i = crate::stdlib::as_int(&arguments[0]);
                                    let j = crate::stdlib::as_int(&arguments[1]);
                                    let len = values.len() as i64;
                                    if i < 0 || i >= len || j < 0 || j >= len {
                                        return Err(self.runtime_error(&frame, format!("swap index out of bounds: indices ({}, {}), length {}", i, j, len)));
                                    }
                                    values.swap(i as usize, j as usize);
                                    let result = Value::Array(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "count" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "count expects one argument (value)"));
                                    }
                                    let target = &arguments[0];
                                    let cnt = values.iter().filter(|v| *v == target).count() as i64;
                                    self.push(&frame, Value::Int(cnt))?;
                                }
                                "take" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "take expects one argument (n)"));
                                    }
                                    let n = crate::stdlib::as_int(&arguments[0]).max(0) as usize;
                                    let taken = values.into_iter().take(n).collect();
                                    self.push(&frame, Value::Array(taken))?;
                                }
                                "drop" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "drop expects one argument (n)"));
                                    }
                                    let n = crate::stdlib::as_int(&arguments[0]).max(0) as usize;
                                    let dropped = values.into_iter().skip(n).collect();
                                    self.push(&frame, Value::Array(dropped))?;
                                }
                                "chunk" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "chunk expects one argument (size)"));
                                    }
                                    let size = crate::stdlib::as_int(&arguments[0]);
                                    if size <= 0 {
                                        return Err(self.runtime_error(&frame, "chunk size must be greater than 0"));
                                    }
                                    let chunk_size = size as usize;
                                    let chunks: Vec<Value> = values.chunks(chunk_size).map(|c| Value::Array(c.to_vec())).collect();
                                    self.push(&frame, Value::Array(chunks))?;
                                }
                                "flatten" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "flatten expects no arguments"));
                                    }
                                    let mut flat = Vec::new();
                                    for item in values {
                                        match item {
                                            Value::Array(inner) => {
                                                if flat.len().saturating_add(inner.len()) > MAX_ARRAY_SIZE {
                                                    return Err(self.runtime_error(&frame, "Array size exceeds maximum limit of 10,000,000"));
                                                }
                                                flat.extend(inner);
                                            }
                                            other => {
                                                if flat.len() >= MAX_ARRAY_SIZE {
                                                    return Err(self.runtime_error(&frame, "Array size exceeds maximum limit of 10,000,000"));
                                                }
                                                flat.push(other);
                                            }
                                        }
                                    }
                                    self.push(&frame, Value::Array(flat))?;
                                }
                                "sum" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "sum expects no arguments"));
                                    }
                                    let mut is_float = false;
                                    let mut total_int: i64 = 0;
                                    let mut total_float: f64 = 0.0;
                                    for item in &values {
                                        match item {
                                            Value::Int(i) => {
                                                total_int = total_int.wrapping_add(*i);
                                                total_float += *i as f64;
                                            }
                                            Value::Float(f) => {
                                                is_float = true;
                                                total_float += *f;
                                            }
                                            _ => {
                                                return Err(self.runtime_error(&frame, "sum can only be called on an array of numbers"));
                                            }
                                        }
                                    }
                                    if is_float {
                                        self.push(&frame, Value::Float(total_float))?;
                                    } else {
                                        self.push(&frame, Value::Int(total_int))?;
                                    }
                                }
                                "average" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "average expects no arguments"));
                                    }
                                    if values.is_empty() {
                                        return Err(self.runtime_error(&frame, "average cannot be called on an empty array"));
                                    }
                                    let mut total_float: f64 = 0.0;
                                    for item in &values {
                                        match item {
                                            Value::Int(i) => total_float += *i as f64,
                                            Value::Float(f) => total_float += *f,
                                            _ => {
                                                return Err(self.runtime_error(&frame, "average can only be called on an array of numbers"));
                                            }
                                        }
                                    }
                                    self.push(&frame, Value::Float(total_float / values.len() as f64))?;
                                }
                                "min" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "min expects no arguments"));
                                    }
                                    if values.is_empty() {
                                        return Err(self.runtime_error(&frame, "Cannot find minimum of an empty array"));
                                    }
                                    let mut m = values[0].clone();
                                    for v in &values[1..] {
                                        if v.compare_order(&m) == std::cmp::Ordering::Less {
                                            m = v.clone();
                                        }
                                    }
                                    self.push(&frame, m)?;
                                }
                                "max" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "max expects no arguments"));
                                    }
                                    if values.is_empty() {
                                        return Err(self.runtime_error(&frame, "Cannot find maximum of an empty array"));
                                    }
                                    let mut m = values[0].clone();
                                    for v in &values[1..] {
                                        if v.compare_order(&m) == std::cmp::Ordering::Greater {
                                            m = v.clone();
                                        }
                                    }
                                    self.push(&frame, m)?;
                                }
                                "zip" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "zip expects one argument (other_array)"));
                                    }
                                    let Value::Array(other) = &arguments[0] else {
                                        return Err(self.runtime_error(&frame, "zip argument must be an array"));
                                    };
                                    let zipped: Vec<Value> = values.into_iter().zip(other.iter().cloned()).map(|(a, b)| Value::Array(vec![a, b])).collect();
                                    self.push(&frame, Value::Array(zipped))?;
                                }
                                "product" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "product expects no arguments"));
                                    }
                                    let mut total_int: i64 = 1;
                                    let mut total_float: f64 = 1.0;
                                    let mut is_float = false;
                                    for item in &values {
                                        match item {
                                            Value::Int(i) => {
                                                total_int = total_int.wrapping_mul(*i);
                                                total_float *= *i as f64;
                                            }
                                            Value::Float(f) => {
                                                is_float = true;
                                                total_float *= *f;
                                            }
                                            _ => {
                                                return Err(self.runtime_error(&frame, "product can only be called on an array of numbers"));
                                            }
                                        }
                                    }
                                    if is_float {
                                        self.push(&frame, Value::Float(total_float))?;
                                    } else {
                                        self.push(&frame, Value::Int(total_int))?;
                                    }
                                }
                                "dedup" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "dedup expects no arguments"));
                                    }
                                    let mut deduped: Vec<Value> = Vec::new();
                                    for v in values {
                                        if deduped.last() != Some(&v) {
                                            deduped.push(v);
                                        }
                                    }
                                    self.push(&frame, Value::Array(deduped))?;
                                }
                                "binary_search" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "binary_search expects one argument (target)"));
                                    }
                                    let target = &arguments[0];
                                    let mut low: isize = 0;
                                    let mut high: isize = values.len() as isize - 1;
                                    let mut found_idx: i64 = -1;
                                    while low <= high {
                                        let mid = (low + high) / 2;
                                        match values[mid as usize].compare_order(target) {
                                            std::cmp::Ordering::Equal => {
                                                found_idx = mid as i64;
                                                break;
                                            }
                                            std::cmp::Ordering::Less => {
                                                low = mid + 1;
                                            }
                                            std::cmp::Ordering::Greater => {
                                                high = mid - 1;
                                            }
                                        }
                                    }
                                    self.push(&frame, Value::Int(found_idx))?;
                                }
                                "rotate" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "rotate expects one argument (count)"));
                                    }
                                    let count = crate::stdlib::as_int(&arguments[0]);
                                    if !values.is_empty() {
                                        let len = values.len() as i64;
                                        let shift = ((count % len) + len) % len;
                                        values.rotate_right(shift as usize);
                                    }
                                    let result = Value::Array(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "windows" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "windows expects one argument (size)"));
                                    }
                                    let size = crate::stdlib::as_int(&arguments[0]);
                                    if size <= 0 || (size as usize) > values.len() {
                                        self.push(&frame, Value::Array(Vec::new()))?;
                                    } else {
                                        let sz = size as usize;
                                        let res: Vec<Value> = values.windows(sz).map(|w| Value::Array(w.to_vec())).collect();
                                        self.push(&frame, Value::Array(res))?;
                                    }
                                }
                                "frequencies" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "frequencies expects no arguments"));
                                    }
                                    let mut map = std::collections::HashMap::new();
                                    for item in values {
                                        let key = match item {
                                            Value::String(s) => s,
                                            Value::Int(i) => i.to_string(),
                                            Value::Bool(b) => b.to_string(),
                                            other => crate::stdlib::value_to_string(&other),
                                        };
                                        let count = map.entry(key).or_insert(0i64);
                                        *count += 1;
                                    }
                                    let out_map = map.into_iter().map(|(k, v)| (crate::value::MapKey::String(k), Value::Int(v))).collect();
                                    self.push(&frame, Value::Map(out_map))?;
                                }
                                "is_sorted" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "is_sorted expects no arguments"));
                                    }
                                    let sorted = crate::stdlib::is_array_sorted(&values);
                                    self.push(&frame, Value::Bool(sorted))?;
                                }
                                "all_unique" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "all_unique expects no arguments"));
                                    }
                                    let unique = crate::stdlib::are_array_elements_unique(&values);
                                    self.push(&frame, Value::Bool(unique))?;
                                }
                                "find_index" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "find_index expects 1 argument (target)"));
                                    }
                                    let target = &arguments[0];
                                    let idx = values.iter().position(|v| v == target).map(|i| i as i64).unwrap_or(-1);
                                    self.push(&frame, Value::Int(idx))?;
                                }
                                "clamp_elements" => {
                                    if arguments.len() != 2 {
                                        return Err(self.runtime_error(&frame, "clamp_elements expects 2 arguments (min, max)"));
                                    }
                                    let min_f = crate::stdlib::as_float(&arguments[0]);
                                    let max_f = crate::stdlib::as_float(&arguments[1]);
                                    let min_i = crate::stdlib::as_int(&arguments[0]);
                                    let max_i = crate::stdlib::as_int(&arguments[1]);
                                    let clamped: Vec<Value> = values.iter().map(|v| match v {
                                        Value::Int(i) => Value::Int(i.clamp(&min_i, &max_i).clone()),
                                        Value::Float(f) => Value::Float(f.clamp(min_f, max_f)),
                                        other => other.clone(),
                                    }).collect();
                                    self.push(&frame, Value::Array(clamped))?;
                                }
                                "compact" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "compact expects no arguments"));
                                    }
                                    let compacted: Vec<Value> = values.into_iter().filter(|v| !matches!(v, Value::Unit)).collect();
                                    self.push(&frame, Value::Array(compacted))?;
                                }
                                "diff" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "diff expects 1 argument (other_array)"));
                                    }
                                    let Value::Array(other) = &arguments[0] else {
                                        return Err(self.runtime_error(&frame, "diff argument must be an array"));
                                    };
                                    let res: Vec<Value> = values.into_iter().filter(|v| !other.contains(v)).collect();
                                    self.push(&frame, Value::Array(res))?;
                                }
                                "intersect" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "intersect expects 1 argument (other_array)"));
                                    }
                                    let Value::Array(other) = &arguments[0] else {
                                        return Err(self.runtime_error(&frame, "intersect argument must be an array"));
                                    };
                                    let mut res: Vec<Value> = Vec::new();
                                    for v in values {
                                        if other.contains(&v) && !res.contains(&v) {
                                            res.push(v);
                                        }
                                    }
                                    self.push(&frame, Value::Array(res))?;
                                }
                                _ => {
                                    return Err(self.runtime_error(
                                        &frame,
                                        format!("Unknown array method: {}", method),
                                    ))
                                }
                            },
                            Value::String(value) => match method.as_str() {
                                "len" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "len expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::Int(value.chars().count() as i64))?;
                                }
                                "is_empty" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "is_empty expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::Bool(value.is_empty()))?;
                                }
                                "trim" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "trim expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::String(value.trim().to_string()))?;
                                }
                                "trim_start" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "trim_start expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::String(value.trim_start().to_string()))?;
                                }
                                "trim_end" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "trim_end expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::String(value.trim_end().to_string()))?;
                                }
                                "rot13" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "rot13 expects no arguments"));
                                    }
                                    let rotated: String = value.chars().map(|c| match c {
                                        'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
                                        'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
                                        _ => c,
                                    }).collect();
                                    self.push(&frame, Value::String(rotated))?;
                                }
                                "levenshtein" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "levenshtein expects 1 argument (other)"));
                                    }
                                    let other = crate::stdlib::value_to_string(&arguments[0]);
                                    let dist = crate::stdlib::levenshtein_distance(&value, &other);
                                    self.push(&frame, Value::Int(dist as i64))?;
                                }
                                "hamming" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "hamming expects 1 argument (other)"));
                                    }
                                    let other = crate::stdlib::value_to_string(&arguments[0]);
                                    let dist = crate::stdlib::hamming_distance(&value, &other);
                                    self.push(&frame, Value::Int(dist))?;
                                }
                                "slugify" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "slugify expects no arguments"));
                                    }
                                    let slug = crate::stdlib::slugify(&value);
                                    self.push(&frame, Value::String(slug))?;
                                }
                                "truncate" => {
                                    if arguments.is_empty() || arguments.len() > 2 {
                                        return Err(self.runtime_error(&frame, "truncate expects 1 or 2 arguments (max_len, [ellipsis])"));
                                    }
                                    let max_len = crate::stdlib::as_int(&arguments[0]).max(0) as usize;
                                    let ellipsis = if arguments.len() > 1 {
                                        crate::stdlib::value_to_string(&arguments[1])
                                    } else {
                                        "...".to_string()
                                    };
                                    let res = crate::stdlib::truncate_string(&value, max_len, &ellipsis);
                                    self.push(&frame, Value::String(res))?;
                                }
                                "hex_encode" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "hex_encode expects no arguments"));
                                    }
                                    let res = crate::stdlib::hex_encode(&value);
                                    self.push(&frame, Value::String(res))?;
                                }
                                "hex_decode" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "hex_decode expects no arguments"));
                                    }
                                    let res = crate::stdlib::hex_decode(&value);
                                    self.push(&frame, Value::String(res))?;
                                }
                                "is_ascii" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "is_ascii expects no arguments"));
                                    }
                                    self.push(&frame, Value::Bool(crate::stdlib::is_string_ascii(&value)))?;
                                }
                                "is_alphanumeric" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "is_alphanumeric expects no arguments"));
                                    }
                                    self.push(&frame, Value::Bool(crate::stdlib::is_string_alphanumeric(&value)))?;
                                }
                                "count_occurrences" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "count_occurrences expects 1 argument (sub)"));
                                    }
                                    let sub = crate::stdlib::value_to_string(&arguments[0]);
                                    let cnt = crate::stdlib::count_string_occurrences(&value, &sub);
                                    self.push(&frame, Value::Int(cnt))?;
                                }
                                "pad_zero" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "pad_zero expects 1 argument (width)"));
                                    }
                                    let width = crate::stdlib::as_int(&arguments[0]).max(0) as usize;
                                    let res = crate::stdlib::pad_zero_string(&value, width);
                                    self.push(&frame, Value::String(res))?;
                                }
                                "lines_count" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "lines_count expects no arguments"));
                                    }
                                    let cnt = crate::stdlib::string_lines_count(&value);
                                    self.push(&frame, Value::Int(cnt))?;
                                }
                                "words_count" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "words_count expects no arguments"));
                                    }
                                    let cnt = crate::stdlib::string_words_count(&value);
                                    self.push(&frame, Value::Int(cnt))?;
                                }
                                "lower" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "lower expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::String(value.to_lowercase()))?;
                                }
                                "upper" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "upper expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::String(value.to_uppercase()))?;
                                }
                                "contains" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "contains expects one argument")
                                        );
                                    }
                                    let needle = crate::stdlib::value_to_string(&arguments[0]);
                                    self.push(&frame, Value::Bool(value.contains(&needle)))?;
                                }
                                "starts_with" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "starts_with expects one argument")
                                        );
                                    }
                                    let prefix = crate::stdlib::value_to_string(&arguments[0]);
                                    self.push(&frame, Value::Bool(value.starts_with(&prefix)))?;
                                }
                                "ends_with" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "ends_with expects one argument")
                                        );
                                    }
                                    let suffix = crate::stdlib::value_to_string(&arguments[0]);
                                    self.push(&frame, Value::Bool(value.ends_with(&suffix)))?;
                                }
                                "replace" => {
                                    if arguments.len() != 2 {
                                        return Err(self.runtime_error(
                                            &frame,
                                            "replace expects two arguments (from, to)",
                                        ));
                                    }
                                    let from = crate::stdlib::value_to_string(&arguments[0]);
                                    let to = crate::stdlib::value_to_string(&arguments[1]);
                                    let replaced = value.replace(&from, &to);
                                    if replaced.len() > MAX_STRING_SIZE {
                                        return Err(self.runtime_error(&frame, "String size exceeds maximum limit of 64MB"));
                                    }
                                    self.push(&frame, Value::String(replaced))?;
                                }
                                "split" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "split expects one argument")
                                        );
                                    }
                                    let delimiter = crate::stdlib::value_to_string(&arguments[0]);
                                    let parts = value
                                        .split(&delimiter)
                                        .map(|s| Value::String(s.to_string()))
                                        .collect::<Vec<_>>();
                                    self.push(&frame, Value::Array(parts))?;
                                }
                                "slice" => {
                                    if arguments.len() != 2 {
                                        return Err(self.runtime_error(&frame, "slice expects two arguments (start, end)"));
                                    }
                                    let start = crate::stdlib::as_int(&arguments[0]);
                                    let end = crate::stdlib::as_int(&arguments[1]);
                                    let chars: Vec<char> = value.chars().collect();
                                    let len = chars.len() as i64;
                                    let s = start.max(0).min(len) as usize;
                                    let e = end.max(0).min(len) as usize;
                                    let sliced: String = if s <= e { chars[s..e].iter().collect() } else { String::new() };
                                    self.push(&frame, Value::String(sliced))?;
                                }
                                "index_of" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "index_of expects one argument (needle)"));
                                    }
                                    let needle = crate::stdlib::value_to_string(&arguments[0]);
                                    let idx = value.find(&needle).map(|byte_idx| {
                                        value[..byte_idx].chars().count() as i64
                                    }).unwrap_or(-1);
                                    self.push(&frame, Value::Int(idx))?;
                                }
                                "repeat" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "repeat expects one argument (count)"));
                                    }
                                    let count = crate::stdlib::as_int(&arguments[0]);
                                    if count < 0 {
                                        return Err(self.runtime_error(&frame, "repeat count cannot be negative"));
                                    }
                                    let count = count as usize;
                                    if value.len().saturating_mul(count) > MAX_STRING_SIZE {
                                        return Err(self.runtime_error(&frame, "String size exceeds maximum limit of 64MB"));
                                    }
                                    let repeated = value.repeat(count);
                                    self.push(&frame, Value::String(repeated))?;
                                }
                                "chars" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "chars expects no arguments"));
                                    }
                                    let char_values = value.chars().map(|c| Value::String(c.to_string())).collect::<Vec<_>>();
                                    self.push(&frame, Value::Array(char_values))?;
                                }
                                "pad_start" => {
                                    if arguments.len() != 2 {
                                        return Err(self.runtime_error(&frame, "pad_start expects two arguments (target_len, pad_char)"));
                                    }
                                    let target_len = crate::stdlib::as_int(&arguments[0]).max(0) as usize;
                                    if target_len > MAX_STRING_SIZE {
                                        return Err(self.runtime_error(&frame, "String size exceeds maximum limit of 64MB"));
                                    }
                                    let pad_str = crate::stdlib::value_to_string(&arguments[1]);
                                    let current_len = value.chars().count();
                                    if current_len >= target_len || pad_str.is_empty() {
                                        self.push(&frame, Value::String(value.clone()))?;
                                    } else {
                                        let needed = target_len - current_len;
                                        let mut pad = String::with_capacity(needed);
                                        for c in pad_str.chars().cycle().take(needed) {
                                            pad.push(c);
                                        }
                                        pad.push_str(&value);
                                        if pad.len() > MAX_STRING_SIZE {
                                            return Err(self.runtime_error(&frame, "String size exceeds maximum limit of 64MB"));
                                        }
                                        self.push(&frame, Value::String(pad))?;
                                    }
                                }
                                "pad_end" => {
                                    if arguments.len() != 2 {
                                        return Err(self.runtime_error(&frame, "pad_end expects two arguments (target_len, pad_char)"));
                                    }
                                    let target_len = crate::stdlib::as_int(&arguments[0]).max(0) as usize;
                                    if target_len > MAX_STRING_SIZE {
                                        return Err(self.runtime_error(&frame, "String size exceeds maximum limit of 64MB"));
                                    }
                                    let pad_str = crate::stdlib::value_to_string(&arguments[1]);
                                    let current_len = value.chars().count();
                                    if current_len >= target_len || pad_str.is_empty() {
                                        self.push(&frame, Value::String(value.clone()))?;
                                    } else {
                                        let needed = target_len - current_len;
                                        let mut res = value.clone();
                                        for c in pad_str.chars().cycle().take(needed) {
                                            res.push(c);
                                        }
                                        if res.len() > MAX_STRING_SIZE {
                                            return Err(self.runtime_error(&frame, "String size exceeds maximum limit of 64MB"));
                                        }
                                        self.push(&frame, Value::String(res))?;
                                    }
                                }
                                "char_at" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "char_at expects one argument (index)"));
                                    }
                                    let idx = crate::stdlib::as_int(&arguments[0]);
                                    let chars: Vec<char> = value.chars().collect();
                                    if idx < 0 || (idx as usize) >= chars.len() {
                                        return Err(self.runtime_error(&frame, format!("char_at index out of bounds: index {}, length {}", idx, chars.len())));
                                    }
                                    self.push(&frame, Value::String(chars[idx as usize].to_string()))?;
                                }
                                "reverse" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "reverse expects no arguments"));
                                    }
                                    let rev: String = value.chars().rev().collect();
                                    self.push(&frame, Value::String(rev))?;
                                }
                                "lines" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "lines expects no arguments"));
                                    }
                                    let lines: Vec<Value> = value.lines().map(|s| Value::String(s.to_string())).collect();
                                    self.push(&frame, Value::Array(lines))?;
                                }
                                "count" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "count expects one argument (pattern)"));
                                    }
                                    let needle = crate::stdlib::value_to_string(&arguments[0]);
                                    let cnt = if needle.is_empty() {
                                        0
                                    } else {
                                        value.matches(&needle).count() as i64
                                    };
                                    self.push(&frame, Value::Int(cnt))?;
                                }
                                "capitalize" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "capitalize expects no arguments"));
                                    }
                                    let mut chars = value.chars();
                                    let res = match chars.next() {
                                        None => String::new(),
                                        Some(first) => {
                                            let mut cap = first.to_uppercase().to_string();
                                            cap.push_str(chars.as_str());
                                            cap
                                        }
                                    };
                                    self.push(&frame, Value::String(res))?;
                                }
                                "is_numeric" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "is_numeric expects no arguments"));
                                    }
                                    let num = !value.is_empty() && value.chars().all(|c| c.is_ascii_digit());
                                    self.push(&frame, Value::Bool(num))?;
                                }
                                "is_alpha" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "is_alpha expects no arguments"));
                                    }
                                    let alpha = !value.is_empty() && value.chars().all(|c| c.is_alphabetic());
                                    self.push(&frame, Value::Bool(alpha))?;
                                }
                                "words" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "words expects no arguments"));
                                    }
                                    let w: Vec<Value> = value.split_whitespace().map(|s| Value::String(s.to_string())).collect();
                                    self.push(&frame, Value::Array(w))?;
                                }
                                "fnv1a" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "fnv1a expects no arguments"));
                                    }
                                    let mut hash: u64 = 0xcbf29ce484222325;
                                    for byte in value.as_bytes() {
                                        hash ^= *byte as u64;
                                        hash = hash.wrapping_mul(0x100000001b3);
                                    }
                                    self.push(&frame, Value::Int(hash as i64))?;
                                }
                                "djb2" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "djb2 expects no arguments"));
                                    }
                                    let mut hash: u64 = 5381;
                                    for byte in value.as_bytes() {
                                        hash = hash.wrapping_mul(33).wrapping_add(*byte as u64);
                                    }
                                    self.push(&frame, Value::Int(hash as i64))?;
                                }
                                "adler32" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "adler32 expects no arguments"));
                                    }
                                    let mut a: u32 = 1;
                                    let mut b: u32 = 0;
                                    for byte in value.as_bytes() {
                                        a = (a + *byte as u32) % 65521;
                                        b = (b + a) % 65521;
                                    }
                                    let adler = ((b << 16) | a) as i64;
                                    self.push(&frame, Value::Int(adler))?;
                                }
                                "crc32" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "crc32 expects no arguments"));
                                    }
                                    let mut crc: u32 = 0xFFFFFFFF;
                                    for byte in value.as_bytes() {
                                        crc ^= *byte as u32;
                                        for _ in 0..8 {
                                            crc = if (crc & 1) != 0 {
                                                (crc >> 1) ^ 0xEDB88320
                                            } else {
                                                crc >> 1
                                            };
                                        }
                                    }
                                    self.push(&frame, Value::Int((!crc) as i64))?;
                                }
                                "murmur3" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "murmur3 expects no arguments"));
                                    }
                                    let h = crate::stdlib::murmur3_32(value.as_bytes(), 0);
                                    self.push(&frame, Value::Int(h as i64))?;
                                }
                                _ => {
                                    // Check if this string is an enum variant (e.g. "Color.Red") or string impl with a user-defined method
                                    let candidate_names = if let Some((enum_name, _)) = value.split_once('.') {
                                        vec![format!("{}::{}", enum_name, method), format!("string::{}", method)]
                                    } else {
                                        vec![format!("string::{}", method)]
                                    };

                                    let mut resolved_func = None;
                                    for candidate in candidate_names {
                                        if let Some(function) = self.program.as_ref().and_then(|p| p.functions.get(&candidate)) {
                                            resolved_func = Some((candidate, function.clone()));
                                            break;
                                        }
                                    }

                                    if let Some((name, function)) = resolved_func {
                                        let arity = function.arity;
                                        let mut method_arguments = Vec::with_capacity(arguments.len() + 1);
                                        if arity == arguments.len() + 1 {
                                            method_arguments.push(Value::String(value.clone()));
                                        }
                                        method_arguments.extend(arguments);
                                        let is_tail_call = !frame.is_nonreentrant
                                            && frame.ip < function.chunk.code.len()
                                            && matches!(function.chunk.code[frame.ip], Opcode::Return);
                                        let new_frame = self.make_frame(
                                            &name,
                                            method_arguments,
                                            Vec::new(),
                                            if is_tail_call { self.frames.last() } else { Some(&frame) },
                                            None,
                                        )?;
                                        if is_tail_call && frame.is_nonreentrant {
                                            self.active_nonreentrant.remove(&frame.function);
                                        }
                                        if new_frame.is_nonreentrant {
                                            self.active_nonreentrant.insert(new_frame.function.clone());
                                        }
                                        if !is_tail_call {
                                            self.frames.push(frame);
                                        }
                                        frame = new_frame;
                                        continue 'frames;
                                    }

                                    return Err(self.runtime_error(
                                        &frame,
                                        format!("Unknown string method: {}", method),
                                    ));
                                }
                            },
                            Value::Map(mut values) => match method.as_str() {
                                "len" | "count" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "len expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::Int(values.len() as i64))?;
                                }
                                "is_empty" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "is_empty expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::Bool(values.is_empty()))?;
                                }
                                "contains" | "contains_key" | "has_key" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "contains expects one argument (key)")
                                        );
                                    }
                                    let key = MapKey::from_value(&arguments[0]).ok_or_else(|| {
                                        self.runtime_error(&frame, "Invalid map key type")
                                    })?;
                                    self.push(&frame, Value::Bool(values.contains_key(&key)))?;
                                }
                                "keys" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "keys expects no arguments")
                                        );
                                    }
                                    let mut keys: Vec<MapKey> = values.keys().cloned().collect();
                                    keys.sort();
                                    let key_vals: Vec<Value> = keys.into_iter().map(|k| k.to_value()).collect();
                                    self.push(&frame, Value::Array(key_vals))?;
                                }
                                "values" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "values expects no arguments")
                                        );
                                    }
                                    let mut entries: Vec<(MapKey, Value)> = values.into_iter().collect();
                                    entries.sort_by(|a, b| a.0.cmp(&b.0));
                                    let val_list: Vec<Value> = entries.into_iter().map(|(_, v)| v).collect();
                                    self.push(&frame, Value::Array(val_list))?;
                                }
                                "get" => {
                                    if arguments.len() != 2 {
                                        return Err(
                                            self.runtime_error(&frame, "get expects two arguments: (key, default)")
                                        );
                                    }
                                    let key = MapKey::from_value(&arguments[0]).ok_or_else(|| {
                                        self.runtime_error(&frame, "Invalid map key type")
                                    })?;
                                    let res = values.get(&key).cloned().unwrap_or_else(|| arguments[1].clone());
                                    self.push(&frame, res)?;
                                }
                                "remove" => {
                                    if arguments.len() != 1 {
                                        return Err(
                                            self.runtime_error(&frame, "remove expects one argument (key)")
                                        );
                                    }
                                    let key = MapKey::from_value(&arguments[0]).ok_or_else(|| {
                                        self.runtime_error(&frame, "Invalid map key type")
                                    })?;
                                    let removed = values.remove(&key);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, Value::Map(values), &frame)?;
                                    }
                                    self.push(&frame, Value::Bool(removed.is_some()))?;
                                }
                                "clear" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "clear expects no arguments")
                                        );
                                    }
                                    values.clear();
                                    let result = Value::Map(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "entries" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "entries expects no arguments")
                                        );
                                    }
                                    let mut entries: Vec<(MapKey, Value)> = values.into_iter().collect();
                                    entries.sort_by(|a, b| a.0.cmp(&b.0));
                                    let list: Vec<Value> = entries
                                        .into_iter()
                                        .map(|(k, v)| Value::Array(vec![k.to_value(), v]))
                                        .collect();
                                    self.push(&frame, Value::Array(list))?;
                                }
                                "clone" => {
                                    if !arguments.is_empty() {
                                        return Err(
                                            self.runtime_error(&frame, "clone expects no arguments")
                                        );
                                    }
                                    self.push(&frame, Value::Map(values.clone()))?;
                                }
                                "merge" | "extend" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(
                                            &frame,
                                            "merge expects one argument (other_map)",
                                        ));
                                    }
                                    let Value::Map(other) = &arguments[0] else {
                                        return Err(self.runtime_error(
                                            &frame,
                                            "merge argument must be a map",
                                        ));
                                    };
                                    if values.len().saturating_add(other.len()) > MAX_MAP_SIZE {
                                        return Err(self.runtime_error(
                                            &frame,
                                            "Map size exceeds maximum limit",
                                        ));
                                    }
                                    for (k, v) in other {
                                        values.insert(k.clone(), v.clone());
                                    }
                                    let result = Value::Map(values);
                                    if let Some(reference) = reference {
                                        self.write_reference(&reference, result.clone(), &frame)?;
                                    }
                                    self.push(&frame, result)?;
                                }
                                "has_value" => {
                                    if arguments.len() != 1 {
                                        return Err(self.runtime_error(&frame, "has_value expects one argument"));
                                    }
                                    let found = values.values().any(|v| v == &arguments[0]);
                                    self.push(&frame, Value::Bool(found))?;
                                }
                                "invert" => {
                                    if !arguments.is_empty() {
                                        return Err(self.runtime_error(&frame, "invert expects no arguments"));
                                    }
                                    let mut inverted = HashMap::new();
                                    for (k, v) in values {
                                        let inv_key = MapKey::from_value(&v).ok_or_else(|| {
                                            self.runtime_error(&frame, "Cannot invert map: value cannot be converted to a map key")
                                        })?;
                                        inverted.insert(inv_key, k.to_value());
                                    }
                                    self.push(&frame, Value::Map(inverted))?;
                                }
                                _ => {
                                    return Err(self.runtime_error(
                                        &frame,
                                        format!("Unknown map method: {}", method),
                                    ));
                                }
                            },
                            instance @ (Value::Instance { .. } | Value::EnumVariant { .. }) => {
                                let data_name = match &instance {
                                    Value::Instance { data_name, .. } => data_name.clone(),
                                    Value::EnumVariant { enum_name, .. } => enum_name.clone(),
                                    _ => unreachable!(),
                                };
                                let name = format!("{}::{}", data_name, method);
                                let arity = self
                                    .program
                                    .as_ref()
                                    .and_then(|program| program.functions.get(&name))
                                    .map(|function| function.arity)
                                    .ok_or_else(|| {
                                        self.runtime_error(
                                            &frame,
                                            format!("Instance method not implemented: {}", name),
                                        )
                                    })?;
                                let mut method_arguments = Vec::with_capacity(arguments.len() + 1);
                                if arity == arguments.len() + 1 {
                                    method_arguments.push(instance.clone());
                                }
                                method_arguments.extend(arguments);
                                let is_tail_call = !frame.is_nonreentrant
                                    && frame.ip < function.chunk.code.len()
                                    && matches!(function.chunk.code[frame.ip], Opcode::Return);
                                let new_frame = self.make_frame(
                                    &name,
                                    method_arguments,
                                    Vec::new(),
                                    if is_tail_call { self.frames.last() } else { Some(&frame) },
                                    None,
                                )?;
                                if is_tail_call && frame.is_nonreentrant {
                                    self.active_nonreentrant.remove(&frame.function);
                                }
                                if new_frame.is_nonreentrant {
                                    self.active_nonreentrant.insert(new_frame.function.clone());
                                }
                                if !is_tail_call {
                                    self.frames.push(frame);
                                }
                                frame = new_frame;
                                continue 'frames;
                            }
                            other => {
                                return Err(self.runtime_error(
                                    &frame,
                                    format!("Method call on {}", other.type_name()),
                                ))
                            }
                        }
                    }
                    Opcode::NativeCall(module_index, method_index, argument_count) => {
                        let module = self.string_constant(&frame, *module_index)?;
                        let method = self.string_constant(&frame, *method_index)?;
                        let arguments = self.pop_many(&frame, *argument_count)?;
                        let value = crate::stdlib::call_native_vm(&module, &method, arguments)
                            .map_err(|message| self.runtime_error(&frame, message))?;
                        self.push(&frame, value)?;
                    }
                    Opcode::Raise(metadata_count) => {
                        let message = self.pop_materialized(&frame)?;
                        let _metadata = self.pop_many(&frame, *metadata_count)?;
                        return Err(
                            self.runtime_error(&frame, crate::stdlib::value_to_string(&message))
                        );
                    }
                    Opcode::Dup => {
                        let value = self.peek(&frame)?.clone();
                        self.push(&frame, value)?;
                    }
                    Opcode::Pop => {
                        self.pop(&frame)?;
                    }
                    Opcode::Len => {
                        let value = self.pop_materialized(&frame)?;
                        let length = match value {
                            Value::Array(values) => values.len(),
                            Value::String(value) => value.chars().count(),
                            Value::Map(values) => values.len(),
                            _ => {
                                return Err(self
                                    .runtime_error(&frame, "len requires array, map, or string"))
                            }
                        };
                        self.push(&frame, Value::Int(length as i64))?;
                    }
                    Opcode::Output(count) => {
                        let values = self.pop_many(&frame, *count)?;
                        self.output.push(
                            values
                                .iter()
                                .map(crate::stdlib::value_to_string)
                                .collect::<Vec<_>>()
                                .join(" "),
                        );
                    }
                    Opcode::TruncateToInt => {
                        let value = self.pop_materialized(&frame)?;
                        let value = match value {
                            Value::Float(value) => Value::Int(value as i64),
                            Value::Int(value) => Value::Int(value),
                            _ => {
                                return Err(self.runtime_error(&frame, "Cannot truncate non-number"))
                            }
                        };
                        self.push(&frame, value)?;
                    }
                    Opcode::Input => {
                        let value =
                            self.input.get(self.input_idx).cloned().ok_or_else(|| {
                                self.runtime_error(&frame, "No more buffered input")
                            })?;
                        self.input_idx += 1;
                        self.push(&frame, Value::String(value))?;
                    }
                    Opcode::Swap => {
                        let first = self.pop(&frame)?;
                        let second = self.pop(&frame)?;
                        self.push(&frame, first)?;
                        self.push(&frame, second)?;
                    }
                    Opcode::Rotate3 => {
                        let top = self.pop(&frame)?;
                        let middle = self.pop(&frame)?;
                        let bottom = self.pop(&frame)?;
                        self.push(&frame, middle)?;
                        self.push(&frame, top)?;
                        self.push(&frame, bottom)?;
                    }
                }
            }

            let result = returned;
            if frame.is_nonreentrant {
                self.active_nonreentrant.remove(&frame.function);
            }
            if frame.is_audit {
                self.audit_trail.borrow_mut().push(format!(
                    "RET {} => {}",
                    frame.function,
                    crate::stdlib::value_to_string(&result)
                ));
            }
            if let Some(key) = frame.memo_key {
                if self.memo_cache.len() < MAX_MEMO_ENTRIES {
                    self.memo_cache.insert(key, result.clone());
                }
            }
            if let Some(caller) = self.frames.pop() {
                frame = caller;
                self.push(&frame, result)?;
                continue 'frames;
            }
            return Ok(result);
        }
    }

    fn make_frame(
        &self,
        name: &str,
        arguments: Vec<Value>,
        captures: Vec<Slot>,
        caller: Option<&CallFrame>,
        memo_key: Option<(String, Vec<MapKey>)>,
    ) -> Result<CallFrame, ViceError> {
        if caller.is_some() && self.frames.len() + 1 >= self.config.max_call_depth {
            return Err(self.error_for_caller(caller, "Maximum call depth exceeded"));
        }
        let function = self
            .program
            .as_ref()
            .and_then(|program| program.functions.get(name))
            .ok_or_else(|| {
                self.error_for_caller(caller, format!("Undefined function: {}", name))
            })?;
        if function.is_unreachable {
            return Err(self.error_for_caller(
                caller,
                format!("Execution reached @unreachable function '{}'", name),
            ));
        }
        if function.is_nonreentrant && self.active_nonreentrant.contains(name) {
            return Err(self.error_for_caller(
                caller,
                format!("Reentrancy violation: function '{}' is marked with @nonreentrant and is already active on the call stack", name),
            ));
        }
        if let Some(limit) = function.rate_limit {
            let mut counts = self.function_call_counts.borrow_mut();
            let count = counts.entry(name.to_string()).or_insert(0);
            *count += 1;
            if *count > limit {
                return Err(self.error_for_caller(
                    caller,
                    format!("Rate limit of {} calls exceeded for function '{}'", limit, name),
                ));
            }
        }
        if function.is_audit {
            let args_str: Vec<String> = arguments.iter().map(crate::stdlib::value_to_string).collect();
            self.audit_trail.borrow_mut().push(format!("CALL {}({})", name, args_str.join(", ")));
        }
        if let Some(limit) = function.max_depth {
            let mut current_depth = if let Some(c) = caller {
                if c.function == name { 1 } else { 0 }
            } else { 0 };
            for f in &self.frames {
                if f.function == name {
                    current_depth += 1;
                }
            }
            if current_depth >= limit {
                return Err(self.error_for_caller(
                    caller,
                    format!("Recursion depth limit of {} exceeded in function '{}'", limit, name),
                ));
            }
        }
        if arguments.len() != function.arity {
            return Err(self.error_for_caller(
                caller,
                format!(
                    "Function '{}' expects {} arguments, got {}",
                    name,
                    function.arity,
                    arguments.len()
                ),
            ));
        }
        if captures.len() != function.capture_count {
            return Err(self.error_for_caller(caller, "Closure capture count mismatch"));
        }
        let mut locals = (0..function.local_count)
            .map(|_| Slot::new(Value::Unit))
            .collect::<Vec<_>>();
        for (index, value) in arguments.into_iter().enumerate() {
            locals[index] = Slot::new(value);
        }
        for (offset, capture) in captures.into_iter().enumerate() {
            locals[function.arity + offset] = capture;
        }
        let (timeout_ms, start_time) = if let Some(ms) = function.timeout_ms {
            (Some(ms), Some(std::time::Instant::now()))
        } else {
            (None, None)
        };
        Ok(CallFrame {
            function: name.to_string(),
            ip: 0,
            locals,
            memo_key,
            instruction_budget: function.budget,
            instructions_executed: 0,
            is_nonreentrant: function.is_nonreentrant,
            is_checked: function.is_checked,
            is_saturating: function.is_saturating,
            is_wrapping: function.is_wrapping,
            timeout_ms,
            start_time,
            is_audit: function.is_audit,
        })
    }

    pub fn audit_trail(&self) -> Vec<String> {
        self.audit_trail.borrow().clone()
    }

    pub fn function_call_counts(&self) -> HashMap<String, usize> {
        self.function_call_counts.borrow().clone()
    }

    fn function<'a>(
        &'a self,
        name: &str,
        frame: &CallFrame,
    ) -> Result<&'a CompiledFunction, ViceError> {
        self.program
            .as_ref()
            .and_then(|program| program.functions.get(name))
            .ok_or_else(|| self.runtime_error(frame, format!("Undefined function: {}", name)))
    }

    fn tick(&mut self, frame: &mut CallFrame) -> Result<(), ViceError> {
        self.executed_instructions = self.executed_instructions.saturating_add(1);
        frame.instructions_executed = frame.instructions_executed.saturating_add(1);
        if let Some(limit) = self.config.instruction_budget {
            if self.executed_instructions > limit {
                return Err(self.runtime_error(frame, "Instruction budget exceeded"));
            }
        }
        if let Some(limit) = frame.instruction_budget {
            if frame.instructions_executed > limit {
                return Err(self.runtime_error(
                    frame,
                    format!(
                        "Instruction budget of {} exceeded in function '{}'",
                        limit, frame.function
                    ),
                ));
            }
        }
        if let (Some(ms), Some(start)) = (frame.timeout_ms, frame.start_time) {
            if frame.instructions_executed.is_multiple_of(32) && start.elapsed().as_millis() as u64 >= ms {
                return Err(self.runtime_error(
                    frame,
                    format!(
                        "Execution timeout of {}ms exceeded in function '{}'",
                        ms, frame.function
                    ),
                ));
            }
        }
        Ok(())
    }

    fn push(&mut self, frame: &CallFrame, value: Value) -> Result<(), ViceError> {
        if self.stack.len() >= self.config.max_stack_size {
            return Err(self.runtime_error(frame, "Maximum operand stack size exceeded"));
        }
        self.stack.push(value);
        Ok(())
    }

    fn pop(&mut self, frame: &CallFrame) -> Result<Value, ViceError> {
        self.stack
            .pop()
            .ok_or_else(|| self.runtime_error(frame, "Operand stack underflow"))
    }

    fn pop_materialized(&mut self, frame: &CallFrame) -> Result<Value, ViceError> {
        let value = self.pop(frame)?;
        self.materialize(value, frame)
    }

    fn peek(&self, frame: &CallFrame) -> Result<&Value, ViceError> {
        self.stack
            .last()
            .ok_or_else(|| self.runtime_error(frame, "Operand stack underflow"))
    }

    fn pop_many(&mut self, frame: &CallFrame, count: usize) -> Result<Vec<Value>, ViceError> {
        if self.stack.len() < count {
            return Err(self.runtime_error(frame, "Operand stack underflow"));
        }
        let start = self.stack.len() - count;
        Ok(self.stack.drain(start..).collect())
    }

    fn local<'a>(&self, frame: &'a CallFrame, index: usize) -> Result<&'a Slot, ViceError> {
        frame
            .locals
            .get(index)
            .ok_or_else(|| self.runtime_error(frame, "Local slot index is out of bounds"))
    }

    fn constant(&self, frame: &CallFrame, index: usize) -> Result<Value, ViceError> {
        self.function(&frame.function, frame)?
            .chunk
            .constants
            .get(index)
            .cloned()
            .ok_or_else(|| self.runtime_error(frame, "Constant pool index is out of bounds"))
    }

    fn string_constant(&self, frame: &CallFrame, index: usize) -> Result<String, ViceError> {
        match self.constant(frame, index)? {
            Value::String(value) => Ok(value),
            _ => Err(self.runtime_error(frame, "Opcode requires a string constant")),
        }
    }

    fn pop_reference(&mut self, frame: &CallFrame) -> Result<ReferenceValue, ViceError> {
        match self.pop(frame)? {
            Value::Reference(reference) => Ok(reference),
            value => Err(self.runtime_error(
                frame,
                format!("Expected reference, got {}", value.type_name()),
            )),
        }
    }

    fn materialize(&self, mut value: Value, frame: &CallFrame) -> Result<Value, ViceError> {
        for _ in 0..64 {
            match value {
                Value::Reference(reference) => value = self.read_reference(&reference, frame)?,
                _ => return Ok(value),
            }
        }
        Err(self.runtime_error(frame, "Reference cycle detected"))
    }

    fn read_reference(
        &self,
        reference: &ReferenceValue,
        frame: &CallFrame,
    ) -> Result<Value, ViceError> {
        let root = reference
            .root
            .read()
            .map_err(|message| self.runtime_error(frame, message))?;
        if let Value::Reference(mut nested) = root {
            nested.path.extend(reference.path.clone());
            return self.read_reference(&nested, frame);
        }
        let mut value = root;
        for step in &reference.path {
            value = self.materialize(value, frame)?;
            value = match step {
                ReferenceStep::Property(name) => self.get_property(frame, value, name)?,
                ReferenceStep::Index(index) => get_index(&value, index)
                    .map_err(|message| self.runtime_error(frame, message))?,
            };
        }
        Ok(value)
    }

    fn read_reference_root(
        &self,
        reference: &ReferenceValue,
        frame: &CallFrame,
    ) -> Result<Value, ViceError> {
        self.materialize(
            reference
                .root
                .read()
                .map_err(|message| self.runtime_error(frame, message))?,
            frame,
        )
    }

    fn write_reference(
        &self,
        reference: &ReferenceValue,
        value: Value,
        frame: &CallFrame,
    ) -> Result<(), ViceError> {
        let root = reference
            .root
            .read()
            .map_err(|message| self.runtime_error(frame, message))?;
        if let Value::Reference(mut nested) = root {
            nested.path.extend(reference.path.clone());
            return self.write_reference(&nested, value, frame);
        }
        if !reference.mutable {
            return Err(self.runtime_error(frame, "Cannot write through immutable reference"));
        }
        let mut root = root;
        update_path(&mut root, &reference.path, value)
            .map_err(|message| self.runtime_error(frame, message))?;
        reference
            .root
            .write(root)
            .map_err(|message| self.runtime_error(frame, message))
    }

    fn get_property(
        &self,
        frame: &CallFrame,
        entity: Value,
        name: &str,
    ) -> Result<Value, ViceError> {
        match entity {
            Value::Instance {
                data_name, fields, ..
            } => {
                if data_name == "Window" {
                    if name == "is_open" {
                        return Ok(Value::Bool(crate::ui::GLOBAL_UI.is_open()));
                    }
                    if let Some(key) = name.strip_prefix("key_") {
                        return Ok(Value::Bool(crate::ui::GLOBAL_UI.is_key_pressed(key)));
                    }
                }
                fields
                    .try_borrow()
                    .map_err(|_| self.runtime_error(frame, "Instance is already mutably borrowed"))?
                    .get(name)
                    .cloned()
                    .ok_or_else(|| {
                        self.runtime_error(frame, format!("Property '{}' not found", name))
                    })
            }
            other => Err(self.runtime_error(
                frame,
                format!("Cannot get property from {}", other.type_name()),
            )),
        }
    }

    fn unwrap_value(&self, frame: &CallFrame, value: &Value) -> Result<Value, ViceError> {
        let Value::Instance {
            data_name, fields, ..
        } = value
        else {
            return Err(self.runtime_error(frame, "Cannot unwrap non-Result/Option value"));
        };
        let fields = fields
            .try_borrow()
            .map_err(|_| self.runtime_error(frame, "Result/Option is already borrowed"))?;
        match data_name.as_str() {
            "Result" => match fields.get("is_ok") {
                Some(Value::Bool(true)) => fields
                    .get("value")
                    .cloned()
                    .ok_or_else(|| self.runtime_error(frame, "Result has no value field")),
                _ => Err(self.runtime_error(
                    frame,
                    fields
                        .get("error")
                        .map(crate::stdlib::value_to_string)
                        .unwrap_or_else(|| "Called unwrap on Err".to_string()),
                )),
            },
            "Option" => match fields.get("is_some") {
                Some(Value::Bool(true)) => fields
                    .get("value")
                    .cloned()
                    .ok_or_else(|| self.runtime_error(frame, "Option has no value field")),
                _ => Err(self.runtime_error(frame, "Called unwrap on None")),
            },
            _ => Err(self.runtime_error(frame, "Cannot unwrap non-Result/Option value")),
        }
    }

    fn try_value(&self, frame: &CallFrame, value: &Value) -> Result<Option<Value>, ViceError> {
        let Value::Instance {
            data_name, fields, ..
        } = value
        else {
            return Err(self.runtime_error(frame, "Cannot apply ? to non-Result/Option value"));
        };
        let fields = fields
            .try_borrow()
            .map_err(|_| self.runtime_error(frame, "Result/Option is already borrowed"))?;
        let success = match data_name.as_str() {
            "Result" => matches!(fields.get("is_ok"), Some(Value::Bool(true))),
            "Option" => matches!(fields.get("is_some"), Some(Value::Bool(true))),
            _ => return Err(self.runtime_error(frame, "Cannot apply ? to non-Result/Option value")),
        };
        if success {
            Ok(Some(fields.get("value").cloned().unwrap_or(Value::Unit)))
        } else {
            Ok(None)
        }
    }

    fn call_builtin(
        &self,
        frame: &CallFrame,
        name: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, ViceError> {
        let one = || {
            arguments
                .first()
                .cloned()
                .ok_or_else(|| self.runtime_error(frame, format!("{} expects one argument", name)))
        };
        let value = match name {
            "to_str" => Value::String(crate::stdlib::value_to_string(&one()?)),
            "to_int" => Value::Int(to_int(&one()?)),
            "to_float" => Value::Float(to_float(&one()?)),
            "to_num" => match one()? {
                Value::Int(value) => Value::Int(value),
                Value::Float(value) => Value::Float(value),
                Value::String(value) => value
                    .parse::<i64>()
                    .map(Value::Int)
                    .or_else(|_| value.parse::<f64>().map(Value::Float))
                    .unwrap_or(Value::Int(0)),
                Value::Bool(value) => Value::Int(i64::from(value)),
                _ => Value::Int(0),
            },
            "ok" => result_value(true, one()?, Value::Unit),
            "err" => result_value(false, Value::Unit, one()?),
            "some" => option_value(true, one()?),
            "none" => {
                if !arguments.is_empty() {
                    return Err(self.runtime_error(frame, "none expects no arguments"));
                }
                option_value(false, Value::Unit)
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    fn arithmetic(
        &self,
        frame: &CallFrame,
        opcode: Opcode,
        left: Value,
        right: Value,
    ) -> Result<Value, ViceError> {
        if opcode == Opcode::Add {
            if let Value::String(left) = left {
                let right_str = crate::stdlib::value_to_string(&right);
                if left.len().saturating_add(right_str.len()) > MAX_STRING_SIZE {
                    return Err(self.runtime_error(frame, "String size exceeds maximum limit of 64MB"));
                }
                return Ok(Value::String(left + &right_str));
            } else if let Value::String(right) = right {
                let left_str = crate::stdlib::value_to_string(&left);
                if left_str.len().saturating_add(right.len()) > MAX_STRING_SIZE {
                    return Err(self.runtime_error(frame, "String size exceeds maximum limit of 64MB"));
                }
                return Ok(Value::String(left_str + &right));
            }
        }
        match (left, right) {
            (Value::Int(left), Value::Int(right)) => {
                if frame.is_checked {
                    match opcode {
                        Opcode::Add => left
                            .checked_add(right)
                            .map(Value::Int)
                            .ok_or_else(|| self.runtime_error(frame, "Integer overflow in @checked function")),
                        Opcode::Sub => left
                            .checked_sub(right)
                            .map(Value::Int)
                            .ok_or_else(|| self.runtime_error(frame, "Integer overflow in @checked function")),
                        Opcode::Mul => left
                            .checked_mul(right)
                            .map(Value::Int)
                            .ok_or_else(|| self.runtime_error(frame, "Integer overflow in @checked function")),
                        Opcode::Div => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Division by zero"))
                            } else {
                                left.checked_div(right)
                                    .map(Value::Int)
                                    .ok_or_else(|| self.runtime_error(frame, "Integer overflow in @checked function"))
                            }
                        }
                        Opcode::HighLevelDiv => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Division by zero"))
                            } else {
                                Ok(Value::Float(left as f64 / right as f64))
                            }
                        }
                        Opcode::Mod => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Modulo by zero"))
                            } else {
                                left.checked_rem(right)
                                    .map(Value::Int)
                                    .ok_or_else(|| self.runtime_error(frame, "Integer overflow in @checked function"))
                            }
                        }
                        Opcode::Pow if right >= 0 => {
                            if let Ok(exp) = u32::try_from(right) {
                                left.checked_pow(exp)
                                    .map(Value::Int)
                                    .ok_or_else(|| self.runtime_error(frame, "Integer overflow in @checked function"))
                            } else {
                                Err(self.runtime_error(frame, "Integer overflow in @checked function"))
                            }
                        }
                        Opcode::Pow => Ok(Value::Float((left as f64).powf(right as f64))),
                        _ => Err(self.runtime_error(frame, "Invalid integer operation")),
                    }
                } else if frame.is_saturating {
                    match opcode {
                        Opcode::Add => Ok(Value::Int(left.saturating_add(right))),
                        Opcode::Sub => Ok(Value::Int(left.saturating_sub(right))),
                        Opcode::Mul => Ok(Value::Int(left.saturating_mul(right))),
                        Opcode::Div => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Division by zero"))
                            } else if left == i64::MIN && right == -1 {
                                Ok(Value::Int(i64::MAX))
                            } else {
                                Ok(Value::Int(left / right))
                            }
                        }
                        Opcode::HighLevelDiv => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Division by zero"))
                            } else {
                                Ok(Value::Float(left as f64 / right as f64))
                            }
                        }
                        Opcode::Mod => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Modulo by zero"))
                            } else {
                                Ok(Value::Int(left.wrapping_rem(right)))
                            }
                        }
                        Opcode::Pow if right >= 0 => {
                            match u32::try_from(right) {
                                Ok(exp) => Ok(Value::Int(left.saturating_pow(exp))),
                                Err(_) => Ok(Value::Int(if left >= 0 { i64::MAX } else { i64::MIN })),
                            }
                        }
                        Opcode::Pow => Ok(Value::Float((left as f64).powf(right as f64))),
                        _ => Err(self.runtime_error(frame, "Invalid integer operation")),
                    }
                } else {
                    match opcode {
                        Opcode::Add => Ok(Value::Int(left.wrapping_add(right))),
                        Opcode::Sub => Ok(Value::Int(left.wrapping_sub(right))),
                        Opcode::Mul => Ok(Value::Int(left.wrapping_mul(right))),
                        Opcode::Div => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Division by zero"))
                            } else {
                                Ok(Value::Int(left.wrapping_div(right)))
                            }
                        }
                        Opcode::HighLevelDiv => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Division by zero"))
                            } else {
                                Ok(Value::Float(left as f64 / right as f64))
                            }
                        }
                        Opcode::Mod => {
                            if right == 0 {
                                Err(self.runtime_error(frame, "Modulo by zero"))
                            } else {
                                Ok(Value::Int(left.wrapping_rem(right)))
                            }
                        }
                        Opcode::Pow if right >= 0 => Ok(Value::Int(left.wrapping_pow(right as u32))),
                        Opcode::Pow => Ok(Value::Float((left as f64).powf(right as f64))),
                        _ => Err(self.runtime_error(frame, "Invalid integer operation")),
                    }
                }
            }
            (left, right) if numeric(&left) && numeric(&right) => {
                let left = to_float(&left);
                let right = to_float(&right);
                match opcode {
                    Opcode::Add => Ok(Value::Float(left + right)),
                    Opcode::Sub => Ok(Value::Float(left - right)),
                    Opcode::Mul => Ok(Value::Float(left * right)),
                    Opcode::Div | Opcode::HighLevelDiv => {
                        if right == 0.0 {
                            Err(self.runtime_error(frame, "Division by zero"))
                        } else {
                            Ok(Value::Float(left / right))
                        }
                    }
                    Opcode::Mod => {
                        if right == 0.0 {
                            Err(self.runtime_error(frame, "Modulo by zero"))
                        } else {
                            Ok(Value::Float(left % right))
                        }
                    }
                    Opcode::Pow => Ok(Value::Float(left.powf(right))),
                    _ => Err(self.runtime_error(frame, "Invalid numeric operation")),
                }
            }
            (left, right) => Err(self.runtime_error(
                frame,
                format!(
                    "Invalid operands for arithmetic: {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            )),
        }
    }

    fn runtime_error(&self, frame: &CallFrame, message: impl Into<String>) -> ViceError {
        let location = self
            .program
            .as_ref()
            .and_then(|program| program.functions.get(&frame.function))
            .and_then(|function| function.chunk.locations.get(frame.ip.saturating_sub(1)))
            .copied()
            .unwrap_or_else(|| crate::ast::Span::new(self.last_line));
        let file = self
            .program
            .as_ref()
            .and_then(|program| program.sources.get(location.source_id))
            .cloned();
        let mut error = ViceError::at(
            ViceErrorType::Runtime,
            &message.into(),
            file.clone(),
            location.line,
            location.column,
            location.length,
        );
        error.stack_trace.push(StackFrame {
            function: frame.function.clone(),
            file,
            line: location.line,
            column: location.column,
        });
        for caller in self.frames.iter().rev() {
            let caller_location = self
                .program
                .as_ref()
                .and_then(|program| program.functions.get(&caller.function))
                .and_then(|function| function.chunk.locations.get(caller.ip.saturating_sub(1)))
                .copied()
                .unwrap_or_else(|| crate::ast::Span::new(0));
            error.stack_trace.push(StackFrame {
                function: caller.function.clone(),
                file: self
                    .program
                    .as_ref()
                    .and_then(|program| program.sources.get(caller_location.source_id))
                    .cloned(),
                line: caller_location.line,
                column: caller_location.column,
            });
        }
        error
    }

    fn error_for_caller(
        &self,
        caller: Option<&CallFrame>,
        message: impl Into<String>,
    ) -> ViceError {
        match caller {
            Some(frame) => self.runtime_error(frame, message),
            None => ViceError::new(ViceErrorType::Runtime, &message.into(), 0),
        }
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Bool(value) => *value,
        Value::Int(value) => *value != 0,
        Value::Unit => false,
        _ => true,
    }
}

fn numeric(value: &Value) -> bool {
    matches!(value, Value::Int(_) | Value::Float(_))
}

fn to_int(value: &Value) -> i64 {
    match value {
        Value::Int(value) => *value,
        Value::Float(value) => *value as i64,
        Value::String(value) => value.parse().unwrap_or(0),
        Value::Bool(value) => i64::from(*value),
        _ => 0,
    }
}

fn to_float(value: &Value) -> f64 {
    match value {
        Value::Int(value) => *value as f64,
        Value::Float(value) => *value,
        Value::String(value) => value.parse().unwrap_or(0.0),
        Value::Bool(value) => f64::from(*value),
        _ => 0.0,
    }
}

fn compare(opcode: &Opcode, left: &Value, right: &Value) -> Option<bool> {
    if let (Value::String(left), Value::String(right)) = (left, right) {
        return Some(match opcode {
            Opcode::Lt => left < right,
            Opcode::Lte => left <= right,
            Opcode::Gt => left > right,
            Opcode::Gte => left >= right,
            _ => return None,
        });
    }
    let left = match left {
        Value::Int(value) => *value as f64,
        Value::Float(value) => *value,
        _ => return None,
    };
    let right = match right {
        Value::Int(value) => *value as f64,
        Value::Float(value) => *value,
        _ => return None,
    };
    Some(match opcode {
        Opcode::Lt => left < right,
        Opcode::Lte => left <= right,
        Opcode::Gt => left > right,
        Opcode::Gte => left >= right,
        _ => return None,
    })
}

fn get_index(entity: &Value, index: &Value) -> Result<Value, String> {
    match (entity, index) {
        (Value::Array(values), Value::Int(index)) if *index >= 0 => values
            .get(*index as usize)
            .cloned()
            .ok_or_else(|| format!("Array index out of bounds: index {}, length {}", index, values.len())),
        (Value::Map(values), index) => {
            let key =
                MapKey::from_value(index).ok_or_else(|| "Invalid map key type".to_string())?;
            values
                .get(&key)
                .cloned()
                .ok_or_else(|| format!("Map key not found: {}", key))
        }
        (Value::String(value), Value::Int(index)) if *index >= 0 => value
            .chars()
            .nth(*index as usize)
            .map(|character| Value::String(character.to_string()))
            .ok_or_else(|| "String index out of bounds".to_string()),
        _ => Err("Invalid index operation".to_string()),
    }
}

fn set_index(entity: &mut Value, index: &Value, value: Value) -> Result<(), String> {
    match (entity, index) {
        (Value::Array(values), Value::Int(index)) if *index >= 0 => {
            let target = values
                .get_mut(*index as usize)
                .ok_or_else(|| "Array index out of bounds".to_string())?;
            *target = value;
            Ok(())
        }
        (Value::Map(values), index) => {
            values.insert(
                MapKey::from_value(index).ok_or_else(|| "Invalid map key type".to_string())?,
                value,
            );
            Ok(())
        }
        _ => Err("Invalid index assignment".to_string()),
    }
}

fn set_property(entity: &Value, name: &str, value: Value) -> Result<(), String> {
    match entity {
        Value::Instance {
            fields,
            field_types,
            ..
        } => {
            fields
                .try_borrow_mut()
                .map_err(|_| "Instance is already borrowed".to_string())?
                .insert(name.to_string(), coerce_field(value, field_types.get(name)));
            Ok(())
        }
        _ => Err("Cannot set property on non-instance".to_string()),
    }
}

fn update_path(target: &mut Value, path: &[ReferenceStep], value: Value) -> Result<(), String> {
    let Some((first, rest)) = path.split_first() else {
        *target = value;
        return Ok(());
    };
    match first {
        ReferenceStep::Property(name) => match target {
            Value::Instance { fields, .. } => {
                let mut fields = fields
                    .try_borrow_mut()
                    .map_err(|_| "Instance is already borrowed".to_string())?;
                let field = fields
                    .get_mut(name)
                    .ok_or_else(|| format!("Property '{}' not found", name))?;
                update_path(field, rest, value)
            }
            _ => Err("Cannot reference a property on non-instance".to_string()),
        },
        ReferenceStep::Index(index) => match target {
            Value::Array(values) => {
                let Value::Int(index) = index else {
                    return Err("Array index must be int".to_string());
                };
                if *index < 0 {
                    return Err("Array index out of bounds".to_string());
                }
                let item = values
                    .get_mut(*index as usize)
                    .ok_or_else(|| "Array index out of bounds".to_string())?;
                update_path(item, rest, value)
            }
            Value::Map(values) => {
                let key =
                    MapKey::from_value(index).ok_or_else(|| "Invalid map key type".to_string())?;
                let item = values
                    .get_mut(&key)
                    .ok_or_else(|| "Map key not found".to_string())?;
                update_path(item, rest, value)
            }
            _ => Err("Cannot reference an index on this value".to_string()),
        },
    }
}

fn coerce_field(value: Value, expected: Option<&crate::types::Type>) -> Value {
    if matches!(expected, Some(crate::types::Type::Int)) {
        if let Value::Float(value) = value {
            return Value::Int(value as i64);
        }
    }
    value
}

fn sorted_opcode_counts(counts: &HashMap<&'static str, u64>) -> Vec<(String, u64)> {
    let mut values = counts
        .iter()
        .map(|(name, count)| ((*name).to_string(), *count))
        .collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    values
}

fn opcode_name(opcode: &Opcode) -> &'static str {
    match opcode {
        Opcode::LoadConst(_) => "LoadConst",
        Opcode::LoadLocal(_) => "LoadLocal",
        Opcode::StoreLocal(_) => "StoreLocal",
        Opcode::LoadGlobal(_) => "LoadGlobal",
        Opcode::StoreGlobal(_) => "StoreGlobal",
        Opcode::ReferenceLocal(_, _) => "ReferenceLocal",
        Opcode::ReferenceProperty(_) => "ReferenceProperty",
        Opcode::ReferenceIndex => "ReferenceIndex",
        Opcode::Dereference => "Dereference",
        Opcode::AssignReference => "AssignReference",
        Opcode::Add => "Add",
        Opcode::Sub => "Sub",
        Opcode::Mul => "Mul",
        Opcode::Div => "Div",
        Opcode::Mod => "Mod",
        Opcode::Pow => "Pow",
        Opcode::Neg => "Neg",
        Opcode::Eq => "Eq",
        Opcode::Neq => "Neq",
        Opcode::Lt => "Lt",
        Opcode::Lte => "Lte",
        Opcode::Gt => "Gt",
        Opcode::Gte => "Gte",
        Opcode::And => "And",
        Opcode::Or => "Or",
        Opcode::Not => "Not",
        Opcode::Jump(_) => "Jump",
        Opcode::JumpIfFalse(_) => "JumpIfFalse",
        Opcode::JumpIfTrue(_) => "JumpIfTrue",
        Opcode::Loop(_) => "Loop",
        Opcode::Call(_) => "Call",
        Opcode::MakeClosure(_, _) => "MakeClosure",
        Opcode::Return => "Return",
        Opcode::MakeArray(_) => "MakeArray",
        Opcode::MakeMap(_) => "MakeMap",
        Opcode::MakeInstance(_, _) => "MakeInstance",
        Opcode::GetProperty(_) => "GetProperty",
        Opcode::SetProperty(_) => "SetProperty",
        Opcode::GetIndex => "GetIndex",
        Opcode::SetIndex => "SetIndex",
        Opcode::Unwrap => "Unwrap",
        Opcode::Try => "Try",
        Opcode::MethodCall(_, _) => "MethodCall",
        Opcode::NativeCall(_, _, _) => "NativeCall",
        Opcode::Raise(_) => "Raise",
        Opcode::Dup => "Dup",
        Opcode::Pop => "Pop",
        Opcode::Len => "Len",
        Opcode::Output(_) => "Output",
        Opcode::HighLevelDiv => "HighLevelDiv",
        Opcode::TruncateToInt => "TruncateToInt",
        Opcode::Input => "Input",
        Opcode::Swap => "Swap",
        Opcode::Rotate3 => "Rotate3",
    }
}

fn result_value(is_ok: bool, value: Value, error: Value) -> Value {
    Value::Instance {
        data_name: "Result".to_string(),
        field_types: HashMap::new(),
        fields: std::rc::Rc::new(std::cell::RefCell::new(HashMap::from([
            ("is_ok".to_string(), Value::Bool(is_ok)),
            ("value".to_string(), value),
            ("error".to_string(), error),
        ]))),
    }
}

fn option_value(is_some: bool, value: Value) -> Value {
    Value::Instance {
        data_name: "Option".to_string(),
        field_types: HashMap::new(),
        fields: std::rc::Rc::new(std::cell::RefCell::new(HashMap::from([
            ("is_some".to_string(), Value::Bool(is_some)),
            ("value".to_string(), value),
        ]))),
    }
}
