use crate::error::{ViceError, ViceErrorType};
use crate::value::Value;
use crate::vm::compiler::{CompiledFunction, CompiledProgram};
use crate::vm::opcodes::Opcode;
use std::collections::VecDeque;

pub fn verify(program: &CompiledProgram) -> Result<(), ViceError> {
    if !program.entry.is_empty() && !program.functions.contains_key(&program.entry) {
        return Err(ViceError::new(
            ViceErrorType::Compile,
            "Entry function does not exist in compiled program",
            0,
        ));
    }
    for function in program.functions.values() {
        verify_function(program, function)?;
    }
    Ok(())
}

fn verify_function(
    program: &CompiledProgram,
    function: &CompiledFunction,
) -> Result<(), ViceError> {
    let chunk = &function.chunk;
    if chunk.locations.len() != chunk.code.len() {
        return Err(error_at(
            program,
            function,
            0,
            "Bytecode/source location table length mismatch",
        ));
    }
    if function.local_count < function.arity + function.capture_count {
        return Err(error_at(
            program,
            function,
            0,
            "Function metadata has too few local slots",
        ));
    }

    for (ip, opcode) in chunk.code.iter().enumerate() {
        match opcode {
            Opcode::LoadConst(index)
            | Opcode::LoadGlobal(index)
            | Opcode::StoreGlobal(index)
            | Opcode::GetProperty(index)
            | Opcode::SetProperty(index)
            | Opcode::ReferenceProperty(index)
            | Opcode::MethodCall(index, _)
            | Opcode::MakeInstance(index, _) => {
                require_constant(program, function, ip, *index)?;
            }
            Opcode::NativeCall(module, method, _) => {
                require_string_constant(program, function, ip, *module)?;
                require_string_constant(program, function, ip, *method)?;
            }
            Opcode::MakeClosure(name, captures) => {
                let function_name = require_string_constant(program, function, ip, *name)?;
                if !program.functions.contains_key(function_name) {
                    return Err(error_at(
                        program,
                        function,
                        ip,
                        format!("Closure references unknown function '{}'", function_name),
                    ));
                }
                if captures.iter().any(|index| *index >= function.local_count) {
                    return Err(error_at(
                        program,
                        function,
                        ip,
                        "Closure capture index is out of bounds",
                    ));
                }
            }
            Opcode::LoadLocal(index)
            | Opcode::StoreLocal(index)
            | Opcode::ReferenceLocal(index, _)
                if *index >= function.local_count =>
            {
                return Err(error_at(
                    program,
                    function,
                    ip,
                    "Local slot index is out of bounds",
                ));
            }
            Opcode::Jump(target)
            | Opcode::JumpIfFalse(target)
            | Opcode::JumpIfTrue(target)
            | Opcode::Loop(target)
                if *target > chunk.code.len() =>
            {
                return Err(error_at(
                    program,
                    function,
                    ip,
                    "Jump target is out of bounds",
                ));
            }
            _ => {}
        }
    }

    verify_stack(program, function)
}

fn verify_stack(program: &CompiledProgram, function: &CompiledFunction) -> Result<(), ViceError> {
    let code = &function.chunk.code;
    if code.is_empty() {
        return Err(error_at(program, function, 0, "Function has no bytecode"));
    }
    let mut depths = vec![None; code.len() + 1];
    let mut queue = VecDeque::from([(0usize, 0usize)]);
    while let Some((ip, depth)) = queue.pop_front() {
        if ip == code.len() {
            continue;
        }
        if let Some(existing) = depths[ip] {
            if existing != depth {
                return Err(error_at(
                    program,
                    function,
                    ip,
                    "Inconsistent operand stack depth at control-flow merge",
                ));
            }
            continue;
        }
        depths[ip] = Some(depth);
        let (required, delta) = stack_effect(&code[ip]);
        if depth < required {
            return Err(error_at(program, function, ip, "Operand stack underflow"));
        }
        let next_depth = (depth as isize + delta) as usize;
        match &code[ip] {
            Opcode::Return | Opcode::Raise(_) => {}
            Opcode::Jump(target) | Opcode::Loop(target) => queue.push_back((*target, next_depth)),
            Opcode::JumpIfFalse(target) | Opcode::JumpIfTrue(target) => {
                queue.push_back((*target, next_depth));
                queue.push_back((ip + 1, next_depth));
            }
            _ => queue.push_back((ip + 1, next_depth)),
        }
    }
    Ok(())
}

fn stack_effect(opcode: &Opcode) -> (usize, isize) {
    match opcode {
        Opcode::LoadConst(_)
        | Opcode::LoadLocal(_)
        | Opcode::LoadGlobal(_)
        | Opcode::ReferenceLocal(_, _)
        | Opcode::MakeClosure(_, _)
        | Opcode::Input => (0, 1),
        Opcode::StoreLocal(_) | Opcode::StoreGlobal(_) => (1, 0),
        Opcode::ReferenceProperty(_)
        | Opcode::Dereference
        | Opcode::Neg
        | Opcode::Not
        | Opcode::Unwrap
        | Opcode::Try
        | Opcode::GetProperty(_)
        | Opcode::Len
        | Opcode::TruncateToInt => (1, 0),
        Opcode::ReferenceIndex | Opcode::AssignReference | Opcode::GetIndex => (2, -1),
        Opcode::Add
        | Opcode::Sub
        | Opcode::Mul
        | Opcode::Div
        | Opcode::Mod
        | Opcode::Pow
        | Opcode::Eq
        | Opcode::Neq
        | Opcode::Lt
        | Opcode::Lte
        | Opcode::Gt
        | Opcode::Gte
        | Opcode::And
        | Opcode::Or
        | Opcode::HighLevelDiv => (2, -1),
        Opcode::Jump(_) | Opcode::JumpIfFalse(_) | Opcode::JumpIfTrue(_) | Opcode::Loop(_) => {
            (0, 0)
        }
        Opcode::Call(arguments) => (arguments + 1, -(*arguments as isize)),
        Opcode::Return => (1, -1),
        Opcode::MakeArray(count) => (*count, 1 - *count as isize),
        Opcode::MakeMap(count) | Opcode::MakeInstance(_, count) => {
            (count * 2, 1 - (count * 2) as isize)
        }
        Opcode::SetProperty(_) => (2, -1),
        Opcode::SetIndex => (3, -2),
        Opcode::MethodCall(_, arguments) => (arguments + 1, -(*arguments as isize)),
        Opcode::NativeCall(_, _, arguments) => (*arguments, 1 - *arguments as isize),
        Opcode::Raise(arguments) => (arguments + 1, -((*arguments + 1) as isize)),
        Opcode::Dup => (1, 1),
        Opcode::Pop => (1, -1),
        Opcode::Output(count) => (*count, -(*count as isize)),
        Opcode::Swap => (2, 0),
        Opcode::Rotate3 => (3, 0),
    }
}

fn require_constant(
    program: &CompiledProgram,
    function: &CompiledFunction,
    ip: usize,
    index: usize,
) -> Result<(), ViceError> {
    if index >= function.chunk.constants.len() {
        return Err(error_at(
            program,
            function,
            ip,
            "Constant pool index is out of bounds",
        ));
    }
    Ok(())
}

fn require_string_constant<'a>(
    program: &CompiledProgram,
    function: &'a CompiledFunction,
    ip: usize,
    index: usize,
) -> Result<&'a str, ViceError> {
    require_constant(program, function, ip, index)?;
    match &function.chunk.constants[index] {
        Value::String(value) => Ok(value),
        _ => Err(error_at(
            program,
            function,
            ip,
            "Opcode requires a string constant",
        )),
    }
}

fn error_at(
    program: &CompiledProgram,
    function: &CompiledFunction,
    ip: usize,
    message: impl Into<String>,
) -> ViceError {
    let location = function
        .chunk
        .locations
        .get(ip)
        .copied()
        .unwrap_or_else(|| crate::ast::Span::new(0).in_source(function.chunk.source_id));
    ViceError::at(
        ViceErrorType::Compile,
        &format!(
            "Invalid bytecode in '{}': {}",
            function.name,
            message.into()
        ),
        program.sources.get(location.source_id).cloned(),
        location.line,
        location.column,
        location.length,
    )
}
