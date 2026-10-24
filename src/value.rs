use crate::ast::*;
use crate::error::ViceError;
use crate::error::ViceErrorType;
use crate::types::Type;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub struct Slot(Rc<RefCell<Value>>);

impl Slot {
    pub fn new(value: Value) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn read(&self) -> Result<Value, String> {
        self.0
            .try_borrow()
            .map(|value| value.clone())
            .map_err(|_| "Reference is already mutably borrowed".to_string())
    }

    pub fn write(&self, value: Value) -> Result<(), String> {
        let mut target = self
            .0
            .try_borrow_mut()
            .map_err(|_| "Reference is already borrowed".to_string())?;
        *target = value;
        Ok(())
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Slot")
            .field(&(Rc::as_ptr(&self.0) as usize))
            .finish()
    }
}

impl PartialEq for Slot {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MapKey {
    String(String),
    Int(i64),
    Float(u64),
    Bool(bool),
}

impl MapKey {
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::String(value) => Some(Self::String(value.clone())),
            Value::Int(value) => Some(Self::Int(*value)),
            Value::Float(value) => {
                let normalized = if value.is_nan() {
                    f64::NAN.to_bits()
                } else if *value == 0.0 {
                    0.0f64.to_bits()
                } else {
                    value.to_bits()
                };
                Some(Self::Float(normalized))
            }
            Value::Bool(value) => Some(Self::Bool(*value)),
            _ => None,
        }
    }

    pub fn to_source_string(&self) -> String {
        match self {
            Self::String(value) => format!("\"{}\"", value),
            _ => self.to_string(),
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Int(value) => Value::Int(*value),
            Self::Float(bits) => Value::Float(f64::from_bits(*bits)),
            Self::Bool(value) => Value::Bool(*value),
        }
    }
}

impl fmt::Display for MapKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => write!(formatter, "{}", value),
            Self::Int(value) => write!(formatter, "{}", value),
            Self::Float(bits) => write!(formatter, "{}", f64::from_bits(*bits)),
            Self::Bool(value) => write!(formatter, "{}", value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceStep {
    Property(String),
    Index(Value),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceValue {
    pub root: Slot,
    pub path: Vec<ReferenceStep>,
    pub mutable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Function {
        attributes: Vec<Attribute>,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },
    DataDefinition {
        attributes: Vec<Attribute>,
        fields: Vec<DataField>,
    },
    Instance {
        data_name: String,
        field_types: HashMap<String, Type>,
        fields: std::rc::Rc<std::cell::RefCell<HashMap<String, Value>>>,
    },
    Module(String),
    Map(HashMap<MapKey, Value>),
    Callable(String),
    Closure {
        function: String,
        captures: Vec<Slot>,
    },
    Reference(ReferenceValue),
    EnumVariant {
        enum_name: String,
        variant_name: String,
        fields: Vec<Value>,
    },
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unit => "unit",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Array(_) => "array",
            Value::Function { .. } => "function",
            Value::DataDefinition { .. } => "data",
            Value::Instance { .. } => "instance",
            Value::Module(_) => "module",
            Value::Map(_) => "map",
            Value::Callable(_) => "function",
            Value::Closure { .. } => "closure",
            Value::Reference(_) => "reference",
            Value::EnumVariant { .. } => "enum_variant",
        }
    }

    pub fn as_bool(&self, line: usize) -> Result<bool, ViceError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(ViceError::new(
                ViceErrorType::Runtime,
                "Condition must be a boolean value",
                line,
            )),
        }
    }

    pub fn compare_order(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Array(a), Value::Array(b)) => {
                let len_cmp = a.len().cmp(&b.len());
                if len_cmp != std::cmp::Ordering::Equal {
                    return len_cmp;
                }
                for (x, y) in a.iter().zip(b.iter()) {
                    let cmp = x.compare_order(y);
                    if cmp != std::cmp::Ordering::Equal {
                        return cmp;
                    }
                }
                std::cmp::Ordering::Equal
            }
            _ => self.type_name().cmp(other.type_name()),
        }
    }
}
