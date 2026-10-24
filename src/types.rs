use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridMode {
    Default,
    LowLevel,
    HighLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Array(Box<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Data(String, Vec<Type>),
    Generic(String),
    Module(String),
    Map(Box<Type>, Box<Type>),
    Enum(String),
    Reference {
        mutable: bool,
        inner: Box<Type>,
    },
    Unknown,
}

impl Type {
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        if self == other || matches!(self, Type::Unknown) || matches!(other, Type::Unknown) {
            return true;
        }
        match (self, other) {
            (Type::Array(a), Type::Array(b)) => a.is_compatible_with(b),
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                k1.is_compatible_with(k2) && v1.is_compatible_with(v2)
            }
            (Type::Data(n1, t1), Type::Data(n2, t2)) => {
                if n1 != n2 {
                    return false;
                }
                if t1.is_empty() || t2.is_empty() {
                    return true;
                }
                if t1.len() != t2.len() {
                    return false;
                }
                for (a, b) in t1.iter().zip(t2.iter()) {
                    if !a.is_compatible_with(b) {
                        return false;
                    }
                }
                true
            }
            (Type::Enum(n1), Type::Data(n2, _)) | (Type::Data(n1, _), Type::Enum(n2)) => n1 == n2,
            (Type::Generic(_), _) | (_, Type::Generic(_)) => true,
            (
                Type::Reference {
                    mutable: m1,
                    inner: i1,
                },
                Type::Reference {
                    mutable: m2,
                    inner: i2,
                },
            ) => m1 == m2 && i1.is_compatible_with(i2),
            (Type::Reference { inner, .. }, other) => inner.is_compatible_with(other),
            (other, Type::Reference { inner, .. }) => other.is_compatible_with(inner),
            _ => false,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "int" => Type::Int,
            "float" => Type::Float,
            "bool" => Type::Bool,
            "string" => Type::String,
            "void" => Type::Void,
            "array" | "Array" => Type::Array(Box::new(Type::Unknown)),
            "map" | "Map" => Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown)),
            "unknown" | "any" => Type::Unknown,
            other => Type::Data(other.to_string(), Vec::new()),
            // Built-in modules are defined directly in the semantic analyzer's global scope.
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),
            Type::Array(inner) => write!(f, "{}[]", inner),
            Type::Function {
                params,
                return_type,
            } => {
                let params = params
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "fn({}) -> {}", params, return_type)
            }
            Type::Data(name, type_args) => {
                if type_args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    let args = type_args
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(f, "{}<{}>", name, args)
                }
            }
            Type::Generic(name) => write!(f, "{}", name),
            Type::Enum(name) => write!(f, "enum {}", name),
            Type::Module(name) => write!(f, "module {}", name),
            Type::Map(k, v) => write!(f, "map[{}, {}]", k, v),
            Type::Float => write!(f, "float"),
            Type::Reference { mutable, inner } => {
                if *mutable {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            Type::Unknown => write!(f, "unknown"),
        }
    }
}
