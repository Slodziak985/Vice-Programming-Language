#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    /// Load a constant from the chunk's constant pool by index
    LoadConst(usize),
    /// Load a local variable by its stack index
    LoadLocal(usize),
    /// Store a local variable by its stack index
    StoreLocal(usize),
    /// Load a global variable by its name's constant index
    LoadGlobal(usize),
    /// Store a global variable by its name's constant index
    StoreGlobal(usize),
    /// Create a reference to a stable local slot.
    ReferenceLocal(usize, bool),
    /// Extend a reference with a property path component.
    ReferenceProperty(usize),
    /// Extend a reference with an index popped from the operand stack.
    ReferenceIndex,
    /// Read through a reference.
    Dereference,
    /// Write a value through a mutable reference.
    AssignReference,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Neg,

    // Logic and Comparison
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
    Not,

    // Control Flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize), // Useful for short-circuit logic
    Loop(usize),       // Jump backwards

    // Functions
    /// Call a function with `argc` arguments
    Call(usize),
    /// Build a closure from a function name and local slots to capture.
    MakeClosure(usize, Vec<usize>),
    /// Return from the current function
    Return,

    // Data structures
    /// Create an array with `count` elements
    MakeArray(usize),
    /// Create a map with `count` key-value pairs
    MakeMap(usize),
    /// Create an instance. `type_id` is the constant index of the type name, `count` is the number of fields.
    MakeInstance(usize, usize),

    // Properties and Indexing
    /// Get a property by name's constant index
    GetProperty(usize),
    /// Set a property by name's constant index
    SetProperty(usize),
    /// Get an element by index (array or map)
    GetIndex,
    /// Set an element by index (array or map)
    SetIndex,

    /// Unpack a Result or Option, panic on error/none
    Unwrap,
    /// Unpack a Result or Option, return from function on error/none
    Try,

    // Method/Native calls
    /// Call a method by name's constant index, with `argc` arguments
    MethodCall(usize, usize),
    /// Call a native function. First is module string index, second is method string index, third is argc
    NativeCall(usize, usize, usize),
    /// Raise a Vice runtime error. Metadata arguments and message are on the stack.
    Raise(usize),

    // Utility
    /// Duplicate the top value on the stack
    Dup,
    /// Pop the top value from the stack
    Pop,
    /// Get length of array or string
    Len,
    /// Emit value to output buffer (for Vice's `emit <- val`) with `count` elements
    Output(usize),
    /// High-level floating-point division
    HighLevelDiv,
    /// Truncate a float on top of stack to an int
    TruncateToInt,
    /// Read from the input buffer
    Input,
    /// Swap the top two values on the stack
    Swap,
    /// Rotate the top three values on the stack (brings the third value to the top)
    Rotate3,
}
