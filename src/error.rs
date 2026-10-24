use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ViceErrorType {
    Syntax,
    Compile,
    Runtime,
    Semantic,
    ReturnEarly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    pub function: String,
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ViceError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub file: Option<String>,
    pub error_type: ViceErrorType,
    pub stack_trace: Vec<StackFrame>,
}

impl ViceError {
    pub fn new(error_type: ViceErrorType, message: &str, line: usize) -> Self {
        Self {
            message: message.to_string(),
            line,
            column: 1,
            length: 1,
            file: None,
            error_type,
            stack_trace: Vec::new(),
        }
    }

    pub fn at(
        error_type: ViceErrorType,
        message: &str,
        file: Option<String>,
        line: usize,
        column: usize,
        length: usize,
    ) -> Self {
        Self {
            message: message.to_string(),
            line,
            column: column.max(1),
            length: length.max(1),
            file,
            error_type,
            stack_trace: Vec::new(),
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_frame(mut self, frame: StackFrame) -> Self {
        self.stack_trace.push(frame);
        self
    }
}

impl fmt::Display for ViceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_str = match self.error_type {
            ViceErrorType::Syntax => "Syntax",
            ViceErrorType::Compile => "Compile",
            ViceErrorType::Runtime => "Runtime",
            ViceErrorType::Semantic => "Semantic",
            ViceErrorType::ReturnEarly => "ReturnEarly",
        };
        if let Some(file) = &self.file {
            write!(
                f,
                "{}:{}:{}: [{} Error] {}",
                file, self.line, self.column, type_str, self.message
            )
        } else {
            write!(
                f,
                "[{} Error] Line {}:{}: {}",
                type_str, self.line, self.column, self.message
            )
        }?;
        for frame in &self.stack_trace {
            write!(
                f,
                "\n  at {} ({}:{}:{})",
                frame.function,
                frame.file.as_deref().unwrap_or("<source>"),
                frame.line,
                frame.column
            )?;
        }
        Ok(())
    }
}
