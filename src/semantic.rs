use crate::ast::*;
use crate::error::{ViceError, ViceErrorType};
use crate::stdlib;
use crate::types::{HybridMode, Type};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub _name: String,
    pub ty: Type,
    pub mutable: bool,
    pub mode: HybridMode,
}

#[derive(Debug, Clone)]
struct DataDef {
    fields: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
struct TraitDefinition {
    type_params: Vec<String>,
    methods: HashMap<String, TraitMethodDef>,
}

pub struct SemanticAnalyzer {
    pub registry: crate::native_registry::NativeRegistry,
    scopes: Vec<HashMap<String, Symbol>>,
    data_defs: HashMap<String, DataDef>,
    enum_defs: HashMap<String, Vec<String>>,
    trait_defs: HashMap<String, TraitDefinition>,
    trait_impls: HashSet<(String, String)>,
    impls: HashMap<String, HashMap<String, Type>>,
    current_return_type: Option<Type>,
    loop_depth: usize,
    module_autocheck: bool,
    current_autocheck: bool,
    module_mode: HybridMode,
    current_mode: HybridMode,
    pub in_read_only_function: bool,
    pub current_function_params: std::collections::HashSet<String>,
    pub read_only_functions: std::collections::HashSet<String>,
    pub read_only_datas: std::collections::HashSet<String>,
    pub read_only_enums: std::collections::HashSet<String>,
    pub in_pure_function: bool,
    pub pure_functions: std::collections::HashSet<String>,
    pub in_safe_function: bool,
    pub safe_functions: std::collections::HashSet<String>,
    pub no_inline_functions: std::collections::HashSet<String>,
    pub cold_functions: std::collections::HashSet<String>,
    pub hot_functions: std::collections::HashSet<String>,
    pub strict_functions: std::collections::HashSet<String>,
    pub in_strict_function: bool,
    pub nonreentrant_functions: std::collections::HashSet<String>,
    pub must_use_functions: std::collections::HashMap<String, Option<String>>,
    pub must_use_types: std::collections::HashMap<String, Option<String>>,
    pub deprecated_items: std::collections::HashMap<String, Option<String>>,
    pub tailrec_functions: std::collections::HashSet<String>,
    pub inlined_functions: std::collections::HashSet<String>,
    pub test_functions: std::collections::HashSet<String>,
    pub const_functions: std::collections::HashSet<String>,
    pub in_const_function: bool,
    pub no_panic_functions: std::collections::HashSet<String>,
    pub in_no_panic_function: bool,
    pub sealed_traits: std::collections::HashMap<String, usize>,
    pub record_datas: std::collections::HashSet<String>,
    pub fallible_functions: std::collections::HashSet<String>,
    pub infallible_functions: std::collections::HashSet<String>,
    pub in_infallible_function: bool,
    pub deterministic_functions: std::collections::HashSet<String>,
    pub in_deterministic_function: bool,
    pub idempotent_functions: std::collections::HashSet<String>,
    pub pure_datas: std::collections::HashSet<String>,
    pub final_datas: std::collections::HashSet<String>,
    pub unreachable_functions: std::collections::HashSet<String>,
    pub benchmark_functions: std::collections::HashSet<String>,
    pub checked_functions: std::collections::HashSet<String>,
    pub saturating_functions: std::collections::HashSet<String>,
    pub zero_alloc_functions: std::collections::HashSet<String>,
    pub in_zero_alloc_function: bool,
    pub in_isolated_function: bool,
    pub isolated_functions: std::collections::HashSet<String>,
    pub in_no_float_function: bool,
    pub no_float_functions: std::collections::HashSet<String>,
    pub immutable_datas: std::collections::HashSet<String>,
    pub audit_functions: std::collections::HashSet<String>,
    pub rate_limited_functions: std::collections::HashMap<String, usize>,
    pub timeout_functions: std::collections::HashMap<String, u64>,
    pub singleton_datas: std::collections::HashSet<String>,
    pub singleton_instantiations: std::cell::RefCell<std::collections::HashMap<String, usize>>,
    pub profiled_functions: std::collections::HashSet<String>,
    pub retry_functions: std::collections::HashMap<String, usize>,
    pub current_function_name: Option<String>,
    pub in_test_function: bool,
    pub test_only_functions: std::collections::HashSet<String>,
    pub no_recurse_functions: std::collections::HashSet<String>,
    pub obsolete_items: std::collections::HashMap<String, Option<String>>,
    pub sealed_datas: std::collections::HashMap<String, usize>,
    pub sealed_enums: std::collections::HashMap<String, usize>,
    pub in_leaf_function: bool,
    pub leaf_functions: std::collections::HashSet<String>,
    pub in_nothrow_function: bool,
    pub nothrow_functions: std::collections::HashSet<String>,
    pub in_no_io_function: bool,
    pub no_io_functions: std::collections::HashSet<String>,
    pub wrapping_functions: std::collections::HashSet<String>,
    pub non_exhaustive_enums: std::collections::HashSet<String>,
    pub private_functions: std::collections::HashMap<String, usize>,
    pub private_datas: std::collections::HashMap<String, usize>,
    pub private_enums: std::collections::HashMap<String, usize>,
    pub packed_datas: std::collections::HashSet<String>,
    pub align_datas: std::collections::HashMap<String, usize>,
    pub exported_items: std::collections::HashSet<String>,
    pub experimental_items: std::collections::HashSet<String>,
    pub stable_items: std::collections::HashSet<String>,
    pub benchmark_only_functions: std::collections::HashSet<String>,
    pub in_benchmark_function: bool,
    pub in_experimental_function: bool,
    pub since_items: std::collections::HashMap<String, String>,
    pub must_mutate_functions: std::collections::HashSet<String>,
    pub transparent_datas: std::collections::HashSet<String>,
    pub security_critical_functions: std::collections::HashSet<String>,
    pub inline_always_functions: std::collections::HashSet<String>,
    pub const_time_functions: std::collections::HashSet<String>,
    pub reentrant_functions: std::collections::HashSet<String>,
    pub must_initialize_datas: std::collections::HashSet<String>,
    pub simd_functions: std::collections::HashSet<String>,
    pub in_simd_function: bool,
    pub in_idempotent_function: bool,
    pub thread_safe_functions: std::collections::HashSet<String>,
    pub in_thread_safe_function: bool,
    pub pure_math_functions: std::collections::HashSet<String>,
    pub in_pure_math_function: bool,
    pub clamped_functions: std::collections::HashMap<String, (f64, f64)>,
    pub non_empty_functions: std::collections::HashSet<String>,
    pub bitwise_only_functions: std::collections::HashSet<String>,
    pub in_bitwise_only_function: bool,
    pub no_alias_functions: std::collections::HashSet<String>,
    pub pure_string_functions: std::collections::HashSet<String>,
    pub in_pure_string_function: bool,
    pub non_negative_functions: std::collections::HashSet<String>,
    pub positive_functions: std::collections::HashSet<String>,
    pub bounded_functions: std::collections::HashMap<String, (f64, f64)>,
    pub ascii_only_functions: std::collections::HashSet<String>,
    pub sorted_functions: std::collections::HashSet<String>,
    pub distinct_functions: std::collections::HashSet<String>,
    pub pure_array_functions: std::collections::HashSet<String>,
    pub in_pure_array_function: bool,
    current_function_mutated_params: std::collections::HashSet<String>,
    pub warnings: std::cell::RefCell<Vec<String>>,
    source_files: Vec<String>,
    current_source: Option<String>,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let registry = crate::native_registry::NativeRegistry::new();
        let mut globals = HashMap::new();
        for module in stdlib::BUILTIN_MODULES {
            globals.insert(
                module.to_string(),
                Symbol {
                    _name: module.to_string(),
                    ty: Type::Module(module.to_string()),
                    mutable: false,
                    mode: HybridMode::Default,
                },
            );
        }

        let mut data_defs = HashMap::new();

        // Wbudowane typy opcjonalne i błędów
        data_defs.insert(
            "Result".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("is_ok".to_string(), Type::Bool),
                    ("value".to_string(), Type::Generic("T".to_string())),
                    ("error".to_string(), Type::Generic("E".to_string())),
                ]),
            },
        );

        data_defs.insert(
            "Option".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("is_some".to_string(), Type::Bool),
                    ("value".to_string(), Type::Generic("T".to_string())),
                ]),
            },
        );

        data_defs.insert(
            "Error".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("message".to_string(), Type::String),
                    ("code".to_string(), Type::Int),
                ]),
            },
        );

        data_defs.insert(
            "Handle".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("id".to_string(), Type::Int),
                    ("path".to_string(), Type::String),
                ]),
            },
        );

        data_defs.insert(
            "HttpResponse".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("status".to_string(), Type::Int),
                    ("body".to_string(), Type::String),
                    ("ok".to_string(), Type::Bool),
                ]),
            },
        );

        data_defs.insert(
            "Matrix".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("rows".to_string(), Type::Int),
                    ("cols".to_string(), Type::Int),
                    ("data".to_string(), Type::Array(Box::new(Type::Float))),
                ]),
            },
        );

        // Rejestracja wbudowanych typów UI, aby checker znał ich pola
        data_defs.insert(
            "Window".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("is_open".to_string(), Type::Bool),
                    ("width".to_string(), Type::Int),
                    ("height".to_string(), Type::Int),
                    ("dpi".to_string(), Type::Float),
                ]),
            },
        );

        globals.insert(
            "Result".to_string(),
            Symbol {
                _name: "Result".to_string(),
                ty: Type::Function {
                    params: vec![
                        Type::Bool,
                        Type::Generic("T".to_string()),
                        Type::Generic("E".to_string()),
                    ],
                    return_type: Box::new(Type::Data(
                        "Result".to_string(),
                        vec![
                            Type::Generic("T".to_string()),
                            Type::Generic("E".to_string()),
                        ],
                    )),
                },
                mutable: false,
                mode: HybridMode::Default,
            },
        );

        globals.insert(
            "Option".to_string(),
            Symbol {
                _name: "Option".to_string(),
                ty: Type::Function {
                    params: vec![Type::Bool, Type::Generic("T".to_string())],
                    return_type: Box::new(Type::Data(
                        "Option".to_string(),
                        vec![Type::Generic("T".to_string())],
                    )),
                },
                mutable: false,
                mode: HybridMode::Default,
            },
        );

        globals.insert(
            "Error".to_string(),
            Symbol {
                _name: "Error".to_string(),
                ty: Type::Function {
                    params: vec![Type::String, Type::Int],
                    return_type: Box::new(Type::Data("Error".to_string(), vec![])),
                },
                mutable: false,
                mode: HybridMode::Default,
            },
        );
        data_defs.insert(
            "UiResponse".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("clicked".to_string(), Type::Bool),
                    ("hovered".to_string(), Type::Bool),
                    ("active".to_string(), Type::Bool),
                    ("focused".to_string(), Type::Bool),
                    ("changed".to_string(), Type::Bool),
                ]),
            },
        );

        data_defs.insert(
            "TextInputResponse".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("value".to_string(), Type::String),
                    ("changed".to_string(), Type::Bool),
                    ("submitted".to_string(), Type::Bool),
                    ("focused".to_string(), Type::Bool),
                ]),
            },
        );

        data_defs.insert(
            "Canvas".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("id".to_string(), Type::String),
                    ("width".to_string(), Type::Int),
                    ("height".to_string(), Type::Int),
                ]),
            },
        );

        data_defs.insert(
            "Theme".to_string(),
            DataDef {
                fields: HashMap::from([("name".to_string(), Type::String)]),
            },
        );

        data_defs.insert(
            "UiEvent".to_string(),
            DataDef {
                fields: HashMap::from([
                    ("kind".to_string(), Type::String),
                    ("id".to_string(), Type::Int),
                    ("key".to_string(), Type::String),
                    ("x".to_string(), Type::Float),
                    ("y".to_string(), Type::Float),
                ]),
            },
        );

        Self {
            scopes: vec![globals],
            data_defs,
            enum_defs: HashMap::new(),
            trait_defs: HashMap::new(),
            trait_impls: HashSet::new(),
            impls: HashMap::new(),
            current_return_type: None,
            loop_depth: 0,
            module_autocheck: false,
            current_autocheck: false,
            module_mode: HybridMode::Default,
            current_mode: HybridMode::Default,
            registry,
            in_read_only_function: false,
            current_function_params: std::collections::HashSet::new(),
            read_only_functions: std::collections::HashSet::new(),
            read_only_datas: std::collections::HashSet::new(),
            read_only_enums: std::collections::HashSet::new(),
            in_pure_function: false,
            pure_functions: std::collections::HashSet::new(),
            in_safe_function: false,
            safe_functions: std::collections::HashSet::new(),
            no_inline_functions: std::collections::HashSet::new(),
            cold_functions: std::collections::HashSet::new(),
            hot_functions: std::collections::HashSet::new(),
            strict_functions: std::collections::HashSet::new(),
            in_strict_function: false,
            nonreentrant_functions: std::collections::HashSet::new(),
            must_use_functions: std::collections::HashMap::new(),
            must_use_types: std::collections::HashMap::new(),
            deprecated_items: std::collections::HashMap::new(),
            tailrec_functions: std::collections::HashSet::new(),
            inlined_functions: std::collections::HashSet::new(),
            test_functions: std::collections::HashSet::new(),
            const_functions: std::collections::HashSet::new(),
            in_const_function: false,
            no_panic_functions: std::collections::HashSet::new(),
            in_no_panic_function: false,
            sealed_traits: std::collections::HashMap::new(),
            record_datas: std::collections::HashSet::new(),
            fallible_functions: std::collections::HashSet::new(),
            infallible_functions: std::collections::HashSet::new(),
            in_infallible_function: false,
            deterministic_functions: std::collections::HashSet::new(),
            in_deterministic_function: false,
            idempotent_functions: std::collections::HashSet::new(),
            pure_datas: std::collections::HashSet::new(),
            final_datas: std::collections::HashSet::new(),
            unreachable_functions: std::collections::HashSet::new(),
            benchmark_functions: std::collections::HashSet::new(),
            checked_functions: std::collections::HashSet::new(),
            saturating_functions: std::collections::HashSet::new(),
            zero_alloc_functions: std::collections::HashSet::new(),
            in_zero_alloc_function: false,
            in_isolated_function: false,
            isolated_functions: std::collections::HashSet::new(),
            in_no_float_function: false,
            no_float_functions: std::collections::HashSet::new(),
            immutable_datas: std::collections::HashSet::new(),
            audit_functions: std::collections::HashSet::new(),
            rate_limited_functions: std::collections::HashMap::new(),
            timeout_functions: std::collections::HashMap::new(),
            singleton_datas: std::collections::HashSet::new(),
            singleton_instantiations: std::cell::RefCell::new(std::collections::HashMap::new()),
            profiled_functions: std::collections::HashSet::new(),
            retry_functions: std::collections::HashMap::new(),
            current_function_name: None,
            in_test_function: false,
            test_only_functions: std::collections::HashSet::new(),
            no_recurse_functions: std::collections::HashSet::new(),
            obsolete_items: std::collections::HashMap::new(),
            sealed_datas: std::collections::HashMap::new(),
            sealed_enums: std::collections::HashMap::new(),
            in_leaf_function: false,
            leaf_functions: std::collections::HashSet::new(),
            in_nothrow_function: false,
            nothrow_functions: std::collections::HashSet::new(),
            in_no_io_function: false,
            no_io_functions: std::collections::HashSet::new(),
            wrapping_functions: std::collections::HashSet::new(),
            non_exhaustive_enums: std::collections::HashSet::new(),
            private_functions: std::collections::HashMap::new(),
            private_datas: std::collections::HashMap::new(),
            private_enums: std::collections::HashMap::new(),
            packed_datas: std::collections::HashSet::new(),
            align_datas: std::collections::HashMap::new(),
            exported_items: std::collections::HashSet::new(),
            experimental_items: std::collections::HashSet::new(),
            stable_items: std::collections::HashSet::new(),
            benchmark_only_functions: std::collections::HashSet::new(),
            in_benchmark_function: false,
            in_experimental_function: false,
            since_items: std::collections::HashMap::new(),
            must_mutate_functions: std::collections::HashSet::new(),
            transparent_datas: std::collections::HashSet::new(),
            security_critical_functions: std::collections::HashSet::new(),
            inline_always_functions: std::collections::HashSet::new(),
            const_time_functions: std::collections::HashSet::new(),
            reentrant_functions: std::collections::HashSet::new(),
            must_initialize_datas: std::collections::HashSet::new(),
            simd_functions: std::collections::HashSet::new(),
            in_simd_function: false,
            in_idempotent_function: false,
            thread_safe_functions: std::collections::HashSet::new(),
            in_thread_safe_function: false,
            pure_math_functions: std::collections::HashSet::new(),
            in_pure_math_function: false,
            clamped_functions: std::collections::HashMap::new(),
            non_empty_functions: std::collections::HashSet::new(),
            bitwise_only_functions: std::collections::HashSet::new(),
            in_bitwise_only_function: false,
            no_alias_functions: std::collections::HashSet::new(),
            pure_string_functions: std::collections::HashSet::new(),
            in_pure_string_function: false,
            non_negative_functions: std::collections::HashSet::new(),
            positive_functions: std::collections::HashSet::new(),
            bounded_functions: std::collections::HashMap::new(),
            ascii_only_functions: std::collections::HashSet::new(),
            sorted_functions: std::collections::HashSet::new(),
            distinct_functions: std::collections::HashSet::new(),
            pure_array_functions: std::collections::HashSet::new(),
            in_pure_array_function: false,
            current_function_mutated_params: std::collections::HashSet::new(),
            warnings: std::cell::RefCell::new(Vec::new()),
            source_files: Vec::new(),
            current_source: None,
        }
    }

    pub fn record_warning(&self, warning: impl Into<String>) {
        self.warnings.borrow_mut().push(warning.into());
    }

    pub fn get_warnings(&self) -> Vec<String> {
        self.warnings.borrow().clone()
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), ViceError> {
        self.source_files = program.sources.clone();
        // Determine module-level @autocheck: if the first non-import top-level element has @autocheck, treat as module default
        let mut module_autocheck = false;
        for top in &program.top_level {
            match top {
                TopLevel::EnumDef(_) => continue,
                TopLevel::TraitDef(_) => continue,
                TopLevel::Impl(_) => continue,
                TopLevel::Import(_) => continue,
                TopLevel::Function(f) => {
                    if f.attributes.iter().any(|a| a.name == "autocheck") {
                        module_autocheck = true;
                    }
                    break;
                }
                TopLevel::Data(stmt) => {
                    if let StmtKind::DataDeclaration { attributes, .. } = &stmt.kind {
                        if attributes.iter().any(|a| a.name == "autocheck") {
                            module_autocheck = true;
                        }
                    }
                    break;
                }
            }
        }
        self.module_autocheck = module_autocheck;
        self.module_mode = Self::module_mode(program);
        self.current_mode = self.module_mode;

        for item in &program.top_level {
            if let TopLevel::EnumDef(stmt) = item {
                if let StmtKind::EnumDef {
                    name,
                    variants,
                    attributes,
                } = &stmt.kind
                {
                    self.enum_defs.insert(name.clone(), variants.clone());
                    if attributes.iter().any(|a| a.name == "read") {
                        self.read_only_enums.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "non_exhaustive") {
                        self.non_exhaustive_enums.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "sealed") {
                        self.sealed_enums.insert(name.clone(), stmt.span.source_id);
                    }
                    if attributes.iter().any(|a| a.name == "private" || a.name == "internal") {
                        self.private_enums.insert(name.clone(), stmt.span.source_id);
                    }
                    let is_export = attributes.iter().any(|a| a.name == "export" || a.name == "public");
                    let is_private = attributes.iter().any(|a| a.name == "private" || a.name == "internal");
                    if is_export && is_private {
                        return Err(self.error_at_span(
                            stmt.span,
                            &format!("Enum '{}' cannot be marked with both @export and @private", name),
                        ));
                    }
                    if is_export {
                        self.exported_items.insert(name.clone());
                    }
                    let is_experimental = attributes.iter().any(|a| a.name == "experimental");
                    let is_stable = attributes.iter().any(|a| a.name == "stable");
                    let is_deprecated = attributes.iter().any(|a| a.name == "deprecated");
                    let is_obsolete = attributes.iter().any(|a| a.name == "obsolete");
                    if is_experimental && is_stable {
                        return Err(self.error_at_span(
                            stmt.span,
                            &format!("Enum '{}' cannot be marked with both @experimental and @stable", name),
                        ));
                    }
                    if is_experimental && (is_deprecated || is_obsolete) {
                        return Err(self.error_at_span(
                            stmt.span,
                            &format!("Enum '{}' cannot be marked with both @experimental and @deprecated", name),
                        ));
                    }
                    if is_stable && (is_deprecated || is_obsolete) {
                        return Err(self.error_at_span(
                            stmt.span,
                            &format!("Enum '{}' cannot be marked with both @stable and @deprecated", name),
                        ));
                    }
                    if is_experimental {
                        self.experimental_items.insert(name.clone());
                    }
                    if is_stable {
                        self.stable_items.insert(name.clone());
                    }
                    if let Some(attr) = attributes.iter().find(|a| a.name == "since") {
                        let version = extract_attribute_version(attr).ok_or_else(|| {
                            self.error_at_span(
                                attr.span,
                                &format!("@since attribute on enum '{}' requires a version string or number", name),
                            )
                        })?;
                        self.since_items.insert(name.clone(), version);
                    }
                    if let Some(attr) = attributes.iter().find(|a| a.name == "deprecated") {
                        self.deprecated_items.insert(name.clone(), extract_attribute_message(attr));
                    }
                    if let Some(attr) = attributes.iter().find(|a| a.name == "obsolete") {
                        self.obsolete_items.insert(name.clone(), extract_attribute_message(attr));
                    }
                }
            } else if let TopLevel::Data(stmt) = item {
                if let StmtKind::DataDeclaration {
                    name, attributes, ..
                } = &stmt.kind
                {
                    self.data_defs.insert(
                        name.clone(),
                        DataDef {
                            fields: HashMap::new(),
                        },
                    );
                    if attributes.iter().any(|a| a.name == "read" || a.name == "immutable" || a.name == "immut" || a.name == "record") {
                        self.read_only_datas.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "immutable" || a.name == "immut") {
                        self.immutable_datas.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "record") {
                        self.record_datas.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "pure_data" || a.name == "value_object") {
                        self.pure_datas.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "final") {
                        self.final_datas.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "singleton") {
                        self.singleton_datas.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "sealed") {
                        self.sealed_datas.insert(name.clone(), stmt.span.source_id);
                    }
                    if attributes.iter().any(|a| a.name == "private" || a.name == "internal") {
                        self.private_datas.insert(name.clone(), stmt.span.source_id);
                    }
                    let is_export = attributes.iter().any(|a| a.name == "export" || a.name == "public");
                    let is_private = attributes.iter().any(|a| a.name == "private" || a.name == "internal");
                    if is_export && is_private {
                        return Err(self.error_at_span(
                            stmt.span,
                            &format!("Data type '{}' cannot be marked with both @export and @private", name),
                        ));
                    }
                    if is_export {
                        self.exported_items.insert(name.clone());
                    }
                    let is_experimental = attributes.iter().any(|a| a.name == "experimental");
                    let is_stable = attributes.iter().any(|a| a.name == "stable");
                    let is_deprecated = attributes.iter().any(|a| a.name == "deprecated");
                    let is_obsolete = attributes.iter().any(|a| a.name == "obsolete");
                    if is_experimental && is_stable {
                        return Err(self.error_at_span(
                            stmt.span,
                            &format!("Data type '{}' cannot be marked with both @experimental and @stable", name),
                        ));
                    }
                    if is_experimental && (is_deprecated || is_obsolete) {
                        return Err(self.error_at_span(
                            stmt.span,
                            &format!("Data type '{}' cannot be marked with both @experimental and @deprecated", name),
                        ));
                    }
                    if is_stable && (is_deprecated || is_obsolete) {
                        return Err(self.error_at_span(
                            stmt.span,
                            &format!("Data type '{}' cannot be marked with both @stable and @deprecated", name),
                        ));
                    }
                    if is_experimental {
                        self.experimental_items.insert(name.clone());
                    }
                    if is_stable {
                        self.stable_items.insert(name.clone());
                    }
                    if let Some(attr) = attributes.iter().find(|a| a.name == "since") {
                        let version = extract_attribute_version(attr).ok_or_else(|| {
                            self.error_at_span(
                                attr.span,
                                &format!("@since attribute on struct '{}' requires a version string or number", name),
                            )
                        })?;
                        self.since_items.insert(name.clone(), version);
                    }
                    if attributes.iter().any(|a| a.name == "packed" || a.name == "flat") {
                        self.packed_datas.insert(name.clone());
                    }
                    if attributes.iter().any(|a| a.name == "transparent") {
                        self.transparent_datas.insert(name.clone());
                    }
                    if let Some(attr) = attributes.iter().find(|a| a.name == "align") {
                        let align_val = extract_attribute_align(attr);
                        let valid = match align_val {
                            Some(v) => v > 0 && (v & (v - 1)) == 0 && v <= 128,
                            None => false,
                        };
                        if !valid {
                            return Err(self.error_at_span(
                                attr.span,
                                &format!("@align attribute on struct '{}' requires a power-of-two integer alignment (1, 2, 4, 8, 16, 32, 64, 128)", name),
                            ));
                        }
                        self.align_datas.insert(name.clone(), align_val.unwrap());
                    }
                    if let Some(attr) = attributes.iter().find(|a| a.name == "deprecated") {
                        self.deprecated_items.insert(name.clone(), extract_attribute_message(attr));
                    }
                    if let Some(attr) = attributes.iter().find(|a| a.name == "obsolete") {
                        self.obsolete_items.insert(name.clone(), extract_attribute_message(attr));
                    }
                    if let Some(attr) = attributes.iter().find(|a| a.name == "must_use" || a.name == "nodiscard" || a.name == "no_discard") {
                        self.must_use_types.insert(name.clone(), extract_attribute_message(attr));
                    }
                }
            } else if let TopLevel::TraitDef(stmt) = item {
                self.register_trait_declaration(stmt)?;
            }
        }

        for item in &program.top_level {
            if let TopLevel::Data(stmt) = item {
                self.register_data_declaration(stmt)?;
            }
        }

        self.validate_trait_declarations()?;

        for implementation in &program.impls {
            self.register_impl_declaration(implementation)?;
        }

        for function in &program.functions {
            let params = function
                .params
                .iter()
                .map(|param| {
                    let ty = param.type_annotation.clone().unwrap_or(Type::Unknown);
                    self.resolve_generics(&ty, &function.type_params)
                })
                .collect::<Vec<_>>();
            let return_type = function.return_type.clone().unwrap_or(Type::Unknown);
            let return_type = self.resolve_generics(&return_type, &function.type_params);
            let mode = self.mode_for_attributes(&function.attributes);
            self.scopes[0].insert(
                function.name.clone(),
                Symbol {
                    _name: function.name.clone(),
                    ty: Type::Function {
                        params,
                        return_type: Box::new(return_type),
                    },
                    mutable: false,
                    mode,
                },
            );
            if function.attributes.iter().any(|a| a.name == "read") {
                self.read_only_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "pure") {
                self.pure_functions.insert(function.name.clone());
            }
            if let Some(attr) = function.attributes.iter().find(|a| a.name == "must_use" || a.name == "nodiscard" || a.name == "no_discard") {
                self.must_use_functions.insert(function.name.clone(), extract_attribute_message(attr));
            }
            if let Some(attr) = function.attributes.iter().find(|a| a.name == "deprecated") {
                self.deprecated_items.insert(function.name.clone(), extract_attribute_message(attr));
            }
            if function.attributes.iter().any(|a| a.name == "tailrec") {
                self.tailrec_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "inline") {
                self.inlined_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "safe") {
                self.safe_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "no_inline") {
                self.no_inline_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "cold") {
                self.cold_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "hot") {
                self.hot_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "strict") {
                self.strict_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "nonreentrant") {
                self.nonreentrant_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "test") {
                self.test_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "const") {
                self.const_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "no_panic") {
                self.no_panic_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "fallible") {
                self.fallible_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "infallible") {
                self.infallible_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "deterministic") {
                self.deterministic_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "idempotent") {
                self.idempotent_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "thread_safe" || a.name == "sync") {
                self.thread_safe_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "pure_math" || a.name == "strict_math") {
                self.pure_math_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "non_empty") {
                self.non_empty_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "bitwise_only") {
                self.bitwise_only_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "no_alias") {
                self.no_alias_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "pure_string" || a.name == "string_only") {
                self.pure_string_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "non_negative") {
                self.non_negative_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "positive") {
                self.positive_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "ascii_only") {
                self.ascii_only_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "sorted") {
                self.sorted_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "distinct") {
                self.distinct_functions.insert(function.name.clone());
            }
            if let Some(attr) = function.attributes.iter().find(|a| a.name == "bounded") {
                let mut min_val: Option<f64> = None;
                let mut max_val: Option<f64> = None;
                if attr.args.len() == 2 && attr.args.iter().all(|a| a.name.is_none()) {
                    if let ExprKind::Number(n) = &attr.args[0].value.kind {
                        min_val = n.parse::<f64>().ok();
                    }
                    if let ExprKind::Number(n) = &attr.args[1].value.kind {
                        max_val = n.parse::<f64>().ok();
                    }
                } else {
                    for arg in &attr.args {
                        if arg.name.as_deref() == Some("min") {
                            if let ExprKind::Number(n) = &arg.value.kind {
                                min_val = n.parse::<f64>().ok();
                            }
                        } else if arg.name.as_deref() == Some("max") {
                            if let ExprKind::Number(n) = &arg.value.kind {
                                max_val = n.parse::<f64>().ok();
                            }
                        }
                    }
                }
                if let (Some(mi), Some(ma)) = (min_val, max_val) {
                    self.bounded_functions.insert(function.name.clone(), (mi, ma));
                }
            }
            if function.attributes.iter().any(|a| a.name == "pure_array") {
                self.pure_array_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "benchmark") {
                self.benchmark_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "unreachable") {
                self.unreachable_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "checked") {
                self.checked_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "saturating") {
                self.saturating_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "zero_alloc") {
                self.zero_alloc_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "profiled") {
                self.profiled_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "test_only") {
                self.test_only_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "no_recurse") {
                self.no_recurse_functions.insert(function.name.clone());
            }
            if let Some(attr) = function.attributes.iter().find(|a| a.name == "obsolete") {
                self.obsolete_items.insert(function.name.clone(), extract_attribute_message(attr));
            }
            if function.attributes.iter().any(|a| a.name == "isolated") {
                self.isolated_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "no_float") {
                self.no_float_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "audit") {
                self.audit_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "leaf") {
                self.leaf_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "wrapping") {
                self.wrapping_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "nothrow" || a.name == "no_fail") {
                self.nothrow_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "no_io") {
                self.no_io_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "private" || a.name == "internal") {
                self.private_functions.insert(function.name.clone(), function.span.source_id);
            }
            if function.attributes.iter().any(|a| a.name == "export" || a.name == "public") {
                self.exported_items.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "experimental") {
                self.experimental_items.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "stable") {
                self.stable_items.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "benchmark_only") {
                self.benchmark_only_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "must_mutate") {
                self.must_mutate_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "security_critical") {
                self.security_critical_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "inline_always" || a.name == "always_inline") {
                self.inline_always_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "reentrant") {
                self.reentrant_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "constant_time" || a.name == "const_time") {
                self.const_time_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "simd" || a.name == "vectorize") {
                self.simd_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "idempotent") {
                self.idempotent_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "thread_safe" || a.name == "sync") {
                self.thread_safe_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "pure_math" || a.name == "strict_math") {
                self.pure_math_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "non_empty") {
                self.non_empty_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "bitwise_only") {
                self.bitwise_only_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "no_alias") {
                self.no_alias_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "pure_string" || a.name == "string_only") {
                self.pure_string_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "non_negative") {
                self.non_negative_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "positive") {
                self.positive_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "ascii_only") {
                self.ascii_only_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "sorted") {
                self.sorted_functions.insert(function.name.clone());
            }
            if function.attributes.iter().any(|a| a.name == "distinct") {
                self.distinct_functions.insert(function.name.clone());
            }
            if let Some(attr) = function.attributes.iter().find(|a| a.name == "bounded") {
                let mut min_val: Option<f64> = None;
                let mut max_val: Option<f64> = None;
                if attr.args.len() == 2 && attr.args.iter().all(|a| a.name.is_none()) {
                    if let ExprKind::Number(n) = &attr.args[0].value.kind {
                        min_val = n.parse::<f64>().ok();
                    }
                    if let ExprKind::Number(n) = &attr.args[1].value.kind {
                        max_val = n.parse::<f64>().ok();
                    }
                } else {
                    for arg in &attr.args {
                        if arg.name.as_deref() == Some("min") {
                            if let ExprKind::Number(n) = &arg.value.kind {
                                min_val = n.parse::<f64>().ok();
                            }
                        } else if arg.name.as_deref() == Some("max") {
                            if let ExprKind::Number(n) = &arg.value.kind {
                                max_val = n.parse::<f64>().ok();
                            }
                        }
                    }
                }
                if let (Some(mi), Some(ma)) = (min_val, max_val) {
                    self.bounded_functions.insert(function.name.clone(), (mi, ma));
                }
            }
            if function.attributes.iter().any(|a| a.name == "pure_array") {
                self.pure_array_functions.insert(function.name.clone());
            }
            if let Some(attr) = function.attributes.iter().find(|a| a.name == "since") {
                if let Some(v) = extract_attribute_version(attr) {
                    self.since_items.insert(function.name.clone(), v);
                }
            }
        }

        for data_decl in &program.data_declarations {
            self.current_source = self.source_files.get(data_decl.span.source_id).cloned();
            self.analyze_statement(data_decl)?;
        }

        for function in &program.functions {
            self.analyze_function(function)?;
        }

        for implementation in &program.impls {
            let self_type =
                self.resolve_generics(&implementation.target, &implementation.type_params);
            for method in &implementation.methods {
                self.analyze_function_with_self_type(method, Some(&self_type))?;
            }
        }

        Ok(())
    }

    fn register_data_declaration(&mut self, stmt: &Stmt) -> Result<(), ViceError> {
        if let StmtKind::DataDeclaration {
            name,
            type_params,
            attributes,
            fields,
            ..
        } = &stmt.kind
        {
            let mode = self.mode_for_attributes(attributes);
            let mut field_types = HashMap::new();
            let mut params = Vec::new();
            for field in fields {
                self.validate_type_annotation(&field.type_annotation, type_params, stmt.span.line)?;
                let resolved_ty = self.resolve_generics(&field.type_annotation, type_params);
                field_types.insert(field.name.clone(), resolved_ty.clone());
                params.push(resolved_ty);
            }
            self.data_defs.insert(
                name.clone(),
                DataDef {
                    fields: field_types,
                },
            );
            if let Some(attr) = attributes.iter().find(|a| a.name == "must_use" || a.name == "nodiscard" || a.name == "no_discard") {
                self.must_use_types.insert(name.clone(), extract_attribute_message(attr));
            }
            if attributes.iter().any(|a| a.name == "immutable" || a.name == "immut") {
                self.immutable_datas.insert(name.clone());
            }
            if attributes.iter().any(|a| a.name == "pure_data" || a.name == "value_object") {
                for field in fields {
                    if !self.is_pure_data_type(&field.type_annotation) {
                        return Err(self.error_at_span(
                            field.span,
                            &format!(
                                "Field '{}' in @pure_data struct '{}' must have a pure value type, got {}",
                                field.name, name, field.type_annotation
                            ),
                        ));
                    }
                }
            }
            if attributes.iter().any(|a| a.name == "transparent") {
                if fields.len() != 1 {
                    return Err(self.error_at_span(
                        stmt.span,
                        &format!(
                            "@transparent struct '{}' must have exactly one field, found {}",
                            name,
                            fields.len()
                        ),
                    ));
                }
                self.transparent_datas.insert(name.clone());
            }
            if attributes.iter().any(|a| a.name == "packed" || a.name == "flat") {
                for field in fields {
                    if !self.is_packed_data_type(&field.type_annotation) {
                        return Err(self.error_at_span(
                            field.span,
                            &format!(
                                "Field '{}' in @packed struct '{}' must have a scalar type (int, float, bool), got {}",
                                field.name, name, field.type_annotation
                            ),
                        ));
                    }
                }
            }
            if attributes.iter().any(|a| a.name == "must_initialize" || a.name == "must_init" || a.name == "require_all_fields") {
                self.must_initialize_datas.insert(name.clone());
            }

            let return_args = type_params
                .iter()
                .map(|p| Type::Generic(p.clone()))
                .collect();
            self.scopes[0].insert(
                name.clone(),
                Symbol {
                    _name: name.clone(),
                    ty: Type::Function {
                        params,
                        return_type: Box::new(Type::Data(name.clone(), return_args)),
                    },
                    mutable: false,
                    mode,
                },
            );
        }
        Ok(())
    }

    fn is_pure_data_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Int | Type::Float | Type::Bool | Type::String => true,
            Type::Data(name, _) => self.pure_datas.contains(name),
            Type::Array(inner) => self.is_pure_data_type(inner),
            Type::Map(k, v) => self.is_pure_data_type(k) && self.is_pure_data_type(v),
            _ => false,
        }
    }

    fn is_packed_data_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Int | Type::Float | Type::Bool => true,
            Type::Data(name, _) => self.packed_datas.contains(name) || self.transparent_datas.contains(name),
            _ => false,
        }
    }

    fn register_trait_declaration(&mut self, stmt: &Stmt) -> Result<(), ViceError> {
        let StmtKind::TraitDef {
            name,
            type_params,
            attributes,
            methods,
        } = &stmt.kind
        else {
            return Ok(());
        };

        if self.trait_defs.contains_key(name) {
            return Err(
                self.error_at_span(stmt.span, &format!("Duplicate trait declaration: {}", name))
            );
        }

        if attributes.iter().any(|a| a.name == "sealed") {
            self.sealed_traits.insert(name.clone(), stmt.span.source_id);
        }

        let is_export = attributes.iter().any(|a| a.name == "export" || a.name == "public");
        let is_private = attributes.iter().any(|a| a.name == "private" || a.name == "internal");
        if is_export && is_private {
            return Err(self.error_at_span(
                stmt.span,
                &format!("Trait '{}' cannot be marked with both @export and @private", name),
            ));
        }
        if is_export {
            self.exported_items.insert(name.clone());
        }
        let is_experimental = attributes.iter().any(|a| a.name == "experimental");
        let is_stable = attributes.iter().any(|a| a.name == "stable");
        let is_deprecated = attributes.iter().any(|a| a.name == "deprecated");
        let is_obsolete = attributes.iter().any(|a| a.name == "obsolete");
        if is_experimental && is_stable {
            return Err(self.error_at_span(
                stmt.span,
                &format!("Trait '{}' cannot be marked with both @experimental and @stable", name),
            ));
        }
        if is_experimental && (is_deprecated || is_obsolete) {
            return Err(self.error_at_span(
                stmt.span,
                &format!("Trait '{}' cannot be marked with both @experimental and @deprecated", name),
            ));
        }
        if is_stable && (is_deprecated || is_obsolete) {
            return Err(self.error_at_span(
                stmt.span,
                &format!("Trait '{}' cannot be marked with both @stable and @deprecated", name),
            ));
        }
        if is_experimental {
            self.experimental_items.insert(name.clone());
        }
        if is_stable {
            self.stable_items.insert(name.clone());
        }
        if let Some(attr) = attributes.iter().find(|a| a.name == "since") {
            let version = extract_attribute_version(attr).ok_or_else(|| {
                self.error_at_span(
                    attr.span,
                    &format!("@since attribute on trait '{}' requires a version string or number", name),
                )
            })?;
            self.since_items.insert(name.clone(), version);
        }

        let mut seen_type_params = HashSet::new();
        for type_param in type_params {
            if !seen_type_params.insert(type_param) {
                return Err(self.error_at_span(
                    stmt.span,
                    &format!(
                        "Duplicate type parameter '{}' in trait '{}'",
                        type_param, name
                    ),
                ));
            }
        }

        let mut registered_methods = HashMap::new();
        for method in methods {
            if registered_methods.contains_key(&method.name) {
                return Err(self.error_at_span(
                    method.span,
                    &format!("Duplicate method '{}' in trait '{}'", method.name, name),
                ));
            }
            let mut has_self = false;
            for (index, param) in method.params.iter().enumerate() {
                if param.name == "self" {
                    if index != 0 || has_self {
                        return Err(self.error_at_span(
                            param.span,
                            &format!(
                                "Trait method '{}.{}' may declare self only once as its first parameter",
                                name, method.name
                            ),
                        ));
                    }
                    has_self = true;
                }
                if param.name != "self" && param.type_annotation.is_none() {
                    return Err(self.error_at_span(
                        param.span,
                        &format!(
                            "Trait method '{}.{}' parameter '{}' requires a type annotation",
                            name, method.name, param.name
                        ),
                    ));
                }
            }
            registered_methods.insert(method.name.clone(), method.clone());
        }

        self.trait_defs.insert(
            name.clone(),
            TraitDefinition {
                type_params: type_params.clone(),
                methods: registered_methods,
            },
        );
        Ok(())
    }

    fn validate_trait_declarations(&self) -> Result<(), ViceError> {
        for definition in self.trait_defs.values() {
            for method in definition.methods.values() {
                for param in &method.params {
                    if let Some(ty) = &param.type_annotation {
                        self.validate_trait_type_annotation(
                            ty,
                            &definition.type_params,
                            param.span,
                        )?;
                    }
                }
                if let Some(ty) = &method.return_type {
                    self.validate_trait_type_annotation(ty, &definition.type_params, method.span)?;
                }
            }
        }
        Ok(())
    }

    fn register_impl_declaration(&mut self, implementation: &ImplBlock) -> Result<(), ViceError> {
        let mut seen_type_params = HashSet::new();
        for type_param in &implementation.type_params {
            if !seen_type_params.insert(type_param) {
                return Err(self.error_at_span(
                    implementation.span,
                    &format!("Duplicate impl type parameter '{}'", type_param),
                ));
            }
        }

        self.validate_type_annotation(
            &implementation.target,
            &implementation.type_params,
            implementation.span.line,
        )?;
        let resolved_target =
            self.resolve_generics(&implementation.target, &implementation.type_params);
        let target_name = match &resolved_target {
            Type::Data(name, _) if !implementation.type_params.contains(name) => name.clone(),
            _ => {
                return Err(self.error_at_span(
                    implementation.span,
                    "Impl target must be a declared data or enum type",
                ));
            }
        };

        if self.final_datas.contains(&target_name) {
            return Err(self.error_at_span(
                implementation.span,
                &format!("Cannot define impl block for @final data '{}'", target_name),
            ));
        }

        if let Some(&sealed_source_id) = self.sealed_datas.get(&target_name) {
            if implementation.span.source_id != sealed_source_id {
                return Err(self.error_at_span(
                    implementation.span,
                    &format!("Data type '{}' is @sealed and cannot be implemented outside its defining file", target_name),
                ));
            }
        }

        if let Some(&sealed_source_id) = self.sealed_enums.get(&target_name) {
            if implementation.span.source_id != sealed_source_id {
                return Err(self.error_at_span(
                    implementation.span,
                    &format!("Enum '{}' is @sealed and cannot be implemented outside its defining file", target_name),
                ));
            }
        }

        let mut method_names = HashSet::new();
        for method in &implementation.methods {
            if !method_names.insert(method.name.as_str()) {
                return Err(self.error_at_span(
                    method.span,
                    &format!(
                        "Duplicate method '{}' in impl for '{}'",
                        method.name, target_name
                    ),
                ));
            }
        }

        if self.pure_datas.contains(&target_name)
            || self.immutable_datas.contains(&target_name)
            || self.read_only_datas.contains(&target_name)
        {
            for method in &implementation.methods {
                if method.params.iter().any(|p| p.name == "self" && p.mutable) {
                    return Err(self.error_at_span(
                        method.span,
                        &format!("Method '{}' on @pure_data struct '{}' cannot take 'mut self'", method.name, target_name),
                    ));
                }
            }
        }

        let trait_key = if let Some(trait_type) = &implementation.trait_name {
            Some(self.validate_trait_impl(implementation, &resolved_target, trait_type)?)
        } else {
            None
        };

        if let Some(existing) = self.impls.get(&target_name) {
            for method in &implementation.methods {
                if existing.contains_key(&method.name) {
                    return Err(self.error_at_span(
                        method.span,
                        &format!(
                            "Method '{}' is already implemented for '{}'",
                            method.name, target_name
                        ),
                    ));
                }
            }
        }

        let mut methods = HashMap::new();
        for method in &implementation.methods {
            let params = method
                .params
                .iter()
                .map(|param| {
                    let ty = param.type_annotation.clone().unwrap_or(Type::Unknown);
                    self.resolve_generics(&ty, &implementation.type_params)
                })
                .collect();
            let return_type = method.return_type.clone().unwrap_or(Type::Unknown);
            let return_type = self.resolve_generics(&return_type, &implementation.type_params);
            methods.insert(
                method.name.clone(),
                Type::Function {
                    params,
                    return_type: Box::new(return_type),
                },
            );
        }

        if let Some(key) = trait_key {
            self.trait_impls.insert(key);
        }
        self.impls.entry(target_name).or_default().extend(methods);
        Ok(())
    }

    fn validate_trait_impl(
        &self,
        implementation: &ImplBlock,
        resolved_target: &Type,
        trait_type: &Type,
    ) -> Result<(String, String), ViceError> {
        let (trait_name, trait_args) = match trait_type {
            Type::Data(name, args) => (name, args),
            _ => {
                return Err(self.error_at_span(
                    implementation.span,
                    "Implemented trait must be a named trait",
                ));
            }
        };
        let definition = self.trait_defs.get(trait_name).ok_or_else(|| {
            self.error_at_span(
                implementation.span,
                &format!("Unknown trait '{}'", trait_name),
            )
        })?;

        if let Some(&sealed_source_id) = self.sealed_traits.get(trait_name) {
            if implementation.span.source_id != sealed_source_id {
                return Err(self.error_at_span(
                    implementation.span,
                    &format!(
                        "Trait '{}' is @sealed and cannot be implemented outside its defining file",
                        trait_name
                    ),
                ));
            }
        }

        if definition.type_params.len() != trait_args.len() {
            return Err(self.error_at_span(
                implementation.span,
                &format!(
                    "Trait '{}' expects {} type arguments, got {}",
                    trait_name,
                    definition.type_params.len(),
                    trait_args.len()
                ),
            ));
        }

        let target_name = resolved_target.to_string();
        let key = (trait_name.clone(), target_name.clone());
        if self.trait_impls.contains(&key) {
            return Err(self.error_at_span(
                implementation.span,
                &format!(
                    "Trait '{}' is already implemented for '{}'",
                    trait_name, target_name
                ),
            ));
        }

        let bindings = definition
            .type_params
            .iter()
            .zip(trait_args.iter())
            .map(|(name, ty)| {
                (
                    name.clone(),
                    self.resolve_generics(ty, &implementation.type_params),
                )
            })
            .collect::<HashMap<_, _>>();
        let actual_methods = implementation
            .methods
            .iter()
            .map(|method| (method.name.as_str(), method))
            .collect::<HashMap<_, _>>();

        for method_name in actual_methods.keys() {
            if !definition.methods.contains_key(*method_name) {
                let method = actual_methods[method_name];
                return Err(self.error_at_span(
                    method.span,
                    &format!(
                        "Method '{}' is not declared by trait '{}'",
                        method_name, trait_name
                    ),
                ));
            }
        }

        for (method_name, expected) in &definition.methods {
            let actual = actual_methods.get(method_name.as_str()).ok_or_else(|| {
                self.error_at_span(
                    implementation.span,
                    &format!(
                        "Impl for '{}' is missing trait method '{}.{}'",
                        target_name, trait_name, method_name
                    ),
                )
            })?;

            if !actual.type_params.is_empty() {
                return Err(self.error_at_span(
                    actual.span,
                    &format!(
                        "Trait method '{}.{}' cannot add method type parameters",
                        trait_name, method_name
                    ),
                ));
            }
            if expected.params.len() != actual.params.len() {
                return Err(self.error_at_span(
                    actual.span,
                    &format!(
                        "Trait method '{}.{}' expects {} parameters, got {}",
                        trait_name,
                        method_name,
                        expected.params.len(),
                        actual.params.len()
                    ),
                ));
            }

            for (index, (expected_param, actual_param)) in
                expected.params.iter().zip(actual.params.iter()).enumerate()
            {
                if (expected_param.name == "self") != (actual_param.name == "self") {
                    return Err(self.error_at_span(
                        actual_param.span,
                        &format!(
                            "Trait method '{}.{}' parameter {} must preserve self position",
                            trait_name,
                            method_name,
                            index + 1
                        ),
                    ));
                }
                if expected_param.mutable != actual_param.mutable {
                    return Err(self.error_at_span(
                        actual_param.span,
                        &format!(
                            "Trait method '{}.{}' parameter {} mutability does not match",
                            trait_name,
                            method_name,
                            index + 1
                        ),
                    ));
                }

                let expected_type = expected_param
                    .type_annotation
                    .as_ref()
                    .map(|ty| Self::substitute_trait_type(ty, &bindings, resolved_target))
                    .unwrap_or_else(|| resolved_target.clone());
                let actual_type = actual_param
                    .type_annotation
                    .as_ref()
                    .map(|ty| self.resolve_generics(ty, &implementation.type_params))
                    .unwrap_or_else(|| {
                        if actual_param.name == "self" {
                            resolved_target.clone()
                        } else {
                            Type::Unknown
                        }
                    });
                if expected_type != actual_type {
                    return Err(self.error_at_span(
                        actual_param.span,
                        &format!(
                            "Trait method '{}.{}' parameter {} expected {}, got {}",
                            trait_name,
                            method_name,
                            index + 1,
                            expected_type,
                            actual_type
                        ),
                    ));
                }
            }

            let expected_return = expected
                .return_type
                .as_ref()
                .map(|ty| Self::substitute_trait_type(ty, &bindings, resolved_target))
                .unwrap_or(Type::Void);
            let actual_return = actual
                .return_type
                .as_ref()
                .map(|ty| self.resolve_generics(ty, &implementation.type_params))
                .unwrap_or(Type::Void);
            if expected_return != actual_return {
                return Err(self.error_at_span(
                    actual.span,
                    &format!(
                        "Trait method '{}.{}' must return {}, got {}",
                        trait_name, method_name, expected_return, actual_return
                    ),
                ));
            }
        }

        Ok(key)
    }

    fn substitute_trait_type(ty: &Type, bindings: &HashMap<String, Type>, target: &Type) -> Type {
        match ty {
            Type::Data(name, args) if name == "Self" && args.is_empty() => target.clone(),
            Type::Data(name, args) if args.is_empty() && bindings.contains_key(name) => {
                bindings[name].clone()
            }
            Type::Data(name, args) => Type::Data(
                name.clone(),
                args.iter()
                    .map(|arg| Self::substitute_trait_type(arg, bindings, target))
                    .collect(),
            ),
            Type::Generic(name) if bindings.contains_key(name) => bindings[name].clone(),
            Type::Array(inner) => Type::Array(Box::new(Self::substitute_trait_type(
                inner, bindings, target,
            ))),
            Type::Map(key, value) => Type::Map(
                Box::new(Self::substitute_trait_type(key, bindings, target)),
                Box::new(Self::substitute_trait_type(value, bindings, target)),
            ),
            Type::Function {
                params,
                return_type,
            } => Type::Function {
                params: params
                    .iter()
                    .map(|param| Self::substitute_trait_type(param, bindings, target))
                    .collect(),
                return_type: Box::new(Self::substitute_trait_type(return_type, bindings, target)),
            },
            Type::Reference { mutable, inner } => Type::Reference {
                mutable: *mutable,
                inner: Box::new(Self::substitute_trait_type(inner, bindings, target)),
            },
            _ => ty.clone(),
        }
    }

    fn analyze_function(&mut self, function: &Function) -> Result<(), ViceError> {
        self.analyze_function_with_self_type(function, None)
    }

    fn analyze_function_with_self_type(
        &mut self,
        function: &Function,
        self_type: Option<&Type>,
    ) -> Result<(), ViceError> {
        self.scopes.push(HashMap::new());
        let previous_return = self.current_return_type.clone();
        let previous_autocheck = self.current_autocheck;
        let previous_mode = self.current_mode;
        let previous_read_only = self.in_read_only_function;
        let previous_pure = self.in_pure_function;
        let previous_safe = self.in_safe_function;
        let previous_strict = self.in_strict_function;
        let previous_const = self.in_const_function;
        let previous_no_panic = self.in_no_panic_function;
        let previous_infallible = self.in_infallible_function;
        let previous_deterministic = self.in_deterministic_function;
        let previous_zero_alloc = self.in_zero_alloc_function;
        let previous_isolated = self.in_isolated_function;
        let previous_no_float = self.in_no_float_function;
        let previous_test = self.in_test_function;
        let previous_leaf = self.in_leaf_function;
        let previous_nothrow = self.in_nothrow_function;
        let previous_no_io = self.in_no_io_function;
        let previous_benchmark = self.in_benchmark_function;
        let previous_experimental = self.in_experimental_function;
        let previous_simd = self.in_simd_function;
        let previous_idempotent = self.in_idempotent_function;
        let previous_thread_safe = self.in_thread_safe_function;
        let previous_pure_math = self.in_pure_math_function;
        let previous_bitwise_only = self.in_bitwise_only_function;
        let previous_pure_string = self.in_pure_string_function;
        let previous_pure_array = self.in_pure_array_function;
        let previous_mutated = std::mem::take(&mut self.current_function_mutated_params);
        let previous_function_name = self.current_function_name.clone();
        let previous_source = self.current_source.clone();
        self.current_source = self.source_files.get(function.span.source_id).cloned();
        self.current_function_name = Some(function.name.clone());

        self.current_return_type = function.return_type.clone();
        // current_autocheck true if module default or function has explicit @autocheck
        self.current_autocheck =
            self.module_autocheck || function.attributes.iter().any(|a| a.name == "autocheck");
        self.current_mode = self.mode_for_attributes(&function.attributes);
        self.in_read_only_function =
            previous_read_only || function.attributes.iter().any(|a| a.name == "read");
        let is_pure = function.attributes.iter().any(|a| a.name == "pure");
        self.in_pure_function = previous_pure || is_pure;
        let is_leaf = function.attributes.iter().any(|a| a.name == "leaf");
        self.in_leaf_function = previous_leaf || is_leaf;
        let is_nothrow = function.attributes.iter().any(|a| a.name == "nothrow" || a.name == "no_fail");
        self.in_nothrow_function = previous_nothrow || is_nothrow;
        let is_no_io = function.attributes.iter().any(|a| a.name == "no_io");
        self.in_no_io_function = previous_no_io || is_no_io;
        let is_safe = function.attributes.iter().any(|a| a.name == "safe");
        self.in_safe_function = previous_safe || is_safe;
        let is_zero_alloc = function.attributes.iter().any(|a| a.name == "zero_alloc");
        self.in_zero_alloc_function = previous_zero_alloc || is_zero_alloc;
        let is_isolated = function.attributes.iter().any(|a| a.name == "isolated");
        self.in_isolated_function = previous_isolated || is_isolated;
        let is_simd = function.attributes.iter().any(|a| a.name == "simd" || a.name == "vectorize");
        self.in_simd_function = previous_simd || is_simd;
        let is_idempotent = function.attributes.iter().any(|a| a.name == "idempotent");
        self.in_idempotent_function = previous_idempotent || is_idempotent;
        let is_thread_safe = function.attributes.iter().any(|a| a.name == "thread_safe" || a.name == "sync");
        self.in_thread_safe_function = previous_thread_safe || is_thread_safe;
        let is_pure_math = function.attributes.iter().any(|a| a.name == "pure_math" || a.name == "strict_math");
        self.in_pure_math_function = previous_pure_math || is_pure_math;
        let is_bitwise_only = function.attributes.iter().any(|a| a.name == "bitwise_only");
        self.in_bitwise_only_function = previous_bitwise_only || is_bitwise_only;
        let is_pure_string = function.attributes.iter().any(|a| a.name == "pure_string" || a.name == "string_only");
        self.in_pure_string_function = previous_pure_string || is_pure_string;
        let is_pure_array = function.attributes.iter().any(|a| a.name == "pure_array");
        self.in_pure_array_function = previous_pure_array || is_pure_array;
        let is_no_float = function.attributes.iter().any(|a| a.name == "no_float");
        self.in_no_float_function = previous_no_float || is_no_float;
        if is_no_float {
            if function.return_type == Some(Type::Float) {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@no_float function '{}' cannot return float", function.name),
                ));
            }
            for param in &function.params {
                if param.type_annotation == Some(Type::Float) {
                    return Err(self.error_at_span(
                        param.span,
                        &format!("Parameter '{}' in @no_float function '{}' cannot have float type", param.name, function.name),
                    ));
                }
            }
        }
        let is_strict = function.attributes.iter().any(|a| a.name == "strict");
        self.in_strict_function = previous_strict || is_strict;
        if is_strict {
            if function.return_type.is_none() {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "@strict function '{}' must specify an explicit return type",
                        function.name
                    ),
                ));
            }
            for param in &function.params {
                if param.type_annotation.is_none() {
                    return Err(self.error_at_span(
                        param.span,
                        &format!(
                            "Parameter '{}' in @strict function '{}' must specify an explicit type annotation",
                            param.name, function.name
                        ),
                    ));
                }
            }
        }

        let is_const = function.attributes.iter().any(|a| a.name == "const");
        self.in_const_function = previous_const || is_const;
        if is_const && (function.return_type.is_none() || function.return_type == Some(Type::Void)) {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "@const function '{}' must specify a non-void return type",
                    function.name
                ),
            ));
        }

        let is_no_panic = function.attributes.iter().any(|a| a.name == "no_panic");
        self.in_no_panic_function = previous_no_panic || is_no_panic;

        let is_test = function.attributes.iter().any(|a| a.name == "test");
        self.in_test_function = previous_test || is_test;
        if is_test {
            if !function.params.is_empty() {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@test function '{}' cannot take parameters", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "entry") {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' cannot be marked with both @test and @entry",
                        function.name
                    ),
                ));
            }
        }

        let is_test_only = function.attributes.iter().any(|a| a.name == "test_only");
        if is_test_only && function.attributes.iter().any(|a| a.name == "entry") {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @test_only and @entry", function.name),
            ));
        }

        let is_private = function.attributes.iter().any(|a| a.name == "private" || a.name == "internal");
        if is_private && function.attributes.iter().any(|a| a.name == "entry") {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @private and @entry", function.name),
            ));
        }

        let is_export = function.attributes.iter().any(|a| a.name == "export" || a.name == "public");
        if is_export && is_private {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @export and @private", function.name),
            ));
        }

        let is_experimental = function.attributes.iter().any(|a| a.name == "experimental");
        self.in_experimental_function = previous_experimental || is_experimental;
        let is_stable = function.attributes.iter().any(|a| a.name == "stable");
        let is_deprecated = function.attributes.iter().any(|a| a.name == "deprecated");
        let is_obsolete = function.attributes.iter().any(|a| a.name == "obsolete");
        if is_experimental && is_stable {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @experimental and @stable", function.name),
            ));
        }
        if is_experimental && (is_deprecated || is_obsolete) {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @experimental and @deprecated", function.name),
            ));
        }
        if is_stable && (is_deprecated || is_obsolete) {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @stable and @deprecated", function.name),
            ));
        }

        if let Some(attr) = function.attributes.iter().find(|a| a.name == "since") {
            let version = extract_attribute_version(attr).ok_or_else(|| {
                self.error_at_span(
                    attr.span,
                    &format!("@since attribute on function '{}' requires a version string or number", function.name),
                )
            })?;
            self.since_items.insert(function.name.clone(), version);
        }

        let is_benchmark_only = function.attributes.iter().any(|a| a.name == "benchmark_only");
        if is_benchmark_only {
            if function.attributes.iter().any(|a| a.name == "entry") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @benchmark_only and @entry", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "test") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @benchmark_only and @test", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "test_only") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @benchmark_only and @test_only", function.name),
                ));
            }
        }

        let is_must_mutate = function.attributes.iter().any(|a| a.name == "must_mutate");
        if is_must_mutate && !function.params.iter().any(|p| p.mutable) {
            return Err(self.error_at_span(
                function.span,
                &format!("@must_mutate function '{}' must have at least one mutable parameter", function.name),
            ));
        }

        let is_no_recurse = function.attributes.iter().any(|a| a.name == "no_recurse");
        if is_no_recurse && function.attributes.iter().any(|a| a.name == "tailrec") {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @no_recurse and @tailrec", function.name),
            ));
        }

        let is_fallible = function.attributes.iter().any(|a| a.name == "fallible");
        if is_fallible {
            let returns_result = match &function.return_type {
                Some(Type::Data(name, _)) => name == "Result",
                _ => false,
            };
            if !returns_result {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@fallible function '{}' must return a Result type", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "infallible") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @fallible and @infallible", function.name),
                ));
            }
        }

        let is_infallible = function.attributes.iter().any(|a| a.name == "infallible");
        self.in_infallible_function = previous_infallible || is_infallible;
        if is_infallible {
            match &function.return_type {
                Some(Type::Data(name, _)) if name == "Result" || name == "Option" => {
                    return Err(self.error_at_span(
                        function.span,
                        &format!("@infallible function '{}' cannot return a Result or Option type", function.name),
                    ));
                }
                _ => {}
            }
        }

        let is_deterministic = function.attributes.iter().any(|a| a.name == "deterministic");
        self.in_deterministic_function = previous_deterministic || is_deterministic;

        let is_idempotent = function.attributes.iter().any(|a| a.name == "idempotent");
        if is_idempotent {
            if function.attributes.iter().any(|a| a.name == "entry") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@idempotent function '{}' cannot be marked with @entry", function.name),
                ));
            }
            if function.params.is_empty() {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@idempotent function '{}' must take at least one parameter", function.name),
                ));
            }
            if function.return_type.is_none() || function.return_type == Some(Type::Void) {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@idempotent function '{}' must return a value", function.name),
                ));
            }
            if let Some(first_param) = function.params.first() {
                if let (Some(ret_ty), Some(param_ty)) = (&function.return_type, &first_param.type_annotation) {
                    if ret_ty != param_ty {
                        return Err(self.error_at_span(
                            function.span,
                            &format!("@idempotent function '{}' return type '{:?}' must match first parameter type '{:?}'", function.name, ret_ty, param_ty),
                        ));
                    }
                }
            }
        }

        let is_benchmark = function.attributes.iter().any(|a| a.name == "benchmark");
        self.in_benchmark_function = previous_benchmark || is_benchmark;
        if is_benchmark {
            if !function.params.is_empty() {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@benchmark function '{}' must take 0 parameters", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "entry") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @benchmark and @entry", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "test") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @benchmark and @test", function.name),
                ));
            }
        }

        let is_unreachable = function.attributes.iter().any(|a| a.name == "unreachable");
        if is_unreachable && function.attributes.iter().any(|a| a.name == "entry") {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @unreachable and @entry", function.name),
            ));
        }

        let is_security_critical = function.attributes.iter().any(|a| a.name == "security_critical");
        if is_security_critical {
            if is_experimental {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @security_critical and @experimental", function.name),
                ));
            }
            if is_unreachable {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @security_critical and @unreachable", function.name),
                ));
            }
        }

        let is_const_time = function.attributes.iter().any(|a| a.name == "constant_time" || a.name == "const_time");
        if is_const_time {
            if function.attributes.iter().any(|a| a.name == "cold") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @constant_time and @cold", function.name),
                ));
            }
            if is_benchmark_only {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @constant_time and @benchmark_only", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "timeout") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @constant_time and @timeout", function.name),
                ));
            }
        }

        if is_simd {
            if count_recursive_calls(&function.body, &function.name) > 0 {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' marked @simd cannot be recursive", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "no_inline") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @simd and @no_inline", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "cold") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @simd and @cold", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "unreachable") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @simd and @unreachable", function.name),
                ));
            }
        }

        if is_idempotent {
            if function.attributes.iter().any(|a| a.name == "cold") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @idempotent and @cold", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "experimental") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @idempotent and @experimental", function.name),
                ));
            }
        }

        if is_thread_safe {
            if function.attributes.iter().any(|a| a.name == "nonreentrant") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @thread_safe and @nonreentrant", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "cold") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @thread_safe and @cold", function.name),
                ));
            }
        }

        if is_pure_math {
            if function.attributes.iter().any(|a| a.name == "entry") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_math function '{}' cannot be marked with @entry", function.name),
                ));
            }
            if function.params.is_empty() {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_math function '{}' must take at least one parameter", function.name),
                ));
            }
            if !matches!(function.return_type, Some(Type::Int) | Some(Type::Float)) {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_math function '{}' must return a number (int or float)", function.name),
                ));
            }
        }

        if is_bitwise_only {
            if function.return_type != Some(Type::Int) {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@bitwise_only function '{}' must return int", function.name),
                ));
            }
            for param in &function.params {
                if param.type_annotation != Some(Type::Int) {
                    return Err(self.error_at_span(
                        param.span,
                        &format!("Parameter '{}' in @bitwise_only function '{}' must have int type", param.name, function.name),
                    ));
                }
            }
        }

        if let Some(clamp_attr) = function.attributes.iter().find(|a| a.name == "clamp") {
            let mut min_val: Option<f64> = None;
            let mut max_val: Option<f64> = None;
            if clamp_attr.args.len() == 2 && clamp_attr.args.iter().all(|a| a.name.is_none()) {
                if let ExprKind::Number(n) = &clamp_attr.args[0].value.kind {
                    min_val = n.parse::<f64>().ok();
                }
                if let ExprKind::Number(n) = &clamp_attr.args[1].value.kind {
                    max_val = n.parse::<f64>().ok();
                }
            } else {
                for arg in &clamp_attr.args {
                    if arg.name.as_deref() == Some("min") {
                        if let ExprKind::Number(n) = &arg.value.kind {
                            min_val = n.parse::<f64>().ok();
                        }
                    } else if arg.name.as_deref() == Some("max") {
                        if let ExprKind::Number(n) = &arg.value.kind {
                            max_val = n.parse::<f64>().ok();
                        }
                    }
                }
            }
            match (min_val, max_val) {
                (Some(mi), Some(ma)) => {
                    if mi > ma {
                        return Err(self.error_at_span(
                            clamp_attr.span,
                            &format!("@clamp attribute min ({}) cannot be greater than max ({}) on function '{}'", mi, ma, function.name),
                        ));
                    }
                    self.clamped_functions.insert(function.name.clone(), (mi, ma));
                }
                _ => {
                    return Err(self.error_at_span(
                        clamp_attr.span,
                        &format!("@clamp attribute on function '{}' requires valid numeric min and max bounds", function.name),
                    ));
                }
            }
            if !matches!(function.return_type, Some(Type::Int) | Some(Type::Float)) {
                return Err(self.error_at_span(
                    clamp_attr.span,
                    &format!("@clamp function '{}' must return a numeric type (int or float)", function.name),
                ));
            }
        }

        if function.attributes.iter().any(|a| a.name == "non_empty") {
            let has_collection_param = function.params.iter().any(|p| {
                matches!(p.type_annotation, Some(Type::String) | Some(Type::Array(_)) | Some(Type::Map(_, _)))
                    || matches!(&p.type_annotation, Some(Type::Data(name, _)) if name == "array" || name == "map")
            });
            if !has_collection_param {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@non_empty function '{}' requires at least one string, array, or map parameter", function.name),
                ));
            }
        }

        if function.attributes.iter().any(|a| a.name == "no_alias") {
            let ref_count = function.params.iter().filter(|p| {
                matches!(p.type_annotation, Some(Type::Reference { .. }))
            }).count();
            if ref_count < 2 {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@no_alias function '{}' requires at least two reference parameters", function.name),
                ));
            }
        }

        if is_pure_string {
            if function.attributes.iter().any(|a| a.name == "entry") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_string function '{}' cannot be marked with @entry", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "pure_math" || a.name == "bitwise_only") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_string function '{}' cannot conflict with math/bitwise attributes", function.name),
                ));
            }
            if function.params.is_empty() {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_string function '{}' requires at least one parameter", function.name),
                ));
            }
            for param in &function.params {
                if !matches!(param.type_annotation, Some(Type::String) | Some(Type::Int)) {
                    return Err(self.error_at_span(
                        param.span,
                        &format!("Parameter '{}' in @pure_string function '{}' must be of type string or int", param.name, function.name),
                    ));
                }
            }
            if !matches!(function.return_type, Some(Type::String) | Some(Type::Int) | Some(Type::Bool)) {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_string function '{}' must return string, int, or bool", function.name),
                ));
            }
        }

        if function.attributes.iter().any(|a| a.name == "non_negative") {
            let has_num_param = function.params.iter().any(|p| {
                matches!(p.type_annotation, Some(Type::Int) | Some(Type::Float))
            });
            if !has_num_param {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@non_negative function '{}' requires at least one numeric parameter (int or float)", function.name),
                ));
            }
            if let Some((min_val, _)) = self.clamped_functions.get(&function.name) {
                if *min_val < 0.0 {
                    return Err(self.error_at_span(
                        function.span,
                        &format!("@non_negative function '{}' cannot have negative @clamp min bound ({})", function.name, min_val),
                    ));
                }
            }
        }

        if is_pure_array {
            if function.attributes.iter().any(|a| a.name == "entry") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_array function '{}' cannot be marked with @entry", function.name),
                ));
            }
            if function.attributes.iter().any(|a| a.name == "pure_math" || a.name == "bitwise_only" || a.name == "pure_string") {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_array function '{}' cannot conflict with math/bitwise/string attributes", function.name),
                ));
            }
            let has_arr = function.params.iter().any(|p| matches!(p.type_annotation, Some(Type::Array(_))))
                || matches!(function.return_type, Some(Type::Array(_)));
            if !has_arr {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@pure_array function '{}' requires at least one array parameter or array return type", function.name),
                ));
            }
        }

        if function.attributes.iter().any(|a| a.name == "ascii_only") {
            let has_string_param = function.params.iter().any(|p| matches!(p.type_annotation, Some(Type::String)));
            if !has_string_param {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@ascii_only function '{}' requires at least one string parameter", function.name),
                ));
            }
        }

        if function.attributes.iter().any(|a| a.name == "sorted") {
            let has_supported = function.params.iter().any(|p| matches!(p.type_annotation, Some(Type::Array(_))))
                || function.params.iter().filter(|p| matches!(p.type_annotation, Some(Type::Int) | Some(Type::Float))).count() >= 2;
            if !has_supported {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@sorted function '{}' requires at least one array parameter or at least two numeric parameters", function.name),
                ));
            }
        }

        if function.attributes.iter().any(|a| a.name == "distinct") {
            let has_arr = function.params.iter().any(|p| matches!(p.type_annotation, Some(Type::Array(_))));
            let has_multi = function.params.len() >= 2;
            if !has_arr && !has_multi {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@distinct function '{}' requires at least one array parameter or at least two parameters", function.name),
                ));
            }
        }

        if function.attributes.iter().any(|a| a.name == "positive") {
            let has_num = function.params.iter().any(|p| matches!(p.type_annotation, Some(Type::Int) | Some(Type::Float)));
            if !has_num {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@positive function '{}' requires at least one numeric parameter (int or float)", function.name),
                ));
            }
        }

        let is_checked = function.attributes.iter().any(|a| a.name == "checked");
        let is_saturating = function.attributes.iter().any(|a| a.name == "saturating");
        let is_wrapping = function.attributes.iter().any(|a| a.name == "wrapping");
        if is_checked && is_saturating {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @checked and @saturating", function.name),
            ));
        }
        if is_wrapping && is_checked {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @wrapping and @checked", function.name),
            ));
        }
        if is_wrapping && is_saturating {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @wrapping and @saturating", function.name),
            ));
        }

        if is_leaf && function.attributes.iter().any(|a| a.name == "tailrec") {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @leaf and @tailrec", function.name),
            ));
        }
        if is_leaf && function.attributes.iter().any(|a| a.name == "retry") {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @leaf and @retry", function.name),
            ));
        }
        if is_leaf && function.attributes.iter().any(|a| a.name == "max_depth") {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @leaf and @max_depth", function.name),
            ));
        }

        if is_nothrow && is_fallible {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @nothrow and @fallible", function.name),
            ));
        }

        if let Some(depth_attr) = function.attributes.iter().find(|a| a.name == "max_depth") {
            let mut limit: Option<usize> = None;
            for arg in &depth_attr.args {
                if arg.name.as_deref() == Some("limit") || arg.name.as_deref() == Some("depth") || arg.name.is_none() {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        if let Ok(val) = n.parse::<usize>() {
                            if val > 0 {
                                limit = Some(val);
                            }
                        }
                    }
                }
            }
            if limit.is_none() {
                return Err(self.error_at_span(
                    depth_attr.span,
                    &format!("@max_depth attribute on function '{}' requires a positive limit", function.name),
                ));
            }
        }

        if let Some(bounded_attr) = function.attributes.iter().find(|a| a.name == "bounded") {
            let mut min_val: Option<f64> = None;
            let mut max_val: Option<f64> = None;
            if bounded_attr.args.len() == 2 && bounded_attr.args.iter().all(|a| a.name.is_none()) {
                if let ExprKind::Number(n) = &bounded_attr.args[0].value.kind {
                    min_val = n.parse::<f64>().ok();
                }
                if let ExprKind::Number(n) = &bounded_attr.args[1].value.kind {
                    max_val = n.parse::<f64>().ok();
                }
            } else {
                for arg in &bounded_attr.args {
                    if arg.name.as_deref() == Some("min") {
                        if let ExprKind::Number(n) = &arg.value.kind {
                            min_val = n.parse::<f64>().ok();
                        }
                    } else if arg.name.as_deref() == Some("max") {
                        if let ExprKind::Number(n) = &arg.value.kind {
                            max_val = n.parse::<f64>().ok();
                        }
                    }
                }
            }
            if let (Some(mi), Some(ma)) = (min_val, max_val) {
                if mi > ma {
                    return Err(self.error_at_span(
                        bounded_attr.span,
                        &format!("@bounded: min ({}) cannot be greater than max ({})", mi, ma),
                    ));
                }
                self.bounded_functions.insert(function.name.clone(), (mi, ma));
            }
            let has_num = function.params.iter().any(|p| matches!(p.type_annotation, Some(Type::Int) | Some(Type::Float)));
            if !has_num {
                return Err(self.error_at_span(
                    bounded_attr.span,
                    &format!("@bounded function '{}' requires at least one numeric parameter (int or float)", function.name),
                ));
            }
        }

        let is_must_use = function.attributes.iter().any(|a| a.name == "must_use");
        if is_must_use
            && (function.return_type.is_none() || function.return_type == Some(Type::Void))
        {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' marked with @must_use must have a return value",
                    function.name
                ),
            ));
        }

        let is_nodiscard = function.attributes.iter().any(|a| a.name == "nodiscard" || a.name == "no_discard");
        if is_nodiscard
            && (function.return_type.is_none() || function.return_type == Some(Type::Void))
        {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' marked with @nodiscard must have a return value",
                    function.name
                ),
            ));
        }

        if is_pure && (function.return_type.is_none() || function.return_type == Some(Type::Void)) {
            return Err(self.error_at_span(
                function.span,
                &format!("@pure function '{}' must return a value", function.name),
            ));
        }

        let is_inline = function.attributes.iter().any(|a| a.name == "inline");
        if is_inline && count_recursive_calls(&function.body, &function.name) > 0 {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with @inline because it is recursive",
                    function.name
                ),
            ));
        }

        let is_no_inline = function.attributes.iter().any(|a| a.name == "no_inline");
        if is_no_inline && is_inline {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @inline and @no_inline",
                    function.name
                ),
            ));
        }

        let is_inline_always = function.attributes.iter().any(|a| a.name == "inline_always" || a.name == "always_inline");
        if is_inline_always && count_recursive_calls(&function.body, &function.name) > 0 {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with @inline_always because it is recursive",
                    function.name
                ),
            ));
        }
        if is_inline_always && is_no_inline {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @inline_always and @no_inline",
                    function.name
                ),
            ));
        }

        let is_cold = function.attributes.iter().any(|a| a.name == "cold");
        if is_cold && is_inline {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @cold and @inline",
                    function.name
                ),
            ));
        }
        if is_cold && is_inline_always {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @inline_always and @cold",
                    function.name
                ),
            ));
        }
        if is_cold && is_benchmark {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @cold and @benchmark",
                    function.name
                ),
            ));
        }

        let is_hot = function.attributes.iter().any(|a| a.name == "hot");
        if is_hot && is_cold {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @hot and @cold",
                    function.name
                ),
            ));
        }
        if is_hot && is_unreachable {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @hot and @unreachable",
                    function.name
                ),
            ));
        }

        let is_profiled = function.attributes.iter().any(|a| a.name == "profiled");
        if is_profiled && is_cold {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @profiled and @cold",
                    function.name
                ),
            ));
        }

        let is_unroll = function.attributes.iter().any(|a| a.name == "unroll");
        if is_unroll && is_cold {
            return Err(self.error_at_span(
                function.span,
                &format!(
                    "Function '{}' cannot be marked with both @unroll and @cold",
                    function.name
                ),
            ));
        }

        if let Some(budget_attr) = function.attributes.iter().find(|a| a.name == "budget") {
            let mut valid = false;
            for arg in &budget_attr.args {
                if arg.name.as_deref() == Some("instructions")
                    || arg.name.as_deref() == Some("limit")
                    || arg.name.is_none()
                {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        if let Ok(val) = n.parse::<u64>() {
                            if val > 0 {
                                valid = true;
                            }
                        }
                    }
                }
            }
            if !valid {
                return Err(self.error_at_span(
                    budget_attr.span,
                    &format!(
                        "@budget attribute on function '{}' requires a positive number of instructions",
                        function.name
                    ),
                ));
            }
        }

        let is_nonreentrant = function.attributes.iter().any(|a| a.name == "nonreentrant");
        let is_reentrant = function.attributes.iter().any(|a| a.name == "reentrant");
        if is_reentrant && is_nonreentrant {
            return Err(self.error_at_span(
                function.span,
                &format!("Function '{}' cannot be marked with both @reentrant and @nonreentrant", function.name),
            ));
        }
        if is_nonreentrant {
            let recursive_count = count_recursive_calls(&function.body, &function.name);
            if recursive_count > 0 {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' marked with @nonreentrant cannot call itself",
                        function.name
                    ),
                ));
            }
        }

        if let Some(retry_attr) = function.attributes.iter().find(|a| a.name == "retry") {
            let mut count_val: Option<usize> = None;
            for arg in &retry_attr.args {
                if arg.name.as_deref() == Some("count")
                    || arg.name.as_deref() == Some("attempts")
                    || arg.name.is_none()
                {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        if let Ok(val) = n.parse::<usize>() {
                            if val > 0 {
                                count_val = Some(val);
                            }
                        }
                    }
                }
            }
            let count = count_val.ok_or_else(|| {
                self.error_at_span(
                    retry_attr.span,
                    &format!(
                        "@retry attribute on function '{}' requires a positive integer count",
                        function.name
                    ),
                )
            })?;
            if is_nonreentrant {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' cannot be marked with both @retry and @nonreentrant",
                        function.name
                    ),
                ));
            }
            let returns_result = match &function.return_type {
                Some(Type::Data(name, _)) => name == "Result",
                _ => false,
            };
            if !returns_result && !is_fallible {
                return Err(self.error_at_span(
                    retry_attr.span,
                    &format!(
                        "@retry function '{}' must return a Result type or be marked @fallible",
                        function.name
                    ),
                ));
            }
            self.retry_functions.insert(function.name.clone(), count);
        }

        let is_memoize = function.attributes.iter().any(|a| a.name == "memoize");
        if is_memoize {
            if function.params.is_empty() {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' marked with @memoize must take at least one parameter",
                        function.name
                    ),
                ));
            }
            if function.return_type.is_none() || function.return_type == Some(Type::Void) {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' marked with @memoize must return a value",
                        function.name
                    ),
                ));
            }
            if !is_pure && !function.attributes.iter().any(|a| a.name == "read") {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' marked with @memoize must also be marked @pure or @read",
                        function.name
                    ),
                ));
            }
        }

        let is_tailrec = function.attributes.iter().any(|a| a.name == "tailrec");
        if is_tailrec {
            if is_nonreentrant {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' cannot be marked with both @tailrec and @nonreentrant",
                        function.name
                    ),
                ));
            }
            let recursive_count = count_recursive_calls(&function.body, &function.name);
            if recursive_count == 0 {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' is marked with @tailrec but contains no recursive calls",
                        function.name
                    ),
                ));
            }
            self.validate_tail_recursion(&function.body, &function.name)?;
        }

        let is_audit = function.attributes.iter().any(|a| a.name == "audit");
        if is_audit {
            if is_unreachable {
                return Err(self.error_at_span(
                    function.span,
                    &format!("Function '{}' cannot be marked with both @audit and @unreachable", function.name),
                ));
            }
            self.audit_functions.insert(function.name.clone());
        }

        if let Some(rate_limit_attr) = function.attributes.iter().find(|a| a.name == "rate_limit") {
            let mut limit_val: Option<usize> = None;
            for arg in &rate_limit_attr.args {
                if arg.name.as_deref() == Some("calls")
                    || arg.name.as_deref() == Some("limit")
                    || arg.name.as_deref() == Some("max")
                    || arg.name.is_none()
                {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        if let Ok(val) = n.parse::<usize>() {
                            if val > 0 {
                                limit_val = Some(val);
                            }
                        }
                    }
                }
            }
            let limit = limit_val.ok_or_else(|| {
                self.error_at_span(
                    rate_limit_attr.span,
                    &format!(
                        "@rate_limit attribute on function '{}' requires a positive integer limit",
                        function.name
                    ),
                )
            })?;
            if is_unreachable {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' cannot be marked with both @rate_limit and @unreachable",
                        function.name
                    ),
                ));
            }
            self.rate_limited_functions.insert(function.name.clone(), limit);
        }

        if let Some(timeout_attr) = function.attributes.iter().find(|a| a.name == "timeout") {
            let mut timeout_val: Option<u64> = None;
            for arg in &timeout_attr.args {
                if arg.name.as_deref() == Some("ms")
                    || arg.name.as_deref() == Some("limit")
                    || arg.name.as_deref() == Some("millis")
                    || arg.name.is_none()
                {
                    if let ExprKind::Number(n) = &arg.value.kind {
                        if let Ok(val) = n.parse::<u64>() {
                            if val > 0 {
                                timeout_val = Some(val);
                            }
                        }
                    }
                }
            }
            let timeout = timeout_val.ok_or_else(|| {
                self.error_at_span(
                    timeout_attr.span,
                    &format!(
                        "@timeout attribute on function '{}' requires a positive integer millisecond limit",
                        function.name
                    ),
                )
            })?;
            if is_unreachable {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' cannot be marked with both @timeout and @unreachable",
                        function.name
                    ),
                ));
            }
            self.timeout_functions.insert(function.name.clone(), timeout);
        }

        self.current_function_params.clear();
        for param in &function.params {
            self.current_function_params.insert(param.name.clone());
            let param_type = param.type_annotation.clone().unwrap_or_else(|| {
                if param.name == "self" {
                    self_type.cloned().unwrap_or(Type::Unknown)
                } else {
                    Type::Unknown
                }
            });
            let symbol = Symbol {
                _name: param.name.clone(),
                ty: param_type,
                mutable: param.mutable,
                mode: self.current_mode,
            };
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(param.name.clone(), symbol);
            }
        }

        for statement in &function.body {
            self.analyze_statement(statement)?;
        }

        if let Some(return_type) = &function.return_type {
            if *return_type != Type::Void && !self.block_terminates(&function.body) {
                return Err(self.error_at_span(
                    function.span,
                    &format!(
                        "Function '{}' can complete without returning {}",
                        function.name, return_type
                    ),
                ));
            }
        }

        if is_must_mutate {
            let mutated = function.params.iter().any(|p| p.mutable && self.current_function_mutated_params.contains(&p.name));
            if !mutated {
                return Err(self.error_at_span(
                    function.span,
                    &format!("@must_mutate function '{}' does not mutate any of its mutable parameters", function.name),
                ));
            }
        }

        self.current_return_type = previous_return;
        self.current_autocheck = previous_autocheck;
        self.current_mode = previous_mode;
        self.in_read_only_function = previous_read_only;
        self.in_pure_function = previous_pure;
        self.in_safe_function = previous_safe;
        self.in_strict_function = previous_strict;
        self.in_const_function = previous_const;
        self.in_no_panic_function = previous_no_panic;
        self.in_infallible_function = previous_infallible;
        self.in_deterministic_function = previous_deterministic;
        self.in_zero_alloc_function = previous_zero_alloc;
        self.in_isolated_function = previous_isolated;
        self.in_no_float_function = previous_no_float;
        self.in_test_function = previous_test;
        self.in_leaf_function = previous_leaf;
        self.in_nothrow_function = previous_nothrow;
        self.in_no_io_function = previous_no_io;
        self.in_benchmark_function = previous_benchmark;
        self.in_experimental_function = previous_experimental;
        self.in_simd_function = previous_simd;
        self.in_idempotent_function = previous_idempotent;
        self.in_thread_safe_function = previous_thread_safe;
        self.in_pure_math_function = previous_pure_math;
        self.in_bitwise_only_function = previous_bitwise_only;
        self.in_pure_string_function = previous_pure_string;
        self.in_pure_array_function = previous_pure_array;
        self.current_function_name = previous_function_name;
        self.current_source = previous_source;
        self.current_function_params.clear();
        self.current_function_mutated_params = previous_mutated;
        self.scopes.pop();
        Ok(())
    }

    fn block_terminates(&self, statements: &[Stmt]) -> bool {
        statements
            .iter()
            .any(|statement| self.statement_terminates(statement))
    }

    fn statement_terminates(&self, statement: &Stmt) -> bool {
        match &statement.kind {
            StmtKind::Return(_) | StmtKind::Error { .. } | StmtKind::Fail { .. } => true,
            StmtKind::Block(statements) => self.block_terminates(statements),
            StmtKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.block_terminates(then_branch)
                    && else_branch
                        .as_ref()
                        .is_some_and(|branch| self.block_terminates(branch))
            }
            StmtKind::Match { condition, arms } => {
                !arms.is_empty()
                    && self.match_is_exhaustive(condition, arms)
                    && arms.iter().all(|arm| self.statement_terminates(&arm.body))
            }
            StmtKind::While { condition, body } => {
                matches!(condition.kind, ExprKind::Bool(true))
                    && !Self::block_breaks_current_loop(body)
            }
            _ => false,
        }
    }

    fn block_breaks_current_loop(statements: &[Stmt]) -> bool {
        statements.iter().any(Self::statement_breaks_current_loop)
    }

    fn statement_breaks_current_loop(statement: &Stmt) -> bool {
        match &statement.kind {
            StmtKind::Break => true,
            StmtKind::Block(statements) => Self::block_breaks_current_loop(statements),
            StmtKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                Self::block_breaks_current_loop(then_branch)
                    || else_branch
                        .as_ref()
                        .is_some_and(|branch| Self::block_breaks_current_loop(branch))
            }
            StmtKind::Match { arms, .. } => arms
                .iter()
                .any(|arm| Self::statement_breaks_current_loop(&arm.body)),
            StmtKind::While { .. } | StmtKind::ForLoop { .. } | StmtKind::ForEach { .. } => false,
            _ => false,
        }
    }

    fn match_is_exhaustive(&self, condition: &Expr, arms: &[MatchArm]) -> bool {
        if arms
            .iter()
            .any(|arm| arm.patterns.iter().any(Self::is_wildcard_pattern))
        {
            return true;
        }

        let Ok(condition_type) = self.analyze_expression(condition) else {
            return false;
        };
        match condition_type {
            Type::Bool => {
                let mut has_true = false;
                let mut has_false = false;
                for pattern in arms.iter().flat_map(|arm| &arm.patterns) {
                    match pattern.kind {
                        ExprKind::Bool(true) => has_true = true,
                        ExprKind::Bool(false) => has_false = true,
                        _ => {}
                    }
                }
                has_true && has_false
            }
            Type::Enum(name) | Type::Data(name, _) => {
                if self.non_exhaustive_enums.contains(&name) {
                    return false;
                }
                let Some(variants) = self.enum_defs.get(&name) else {
                    return false;
                };
                let covered = arms
                    .iter()
                    .flat_map(|arm| &arm.patterns)
                    .filter_map(Self::enum_variant_pattern)
                    .filter_map(|(enum_name, variant_name)| {
                        (enum_name == name).then_some(variant_name)
                    })
                    .collect::<HashSet<_>>();
                variants
                    .iter()
                    .all(|variant| covered.contains(variant.as_str()))
            }
            _ => false,
        }
    }

    fn is_wildcard_pattern(pattern: &Expr) -> bool {
        matches!(&pattern.kind, ExprKind::Identifier(name) if name == "_")
    }

    fn enum_variant_pattern(pattern: &Expr) -> Option<(&str, &str)> {
        match &pattern.kind {
            ExprKind::EnumVariant {
                enum_name,
                variant_name,
            } => Some((enum_name, variant_name)),
            ExprKind::Get { entity, name } => match &entity.kind {
                ExprKind::Identifier(enum_name) => Some((enum_name, name)),
                _ => None,
            },
            _ => None,
        }
    }

    fn validate_match_patterns(
        &self,
        condition_type: &Type,
        arms: &[MatchArm],
    ) -> Result<(), ViceError> {
        let mut wildcard_seen = false;
        for (arm_index, arm) in arms.iter().enumerate() {
            for pattern in &arm.patterns {
                if Self::is_wildcard_pattern(pattern) {
                    if arm.patterns.len() != 1 {
                        return Err(self.error_at_span(
                            pattern.span,
                            "Wildcard must be the only pattern in a match arm",
                        ));
                    }
                    if arm_index + 1 != arms.len() {
                        return Err(
                            self.error_at_span(pattern.span, "Wildcard match arm must be last")
                        );
                    }
                    wildcard_seen = true;
                    continue;
                }
                if wildcard_seen {
                    return Err(self.error_at_span(pattern.span, "Unreachable match arm"));
                }

                match &pattern.kind {
                    ExprKind::Range { start, end } => {
                        let start_type = self.analyze_expression(start)?;
                        let end_type = self.analyze_expression(end)?;
                        if (!self.is_numeric(&start_type) && start_type != Type::Unknown)
                            || (!self.is_numeric(&end_type) && end_type != Type::Unknown)
                        {
                            return Err(self.error_at_span(
                                pattern.span,
                                "Match range boundaries must be numeric",
                            ));
                        }
                        for boundary_type in [&start_type, &end_type] {
                            if !self.types_compatible(
                                condition_type,
                                boundary_type,
                                self.current_mode,
                            ) {
                                return Err(self.error_at_span(
                                    pattern.span,
                                    &format!(
                                        "Match pattern type {} is incompatible with {}",
                                        boundary_type, condition_type
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {
                        let pattern_type = self.analyze_expression(pattern)?;
                        if !self.types_compatible(condition_type, &pattern_type, self.current_mode)
                        {
                            return Err(self.error_at_span(
                                pattern.span,
                                &format!(
                                    "Match pattern type {} is incompatible with {}",
                                    pattern_type, condition_type
                                ),
                            ));
                        }
                    }
                }
            }
        }
        if let Type::Enum(enum_name) = condition_type {
            if self.non_exhaustive_enums.contains(enum_name) && !wildcard_seen {
                let span = arms.first().map(|a| a.body.span).unwrap_or_else(|| crate::ast::Span::new(0));
                return Err(self.error_at_span(
                    span,
                    &format!("Matching @non_exhaustive enum '{}' must include a wildcard '_' arm", enum_name),
                ));
            }
        }
        Ok(())
    }

    fn analyze_statement(&mut self, statement: &Stmt) -> Result<(), ViceError> {
        match &statement.kind {
            StmtKind::Variable {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                if self.in_const_function && *mutable {
                    return Err(self.error(
                        statement.span.line,
                        &format!(
                            "Cannot declare mutable variable '{}' inside @const function",
                            name
                        ),
                    ));
                }
                if self.in_strict_function && type_annotation.is_none() {
                    return Err(self.error(
                        statement.span.line,
                        &format!(
                            "Variable '{}' in @strict function '{}' must specify an explicit type annotation",
                            name,
                            self.current_function_name.as_deref().unwrap_or("anonymous")
                        ),
                    ));
                }
                if let Some(expected) = type_annotation {
                    let value_type = self.analyze_expression(value)?;
                    if !self.types_compatible(expected, &value_type, self.current_mode) {
                        return Err(self.error(
                            statement.span.line,
                            &format!(
                                "Type mismatch for '{}': expected {}, got {}",
                                name, expected, value_type
                            ),
                        ));
                    }
                    let stored_type = expected.clone();
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert(
                            name.clone(),
                            Symbol {
                                _name: name.clone(),
                                ty: stored_type,
                                mutable: *mutable,
                                mode: self.current_mode,
                            },
                        );
                    }
                } else {
                    let value_type = self.analyze_expression(value)?;
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert(
                            name.clone(),
                            Symbol {
                                _name: name.clone(),
                                ty: value_type,
                                mutable: *mutable,
                                mode: self.current_mode,
                            },
                        );
                    }
                }
            }
            StmtKind::Output(expressions) | StmtKind::Emit(expressions) => {
                if self.in_const_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot perform I/O (output/emit) inside @const function",
                    ));
                }
                if self.in_pure_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot perform I/O (output/emit) inside pure function",
                    ));
                }
                if self.in_isolated_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot perform I/O (output/emit) inside @isolated function",
                    ));
                }
                if self.in_no_io_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot perform I/O (output/emit) inside @no_io function",
                    ));
                }
                if self.in_simd_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot perform I/O (output/emit) inside @simd function",
                    ));
                }
                for expr in expressions {
                    self.analyze_expression(expr)?;
                }
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.analyze_expression(condition)?;
                if !self.types_compatible(&Type::Bool, &cond_type, self.current_mode) {
                    return Err(self.error(
                        condition.span.line,
                        &format!("If condition must be bool, got {}", cond_type),
                    ));
                }
                self.analyze_block(then_branch)?;
                if let Some(else_stmts) = else_branch {
                    self.analyze_block(else_stmts)?;
                }
            }
            StmtKind::While { condition, body } => {
                let cond_type = self.analyze_expression(condition)?;
                if !self.types_compatible(&Type::Bool, &cond_type, self.current_mode) {
                    return Err(self.error(
                        condition.span.line,
                        &format!("While condition must be bool, got {}", cond_type),
                    ));
                }
                self.loop_depth += 1;
                self.analyze_block(body)?;
                self.loop_depth -= 1;
            }
            StmtKind::ForLoop {
                iterator,
                start,
                end,
                body,
            } => {
                let start_ty = self.analyze_expression(start)?;
                let end_ty = self.analyze_expression(end)?;
                if !self.types_compatible(&Type::Int, &start_ty, self.current_mode)
                    || !self.types_compatible(&Type::Int, &end_ty, self.current_mode)
                {
                    return Err(self.error(statement.span.line, "For loop range must be integers"));
                }
                self.loop_depth += 1;
                self.scopes.push(HashMap::new());
                self.define_local(iterator, Type::Int, false);
                self.analyze_block_without_new_scope(body)?;
                self.scopes.pop();
                self.loop_depth -= 1;
            }
            StmtKind::ForEach {
                iterator,
                collection,
                body,
            } => {
                let collection_ty = self.analyze_expression(collection)?;
                let item_ty = match collection_ty {
                    Type::Array(inner) => *inner,
                    Type::Unknown => Type::Unknown,
                    other => {
                        return Err(self.error(
                            collection.span.line,
                            &format!("ForEach expects array, got {}", other),
                        ))
                    }
                };
                self.loop_depth += 1;
                self.scopes.push(HashMap::new());
                self.define_local(iterator, item_ty, false);
                self.analyze_block_without_new_scope(body)?;
                self.scopes.pop();
                self.loop_depth -= 1;
            }
            StmtKind::Assign { name, value } => {
                if self.current_function_params.contains(name) {
                    self.current_function_mutated_params.insert(name.clone());
                }
                if self.in_const_function {
                    return Err(self.error(
                        statement.span.line,
                        &format!(
                            "Cannot mutate variable '{}' inside @const function",
                            name
                        ),
                    ));
                }
                if (self.in_read_only_function || self.in_pure_function) && self.is_global(name) {
                    let fn_kind = if self.in_pure_function { "pure" } else { "read-only" };
                    return Err(self.error(
                        statement.span.line,
                        &format!(
                            "Cannot assign to global variable '{}' inside {} function",
                            name, fn_kind
                        ),
                    ));
                }
                if self.in_isolated_function && self.is_global(name) {
                    return Err(self.error(
                        statement.span.line,
                        &format!("Cannot mutate global variable '{}' inside @isolated function", name),
                    ));
                }
                if self.in_idempotent_function && self.is_global(name) {
                    return Err(self.error(
                        statement.span.line,
                        &format!("Cannot mutate global variable '{}' inside @idempotent function", name),
                    ));
                }
                if self.in_thread_safe_function && self.is_global(name) {
                    return Err(self.error(
                        statement.span.line,
                        &format!("Cannot mutate global variable '{}' inside @thread_safe function", name),
                    ));
                }
                if self.in_pure_string_function && self.is_global(name) {
                    return Err(self.error(
                        statement.span.line,
                        &format!("Cannot mutate global variable '{}' inside @pure_string function", name),
                    ));
                }
                if self.in_pure_array_function && self.is_global(name) {
                    return Err(self.error(
                        statement.span.line,
                        &format!("Cannot mutate global variable '{}' inside @pure_array function", name),
                    ));
                }
                if self.in_pure_function && self.current_function_params.contains(name) {
                    return Err(self.error(
                        statement.span.line,
                        &format!(
                            "Cannot mutate parameter '{}' inside pure function",
                            name
                        ),
                    ));
                }
                let value_type = self.analyze_expression(value)?;
                let symbol = self.resolve_variable(name).cloned().ok_or_else(|| {
                    self.error(
                        statement.span.line,
                        &format!("Cannot assign to undefined variable: {}", name),
                    )
                })?;
                if !symbol.mutable {
                    return Err(self.error(
                        statement.span.line,
                        &format!("Cannot assign to immutable variable: {}", name),
                    ));
                }
                if !self.types_compatible(&symbol.ty, &value_type, symbol.mode) {
                    return Err(self.error(
                        statement.span.line,
                        &format!("Type mismatch: expected {}, got {}", symbol.ty, value_type),
                    ));
                }
            }
            StmtKind::DerefAssign { pointer, value } => {
                if self.in_read_only_function || self.in_pure_function {
                    if let Some(root_var) = self.get_root_variable(pointer) {
                        if self.is_global(root_var)
                            || self.current_function_params.contains(root_var)
                        {
                            let fn_kind = if self.in_pure_function { "pure" } else { "read-only" };
                            return Err(self.error(
                                statement.span.line,
                                &format!("Cannot assign through reference of parameter or global variable '{}' inside {} function", root_var, fn_kind),
                            ));
                        }
                    }
                }
                if self.in_isolated_function {
                    if let Some(root_var) = self.get_root_variable(pointer) {
                        if self.is_global(root_var) {
                            return Err(self.error(
                                statement.span.line,
                                &format!("Cannot assign through reference of global variable '{}' inside @isolated function", root_var),
                            ));
                        }
                    }
                }
                let pointer_type = self.analyze_expression(pointer)?;
                let value_type = self.analyze_expression(value)?;

                if let Type::Reference { mutable, inner } = pointer_type {
                    if !mutable {
                        return Err(self.error(
                            statement.span.line,
                            "Cannot assign through an immutable reference",
                        ));
                    }
                    if !self.types_compatible(&inner, &value_type, self.current_mode) {
                        return Err(self.error(
                            statement.span.line,
                            &format!("Type mismatch: expected {}, got {}", inner, value_type),
                        ));
                    }
                } else {
                    return Err(self.error(
                        statement.span.line,
                        &format!(
                            "Cannot dereference and assign to non-reference type: {}",
                            pointer_type
                        ),
                    ));
                }
            }
            StmtKind::MemberAssign {
                entity,
                name,
                value,
            } => {
                if let Some(root_var) = self.get_root_variable(entity) {
                    if self.current_function_params.contains(root_var) {
                        self.current_function_mutated_params.insert(root_var.to_string());
                    }
                }
                if self.in_read_only_function || self.in_pure_function {
                    if let Some(root_var) = self.get_root_variable(entity) {
                        if self.is_global(root_var)
                            || self.current_function_params.contains(root_var)
                        {
                            let fn_kind = if self.in_pure_function { "pure" } else { "read-only" };
                            return Err(self.error(
                                statement.span.line,
                                &format!("Cannot mutate field of parameter or global variable '{}' inside {} function", root_var, fn_kind),
                            ));
                        }
                    }
                }
                if self.in_isolated_function {
                    if let Some(root_var) = self.get_root_variable(entity) {
                        if self.is_global(root_var) {
                            return Err(self.error(
                                statement.span.line,
                                &format!("Cannot mutate field of global variable '{}' inside @isolated function", root_var),
                            ));
                        }
                    }
                }
                if self.in_const_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot mutate field of entity inside @const function",
                    ));
                }
                let entity_type = self.analyze_expression(entity)?;
                let value_type = self.analyze_expression(value)?;
                if let Type::Data(data_name, _) = entity_type {
                    if self.record_datas.contains(&data_name) {
                        return Err(self.error(
                            statement.span.line,
                            &format!("Cannot mutate field of @record type: {}", data_name),
                        ));
                    }
                    if self.read_only_datas.contains(&data_name) {
                        return Err(self.error(
                            statement.span.line,
                            &format!("Cannot mutate field of read-only data type: {}", data_name),
                        ));
                    }
                    let expected = self.field_type(&data_name, name, statement.span.line)?;
                    if !self.types_compatible(&expected, &value_type, self.current_mode) {
                        return Err(self.error(
                            statement.span.line,
                            &format!(
                                "Type mismatch for field '{}.{}': expected {}, got {}",
                                data_name, name, expected, value_type
                            ),
                        ));
                    }
                }
            }
            StmtKind::IndexAssign {
                entity,
                index,
                value,
            } => {
                if self.in_const_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot mutate array or map index inside @const function",
                    ));
                }
                if let Some(root_var) = self.get_root_variable(entity) {
                    if self.current_function_params.contains(root_var) {
                        self.current_function_mutated_params.insert(root_var.to_string());
                    }
                }
                if self.in_read_only_function || self.in_pure_function {
                    if let Some(root_var) = self.get_root_variable(entity) {
                        if self.is_global(root_var)
                            || self.current_function_params.contains(root_var)
                        {
                            let fn_kind = if self.in_pure_function { "pure" } else { "read-only" };
                            return Err(self.error(
                                statement.span.line,
                                &format!("Cannot mutate array/map of parameter or global variable '{}' inside {} function", root_var, fn_kind),
                            ));
                        }
                    }
                }
                let entity_ty = self.analyze_expression(entity)?;
                let index_ty = self.analyze_expression(index)?;
                let value_ty = self.analyze_expression(value)?;

                let mut effective_entity = entity_ty.clone();
                if let Type::Reference { inner, .. } = effective_entity {
                    effective_entity = *inner;
                }

                match effective_entity {
                    Type::Array(inner) => {
                        if !self.types_compatible(&Type::Int, &index_ty, self.current_mode) {
                            return Err(self.error(index.span.line, "Array index must be int"));
                        }
                        if !self.types_compatible(&inner, &value_ty, self.current_mode) {
                            return Err(self
                                .error(statement.span.line, "Type mismatch in array assignment"));
                        }
                    }
                    Type::Map(key_ty, val_ty) => {
                        if !self.types_compatible(&key_ty, &index_ty, self.current_mode) {
                            return Err(self.error(
                                index.span.line,
                                &format!("Map key must be {}, got {}", key_ty, index_ty),
                            ));
                        }
                        if !self.types_compatible(&val_ty, &value_ty, self.current_mode) {
                            return Err(
                                self.error(statement.span.line, "Type mismatch in map assignment")
                            );
                        }
                    }
                    Type::Unknown => {}
                    _ => {
                        return Err(self.error(
                            statement.span.line,
                            "Indexing assignment requires an array or map",
                        ))
                    }
                }
            }
            StmtKind::Return(expr) => {
                let actual = if let Some(e) = expr {
                    if let Some(&(min_val, max_val)) = self
                        .current_function_name
                        .as_ref()
                        .and_then(|fn_name| self.clamped_functions.get(fn_name))
                    {
                        if let ExprKind::Number(ref n) = e.kind {
                            if let Ok(num) = n.parse::<f64>() {
                                if num < min_val || num > max_val {
                                    return Err(self.error(
                                        statement.span.line,
                                        &format!(
                                            "Constant return value {} is outside @clamp({}, {}) range on function '{}'",
                                            n, min_val, max_val, self.current_function_name.as_deref().unwrap_or("")
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    self.analyze_expression(e)?
                } else {
                    Type::Void
                };
                if let Some(expected) = self.current_return_type.clone() {
                    if !self.types_compatible(&expected, &actual, self.current_mode) {
                        return Err(self.error(
                            statement.span.line,
                            &format!(
                                "Return type mismatch: expected {}, got {}",
                                expected, actual
                            ),
                        ));
                    }
                }
            }
            StmtKind::Block(statements) => self.analyze_block(statements)?,
            StmtKind::Input(name) => {
                if self.in_const_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot read input inside @const function",
                    ));
                }
                if self.in_pure_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot read input inside pure function",
                    ));
                }
                if self.in_read_only_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot read input inside read-only function",
                    ));
                }
                if self.in_isolated_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot read input inside @isolated function",
                    ));
                }
                if self.in_no_io_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot read input inside @no_io function",
                    ));
                }
                if self.in_simd_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot read input inside @simd function",
                    ));
                }
                self.define_local(name, Type::String, true);
            }
            StmtKind::Expression(expr) => {
                self.check_unused_must_use(expr, statement.span.line)?;
                self.analyze_expression(expr)?;
            }
            StmtKind::Error { args, message } => {
                if self.in_const_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot raise runtime error inside @const function",
                    ));
                }
                if self.in_no_panic_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot raise runtime error inside @no_panic function",
                    ));
                }
                if self.in_pure_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot raise runtime error inside pure function",
                    ));
                }
                if self.in_safe_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot raise runtime error inside @safe function",
                    ));
                }
                if self.in_infallible_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot raise runtime error inside @infallible function",
                    ));
                }
                for arg in args {
                    self.analyze_expression(&arg.value)?;
                }
                self.analyze_expression(message)?;
            }
            StmtKind::Fail { message } => {
                if self.in_const_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot fail inside @const function",
                    ));
                }
                if self.in_no_panic_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot fail inside @no_panic function",
                    ));
                }
                if self.in_pure_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot fail inside pure function",
                    ));
                }
                if self.in_safe_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot fail inside @safe function",
                    ));
                }
                if self.in_infallible_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot fail inside @infallible function",
                    ));
                }
                if self.in_nothrow_function {
                    return Err(self.error(
                        statement.span.line,
                        "Cannot fail inside @nothrow function",
                    ));
                }
                self.analyze_expression(message)?;
            }
            StmtKind::Break if self.loop_depth == 0 => {
                return Err(self.error(statement.span.line, "break outside of loop"));
            }
            StmtKind::Continue if self.loop_depth == 0 => {
                return Err(self.error(statement.span.line, "continue outside of loop"));
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Match { condition, arms } => {
                let condition_type = self.analyze_expression(condition)?;
                self.validate_match_patterns(&condition_type, arms)?;
                for arm in arms {
                    match &arm.body.kind {
                        StmtKind::Expression(expr) => {
                            self.check_unused_must_use(expr, arm.body.span.line)?;
                            self.analyze_expression(expr)?;
                        }
                        StmtKind::Block(stmts) => {
                            self.analyze_block(stmts)?;
                        }
                        _ => {
                            self.analyze_statement(&arm.body)?;
                        }
                    }
                }
            }
            StmtKind::DataDeclaration {
                name,
                attributes,
                fields,
                type_params,
            } => {
                let previous_mode = self.current_mode;
                self.current_mode = self.mode_for_attributes(attributes);
                let mut field_types = HashMap::new();
                let mut params = Vec::new();
                for field in fields {
                    self.validate_type_annotation(
                        &field.type_annotation,
                        type_params,
                        field.span.line,
                    )?;
                    let default_type = self.analyze_expression(&field.default_value)?;
                    let resolved_ty = self.resolve_generics(&field.type_annotation, type_params);
                    if !self.types_compatible(&resolved_ty, &default_type, self.current_mode) {
                        return Err(self.error(
                            field.span.line,
                            &format!(
                                "Type mismatch in data '{}' field '{}': expected {}, got {}",
                                name, field.name, resolved_ty, default_type
                            ),
                        ));
                    }
                    field_types.insert(field.name.clone(), resolved_ty.clone());
                    params.push(resolved_ty);
                }
                self.data_defs.insert(
                    name.clone(),
                    DataDef {
                        fields: field_types,
                    },
                );
                if let Some(attr) = attributes.iter().find(|a| a.name == "must_use" || a.name == "nodiscard" || a.name == "no_discard") {
                    self.must_use_types.insert(name.clone(), extract_attribute_message(attr));
                }
                let return_args = type_params
                    .iter()
                    .map(|p| Type::Generic(p.clone()))
                    .collect();
                self.define_local(
                    name,
                    Type::Function {
                        params,
                        return_type: Box::new(Type::Data(name.clone(), return_args)),
                    },
                    false,
                );
                self.current_mode = previous_mode;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_unused_must_use(&self, expr: &Expr, line: usize) -> Result<(), ViceError> {
        match &expr.kind {
            ExprKind::Call { name, .. } => {
                if let Some(msg_opt) = self.must_use_functions.get(name) {
                    let note = match msg_opt {
                        Some(m) => format!(": {}", m),
                        None => String::new(),
                    };
                    return Err(self.error(
                        line,
                        &format!("Unused return value from function '{}' marked with @must_use{}", name, note),
                    ));
                }
                if let Some(msg_opt) = self.must_use_types.get(name) {
                    let note = match msg_opt {
                        Some(m) => format!(": {}", m),
                        None => String::new(),
                    };
                    return Err(self.error(
                        line,
                        &format!("Unused value of type '{}' marked with @must_use{}", name, note),
                    ));
                }
                if let Some(symbol) = self.resolve_variable(name) {
                    if let Type::Function { return_type, .. } = &symbol.ty {
                        if let Type::Data(data_name, _) = return_type.as_ref() {
                            if let Some(msg_opt) = self.must_use_types.get(data_name) {
                                let note = match msg_opt {
                                    Some(m) => format!(": {}", m),
                                    None => String::new(),
                                };
                                return Err(self.error(
                                    line,
                                    &format!("Unused value of type '{}' marked with @must_use{}", data_name, note),
                                ));
                            }
                        }
                    }
                }
            }
            ExprKind::MethodCall { method, .. } => {
                if let Some(msg_opt) = self.must_use_functions.get(method) {
                    let note = match msg_opt {
                        Some(m) => format!(": {}", m),
                        None => String::new(),
                    };
                    return Err(self.error(
                        line,
                        &format!("Unused return value from method '{}' marked with @must_use{}", method, note),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_block(&mut self, statements: &[Stmt]) -> Result<(), ViceError> {
        self.scopes.push(HashMap::new());
        self.analyze_block_without_new_scope(statements)?;
        self.scopes.pop();
        Ok(())
    }

    fn analyze_block_without_new_scope(&mut self, statements: &[Stmt]) -> Result<(), ViceError> {
        for stmt in statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    fn define_local(&mut self, name: &str, ty: Type, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name.to_string(),
                Symbol {
                    _name: name.to_string(),
                    ty,
                    mutable,
                    mode: self.current_mode,
                },
            );
        }
    }

    fn resolve_variable(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    fn is_global(&self, name: &str) -> bool {
        if self.scopes.len() <= 1 {
            return true;
        }
        // Search local scopes (scopes[1..])
        for scope in self.scopes.iter().skip(1) {
            if scope.contains_key(name) {
                return false;
            }
        }
        true
    }

    fn get_root_variable<'a>(&self, expr: &'a Expr) -> Option<&'a String> {
        match &expr.kind {
            ExprKind::Identifier(name) => Some(name),
            ExprKind::Get { entity, .. } => self.get_root_variable(entity),
            ExprKind::Index { entity, .. } => self.get_root_variable(entity),
            ExprKind::Unary {
                operator: UnaryOperator::Dereference,
                right,
            } => self.get_root_variable(right),
            _ => None,
        }
    }

    fn analyze_expression(&self, expression: &Expr) -> Result<Type, ViceError> {
        let ty = self.analyze_expression_inner(expression)?;
        if self.in_no_float_function && ty == Type::Float {
            return Err(self.error(
                expression.span.line,
                "Floating point operation or value is not allowed inside @no_float function",
            ));
        }
        Ok(ty)
    }

    fn analyze_expression_inner(&self, expression: &Expr) -> Result<Type, ViceError> {
        match &expression.kind {
            ExprKind::Number(lit) => {
                if lit.contains('.') || lit.contains('e') || lit.contains('E') {
                    if self.in_no_float_function {
                        return Err(self.error(
                            expression.span.line,
                            &format!("Float literal '{}' is not allowed inside @no_float function", lit),
                        ));
                    }
                    if self.in_bitwise_only_function {
                        return Err(self.error(
                            expression.span.line,
                            &format!("Float literal '{}' is forbidden inside @bitwise_only function", lit),
                        ));
                    }
                    Ok(Type::Float)
                } else {
                    Ok(Type::Int)
                }
            }
            ExprKind::Bool(_) => Ok(Type::Bool),
            ExprKind::String(_) => Ok(Type::String),
            ExprKind::Identifier(name) => {
                if name == "_" {
                    Ok(Type::Unknown)
                } else if let Some(sym) = self.resolve_variable(name) {
                    if self.in_isolated_function
                        && self.is_global(name)
                        && !self.current_function_params.contains(name)
                        && !matches!(sym.ty, Type::Function { .. } | Type::Module(_))
                    {
                        return Err(self.error(
                            expression.span.line,
                            &format!("Cannot access global variable '{}' inside @isolated function", name),
                        ));
                    }
                    Ok(sym.ty.clone())
                } else if self.enum_defs.contains_key(name) {
                    Ok(Type::Enum(name.clone()))
                } else if self.data_defs.contains_key(name) {
                    Ok(Type::Data(name.clone(), vec![]))
                } else if crate::stdlib::BUILTIN_MODULES.contains(&name.as_str()) {
                    Ok(Type::Module(name.clone()))
                } else {
                    Err(self.error(
                        expression.span.line,
                        &format!("Undefined variable: {}", name),
                    ))
                }
            }
            ExprKind::Unary {
                operator: UnaryOperator::Reference,
                right,
            } => {
                let right_type = self.analyze_expression(right)?;
                let mutable = self
                    .get_root_variable(right)
                    .and_then(|name| self.resolve_variable(name))
                    .map(|symbol| symbol.mutable)
                    .unwrap_or(false);
                Ok(Type::Reference {
                    mutable,
                    inner: Box::new(right_type),
                })
            }
            ExprKind::Unary { operator, right } => {
                let right_type = self.analyze_expression(right)?;
                self.analyze_unary(expression.span.line, operator, right_type)
            }
            ExprKind::Array(elements) => {
                if self.in_zero_alloc_function {
                    return Err(self.error(
                        expression.span.line,
                        "Cannot allocate array literal inside a @zero_alloc function",
                    ));
                }
                let mut item_ty = Type::Unknown;
                for element in elements {
                    let element_ty = self.analyze_expression(element)?;
                    if item_ty == Type::Unknown {
                        item_ty = element_ty;
                    } else if !self.types_compatible(&item_ty, &element_ty, self.current_mode) {
                        return Err(self.error(
                            element.span.line,
                            &format!(
                                "Array elements must have one type, got {} and {}",
                                item_ty, element_ty
                            ),
                        ));
                    }
                }
                Ok(Type::Array(Box::new(item_ty)))
            }
            ExprKind::Map(entries) => {
                if self.in_zero_alloc_function {
                    return Err(self.error(
                        expression.span.line,
                        "Cannot allocate map literal inside a @zero_alloc function",
                    ));
                }
                let mut key_ty = Type::Unknown;
                let mut val_ty = Type::Unknown;
                for (key, val) in entries {
                    let k_ty = self.analyze_expression(key)?;
                    let v_ty = self.analyze_expression(val)?;
                    if key_ty == Type::Unknown {
                        key_ty = k_ty;
                    } else if !self.types_compatible(&key_ty, &k_ty, self.current_mode) {
                        return Err(self.error(
                            key.span.line,
                            &format!("Map keys must have one type, got {} and {}", key_ty, k_ty),
                        ));
                    }
                    if val_ty == Type::Unknown {
                        val_ty = v_ty;
                    } else if !self.types_compatible(&val_ty, &v_ty, self.current_mode) {
                        return Err(self.error(
                            val.span.line,
                            &format!("Map values must have one type, got {} and {}", val_ty, v_ty),
                        ));
                    }
                }
                Ok(Type::Map(Box::new(key_ty), Box::new(val_ty)))
            }
            ExprKind::Index { entity, index } => {
                let entity_type = self.analyze_expression(entity)?;
                let index_type = self.analyze_expression(index)?;

                let mut effective_entity = entity_type.clone();
                if let Type::Reference { inner, .. } = effective_entity {
                    effective_entity = *inner;
                }

                match effective_entity {
                    Type::Array(inner) => {
                        if !self.types_compatible(&Type::Int, &index_type, self.current_mode) {
                            return Err(self.error(
                                index.span.line,
                                &format!("Array index must be int, got {}", index_type),
                            ));
                        }
                        Ok(*inner)
                    }
                    Type::Map(key_ty, val_ty) => {
                        if !self.types_compatible(&key_ty, &index_type, self.current_mode) {
                            return Err(self.error(
                                index.span.line,
                                &format!("Map key must be {}, got {}", key_ty, index_type),
                            ));
                        }
                        Ok(*val_ty)
                    }
                    Type::Unknown => Ok(Type::Unknown),
                    other => Err(self.error(
                        entity.span.line,
                        &format!("Indexing requires array or map, got {}", other),
                    )),
                }
            }
            ExprKind::Get { entity, name } => {
                let entity_type = self.analyze_expression(entity)?;
                if let Type::Data(_, _) = &entity_type {
                    self.field_type_for_type(&entity_type, name, expression.span.line)
                } else if let Some(data_name) = self.as_data_name(&entity_type) {
                    self.field_type(&data_name, name, expression.span.line)
                } else if let Type::Enum(enum_name) = &entity_type {
                    if let Some(&defining_source) = self.private_enums.get(enum_name) {
                        if expression.span.source_id != defining_source {
                            return Err(self.error_at_span(
                                expression.span,
                                &format!(
                                    "Enum '{}' is marked @private and cannot be accessed from another file",
                                    enum_name
                                ),
                            ));
                        }
                    }
                    if let Some(reason_opt) = self.deprecated_items.get(enum_name).or_else(|| self.obsolete_items.get(enum_name)) {
                        let note = match reason_opt {
                            Some(reason) => format!(": {}", reason),
                            None => String::new(),
                        };
                        return Err(self.error(
                            expression.span.line,
                            &format!("Item '{}' is marked @obsolete and cannot be used{}", enum_name, note),
                        ));
                    }
                    if let Some(variants) = self.enum_defs.get(enum_name) {
                        if variants.contains(name) {
                            Ok(Type::Enum(enum_name.clone()))
                        } else {
                            Err(self.error(
                                expression.span.line,
                                &format!("Enum {} has no variant {}", enum_name, name),
                            ))
                        }
                    } else {
                        Ok(Type::Unknown)
                    }
                } else if entity_type == Type::Unknown {
                    Ok(Type::Unknown)
                } else {
                    Err(self.error(
                        expression.span.line,
                        &format!(
                            "Only data values and enums have fields, got {}",
                            entity_type
                        ),
                    ))
                }
            }
            ExprKind::PropertyCheck { entity, name } => {
                let entity_ty = self.analyze_expression(entity)?;
                if matches!(
                    &entity_ty,
                    Type::Data(data_name, _)
                        if (data_name == "NetListener" || data_name == "NetStream")
                            && name == "id"
                ) {
                    return Ok(Type::String);
                }
                Ok(Type::Unknown)
            }
            ExprKind::Match { condition, arms } => {
                if arms.is_empty() {
                    return Err(self.error(expression.span.line, "Match expression has no arms"));
                }
                let condition_type = self.analyze_expression(condition)?;
                self.validate_match_patterns(&condition_type, arms)?;
                let mut arm_type: Option<Type> = None;
                for arm in arms {
                    let t = match &arm.body.kind {
                        StmtKind::Expression(expr) => self.analyze_expression(expr)?,
                        _ => {
                            return Err(
                                self.error(expression.span.line, "Match arms must be expressions")
                            )
                        }
                    };
                    if let Some(existing) = &arm_type {
                        if !self.types_compatible(existing, &t, self.current_mode) {
                            return Err(self.error(
                                expression.span.line,
                                &format!(
                                    "Mismatched match arm types: expected {}, got {}",
                                    existing, t
                                ),
                            ));
                        }
                    } else {
                        arm_type = Some(t);
                    }
                }
                Ok(arm_type.unwrap_or(Type::Unknown))
            }
            ExprKind::Logical { left, right, .. } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;
                if self.types_compatible(&Type::Bool, &left_type, self.current_mode)
                    && self.types_compatible(&Type::Bool, &right_type, self.current_mode)
                {
                    Ok(Type::Bool)
                } else {
                    Err(self.error(
                        expression.span.line,
                        &format!(
                            "Logical operators expect bool, got {} and {}",
                            left_type, right_type
                        ),
                    ))
                }
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;
                self.analyze_binary(expression.span.line, operator, left_type, right_type)
            }
            ExprKind::Call {
                name,
                type_args,
                args,
            } => self.analyze_call(expression.span, name, type_args, args),
            ExprKind::MethodCall {
                entity,
                method,
                type_args,
                args,
            } => self.analyze_method_call(expression.span.line, entity, method, type_args, args),
            ExprKind::Lambda {
                params,
                return_type,
                ..
            } => {
                if self.in_leaf_function {
                    return Err(self.error(
                        expression.span.line,
                        "Cannot define lambda inside @leaf function",
                    ));
                }
                Ok(Type::Function {
                    params: params
                        .iter()
                        .map(|param| param.type_annotation.clone().unwrap_or(Type::Unknown))
                        .collect(),
                    return_type: Box::new(return_type.clone().unwrap_or(Type::Unknown)),
                })
            }
            ExprKind::Unwrap { expr } => {
                if self.in_no_panic_function {
                    return Err(self.error(
                        expression.span.line,
                        "Cannot use unwrap operator '!' inside @no_panic function; handle Option/Result safely",
                    ));
                }
                if self.in_safe_function {
                    return Err(self.error(
                        expression.span.line,
                        "Cannot use unwrap operator '!' inside @safe function; handle Option/Result safely",
                    ));
                }
                if self.in_infallible_function {
                    return Err(self.error(
                        expression.span.line,
                        "Cannot use unwrap operator '!' inside @infallible function; handle Option/Result safely",
                    ));
                }
                if self.in_nothrow_function {
                    return Err(self.error(
                        expression.span.line,
                        "Cannot use unwrap operator '!' inside @nothrow function; handle Option/Result safely",
                    ));
                }
                let inner_type = self.analyze_expression(expr)?;
                match &inner_type {
                    Type::Data(data_name, args) if data_name == "Option" || data_name == "Result" => {
                        Ok(args.first().cloned().unwrap_or(Type::Unknown))
                    }
                    Type::Unknown => Ok(Type::Unknown),
                    _ => Ok(inner_type),
                }
            }
            ExprKind::Try { expr } => {
                let inner_type = self.analyze_expression(expr)?;
                match &inner_type {
                    Type::Data(data_name, args) if data_name == "Option" || data_name == "Result" => {
                        Ok(args.first().cloned().unwrap_or(Type::Unknown))
                    }
                    Type::Unknown => Ok(Type::Unknown),
                    _ => Ok(inner_type),
                }
            }
            _ => Ok(Type::Unknown),
        }
    }

    fn analyze_unary(
        &self,
        line: usize,
        operator: &UnaryOperator,
        right: Type,
    ) -> Result<Type, ViceError> {
        match operator {
            UnaryOperator::Negate => {
                // Accept int or float (or unknown)
                match right {
                    Type::Int => Ok(Type::Int),
                    Type::Float => Ok(Type::Float),
                    Type::Unknown => Ok(Type::Unknown),
                    other => Err(self.error(
                        line,
                        &format!("Unary '-' expects numeric type, got {}", other),
                    )),
                }
            }
            UnaryOperator::Not => {
                if self.types_compatible(&Type::Bool, &right, self.current_mode) {
                    Ok(Type::Bool)
                } else {
                    Err(self.error(line, &format!("Unary '!' expects bool, got {}", right)))
                }
            }
            UnaryOperator::Reference => Ok(Type::Reference {
                mutable: true,
                inner: Box::new(right),
            }),
            UnaryOperator::Dereference => match right {
                Type::Reference { inner, .. } => Ok(*inner),
                Type::Unknown => Ok(Type::Unknown),
                other => Err(self.error(
                    line,
                    &format!("Cannot dereference non-reference type: {}", other),
                )),
            },
        }
    }

    fn analyze_binary(
        &self,
        line: usize,
        operator: &BinaryOperator,
        left: Type,
        right: Type,
    ) -> Result<Type, ViceError> {
        match operator {
            BinaryOperator::Add => match (&left, &right) {
                (Type::Int, Type::Int) => Ok(Type::Int),
                (Type::Float, Type::Float) => Ok(Type::Float),
                (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                (Type::String, Type::String)
                | (Type::String, Type::Int)
                | (Type::String, Type::Float)
                | (Type::String, Type::Bool)
                | (Type::Int, Type::String)
                | (Type::Float, Type::String)
                | (Type::Bool, Type::String) => Ok(Type::String),
                _ if left == Type::Unknown || right == Type::Unknown => Ok(Type::Unknown),
                _ => Err(self.error(line, &format!("Type error: {} + {}", left, right))),
            },
            BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Modulo => {
                // Numeric operations: if either is float -> float; if both int -> int
                if left == Type::Unknown || right == Type::Unknown {
                    Ok(Type::Unknown)
                } else if matches!(left, Type::Float) || matches!(right, Type::Float) {
                    Ok(Type::Float)
                } else if matches!(left, Type::Int) && matches!(right, Type::Int) {
                    Ok(Type::Int)
                } else {
                    Err(self.error(
                        line,
                        &format!(
                            "Arithmetic expects numeric types, got {} and {}",
                            left, right
                        ),
                    ))
                }
            }
            BinaryOperator::Divide => {
                if left == Type::Unknown || right == Type::Unknown {
                    Ok(Type::Unknown)
                } else if matches!(left, Type::Float) || matches!(right, Type::Float) {
                    Ok(Type::Float)
                } else if matches!(left, Type::Int) && matches!(right, Type::Int) {
                    if self.current_mode == HybridMode::HighLevel {
                        Ok(Type::Float)
                    } else {
                        Ok(Type::Int)
                    }
                } else {
                    Err(self.error(
                        line,
                        &format!(
                            "Arithmetic expects numeric types, got {} and {}",
                            left, right
                        ),
                    ))
                }
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if self.types_compatible(&left, &right, self.current_mode)
                    || self.types_compatible(&right, &left, self.current_mode)
                {
                    Ok(Type::Bool)
                } else {
                    Err(self.error(line, &format!("Cannot compare {} with {}", left, right)))
                }
            }
            BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual => {
                if left == Type::Unknown
                    || right == Type::Unknown
                    || ((matches!(left, Type::Int) || matches!(left, Type::Float))
                        && (matches!(right, Type::Int) || matches!(right, Type::Float)))
                    || (matches!(left, Type::String) && matches!(right, Type::String))
                {
                    Ok(Type::Bool)
                } else {
                    Err(self.error(
                        line,
                        &format!(
                            "Comparison expects numeric or string types, got {} and {}",
                            left, right
                        ),
                    ))
                }
            }
        }
    }

    fn analyze_call(
        &self,
        span: Span,
        name: &str,
        _type_args: &[Type],
        args: &[CallArg],
    ) -> Result<Type, ViceError> {
        let line = span.line;
        if let Some(&defining_source) = self.private_functions.get(name) {
            if span.source_id != defining_source {
                return Err(self.error_at_span(
                    span,
                    &format!(
                        "Function '{}' is marked @private and cannot be called from another file",
                        name
                    ),
                ));
            }
        }

        if self.in_leaf_function {
            return Err(self.error(
                line,
                &format!("Cannot call function '{}' inside @leaf function", name),
            ));
        }

        if self.in_no_io_function
            && (name == "print"
                || name == "println"
                || name == "eprint"
                || name == "eprintln"
                || name.starts_with("io.")
                || name.starts_with("fs."))
        {
            return Err(self.error(
                line,
                &format!("Cannot call I/O function '{}' inside @no_io function", name),
            ));
        }

        if self.in_simd_function
            && (name == "print"
                || name == "println"
                || name == "eprint"
                || name == "eprintln"
                || name.starts_with("io.")
                || name.starts_with("fs."))
        {
            return Err(self.error(
                line,
                &format!("Cannot call I/O function '{}' inside @simd function", name),
            ));
        }

        if self.in_nothrow_function && self.fallible_functions.contains(name) {
            return Err(self.error(
                line,
                &format!(
                    "Cannot call @fallible function '{}' directly inside @nothrow function",
                    name
                ),
            ));
        }

        if let Some(return_type) = self.native_call_type(line, name, args)? {
            if self.in_zero_alloc_function && !name.starts_with("math.") {
                return Err(self.error(
                    line,
                    &format!(
                        "Cannot call allocating native function '{}' inside a @zero_alloc function",
                        name
                    ),
                ));
            }
            return Ok(return_type);
        }

        let Some(symbol) = self.resolve_variable(name) else {
            return Ok(Type::Unknown);
        };

        if self.in_read_only_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor && !self.read_only_functions.contains(name) {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-read-only function '{}' inside a read-only function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_pure_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor && !self.pure_functions.contains(name) {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-pure function '{}' inside a pure function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_isolated_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor
                    && !self.isolated_functions.contains(name)
                    && !self.pure_functions.contains(name)
                    && !self.const_functions.contains(name)
                {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-isolated function '{}' inside an @isolated function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_thread_safe_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor
                    && !self.thread_safe_functions.contains(name)
                    && !self.pure_functions.contains(name)
                    && !self.isolated_functions.contains(name)
                    && !self.const_functions.contains(name)
                {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-thread_safe function '{}' inside @thread_safe function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_pure_math_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor
                    && !self.pure_math_functions.contains(name)
                    && !self.pure_functions.contains(name)
                    && !self.const_functions.contains(name)
                {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-pure_math function '{}' inside @pure_math function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_pure_string_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor
                    && !self.pure_string_functions.contains(name)
                    && !self.pure_functions.contains(name)
                    && !self.const_functions.contains(name)
                {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-pure_string function '{}' inside @pure_string function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_pure_array_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor
                    && !self.pure_array_functions.contains(name)
                    && !self.pure_functions.contains(name)
                    && !self.const_functions.contains(name)
                {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-pure_array function '{}' inside @pure_array function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_idempotent_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor
                    && !self.idempotent_functions.contains(name)
                    && !self.pure_functions.contains(name)
                    && !self.const_functions.contains(name)
                {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-idempotent function '{}' inside @idempotent function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.ascii_only_functions.contains(name) {
            for (i, arg) in args.iter().enumerate() {
                if let ExprKind::String(ref s) = arg.value.kind {
                    if !s.is_ascii() {
                        return Err(self.error(
                            line,
                            &format!("Constant argument to @ascii_only function '{}' at parameter {} contains non-ASCII characters", name, i + 1),
                        ));
                    }
                }
            }
        }

        if self.sorted_functions.contains(name) {
            for (i, arg) in args.iter().enumerate() {
                if let ExprKind::Array(elems) = &arg.value.kind {
                    let mut nums = Vec::new();
                    let mut all_nums = true;
                    for elem in elems {
                        if let ExprKind::Number(n) = &elem.kind {
                            if let Ok(val) = n.parse::<f64>() {
                                nums.push(val);
                            } else {
                                all_nums = false;
                                break;
                            }
                        } else {
                            all_nums = false;
                            break;
                        }
                    }
                    if all_nums && nums.len() > 1 {
                        for w in nums.windows(2) {
                            if w[0] > w[1] {
                                return Err(self.error(
                                    line,
                                    &format!("Constant array argument to @sorted function '{}' at parameter {} is not sorted", name, i + 1),
                                ));
                            }
                        }
                    }
                }
            }
            let mut const_num_args = Vec::new();
            for arg in args {
                if let ExprKind::Number(n) = &arg.value.kind {
                    if let Ok(v) = n.parse::<f64>() {
                        const_num_args.push(v);
                    }
                }
            }
            if const_num_args.len() == args.len() && const_num_args.len() >= 2 {
                for w in const_num_args.windows(2) {
                    if w[0] > w[1] {
                        return Err(self.error(
                            line,
                            &format!("Constant arguments to @sorted function '{}' are not in sorted order", name),
                        ));
                    }
                }
            }
        }

        if self.distinct_functions.contains(name) {
            for (i, arg) in args.iter().enumerate() {
                if let ExprKind::Array(elems) = &arg.value.kind {
                    let mut seen = std::collections::HashSet::new();
                    for elem in elems {
                        let key = match &elem.kind {
                            ExprKind::Number(n) => Some(n.clone()),
                            ExprKind::String(s) => Some(s.clone()),
                            ExprKind::Bool(b) => Some(b.to_string()),
                            _ => None,
                        };
                        if let Some(k) = key {
                            if !seen.insert(k) {
                                return Err(self.error(
                                    line,
                                    &format!("Constant array argument to @distinct function '{}' at parameter {} contains duplicate elements", name, i + 1),
                                ));
                            }
                        }
                    }
                }
            }
            let mut seen_args = std::collections::HashSet::new();
            let mut has_dups = false;
            for arg in args {
                let key = match &arg.value.kind {
                    ExprKind::Number(n) => Some(n.clone()),
                    ExprKind::String(s) => Some(s.clone()),
                    ExprKind::Bool(b) => Some(b.to_string()),
                    _ => None,
                };
                if let Some(k) = key {
                    if !seen_args.insert(k) {
                        has_dups = true;
                        break;
                    }
                }
            }
            if has_dups {
                return Err(self.error(
                    line,
                    &format!("Call to @distinct function '{}' contains duplicate constant arguments", name),
                ));
            }
        }

        if self.positive_functions.contains(name) {
            for (i, arg) in args.iter().enumerate() {
                let is_non_pos = match &arg.value.kind {
                    ExprKind::Number(n) => {
                        n.starts_with('-') || n == "0" || n.parse::<f64>().map(|v| v <= 0.0).unwrap_or(false)
                    }
                    ExprKind::Unary { operator: crate::ast::UnaryOperator::Negate, .. } => true,
                    _ => false,
                };
                if is_non_pos {
                    return Err(self.error(
                        line,
                        &format!("Constant argument to @positive function '{}' at parameter {} must be positive (> 0)", name, i + 1),
                    ));
                }
            }
        }

        if let Some((min_b, max_b)) = self.bounded_functions.get(name).copied() {
            for (i, arg) in args.iter().enumerate() {
                if let ExprKind::Number(n) = &arg.value.kind {
                    if let Ok(val) = n.parse::<f64>() {
                        if val < min_b || val > max_b {
                            return Err(self.error(
                                line,
                                &format!("Constant argument {} to @bounded function '{}' at parameter {} is out of bounds [{}, {}]", val, name, i + 1, min_b, max_b),
                            ));
                        }
                    }
                }
            }
        }

        if self.no_alias_functions.contains(name) {
            let mut seen_roots: std::collections::HashSet<String> = std::collections::HashSet::new();
            for arg in args {
                let inner_expr = match &arg.value.kind {
                    ExprKind::Unary { operator: crate::ast::UnaryOperator::Reference, right } => Some(&**right),
                    _ => None,
                };
                if let Some(inner) = inner_expr {
                    if let Some(root) = self.get_root_variable(inner) {
                        if !seen_roots.insert(root.to_string()) {
                            return Err(self.error(
                                line,
                                &format!("Call to @no_alias function '{}' passes conflicting reference to variable '{}' in multiple arguments", name, root),
                            ));
                        }
                    }
                }
            }
        }

        if self.non_negative_functions.contains(name) {
            for (i, arg) in args.iter().enumerate() {
                let is_neg = match &arg.value.kind {
                    ExprKind::Number(n) => n.starts_with('-') || n.parse::<f64>().map(|v| v < 0.0).unwrap_or(false),
                    ExprKind::Unary { operator: crate::ast::UnaryOperator::Negate, .. } => true,
                    _ => false,
                };
                if is_neg {
                    return Err(self.error(
                        line,
                        &format!("Constant argument to @non_negative function '{}' at parameter {} must be non-negative (>= 0)", name, i + 1),
                    ));
                }
            }
        }

        if self.non_empty_functions.contains(name) {
            if let Type::Function { params, .. } = &symbol.ty {
                for (i, arg) in args.iter().enumerate() {
                    if let Some(param_ty) = params.get(i) {
                        let is_empty_lit = match &arg.value.kind {
                            ExprKind::String(s) if s.is_empty() => true,
                            ExprKind::Array(elems) if elems.is_empty() => true,
                            ExprKind::Map(entries) if entries.is_empty() => true,
                            _ => false,
                        };
                        if is_empty_lit && matches!(param_ty, Type::String | Type::Array(_) | Type::Map(_, _)) {
                            return Err(self.error(
                                line,
                                &format!(
                                    "@non_empty function '{}' cannot be called with an empty collection argument at position {}",
                                    name,
                                    i + 1
                                ),
                            ));
                        }
                    }
                }
            }
        }

        if self.in_safe_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor && !self.safe_functions.contains(name) {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-safe function '{}' inside a @safe function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_const_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor && !self.const_functions.contains(name) {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-@const function '{}' inside @const function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_no_panic_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor && !self.no_panic_functions.contains(name) && !self.safe_functions.contains(name) {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-@no_panic function '{}' inside @no_panic function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_deterministic_function {
            if let Type::Function { return_type, .. } = &symbol.ty {
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if !is_constructor
                    && !self.deterministic_functions.contains(name)
                    && !self.pure_functions.contains(name)
                    && !self.const_functions.contains(name)
                {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-deterministic function '{}' inside @deterministic function",
                            name
                        ),
                    ));
                }
            }
        }

        if self.in_infallible_function && self.fallible_functions.contains(name) {
            return Err(self.error(
                line,
                &format!(
                    "Cannot call @fallible function '{}' directly inside @infallible function",
                    name
                ),
            ));
        }

        if self.security_critical_functions.contains(name) && self.in_experimental_function {
            return Err(self.error(
                line,
                &format!(
                    "Cannot call @security_critical function '{}' from @experimental function '{}'",
                    name,
                    self.current_function_name.as_deref().unwrap_or("<top-level>")
                ),
            ));
        }

        if self.unreachable_functions.contains(name) {
            return Err(self.error(
                line,
                &format!("Cannot call @unreachable function '{}'", name),
            ));
        }

        if let Some(reason_opt) = self.obsolete_items.get(name) {
            let note = match reason_opt {
                Some(reason) => format!(": {}", reason),
                None => String::new(),
            };
            return Err(self.error(
                line,
                &format!("Item '{}' is marked @obsolete and cannot be used{}", name, note),
            ));
        }

        if self.test_only_functions.contains(name) && !self.in_test_function {
            return Err(self.error(
                line,
                &format!(
                    "Cannot call @test_only function '{}' from non-test function '{}'",
                    name,
                    self.current_function_name.as_deref().unwrap_or("<top-level>")
                ),
            ));
        }

        if self.no_recurse_functions.contains(name) && self.current_function_name.as_deref() == Some(name) {
            return Err(self.error(
                line,
                &format!("Function '{}' is marked @no_recurse and cannot call itself", name),
            ));
        }

        if let Some(reason_opt) = self.deprecated_items.get(name) {
            let note = match reason_opt {
                Some(reason) => format!("Item '{}' is deprecated: {}", name, reason),
                None => format!("Item '{}' is deprecated", name),
            };
            self.record_warning(format!("{}:{}: Warning: {}", self.current_source.as_deref().unwrap_or("<unknown>"), line, note));
        }

        if self.benchmark_only_functions.contains(name) && !self.in_benchmark_function {
            return Err(self.error(
                line,
                &format!(
                    "Cannot call @benchmark_only function '{}' from non-benchmark function '{}'",
                    name,
                    self.current_function_name.as_deref().unwrap_or("<top-level>")
                ),
            ));
        }

        if self.experimental_items.contains(name)
            && (self.in_safe_function || self.in_strict_function || self.in_pure_function)
            && !self.in_experimental_function
        {
            return Err(self.error(
                line,
                &format!(
                    "Cannot call @experimental function '{}' from safe/strict function '{}' without @experimental",
                    name,
                    self.current_function_name.as_deref().unwrap_or("<top-level>")
                ),
            ));
        }

        match &symbol.ty {
            Type::Function {
                params,
                return_type,
            } => {
                // Konstruktory danych pozwalają na pominięcie argumentów (użycie domyślnych)
                let is_constructor = matches!(return_type.as_ref(), Type::Data(_, _));
                if is_constructor {
                    if let Type::Data(data_name, _) = return_type.as_ref() {
                        if let Some(&defining_source) = self.private_datas.get(data_name) {
                            if span.source_id != defining_source {
                                return Err(self.error_at_span(
                                    span,
                                    &format!(
                                        "Data type '{}' is marked @private and cannot be constructed from another file",
                                        data_name
                                    ),
                                ));
                            }
                        }
                        if let Some(reason_opt) = self.deprecated_items.get(data_name).or_else(|| self.obsolete_items.get(data_name)) {
                            let note = match reason_opt {
                                Some(reason) => format!(": {}", reason),
                                None => String::new(),
                            };
                            return Err(self.error(
                                line,
                                &format!("Item '{}' is marked @obsolete and cannot be used{}", data_name, note),
                            ));
                        }
                        if self.singleton_datas.contains(data_name) {
                            let mut inst = self.singleton_instantiations.borrow_mut();
                            let count = inst.entry(data_name.clone()).or_insert(0);
                            *count += 1;
                            if *count > 1 {
                                return Err(self.error(
                                    line,
                                    &format!(
                                        "Singleton struct '{}' cannot be instantiated more than once",
                                        data_name
                                    ),
                                ));
                            }
                        }
                        if self.experimental_items.contains(data_name)
                            && (self.in_safe_function || self.in_strict_function || self.in_pure_function)
                            && !self.in_experimental_function
                        {
                            return Err(self.error(
                                line,
                                &format!(
                                    "Cannot instantiate @experimental struct '{}' from safe/strict function '{}' without @experimental",
                                    data_name,
                                    self.current_function_name.as_deref().unwrap_or("<top-level>")
                                ),
                            ));
                        }
                        if self.must_initialize_datas.contains(data_name) {
                            if let Some(data_def) = self.data_defs.get(data_name) {
                                if args.iter().all(|a| a.name.is_none()) {
                                    if args.len() < data_def.fields.len() {
                                        return Err(self.error_at_span(
                                            span,
                                            &format!(
                                                "Instantiation of @must_initialize struct '{}' requires all {} fields, but only {} provided",
                                                data_name,
                                                data_def.fields.len(),
                                                args.len()
                                            ),
                                        ));
                                    }
                                } else {
                                    let provided_names: std::collections::HashSet<&str> = args
                                        .iter()
                                        .filter_map(|a| a.name.as_deref())
                                        .collect();
                                    for field_name in data_def.fields.keys() {
                                        if !provided_names.contains(field_name.as_str()) {
                                            return Err(self.error_at_span(
                                                span,
                                                &format!(
                                                    "Instantiation of @must_initialize struct '{}' is missing field '{}'",
                                                    data_name, field_name
                                                ),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if self.in_zero_alloc_function {
                        return Err(self.error(
                            line,
                            &format!(
                                "Cannot allocate struct '{}' inside a @zero_alloc function",
                                name
                            ),
                        ));
                    }
                } else {
                    if self.in_zero_alloc_function && !self.zero_alloc_functions.contains(name) {
                        return Err(self.error(
                            line,
                            &format!(
                                "Cannot call non-@zero_alloc function '{}' inside a @zero_alloc function",
                                name
                            ),
                        ));
                    }
                    self.check_arg_count(line, name, params.len(), args.len())?;
                }

                for (idx, arg) in args.iter().enumerate() {
                    let actual = self.analyze_expression(&arg.value)?;

                    let expected = if let (true, Some(arg_name)) = (is_constructor, &arg.name) {
                        let Type::Data(data_name, _) = return_type.as_ref() else {
                            return Err(self.error(line, "Invalid constructor return type"));
                        };
                        self.field_type(data_name, arg_name, line)?
                    } else if idx < params.len() {
                        params[idx].clone()
                    } else {
                        return Err(self.error(line, "Too many arguments"));
                    };

                    if !self.types_compatible(&expected, &actual, symbol.mode) {
                        return Err(self.error(
                            arg.value.span.line,
                            &format!(
                                "Argument {} for '{}' expected {}, got {}",
                                idx + 1,
                                name,
                                expected,
                                actual
                            ),
                        ));
                    }
                }
                Ok((**return_type).clone())
            }
            _ => Err(self.error(line, &format!("'{}' is not a function", name))),
        }
    }

    fn native_call_type(
        &self,
        line: usize,
        name: &str,
        args: &[CallArg],
    ) -> Result<Option<Type>, ViceError> {
        match name {
            "to_num" => {
                self.check_arg_count(line, name, 1, args.len())?;
                self.analyze_expression(&args[0].value)?;
                Ok(Some(Type::Unknown))
            }
            "to_int" => {
                self.check_arg_count(line, name, 1, args.len())?;
                self.analyze_expression(&args[0].value)?;
                Ok(Some(Type::Int))
            }
            "to_float" => {
                self.check_arg_count(line, name, 1, args.len())?;
                self.analyze_expression(&args[0].value)?;
                Ok(Some(Type::Float))
            }
            "to_str" => {
                self.check_arg_count(line, name, 1, args.len())?;
                self.analyze_expression(&args[0].value)?;
                Ok(Some(Type::String))
            }
            "len" => {
                self.check_arg_count(line, name, 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if matches!(arg_ty, Type::String | Type::Array(_) | Type::Unknown) {
                    Ok(Some(Type::Int))
                } else {
                    Err(self.error(
                        line,
                        &format!("len expects string or array, got {}", arg_ty),
                    ))
                }
            }
            "ok" => {
                self.check_arg_count(line, name, 1, args.len())?;
                let t = self.analyze_expression(&args[0].value)?;
                Ok(Some(Type::Data(
                    "Result".to_string(),
                    vec![t, Type::Unknown],
                )))
            }
            "err" => {
                self.check_arg_count(line, name, 1, args.len())?;
                let e = self.analyze_expression(&args[0].value)?;
                Ok(Some(Type::Data(
                    "Result".to_string(),
                    vec![Type::Unknown, e],
                )))
            }
            "some" => {
                self.check_arg_count(line, name, 1, args.len())?;
                let t = self.analyze_expression(&args[0].value)?;
                Ok(Some(Type::Data("Option".to_string(), vec![t])))
            }
            "none" => {
                self.check_arg_count(line, name, 0, args.len())?;
                Ok(Some(Type::Data("Option".to_string(), vec![Type::Unknown])))
            }
            _ => Ok(None),
        }
    }

    fn analyze_method_call(
        &self,
        line: usize,
        entity: &Expr,
        method: &str,
        _type_args: &[Type],
        args: &[CallArg],
    ) -> Result<Type, ViceError> {
        let entity_type = {
            let mut ty = self.analyze_expression(entity)?;
            if let Type::Reference { inner, .. } = ty {
                ty = *inner;
            }
            ty
        };

        if self.in_leaf_function {
            return Err(self.error(
                line,
                &format!("Cannot call method '{}' inside @leaf function", method),
            ));
        }

        if self.in_no_io_function {
            if let Type::Module(ref name) = entity_type {
                if matches!(name.as_str(), "fs" | "sys" | "os" | "io") {
                    return Err(self.error(
                        line,
                        &format!("Cannot call I/O module method '{}.{}' inside @no_io function", name, method),
                    ));
                }
            }
        }

        if self.in_pure_function {
            if let Type::Module(ref name) = entity_type {
                if matches!(name.as_str(), "ui" | "game2d" | "fs" | "audio" | "net" | "time" | "env" | "sys" | "os") {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call impure module method '{}.{}' inside a pure function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_isolated_function {
            if let Type::Module(ref name) = entity_type {
                if matches!(name.as_str(), "ui" | "game2d" | "fs" | "audio" | "net" | "time" | "env" | "sys" | "os") {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call impure module method '{}.{}' inside an @isolated function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_thread_safe_function {
            if let Type::Module(ref name) = entity_type {
                if matches!(name.as_str(), "ui" | "game2d" | "fs" | "audio" | "net" | "time" | "env" | "sys" | "os" | "process" | "engine") {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call impure module method '{}.{}' inside @thread_safe function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_pure_math_function {
            if let Type::Module(ref name) = entity_type {
                if name != "math" && name != "hash" {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-math module '{}.{}' inside @pure_math function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_pure_string_function {
            if let Type::Module(ref name) = entity_type {
                if name != "string" && name != "hash" {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-string module '{}.{}' inside @pure_string function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_pure_array_function {
            if let Type::Module(ref name) = entity_type {
                if name != "array" && name != "hash" {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-array module '{}.{}' inside @pure_array function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_idempotent_function {
            if let Type::Module(ref name) = entity_type {
                if matches!(name.as_str(), "fs" | "audio" | "process" | "engine" | "ui") {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call impure module method '{}.{}' inside @idempotent function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_const_function {
            if let Type::Module(ref name) = entity_type {
                if name != "math" {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call module method '{}.{}' inside @const function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_safe_function {
            if let Type::Module(ref name) = entity_type {
                if matches!(name.as_str(), "fs" | "audio" | "process" | "engine" | "ui") {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call impure module method '{}.{}' inside a @safe function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_no_panic_function {
            if let Type::Module(ref name) = entity_type {
                if matches!(name.as_str(), "fs" | "audio" | "process" | "engine" | "ui") {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call impure module method '{}.{}' inside @no_panic function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_deterministic_function {
            if let Type::Module(ref name) = entity_type {
                if matches!(name.as_str(), "random" | "time" | "env" | "process" | "net")
                    || (name == "math" && method == "random")
                {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call non-deterministic module method '{}.{}' inside @deterministic function",
                            name, method
                        ),
                    ));
                }
            }
        }

        if self.in_zero_alloc_function {
            if let Type::Module(ref name) = entity_type {
                if name != "math" {
                    return Err(self.error(
                        line,
                        &format!(
                            "Cannot call module method '{}.{}' inside @zero_alloc function",
                            name, method
                        ),
                    ));
                }
            } else if matches!(entity_type, Type::Array(_) | Type::Map(..) | Type::String)
                && matches!(method, "push" | "append" | "insert" | "split" | "replace" | "clone")
            {
                return Err(self.error(
                    line,
                    &format!(
                        "Cannot call allocating method '{}' inside @zero_alloc function",
                        method
                    ),
                ));
            }
        }

        if let Some(reason_opt) = self.obsolete_items.get(method) {
            let note = match reason_opt {
                Some(reason) => format!(": {}", reason),
                None => String::new(),
            };
            return Err(self.error(
                line,
                &format!("Method '{}' is marked @obsolete and cannot be used{}", method, note),
            ));
        }

        if let Some(reason_opt) = self.deprecated_items.get(method) {
            let note = match reason_opt {
                Some(reason) => format!("Method '{}' is deprecated: {}", method, reason),
                None => format!("Method '{}' is deprecated", method),
            };
            self.record_warning(format!("{}:{}: Warning: {}", self.current_source.as_deref().unwrap_or("<unknown>"), line, note));
        }

        if self.in_read_only_function || self.in_pure_function {
            if self.in_read_only_function {
                if let Type::Module(ref name) = entity_type {
                    if name == "ui" {
                        return Err(self.error(
                            line,
                            &format!(
                                "Cannot call side-effecting UI method 'ui.{}' inside a read-only function",
                                method
                            ),
                        ));
                    }
                    if name == "game2d" {
                        return Err(self.error(
                            line,
                            &format!(
                                "Cannot call side-effecting game2d method 'game2d.{}' inside a read-only function",
                                method
                            ),
                        ));
                    }
                    if name == "fs" && method == "write_text" {
                        return Err(self.error(line, &format!("Cannot call side-effecting FS method 'fs.{}' inside a read-only function", method)));
                    }
                    if name == "audio" && (method == "play_sound" || method == "play_music") {
                        return Err(self.error(line, &format!("Cannot call side-effecting audio method 'audio.{}' inside a read-only function", method)));
                    }
                }
            }

            if matches!(entity_type, Type::Array(_) | Type::Map(_, _))
                && (method == "push"
                    || method == "pop"
                    || method == "insert"
                    || method == "remove"
                    || method == "clear")
            {
                let forbidden_root = self.get_root_variable(entity).filter(|root_var| {
                    self.is_global(root_var) || self.current_function_params.contains(*root_var)
                });
                if let Some(root_var) = forbidden_root {
                    let fn_kind = if self.in_pure_function { "pure" } else { "read-only" };
                    return Err(self.error(line, &format!("Cannot mutate array/map of parameter or global variable '{}' inside a {} function", root_var, fn_kind)));
                }
            }
        }

        if matches!(entity_type, Type::Module(ref name) if name == "ui") {
            if let Some(replacement) = removed_ui_method_replacement(method) {
                return Err(self.error(
                    line,
                    &format!("ui.{} was removed in UI v0.3; use {}", method, replacement),
                ));
            }
        }

        match (&entity_type, method) {
            (Type::Module(name) | Type::Data(name, _), method)
                if !matches!(name.as_str(), "array" | "math" | "map")
                    && self.registry.get_method(name, method).is_some() =>
            {
                let native_def = self.registry.get_method(name, method).ok_or_else(|| {
                    self.error(
                        line,
                        &format!("Unknown native method '{}.{}'", name, method),
                    )
                })?;
                if !native_def.varargs {
                    let min_args = native_def.args.iter().filter(|a| !a.optional).count();
                    let max_args = native_def.args.len();
                    if args.len() < min_args || args.len() > max_args {
                        return Err(self.error(
                            line,
                            &format!(
                                "{} expected {}..{} arguments, got {}",
                                method,
                                min_args,
                                max_args,
                                args.len()
                            ),
                        ));
                    }
                }
                for (i, arg) in args.iter().enumerate() {
                    let arg_ty = self.analyze_expression(&arg.value)?;
                    if i < native_def.args.len() {
                        let expected_ty = &native_def.args[i].ty;
                        if expected_ty != &Type::Unknown
                            && !self.types_compatible(expected_ty, &arg_ty, self.current_mode)
                        {
                            return Err(self.error(
                                arg.value.span.line,
                                &format!(
                                    "Argument {} for '{}' expected {}, got {}",
                                    i + 1,
                                    method,
                                    expected_ty,
                                    arg_ty
                                ),
                            ));
                        }
                    }
                }
                Ok(native_def.return_type.clone())
            }
            (Type::Module(name) | Type::Data(name, _), method) if name == "fs" => match method {
                "read_text" => {
                    self.check_arg_count(line, "fs.read_text", 1, args.len())?;
                    let arg_ty = self.analyze_expression(&args[0].value)?;
                    if self.types_compatible(&Type::String, &arg_ty, self.current_mode) {
                        Ok(Type::String)
                    } else {
                        Err(self.error(
                            line,
                            &format!("fs.read_text expects string, got {}", arg_ty),
                        ))
                    }
                }
                "write_text" => {
                    self.check_arg_count(line, "fs.write_text", 2, args.len())?;
                    for arg in args {
                        let arg_ty = self.analyze_expression(&arg.value)?;
                        if !self.types_compatible(&Type::String, &arg_ty, self.current_mode) {
                            return Err(self.error(
                                line,
                                &format!("fs.write_text expects string args, got {}", arg_ty),
                            ));
                        }
                    }
                    Ok(Type::Bool)
                }
                "read_json" => {
                    self.check_arg_count(line, "fs.read_json", 1, args.len())?;
                    let arg_ty = self.analyze_expression(&args[0].value)?;
                    if self.types_compatible(&Type::String, &arg_ty, self.current_mode) {
                        Ok(Type::Unknown)
                    } else {
                        Err(self.error(
                            line,
                            &format!("fs.read_json expects string, got {}", arg_ty),
                        ))
                    }
                }
                _ => Err(self.error(
                    line,
                    &format!("Unknown method '{}' for {}", method, entity_type),
                )),
            },
            (Type::Module(name) | Type::Data(name, _), method) if name == "string" => {
                match method {
                    "contains" | "starts_with" | "ends_with" => {
                        self.expect_arg_types(
                            line,
                            &format!("string.{}", method),
                            args,
                            &[Type::String, Type::String],
                        )?;
                        Ok(Type::Bool)
                    }
                    "trim" | "trim_start" | "trim_end" | "upper" | "lower" | "to_upper" | "to_lower" => {
                        self.expect_arg_types(
                            line,
                            &format!("string.{}", method),
                            args,
                            &[Type::String],
                        )?;
                        Ok(Type::String)
                    }
                    "from_int" | "from_float" | "from_bool" | "to_string" => {
                        self.check_arg_count(line, &format!("string.{}", method), 1, args.len())?;
                        self.analyze_expression(&args[0].value)?;
                        Ok(Type::String)
                    }
                    "to_int" | "parse_int" => {
                        self.expect_arg_types(
                            line,
                            &format!("string.{}", method),
                            args,
                            &[Type::String],
                        )?;
                        Ok(Type::Int)
                    }
                    "to_float" | "parse_float" => {
                        self.expect_arg_types(
                            line,
                            &format!("string.{}", method),
                            args,
                            &[Type::String],
                        )?;
                        Ok(Type::Float)
                    }
                    "lines" => {
                        self.expect_arg_types(
                            line,
                            "string.lines",
                            args,
                            &[Type::String],
                        )?;
                        Ok(Type::Array(Box::new(Type::String)))
                    }
                    "split" => {
                        self.expect_arg_types(
                            line,
                            "string.split",
                            args,
                            &[Type::String, Type::String],
                        )?;
                        Ok(Type::Array(Box::new(Type::String)))
                    }
                    "replace" => {
                        self.expect_arg_types(
                            line,
                            "string.replace",
                            args,
                            &[Type::String, Type::String, Type::String],
                        )?;
                        Ok(Type::String)
                    }
                    "is_empty" => {
                        self.expect_arg_types(
                            line,
                            "string.is_empty",
                            args,
                            &[Type::String],
                        )?;
                        Ok(Type::Bool)
                    }
                    "len" => {
                        self.expect_arg_types(
                            line,
                            "string.len",
                            args,
                            &[Type::String],
                        )?;
                        Ok(Type::Int)
                    }
                    "slice" => {
                        self.expect_arg_types(
                            line,
                            "string.slice",
                            args,
                            &[Type::String, Type::Int, Type::Int],
                        )?;
                        Ok(Type::String)
                    }
                    "reverse" => {
                        self.expect_arg_types(
                            line,
                            "string.reverse",
                            args,
                            &[Type::String],
                        )?;
                        Ok(Type::String)
                    }
                    "repeat" => {
                        self.expect_arg_types(
                            line,
                            "string.repeat",
                            args,
                            &[Type::String, Type::Int],
                        )?;
                        Ok(Type::String)
                    }
                    "char_at" => {
                        self.expect_arg_types(
                            line,
                            "string.char_at",
                            args,
                            &[Type::String, Type::Int],
                        )?;
                        Ok(Type::String)
                    }
                    "count" => {
                        self.expect_arg_types(
                            line,
                            "string.count",
                            args,
                            &[Type::String, Type::String],
                        )?;
                        Ok(Type::Int)
                    }
                    "capitalize" => {
                        self.expect_arg_types(
                            line,
                            "string.capitalize",
                            args,
                            &[Type::String],
                        )?;
                        Ok(Type::String)
                    }
                    "index_of" => {
                        self.expect_arg_types(
                            line,
                            "string.index_of",
                            args,
                            &[Type::String, Type::String],
                        )?;
                        Ok(Type::Int)
                    }
                    "last_index_of" => {
                        self.expect_arg_types(
                            line,
                            "string.last_index_of",
                            args,
                            &[Type::String, Type::String],
                        )?;
                        Ok(Type::Int)
                    }
                    "pad_start" => {
                        self.expect_arg_types(
                            line,
                            "string.pad_start",
                            args,
                            &[Type::String, Type::Int, Type::String],
                        )?;
                        Ok(Type::String)
                    }
                    "pad_end" => {
                        self.expect_arg_types(
                            line,
                            "string.pad_end",
                            args,
                            &[Type::String, Type::Int, Type::String],
                        )?;
                        Ok(Type::String)
                    }
                    "is_numeric" | "is_alpha" => {
                        self.expect_arg_types(line, &format!("string.{}", method), args, &[Type::String])?;
                        Ok(Type::Bool)
                    }
                    "words" => {
                        self.expect_arg_types(line, "string.words", args, &[Type::String])?;
                        Ok(Type::Array(Box::new(Type::String)))
                    }
                    "fnv1a" | "djb2" | "crc32" | "adler32" | "murmur3" => {
                        self.expect_arg_types(line, &format!("string.{}", method), args, &[Type::String])?;
                        Ok(Type::Int)
                    }
                    "lines_count" | "words_count" => {
                        self.expect_arg_types(line, &format!("string.{}", method), args, &[Type::String])?;
                        Ok(Type::Int)
                    }
                    _ => Err(self.error(
                        line, // The error message will now correctly refer to Type::Module
                        &format!("Unknown method '{}' for {}", method, entity_type),
                    )),
                }
            }
            (Type::Module(name) | Type::Data(name, _), method) if name == "array" => match method {
                "push" | "append" => {
                    self.check_arg_count(line, &format!("array.{}", method), 2, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let val_ty = self.analyze_expression(&args[1].value)?;
                    match arr_ty {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Array(Box::new(val_ty))),
                        other => Err(self.error(line, &format!("array.{} expects array as first argument, got {}", method, other))),
                    }
                }
                "pop" => {
                    self.check_arg_count(line, "array.pop", 1, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    match arr_ty {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.pop expects array, got {}", other))),
                    }
                }
                "len" => {
                    self.check_arg_count(line, "array.len", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(_) | Type::Unknown => Ok(Type::Int),
                        other => {
                            Err(self
                                .error(line, &format!("array.len expects array, got {}", other)))
                        }
                    }
                }
                "is_empty" => {
                    self.check_arg_count(line, "array.is_empty", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(_) | Type::Unknown => Ok(Type::Bool),
                        other => Err(self.error(line, &format!("array.is_empty expects array, got {}", other))),
                    }
                }
                "contains" => {
                    self.check_arg_count(line, "array.contains", 2, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let _val_ty = self.analyze_expression(&args[1].value)?;
                    match arr_ty {
                        Type::Array(_) | Type::Unknown => Ok(Type::Bool),
                        other => Err(self.error(line, &format!("array.contains expects array, got {}", other))),
                    }
                }
                "reverse" => {
                    self.check_arg_count(line, "array.reverse", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.reverse expects array, got {}", other))),
                    }
                }
                "sort" => {
                    self.check_arg_count(line, "array.sort", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.sort expects array, got {}", other))),
                    }
                }
                "slice" => {
                    self.check_arg_count(line, "array.slice", 3, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let start_ty = self.analyze_expression(&args[1].value)?;
                    let end_ty = self.analyze_expression(&args[2].value)?;
                    if !self.types_compatible(&Type::Int, &start_ty, self.current_mode)
                        || !self.types_compatible(&Type::Int, &end_ty, self.current_mode)
                    {
                        return Err(self.error(line, "array.slice start and end must be integers"));
                    }
                    match arr_ty {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.slice expects array, got {}", other))),
                    }
                }
                "index_of" => {
                    self.check_arg_count(line, "array.index_of", 2, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let _val_ty = self.analyze_expression(&args[1].value)?;
                    match arr_ty {
                        Type::Array(_) | Type::Unknown => Ok(Type::Int),
                        other => Err(self.error(line, &format!("array.index_of expects array, got {}", other))),
                    }
                }
                "concat" => {
                    self.check_arg_count(line, "array.concat", 2, args.len())?;
                    let a_ty = self.analyze_expression(&args[0].value)?;
                    let b_ty = self.analyze_expression(&args[1].value)?;
                    match (&a_ty, &b_ty) {
                        (Type::Array(inner), Type::Array(_)) => Ok(Type::Array(inner.clone())),
                        (Type::Array(inner), Type::Unknown) | (Type::Unknown, Type::Array(inner)) => Ok(Type::Array(inner.clone())),
                        (Type::Unknown, Type::Unknown) => Ok(Type::Unknown),
                        _ => Err(self.error(line, "array.concat expects two arrays")),
                    }
                }
                "first" | "last" => {
                    self.check_arg_count(line, &format!("array.{}", method), 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => Ok(*inner),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(
                            line,
                            &format!("array.{} expects array, got {}", method, other),
                        )),
                    }
                }
                "join" => {
                    self.check_arg_count(line, "array.join", 2, args.len())?;
                    let array_ty = self.analyze_expression(&args[0].value)?;
                    if !matches!(array_ty, Type::Array(_) | Type::Unknown) {
                        return Err(self
                            .error(line, &format!("array.join expects array, got {}", array_ty)));
                    }
                    let delimiter_ty = self.analyze_expression(&args[1].value)?;
                    if !self.types_compatible(&Type::String, &delimiter_ty, self.current_mode) {
                        return Err(self.error(
                            line,
                            &format!("array.join expects string delimiter, got {}", delimiter_ty),
                        ));
                    }
                    Ok(Type::String)
                }
                "remove_at" => {
                    self.check_arg_count(line, "array.remove_at", 2, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let idx_ty = self.analyze_expression(&args[1].value)?;
                    if !self.types_compatible(&Type::Int, &idx_ty, self.current_mode) {
                        return Err(self.error(line, "array.remove_at index must be an integer"));
                    }
                    match arr_ty {
                        Type::Array(inner) => Ok(*inner),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.remove_at expects array, got {}", other))),
                    }
                }
                "insert" => {
                    self.check_arg_count(line, "array.insert", 3, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let idx_ty = self.analyze_expression(&args[1].value)?;
                    let val_ty = self.analyze_expression(&args[2].value)?;
                    if !self.types_compatible(&Type::Int, &idx_ty, self.current_mode) {
                        return Err(self.error(line, "array.insert index must be an integer"));
                    }
                    match arr_ty {
                        Type::Array(inner) => {
                            if !self.types_compatible(&inner, &val_ty, self.current_mode) {
                                return Err(self.error(line, &format!("array.insert expected element of type {}, got {}", inner, val_ty)));
                            }
                            Ok(Type::Array(inner))
                        }
                        Type::Unknown => Ok(Type::Array(Box::new(val_ty))),
                        other => Err(self.error(line, &format!("array.insert expects array, got {}", other))),
                    }
                }
                "fill" => {
                    self.check_arg_count(line, "array.fill", 2, args.len())?;
                    let count_ty = self.analyze_expression(&args[0].value)?;
                    let val_ty = self.analyze_expression(&args[1].value)?;
                    if !self.types_compatible(&Type::Int, &count_ty, self.current_mode) {
                        return Err(self.error(line, "array.fill count must be an integer"));
                    }
                    Ok(Type::Array(Box::new(val_ty)))
                }
                "unique" => {
                    self.check_arg_count(line, "array.unique", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.unique expects array, got {}", other))),
                    }
                }
                "chunk" => {
                    self.check_arg_count(line, "array.chunk", 2, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let size_ty = self.analyze_expression(&args[1].value)?;
                    if !self.types_compatible(&Type::Int, &size_ty, self.current_mode) {
                        return Err(self.error(line, "array.chunk size must be an integer"));
                    }
                    match arr_ty {
                        Type::Array(inner) => Ok(Type::Array(Box::new(Type::Array(inner)))),
                        Type::Unknown => Ok(Type::Array(Box::new(Type::Unknown))),
                        other => Err(self.error(line, &format!("array.chunk expects array, got {}", other))),
                    }
                }
                "flatten" => {
                    self.check_arg_count(line, "array.flatten", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => match *inner {
                            Type::Array(nested) => Ok(Type::Array(nested)),
                            other => Ok(Type::Array(Box::new(other))),
                        },
                        Type::Unknown => Ok(Type::Array(Box::new(Type::Unknown))),
                        other => Err(self.error(line, &format!("array.flatten expects array, got {}", other))),
                    }
                }
                "sum" => {
                    self.check_arg_count(line, "array.sum", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => {
                            if *inner == Type::Float {
                                Ok(Type::Float)
                            } else if *inner == Type::Int {
                                Ok(Type::Int)
                            } else {
                                Ok(Type::Unknown)
                            }
                        }
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.sum expects array, got {}", other))),
                    }
                }
                "average" => {
                    self.check_arg_count(line, "array.average", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(_) | Type::Unknown => Ok(Type::Float),
                        other => Err(self.error(line, &format!("array.average expects array, got {}", other))),
                    }
                }
                "product" => {
                    self.check_arg_count(line, "array.product", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => {
                            if *inner == Type::Float {
                                Ok(Type::Float)
                            } else if *inner == Type::Int {
                                Ok(Type::Int)
                            } else {
                                Ok(Type::Unknown)
                            }
                        }
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.product expects array, got {}", other))),
                    }
                }
                "dedup" => {
                    self.check_arg_count(line, "array.dedup", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.dedup expects array, got {}", other))),
                    }
                }
                "min" | "max" => {
                    self.check_arg_count(line, &format!("array.{}", method), 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => Ok(*inner),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.{} expects array, got {}", method, other))),
                    }
                }
                "zip" => {
                    self.check_arg_count(line, "array.zip", 2, args.len())?;
                    let a_ty = self.analyze_expression(&args[0].value)?;
                    let b_ty = self.analyze_expression(&args[1].value)?;
                    if !matches!(a_ty, Type::Array(_) | Type::Unknown) || !matches!(b_ty, Type::Array(_) | Type::Unknown) {
                        return Err(self.error(line, "array.zip expects two arrays"));
                    }
                    Ok(Type::Array(Box::new(Type::Array(Box::new(Type::Unknown)))))
                }
                "binary_search" => {
                    self.check_arg_count(line, "array.binary_search", 2, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let _val_ty = self.analyze_expression(&args[1].value)?;
                    match arr_ty {
                        Type::Array(_) | Type::Unknown => Ok(Type::Int),
                        other => Err(self.error(line, &format!("array.binary_search expects array, got {}", other))),
                    }
                }
                "rotate" => {
                    self.check_arg_count(line, "array.rotate", 2, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let count_ty = self.analyze_expression(&args[1].value)?;
                    if !self.types_compatible(&Type::Int, &count_ty, self.current_mode) {
                        return Err(self.error(line, "array.rotate count must be an integer"));
                    }
                    match arr_ty {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.rotate expects array, got {}", other))),
                    }
                }
                "repeat" => {
                    self.check_arg_count(line, "array.repeat", 2, args.len())?;
                    let val_ty = self.analyze_expression(&args[0].value)?;
                    let count_ty = self.analyze_expression(&args[1].value)?;
                    if !self.types_compatible(&Type::Int, &count_ty, self.current_mode) {
                        return Err(self.error(line, "array.repeat count must be an integer"));
                    }
                    Ok(Type::Array(Box::new(val_ty)))
                }
                "windows" => {
                    self.check_arg_count(line, "array.windows", 2, args.len())?;
                    let arr_ty = self.analyze_expression(&args[0].value)?;
                    let size_ty = self.analyze_expression(&args[1].value)?;
                    if !self.types_compatible(&Type::Int, &size_ty, self.current_mode) {
                        return Err(self.error(line, "array.windows size must be an integer"));
                    }
                    match arr_ty {
                        Type::Array(inner) => Ok(Type::Array(Box::new(Type::Array(inner)))),
                        Type::Unknown => Ok(Type::Array(Box::new(Type::Unknown))),
                        other => Err(self.error(line, &format!("array.windows expects array, got {}", other))),
                    }
                }
                "frequencies" => {
                    self.check_arg_count(line, "array.frequencies", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(_) | Type::Unknown => Ok(Type::Map(Box::new(Type::String), Box::new(Type::Int))),
                        other => Err(self.error(line, &format!("array.frequencies expects array, got {}", other))),
                    }
                }
                "compact" => {
                    self.check_arg_count(line, "array.compact", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Array(inner) => Ok(Type::Array(inner)),
                        Type::Unknown => Ok(Type::Unknown),
                        other => Err(self.error(line, &format!("array.compact expects array, got {}", other))),
                    }
                }
                "diff" | "intersect" => {
                    self.check_arg_count(line, &format!("array.{}", method), 2, args.len())?;
                    let a_ty = self.analyze_expression(&args[0].value)?;
                    let b_ty = self.analyze_expression(&args[1].value)?;
                    match (&a_ty, &b_ty) {
                        (Type::Array(inner), Type::Array(_)) => Ok(Type::Array(inner.clone())),
                        (Type::Array(inner), Type::Unknown) | (Type::Unknown, Type::Array(inner)) => Ok(Type::Array(inner.clone())),
                        (Type::Unknown, Type::Unknown) => Ok(Type::Unknown),
                        _ => Err(self.error(line, &format!("array.{} expects two arrays", method))),
                    }
                }
                _ => Err(self.error(
                    line, // The error message will now correctly refer to Type::Module
                    &format!("Unknown method '{}' for {}", method, entity_type),
                )),
            },
            (Type::Module(name) | Type::Data(name, _), method) if name == "math" => match method {
                "floor" | "ceil" | "round" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Float])?;
                    Ok(Type::Int)
                }
                "sin" | "cos" | "tan" | "sqrt" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Float])?;
                    Ok(Type::Float)
                }
                "pow" => {
                    self.expect_arg_types(line, "math.pow", args, &[Type::Float, Type::Float])?;
                    Ok(Type::Float)
                }
                "random" => {
                    self.check_arg_count(line, "math.random", 0, args.len())?;
                    Ok(Type::Float)
                }
                "abs" => {
                    self.check_arg_count(line, "math.abs", 1, args.len())?;
                    let arg_ty = self.analyze_expression(&args[0].value)?;
                    if self.is_numeric(&arg_ty) || arg_ty == Type::Unknown {
                        Ok(arg_ty)
                    } else {
                        Err(self.error(line, &format!("math.abs expects number, got {}", arg_ty)))
                    }
                }
                "min" | "max" => {
                    self.check_arg_count(line, &format!("math.{}", method), 2, args.len())?;
                    let left = self.analyze_expression(&args[0].value)?;
                    let right = self.analyze_expression(&args[1].value)?;
                    if (!self.is_numeric(&left) && left != Type::Unknown)
                        || (!self.is_numeric(&right) && right != Type::Unknown)
                    {
                        return Err(
                            self.error(line, &format!("math.{} expects number args", method))
                        );
                    }
                    if left == Type::Float || right == Type::Float {
                        Ok(Type::Float)
                    } else if left == Type::Unknown || right == Type::Unknown {
                        Ok(Type::Unknown)
                    } else {
                        Ok(Type::Int)
                    }
                }
                "clamp" => {
                    self.check_arg_count(line, "math.clamp", 3, args.len())?;
                    let val_ty = self.analyze_expression(&args[0].value)?;
                    let min_ty = self.analyze_expression(&args[1].value)?;
                    let max_ty = self.analyze_expression(&args[2].value)?;
                    if (!self.is_numeric(&val_ty) && val_ty != Type::Unknown)
                        || (!self.is_numeric(&min_ty) && min_ty != Type::Unknown)
                        || (!self.is_numeric(&max_ty) && max_ty != Type::Unknown)
                    {
                        return Err(self.error(line, "math.clamp expects numeric arguments (val, min, max)"));
                    }
                    if val_ty == Type::Float || min_ty == Type::Float || max_ty == Type::Float {
                        Ok(Type::Float)
                    } else if val_ty == Type::Unknown || min_ty == Type::Unknown || max_ty == Type::Unknown {
                        Ok(Type::Unknown)
                    } else {
                        Ok(Type::Int)
                    }
                }
                "sign" => {
                    self.check_arg_count(line, "math.sign", 1, args.len())?;
                    let arg_ty = self.analyze_expression(&args[0].value)?;
                    if !self.is_numeric(&arg_ty) && arg_ty != Type::Unknown {
                        return Err(self.error(line, &format!("math.sign expects number, got {}", arg_ty)));
                    }
                    Ok(Type::Int)
                }
                "log" | "log10" | "log2" | "cosh" | "sinh" | "tanh" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Float])?;
                    Ok(Type::Float)
                }
                "copysign" => {
                    self.expect_arg_types(line, "math.copysign", args, &[Type::Float, Type::Float])?;
                    Ok(Type::Float)
                }
                "dist" => {
                    self.expect_arg_types(line, "math.dist", args, &[Type::Float, Type::Float, Type::Float, Type::Float])?;
                    Ok(Type::Float)
                }
                "hypot" => {
                    self.expect_arg_types(line, "math.hypot", args, &[Type::Float, Type::Float])?;
                    Ok(Type::Float)
                }
                "atan2" => {
                    self.expect_arg_types(line, "math.atan2", args, &[Type::Float, Type::Float])?;
                    Ok(Type::Float)
                }
                "asin" | "acos" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Float])?;
                    Ok(Type::Float)
                }
                "pi" | "e" => {
                    self.check_arg_count(line, &format!("math.{}", method), 0, args.len())?;
                    Ok(Type::Float)
                }
                "deg" | "rad" | "radians" | "degrees" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Float])?;
                    Ok(Type::Float)
                }
                "signum" => {
                    self.check_arg_count(line, "math.signum", 1, args.len())?;
                    let ty = self.analyze_expression(&args[0].value)?;
                    if !self.is_numeric(&ty) && ty != Type::Unknown {
                        return Err(self.error(line, "math.signum expects numeric arg"));
                    }
                    Ok(ty)
                }
                "trunc" | "fract" | "cbrt" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Float])?;
                    Ok(Type::Float)
                }
                "gcd" | "lcm" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Int, Type::Int])?;
                    Ok(Type::Int)
                }
                "is_nan" | "is_finite" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Float])?;
                    Ok(Type::Bool)
                }
                "factorial" => {
                    self.expect_arg_types(line, "math.factorial", args, &[Type::Int])?;
                    Ok(Type::Int)
                }
                "round_to" => {
                    self.expect_arg_types(line, "math.round_to", args, &[Type::Float, Type::Int])?;
                    Ok(Type::Float)
                }
                "lerp" => {
                    self.check_arg_count(line, "math.lerp", 3, args.len())?;
                    let a = self.analyze_expression(&args[0].value)?;
                    let b = self.analyze_expression(&args[1].value)?;
                    let t = self.analyze_expression(&args[2].value)?;
                    if (!self.is_numeric(&a) && a != Type::Unknown)
                        || (!self.is_numeric(&b) && b != Type::Unknown)
                        || (!self.is_numeric(&t) && t != Type::Unknown)
                    {
                        return Err(self.error(line, "math.lerp expects 3 numeric arguments"));
                    }
                    Ok(Type::Float)
                }
                "exp" => {
                    self.expect_arg_types(line, "math.exp", args, &[Type::Float])?;
                    Ok(Type::Float)
                }
                "is_even" | "is_odd" | "is_square" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Int])?;
                    Ok(Type::Bool)
                }
                "bitwise_and" | "bitwise_or" | "bitwise_xor" | "bitwise_shl" | "bitwise_shr" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Int, Type::Int])?;
                    Ok(Type::Int)
                }
                "bitwise_not" => {
                    self.expect_arg_types(line, "math.bitwise_not", args, &[Type::Int])?;
                    Ok(Type::Int)
                }
                "wrap" => {
                    self.check_arg_count(line, "math.wrap", 3, args.len())?;
                    let val = self.analyze_expression(&args[0].value)?;
                    let min = self.analyze_expression(&args[1].value)?;
                    let max = self.analyze_expression(&args[2].value)?;
                    if (!self.is_numeric(&val) && val != Type::Unknown)
                        || (!self.is_numeric(&min) && min != Type::Unknown)
                        || (!self.is_numeric(&max) && max != Type::Unknown)
                    {
                        return Err(self.error(line, "math.wrap expects numeric arguments"));
                    }
                    if val == Type::Float || min == Type::Float || max == Type::Float {
                        Ok(Type::Float)
                    } else if val == Type::Unknown || min == Type::Unknown || max == Type::Unknown {
                        Ok(Type::Unknown)
                    } else {
                        Ok(Type::Int)
                    }
                }
                "smoothstep" => {
                    self.expect_arg_types(line, "math.smoothstep", args, &[Type::Float, Type::Float, Type::Float])?;
                    Ok(Type::Float)
                }
                "deg_to_rad" | "rad_to_deg" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Float])?;
                    Ok(Type::Float)
                }
                "is_prime" => {
                    self.expect_arg_types(line, "math.is_prime", args, &[Type::Int])?;
                    Ok(Type::Bool)
                }
                "count_ones" | "leading_zeros" | "trailing_zeros" => {
                    self.expect_arg_types(line, &format!("math.{}", method), args, &[Type::Int])?;
                    Ok(Type::Int)
                }
                "is_power_of_two" => {
                    self.expect_arg_types(line, "math.is_power_of_two", args, &[Type::Int])?;
                    Ok(Type::Bool)
                }
                "next_power_of_two" => {
                    self.expect_arg_types(line, "math.next_power_of_two", args, &[Type::Int])?;
                    Ok(Type::Int)
                }
                "clamp01" => {
                    self.check_arg_count(line, "math.clamp01", 1, args.len())?;
                    let arg_ty = self.analyze_expression(&args[0].value)?;
                    if !self.is_numeric(&arg_ty) && arg_ty != Type::Unknown {
                        return Err(self.error(line, "math.clamp01 expects a numeric argument"));
                    }
                    if arg_ty == Type::Int {
                        Ok(Type::Int)
                    } else {
                        Ok(Type::Float)
                    }
                }
                "lerp_inv" => {
                    self.expect_arg_types(line, "math.lerp_inv", args, &[Type::Float, Type::Float, Type::Float])?;
                    Ok(Type::Float)
                }
                "remap" => {
                    self.expect_arg_types(line, "math.remap", args, &[Type::Float, Type::Float, Type::Float, Type::Float, Type::Float])?;
                    Ok(Type::Float)
                }
                "midpoint" => {
                    self.check_arg_count(line, "math.midpoint", 2, args.len())?;
                    let a = self.analyze_expression(&args[0].value)?;
                    let b = self.analyze_expression(&args[1].value)?;
                    if (!self.is_numeric(&a) && a != Type::Unknown) || (!self.is_numeric(&b) && b != Type::Unknown) {
                        return Err(self.error(line, "math.midpoint expects 2 numeric arguments"));
                    }
                    if a == Type::Float || b == Type::Float {
                        Ok(Type::Float)
                    } else {
                        Ok(Type::Int)
                    }
                }
                "approx_eq" => {
                    if args.len() < 2 || args.len() > 3 {
                        return Err(self.error(line, "math.approx_eq expects 2 or 3 arguments (a, b, [eps])"));
                    }
                    let a = self.analyze_expression(&args[0].value)?;
                    let b = self.analyze_expression(&args[1].value)?;
                    if (!self.is_numeric(&a) && a != Type::Unknown) || (!self.is_numeric(&b) && b != Type::Unknown) {
                        return Err(self.error(line, "math.approx_eq expects numeric arguments"));
                    }
                    if args.len() == 3 {
                        let eps = self.analyze_expression(&args[2].value)?;
                        if !self.is_numeric(&eps) && eps != Type::Unknown {
                            return Err(self.error(line, "math.approx_eq epsilon must be a number"));
                        }
                    }
                    Ok(Type::Bool)
                }
                _ => Err(self.error(
                    line, // The error message will now correctly refer to Type::Module
                    &format!("Unknown method '{}' for {}", method, entity_type),
                )),
            },
            (Type::Module(name) | Type::Data(name, _), method) if name == "map" => match method {
                "len" => {
                    self.check_arg_count(line, "map.len", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Map(_, _) | Type::Unknown => Ok(Type::Int),
                        other => Err(self.error(line, &format!("map.len expects map, got {}", other))),
                    }
                }
                "is_empty" => {
                    self.check_arg_count(line, "map.is_empty", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Map(_, _) | Type::Unknown => Ok(Type::Bool),
                        other => Err(self.error(line, &format!("map.is_empty expects map, got {}", other))),
                    }
                }
                "contains_key" => {
                    self.check_arg_count(line, "map.contains_key", 2, args.len())?;
                    let m_ty = self.analyze_expression(&args[0].value)?;
                    let _k_ty = self.analyze_expression(&args[1].value)?;
                    match m_ty {
                        Type::Map(_, _) | Type::Unknown => Ok(Type::Bool),
                        other => Err(self.error(line, &format!("map.contains_key expects map, got {}", other))),
                    }
                }
                "keys" => {
                    self.check_arg_count(line, "map.keys", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Map(k, _) => Ok(Type::Array(k)),
                        Type::Unknown => Ok(Type::Array(Box::new(Type::Unknown))),
                        other => Err(self.error(line, &format!("map.keys expects map, got {}", other))),
                    }
                }
                "values" => {
                    self.check_arg_count(line, "map.values", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Map(_, v) => Ok(Type::Array(v)),
                        Type::Unknown => Ok(Type::Array(Box::new(Type::Unknown))),
                        other => Err(self.error(line, &format!("map.values expects map, got {}", other))),
                    }
                }
                "get" => {
                    self.check_arg_count(line, "map.get", 3, args.len())?;
                    let m_ty = self.analyze_expression(&args[0].value)?;
                    let _k_ty = self.analyze_expression(&args[1].value)?;
                    let def_ty = self.analyze_expression(&args[2].value)?;
                    match m_ty {
                        Type::Map(_, v) => Ok(*v),
                        Type::Unknown => Ok(def_ty),
                        other => Err(self.error(line, &format!("map.get expects map, got {}", other))),
                    }
                }
                "remove" => {
                    self.check_arg_count(line, "map.remove", 2, args.len())?;
                    let m_ty = self.analyze_expression(&args[0].value)?;
                    let _k_ty = self.analyze_expression(&args[1].value)?;
                    match m_ty {
                        Type::Map(_, _) | Type::Unknown => Ok(Type::Bool),
                        other => Err(self.error(line, &format!("map.remove expects map, got {}", other))),
                    }
                }
                "merge" => {
                    self.check_arg_count(line, "map.merge", 2, args.len())?;
                    let m1 = self.analyze_expression(&args[0].value)?;
                    let m2 = self.analyze_expression(&args[1].value)?;
                    match (&m1, &m2) {
                        (Type::Map(k, v), Type::Map(_, _)) => Ok(Type::Map(k.clone(), v.clone())),
                        (Type::Map(k, v), Type::Unknown) | (Type::Unknown, Type::Map(k, v)) => Ok(Type::Map(k.clone(), v.clone())),
                        (Type::Unknown, Type::Unknown) => Ok(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
                        _ => Err(self.error(line, "map.merge expects two maps")),
                    }
                }
                "clear" => {
                    self.check_arg_count(line, "map.clear", 1, args.len())?;
                    let m_ty = self.analyze_expression(&args[0].value)?;
                    match m_ty {
                        Type::Map(k, v) => Ok(Type::Map(k, v)),
                        Type::Unknown => Ok(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
                        other => Err(self.error(line, &format!("map.clear expects map, got {}", other))),
                    }
                }
                "has_value" => {
                    self.check_arg_count(line, "map.has_value", 2, args.len())?;
                    let m_ty = self.analyze_expression(&args[0].value)?;
                    let _v_ty = self.analyze_expression(&args[1].value)?;
                    match m_ty {
                        Type::Map(_, _) | Type::Unknown => Ok(Type::Bool),
                        other => Err(self.error(line, &format!("map.has_value expects map, got {}", other))),
                    }
                }
                "invert" => {
                    self.check_arg_count(line, "map.invert", 1, args.len())?;
                    match self.analyze_expression(&args[0].value)? {
                        Type::Map(k, v) => Ok(Type::Map(v, k)),
                        Type::Unknown => Ok(Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown))),
                        other => Err(self.error(line, &format!("map.invert expects map, got {}", other))),
                    }
                }
                _ => Err(self.error(
                    line,
                    &format!("Unknown method '{}' for {}", method, entity_type),
                )),
            },
            (Type::Module(name) | Type::Data(name, _), method) if name == "random" => {
                match method {
                    "int" => {
                        self.check_arg_count(line, "random.int", 2, args.len())?;
                        let left = self.analyze_expression(&args[0].value)?;
                        let right = self.analyze_expression(&args[1].value)?;
                        if (!self.is_numeric(&left) && left != Type::Unknown)
                            || (!self.is_numeric(&right) && right != Type::Unknown)
                        {
                            return Err(self.error(line, "random.int expects numeric args "));
                        }
                        Ok(Type::Int)
                    }
                    "float" => {
                        self.check_arg_count(line, "random.float ", 0, args.len())?;
                        Ok(Type::Float)
                    }
                    _ => Err(self.error(
                        line,
                        &format!("Unknown method '{}' for {}", method, entity_type),
                    )),
                }
            }
            (Type::Module(name) | Type::Data(name, _), method) if name == "json" => match method {
                "parse" => {
                    self.check_arg_count(line, "json.parse ", 1, args.len())?;
                    let arg_ty = self.analyze_expression(&args[0].value)?;
                    if self.types_compatible(&Type::String, &arg_ty, self.current_mode) {
                        Ok(Type::Unknown)
                    } else {
                        Err(self.error(line, &format!("json.parse expects string, got {}", arg_ty)))
                    }
                }
                "stringify" => {
                    self.check_arg_count(line, "json.stringify ", 1, args.len())?;
                    let _ = self.analyze_expression(&args[0].value)?;
                    Ok(Type::String)
                }
                _ => Err(self.error(
                    line,
                    &format!("Unknown method '{}' for {}", method, entity_type),
                )),
            },
            (Type::Module(name) | Type::Data(name, _), method) if name == "path" => match method {
                "join" => {
                    self.expect_arg_types(line, "path.join ", args, &[Type::String, Type::String])?;
                    Ok(Type::String)
                }
                "dirname" | "basename" => {
                    self.expect_arg_types(
                        line,
                        &format!("path.{}", method),
                        args,
                        &[Type::String],
                    )?;
                    Ok(Type::String)
                }
                "exists" => {
                    self.expect_arg_types(line, "path.exists ", args, &[Type::String])?;
                    Ok(Type::Bool)
                }
                _ => Err(self.error(
                    line, // The error message will now correctly refer to Type::Module
                    &format!("Unknown method '{}' for {}", method, entity_type),
                )),
            },
            (Type::Module(name) | Type::Data(name, _), method) if name == "hash" => match method {
                "fnv1a" | "djb2" | "adler32" | "crc32" | "murmur3" => {
                    self.expect_arg_types(line, &format!("hash.{}", method), args, &[Type::String])?;
                    Ok(Type::Int)
                }
                "combine" => {
                    self.expect_arg_types(line, "hash.combine", args, &[Type::Int, Type::Int])?;
                    Ok(Type::Int)
                }
                _ => Err(self.error(
                    line,
                    &format!("Unknown method '{}' for {}", method, entity_type),
                )),
            },
            (Type::Data(name, _) | Type::Enum(name), method) => {
                if let Some(methods) = self.impls.get(name) {
                    if let Some(Type::Function {
                        params,
                        return_type,
                    }) = methods.get(method)
                    {
                        // Omijamy pierwszy parametr (self), jeśli istnieje.
                        // Ale czekaj, w vice metody mają explicit self? Tak.
                        // Wait, how does analyze_method_call receive args?
                        // `args` in analyze_method_call DOES NOT include `self`.
                        // It only includes arguments passed in parenthesis.
                        // So we should compare args with params starting from index 1 if the first param is `self`.
                        let is_self_param = !params.is_empty()
                            && (params[0] == entity_type
                                || params[0] == Type::Unknown
                                || matches!(params[0], Type::Reference { .. }));
                        let expected_params = if is_self_param {
                            &params[1..]
                        } else {
                            // Czasami parser pozwala na definicję bez self.
                            &params[..]
                        };

                        self.check_arg_count(line, method, expected_params.len(), args.len())?;
                        for (idx, expected) in expected_params.iter().enumerate() {
                            let actual = self.analyze_expression(&args[idx].value)?;
                            if !self.types_compatible(expected, &actual, self.current_mode) {
                                return Err(self.error(
                                    args[idx].value.span.line,
                                    &format!(
                                        "Argument {} for '{}' expected {}, got {}",
                                        idx + 1,
                                        method,
                                        expected,
                                        actual
                                    ),
                                ));
                            }
                        }
                        return Ok(*return_type.clone());
                    }
                }

                if name == "env" && method == "get" {
                    self.expect_arg_types(line, "env.get ", args, &[Type::String])?;
                    Ok(Type::String)
                } else if name == "env" && method == "set" {
                    self.expect_arg_types(line, "env.set ", args, &[Type::String, Type::String])?;
                    Ok(Type::Unknown)
                } else if name == "time" && method == "now_ms" {
                    self.check_arg_count(line, "time.now_ms ", 0, args.len())?;
                    Ok(Type::Int)
                } else {
                    Err(self.error(
                        line,
                        &format!("Unknown method '{}' for {}", method, entity_type),
                    ))
                }
            }
            (Type::Module(name), "get") if name == "env" => {
                self.expect_arg_types(line, "env.get ", args, &[Type::String])?;
                Ok(Type::String)
            }
            (Type::Module(name), "set") if name == "env" => {
                self.expect_arg_types(line, "env.set ", args, &[Type::String, Type::String])?;
                Ok(Type::Unknown)
            }
            (Type::Module(name), "now_ms") if name == "time" => {
                self.check_arg_count(line, "time.now_ms ", 0, args.len())?;
                Ok(Type::Int)
            }
            (Type::Array(inner), "push") => {
                self.check_arg_count(line, "push", 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(inner, &arg_ty, self.current_mode) {
                    Ok(Type::Array(inner.clone()))
                } else {
                    Err(self.error(line, &format!("push expected {}, got {}", inner, arg_ty)))
                }
            }
            (Type::Array(inner), "pop") => {
                self.check_arg_count(line, "pop", 0, args.len())?;
                Ok((**inner).clone())
            }
            (Type::Array(_), "len") => {
                self.check_arg_count(line, "len", 0, args.len())?;
                Ok(Type::Int)
            }
            (Type::Array(_), "is_empty") => {
                self.check_arg_count(line, "is_empty", 0, args.len())?;
                Ok(Type::Bool)
            }
            (Type::Array(inner), "first" | "last") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok((**inner).clone())
            }
            (Type::Array(inner), "contains") => {
                self.check_arg_count(line, "contains", 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(inner, &arg_ty, self.current_mode) {
                    Ok(Type::Bool)
                } else {
                    Err(self.error(line, &format!("contains expected {}, got {}", inner, arg_ty)))
                }
            }
            (Type::Array(inner), "reverse") => {
                self.check_arg_count(line, "reverse", 0, args.len())?;
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "clear") => {
                self.check_arg_count(line, "clear", 0, args.len())?;
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(_), "join") => {
                self.check_arg_count(line, "join", 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(&Type::String, &arg_ty, self.current_mode) {
                    Ok(Type::String)
                } else {
                    Err(self.error(line, &format!("join expected string separator, got {}", arg_ty)))
                }
            }
            (Type::Array(inner), "slice") => {
                self.check_arg_count(line, "slice", 2, args.len())?;
                let start_ty = self.analyze_expression(&args[0].value)?;
                let end_ty = self.analyze_expression(&args[1].value)?;
                if self.types_compatible(&Type::Int, &start_ty, self.current_mode)
                    && self.types_compatible(&Type::Int, &end_ty, self.current_mode)
                {
                    Ok(Type::Array(inner.clone()))
                } else {
                    Err(self.error(line, "slice expects (int, int)"))
                }
            }
            (Type::Array(inner), "index_of" | "find") => {
                self.check_arg_count(line, method, 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(inner, &arg_ty, self.current_mode) {
                    Ok(Type::Int)
                } else {
                    Err(self.error(line, &format!("{} expected {}, got {}", method, inner, arg_ty)))
                }
            }
            (Type::Array(inner), "insert") => {
                self.check_arg_count(line, "insert", 2, args.len())?;
                let idx_ty = self.analyze_expression(&args[0].value)?;
                let val_ty = self.analyze_expression(&args[1].value)?;
                if !self.types_compatible(&Type::Int, &idx_ty, self.current_mode) {
                    return Err(self.error(line, "insert index must be an int"));
                }
                if !self.types_compatible(inner, &val_ty, self.current_mode) {
                    return Err(self.error(line, &format!("insert expected value of type {}, got {}", inner, val_ty)));
                }
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "remove_at") => {
                self.check_arg_count(line, "remove_at", 1, args.len())?;
                let idx_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::Int, &idx_ty, self.current_mode) {
                    return Err(self.error(line, "remove_at index must be an int"));
                }
                Ok((**inner).clone())
            }
            (Type::Array(inner), "clone") => {
                self.check_arg_count(line, "clone", 0, args.len())?;
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "sort") => {
                self.check_arg_count(line, "sort", 0, args.len())?;
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "concat") => {
                self.check_arg_count(line, "concat", 1, args.len())?;
                let other_ty = self.analyze_expression(&args[0].value)?;
                if !matches!(other_ty, Type::Array(_) | Type::Unknown) {
                    return Err(self.error(line, "concat expects an array argument"));
                }
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "swap") => {
                self.check_arg_count(line, "swap", 2, args.len())?;
                let i_ty = self.analyze_expression(&args[0].value)?;
                let j_ty = self.analyze_expression(&args[1].value)?;
                if !self.types_compatible(&Type::Int, &i_ty, self.current_mode)
                    || !self.types_compatible(&Type::Int, &j_ty, self.current_mode)
                {
                    return Err(self.error(line, "swap expects two int indices"));
                }
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(_), "count") => {
                self.check_arg_count(line, "count", 1, args.len())?;
                let _ = self.analyze_expression(&args[0].value)?;
                Ok(Type::Int)
            }
            (Type::Array(inner), "take") => {
                self.check_arg_count(line, "take", 1, args.len())?;
                let n_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::Int, &n_ty, self.current_mode) {
                    return Err(self.error(line, "take expects an int count"));
                }
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "drop") => {
                self.check_arg_count(line, "drop", 1, args.len())?;
                let n_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::Int, &n_ty, self.current_mode) {
                    return Err(self.error(line, "drop expects an int count"));
                }
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "unique") => {
                self.check_arg_count(line, "unique", 0, args.len())?;
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "fill") => {
                self.check_arg_count(line, "fill", 1, args.len())?;
                let val_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(inner, &val_ty, self.current_mode) {
                    return Err(self.error(line, &format!("fill expected value of type {}, got {}", inner, val_ty)));
                }
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "chunk") => {
                self.check_arg_count(line, "chunk", 1, args.len())?;
                let size_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::Int, &size_ty, self.current_mode) {
                    return Err(self.error(line, "chunk expects an int size"));
                }
                Ok(Type::Array(Box::new(Type::Array(inner.clone()))))
            }
            (Type::Array(inner), "flatten") => {
                self.check_arg_count(line, "flatten", 0, args.len())?;
                match &**inner {
                    Type::Array(nested) => Ok(Type::Array(nested.clone())),
                    other => Ok(Type::Array(Box::new(other.clone()))),
                }
            }
            (Type::Array(inner), "sum") => {
                self.check_arg_count(line, "sum", 0, args.len())?;
                if **inner == Type::Float {
                    Ok(Type::Float)
                } else if **inner == Type::Int {
                    Ok(Type::Int)
                } else {
                    Ok(Type::Unknown)
                }
            }
            (Type::Array(_inner), "average") => {
                self.check_arg_count(line, "average", 0, args.len())?;
                Ok(Type::Float)
            }
            (Type::Array(inner), "product") => {
                self.check_arg_count(line, "product", 0, args.len())?;
                if **inner == Type::Float {
                    Ok(Type::Float)
                } else if **inner == Type::Int {
                    Ok(Type::Int)
                } else {
                    Ok(Type::Unknown)
                }
            }
            (Type::Array(inner), "dedup") => {
                self.check_arg_count(line, "dedup", 0, args.len())?;
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(_inner), "binary_search") => {
                self.check_arg_count(line, "binary_search", 1, args.len())?;
                self.analyze_expression(&args[0].value)?;
                Ok(Type::Int)
            }
            (Type::Array(inner), "rotate") => {
                self.check_arg_count(line, "rotate", 1, args.len())?;
                let count_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::Int, &count_ty, self.current_mode) {
                    return Err(self.error(line, "rotate count must be an integer"));
                }
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "windows") => {
                self.check_arg_count(line, "windows", 1, args.len())?;
                let size_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::Int, &size_ty, self.current_mode) {
                    return Err(self.error(line, "windows size must be an integer"));
                }
                Ok(Type::Array(Box::new(Type::Array(inner.clone()))))
            }
            (Type::Array(_inner), "frequencies") => {
                self.check_arg_count(line, "frequencies", 0, args.len())?;
                Ok(Type::Map(Box::new(Type::String), Box::new(Type::Int)))
            }
            (Type::Array(inner), "min" | "max") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok((**inner).clone())
            }
            (Type::Array(_inner), "zip") => {
                self.check_arg_count(line, "zip", 1, args.len())?;
                let other_ty = self.analyze_expression(&args[0].value)?;
                if !matches!(other_ty, Type::Array(_) | Type::Unknown) {
                    return Err(self.error(line, "zip expects an array argument"));
                }
                Ok(Type::Array(Box::new(Type::Array(Box::new(Type::Unknown)))))
            }
            (Type::Array(inner), "compact") => {
                self.check_arg_count(line, "compact", 0, args.len())?;
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(inner), "diff" | "intersect") => {
                self.check_arg_count(line, method, 1, args.len())?;
                let other_ty = self.analyze_expression(&args[0].value)?;
                if !matches!(other_ty, Type::Array(_) | Type::Unknown) {
                    return Err(self.error(line, &format!("{} expects an array argument", method)));
                }
                Ok(Type::Array(inner.clone()))
            }
            (Type::Array(_inner), "is_sorted" | "all_unique") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok(Type::Bool)
            }
            (Type::String, "len") => {
                self.check_arg_count(line, "len", 0, args.len())?;
                Ok(Type::Int)
            }
            (Type::String, "is_empty") => {
                self.check_arg_count(line, "is_empty", 0, args.len())?;
                Ok(Type::Bool)
            }
            (Type::String, "trim" | "trim_start" | "trim_end" | "lower" | "upper" | "rot13") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok(Type::String)
            }
            (Type::String, "contains" | "starts_with" | "ends_with") => {
                self.check_arg_count(line, method, 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(&Type::String, &arg_ty, self.current_mode) {
                    Ok(Type::Bool)
                } else {
                    Err(self.error(line, &format!("{} expected string argument, got {}", method, arg_ty)))
                }
            }
            (Type::String, "replace") => {
                self.check_arg_count(line, "replace", 2, args.len())?;
                let from_ty = self.analyze_expression(&args[0].value)?;
                let to_ty = self.analyze_expression(&args[1].value)?;
                if self.types_compatible(&Type::String, &from_ty, self.current_mode)
                    && self.types_compatible(&Type::String, &to_ty, self.current_mode)
                {
                    Ok(Type::String)
                } else {
                    Err(self.error(line, "replace expects (string, string)"))
                }
            }
            (Type::String, "split") => {
                self.check_arg_count(line, "split", 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(&Type::String, &arg_ty, self.current_mode) {
                    Ok(Type::Array(Box::new(Type::String)))
                } else {
                    Err(self.error(line, &format!("split expected string delimiter, got {}", arg_ty)))
                }
            }
            (Type::String, "slice") => {
                self.check_arg_count(line, "slice", 2, args.len())?;
                let start_ty = self.analyze_expression(&args[0].value)?;
                let end_ty = self.analyze_expression(&args[1].value)?;
                if self.types_compatible(&Type::Int, &start_ty, self.current_mode)
                    && self.types_compatible(&Type::Int, &end_ty, self.current_mode)
                {
                    Ok(Type::String)
                } else {
                    Err(self.error(line, "slice expects (int, int)"))
                }
            }
            (Type::String, "index_of" | "last_index_of") => {
                self.check_arg_count(line, method, 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(&Type::String, &arg_ty, self.current_mode) {
                    Ok(Type::Int)
                } else {
                    Err(self.error(line, &format!("{} expected string, got {}", method, arg_ty)))
                }
            }
            (Type::String, "repeat") => {
                self.check_arg_count(line, "repeat", 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(&Type::Int, &arg_ty, self.current_mode) {
                    Ok(Type::String)
                } else {
                    Err(self.error(line, &format!("repeat expected int count, got {}", arg_ty)))
                }
            }
            (Type::String, "chars") => {
                self.check_arg_count(line, "chars", 0, args.len())?;
                Ok(Type::Array(Box::new(Type::String)))
            }
            (Type::String, "pad_start" | "pad_end") => {
                self.check_arg_count(line, method, 2, args.len())?;
                let len_ty = self.analyze_expression(&args[0].value)?;
                let pad_ty = self.analyze_expression(&args[1].value)?;
                if !self.types_compatible(&Type::Int, &len_ty, self.current_mode) {
                    return Err(self.error(line, &format!("{} target length must be an int", method)));
                }
                if !self.types_compatible(&Type::String, &pad_ty, self.current_mode) {
                    return Err(self.error(line, &format!("{} pad character must be a string", method)));
                }
                Ok(Type::String)
            }
            (Type::String, "char_at") => {
                self.check_arg_count(line, "char_at", 1, args.len())?;
                let idx_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::Int, &idx_ty, self.current_mode) {
                    return Err(self.error(line, "char_at expects an int index"));
                }
                Ok(Type::String)
            }
            (Type::String, "reverse") => {
                self.check_arg_count(line, "reverse", 0, args.len())?;
                Ok(Type::String)
            }
            (Type::String, "lines") => {
                self.check_arg_count(line, "lines", 0, args.len())?;
                Ok(Type::Array(Box::new(Type::String)))
            }
            (Type::String, "count") => {
                self.check_arg_count(line, "count", 1, args.len())?;
                let sub_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::String, &sub_ty, self.current_mode) {
                    return Err(self.error(line, "count expects string argument"));
                }
                Ok(Type::Int)
            }
            (Type::String, "capitalize") => {
                self.check_arg_count(line, "capitalize", 0, args.len())?;
                Ok(Type::String)
            }
            (Type::String, "is_numeric" | "is_alpha") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok(Type::Bool)
            }
            (Type::String, "words") => {
                self.check_arg_count(line, "words", 0, args.len())?;
                Ok(Type::Array(Box::new(Type::String)))
            }
            (Type::String, "fnv1a" | "djb2" | "crc32" | "adler32" | "murmur3") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok(Type::Int)
            }
            (Type::String, "xxhash32") => {
                if args.len() > 1 {
                    return Err(self.error(line, "xxhash32 expects 0 or 1 argument"));
                }
                if args.len() == 1 {
                    let seed_ty = self.analyze_expression(&args[0].value)?;
                    if !self.types_compatible(&Type::Int, &seed_ty, self.current_mode) {
                        return Err(self.error(line, "xxhash32 seed must be an int"));
                    }
                }
                Ok(Type::Int)
            }
            (Type::String, "levenshtein" | "hamming") => {
                self.check_arg_count(line, method, 1, args.len())?;
                let other_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::String, &other_ty, self.current_mode) {
                    return Err(self.error(line, &format!("{} expects string argument", method)));
                }
                Ok(Type::Int)
            }
            (Type::String, "slugify" | "hex_encode" | "hex_decode") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok(Type::String)
            }
            (Type::String, "lines_count" | "words_count") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok(Type::Int)
            }
            (Type::String, "truncate") => {
                if args.is_empty() || args.len() > 2 {
                    return Err(self.error(line, "truncate expects 1 or 2 arguments (max_len, [ellipsis])"));
                }
                let len_ty = self.analyze_expression(&args[0].value)?;
                if !self.types_compatible(&Type::Int, &len_ty, self.current_mode) {
                    return Err(self.error(line, "truncate max_len must be an int"));
                }
                if args.len() == 2 {
                    let el_ty = self.analyze_expression(&args[1].value)?;
                    if !self.types_compatible(&Type::String, &el_ty, self.current_mode) {
                        return Err(self.error(line, "truncate ellipsis must be a string"));
                    }
                }
                Ok(Type::String)
            }
            (Type::Map(_, _), "len" | "count") => {
                self.check_arg_count(line, method, 0, args.len())?;
                Ok(Type::Int)
            }
            (Type::Map(_, _), "is_empty") => {
                self.check_arg_count(line, "is_empty", 0, args.len())?;
                Ok(Type::Bool)
            }
            (Type::Map(key_ty, _), "contains" | "contains_key" | "has_key") => {
                self.check_arg_count(line, method, 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(key_ty, &arg_ty, self.current_mode) {
                    Ok(Type::Bool)
                } else {
                    Err(self.error(line, &format!("{} expected key type {}, got {}", method, key_ty, arg_ty)))
                }
            }
            (Type::Map(key_ty, _), "keys") => {
                self.check_arg_count(line, "keys", 0, args.len())?;
                Ok(Type::Array(key_ty.clone()))
            }
            (Type::Map(_, val_ty), "values") => {
                self.check_arg_count(line, "values", 0, args.len())?;
                Ok(Type::Array(val_ty.clone()))
            }
            (Type::Map(key_ty, val_ty), "get") => {
                self.check_arg_count(line, "get", 2, args.len())?;
                let k_ty = self.analyze_expression(&args[0].value)?;
                let def_ty = self.analyze_expression(&args[1].value)?;
                if !self.types_compatible(key_ty, &k_ty, self.current_mode) {
                    return Err(self.error(line, &format!("get expected key type {}, got {}", key_ty, k_ty)));
                }
                if !self.types_compatible(val_ty, &def_ty, self.current_mode) {
                    return Err(self.error(line, &format!("get default value expected {}, got {}", val_ty, def_ty)));
                }
                Ok((**val_ty).clone())
            }
            (Type::Map(key_ty, _), "remove") => {
                self.check_arg_count(line, "remove", 1, args.len())?;
                let arg_ty = self.analyze_expression(&args[0].value)?;
                if self.types_compatible(key_ty, &arg_ty, self.current_mode) {
                    Ok(Type::Bool)
                } else {
                    Err(self.error(line, &format!("remove expected key type {}, got {}", key_ty, arg_ty)))
                }
            }
            (Type::Map(key_ty, val_ty), "clear") => {
                self.check_arg_count(line, "clear", 0, args.len())?;
                Ok(Type::Map(key_ty.clone(), val_ty.clone()))
            }
            (Type::Map(_key_ty, _val_ty), "entries") => {
                self.check_arg_count(line, "entries", 0, args.len())?;
                Ok(Type::Array(Box::new(Type::Array(Box::new(Type::Unknown)))))
            }
            (Type::Map(key_ty, val_ty), "clone") => {
                self.check_arg_count(line, "clone", 0, args.len())?;
                Ok(Type::Map(key_ty.clone(), val_ty.clone()))
            }
            (Type::Map(key_ty, val_ty), "merge" | "extend") => {
                self.check_arg_count(line, method, 1, args.len())?;
                let other_ty = self.analyze_expression(&args[0].value)?;
                match other_ty {
                    Type::Map(ref ok, ref ov) => {
                        if !self.types_compatible(key_ty, ok, self.current_mode) {
                            return Err(self.error(line, &format!("{} key type mismatch: expected {}, got {}", method, key_ty, ok)));
                        }
                        if !self.types_compatible(val_ty, ov, self.current_mode) {
                            return Err(self.error(line, &format!("{} value type mismatch: expected {}, got {}", method, val_ty, ov)));
                        }
                    }
                    Type::Unknown => {}
                    _ => {
                        return Err(self.error(line, &format!("{} expects a map argument, got {}", method, other_ty)));
                    }
                }
                Ok(Type::Map(key_ty.clone(), val_ty.clone()))
            }
            (Type::Map(_key_ty, _val_ty), "has_value") => {
                self.check_arg_count(line, "has_value", 1, args.len())?;
                let _ = self.analyze_expression(&args[0].value)?;
                Ok(Type::Bool)
            }
            (Type::Map(key_ty, val_ty), "invert") => {
                self.check_arg_count(line, "invert", 0, args.len())?;
                Ok(Type::Map(val_ty.clone(), key_ty.clone()))
            }

            (Type::Unknown, _) => {
                for arg in args {
                    self.analyze_expression(&arg.value)?;
                }
                Ok(Type::Unknown)
            }
            _ => {
                if self.current_autocheck {
                    for arg in args {
                        let _ = self.analyze_expression(&arg.value);
                    }
                    Ok(Type::Unknown)
                } else {
                    Err(self.error(
                        line,
                        &format!("Unknown method '{}' for {}", method, entity_type),
                    ))
                }
            }
        }
    }

    fn field_type(&self, data_name: &str, field: &str, line: usize) -> Result<Type, ViceError> {
        self.data_defs
            .get(data_name)
            .and_then(|data| data.fields.get(field))
            .cloned()
            .ok_or_else(|| {
                self.error(
                    line,
                    &format!("Field '{}' not found in data '{}'", field, data_name),
                )
            })
    }

    fn field_type_for_type(&self, ty: &Type, field: &str, line: usize) -> Result<Type, ViceError> {
        let Type::Data(data_name, args) = ty else {
            return Ok(Type::Unknown);
        };
        let raw = self.field_type(data_name, field, line)?;
        Ok(match (data_name.as_str(), raw) {
            ("Result", Type::Generic(name)) if name == "T" => {
                args.first().cloned().unwrap_or(Type::Unknown)
            }
            ("Result", Type::Generic(name)) if name == "E" => {
                args.get(1).cloned().unwrap_or(Type::Unknown)
            }
            ("Option", Type::Generic(name)) if name == "T" => {
                args.first().cloned().unwrap_or(Type::Unknown)
            }
            (_, other) => other,
        })
    }

    fn as_data_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Data(name, _) => Some(name.clone()),
            Type::Function { return_type, .. } => match return_type.as_ref() {
                Type::Data(name, _) => Some(name.clone()),
                _ => None,
            },
            Type::Module(_) => None, // Modules are not data types in this context
            _ => None,
        }
    }

    fn check_arg_count(
        &self,
        line: usize,
        name: &str,
        expected: usize,
        actual: usize,
    ) -> Result<(), ViceError> {
        if expected != actual {
            Err(self.error(
                line,
                &format!(
                    "function '{}' expects {} arguments, got {}",
                    name, expected, actual
                ),
            ))
        } else {
            Ok(())
        }
    }

    fn expect_arg_types(
        &self,
        line: usize,
        name: &str,
        args: &[CallArg],
        expected: &[Type],
    ) -> Result<(), ViceError> {
        self.check_arg_count(line, name, expected.len(), args.len())?;
        for (idx, expected_ty) in expected.iter().enumerate() {
            let actual = self.analyze_expression(&args[idx].value)?;
            if !self.types_compatible(expected_ty, &actual, self.current_mode) {
                return Err(self.error(
                    args[idx].value.span.line,
                    &format!(
                        "Argument {} for '{}' expected {}, got {}",
                        idx + 1,
                        name,
                        expected_ty,
                        actual
                    ),
                ));
            }
        }
        Ok(())
    }

    fn module_mode(program: &Program) -> HybridMode {
        for top in &program.top_level {
            match top {
                TopLevel::EnumDef(_) => continue,
                TopLevel::TraitDef(_) => continue,
                TopLevel::Impl(_) => continue,
                TopLevel::Import(_) => continue,
                TopLevel::Function(function) => {
                    return Self::mode_from_attributes(&function.attributes, HybridMode::Default);
                }
                TopLevel::Data(statement) => {
                    if let StmtKind::DataDeclaration { attributes, .. } = &statement.kind {
                        return Self::mode_from_attributes(attributes, HybridMode::Default);
                    }
                    return HybridMode::Default;
                }
            }
        }
        HybridMode::Default
    }

    fn mode_for_attributes(&self, attributes: &[Attribute]) -> HybridMode {
        Self::mode_from_attributes(attributes, self.module_mode)
    }

    fn mode_from_attributes(attributes: &[Attribute], fallback: HybridMode) -> HybridMode {
        if attributes
            .iter()
            .any(|attribute| attribute.name == "lowlevel")
        {
            HybridMode::LowLevel
        } else if attributes
            .iter()
            .any(|attribute| attribute.name == "highlevel")
        {
            HybridMode::HighLevel
        } else {
            fallback
        }
    }

    fn types_compatible(&self, expected: &Type, actual: &Type, mode: HybridMode) -> bool {
        if expected == actual
            || matches!(expected, Type::Unknown)
            || matches!(actual, Type::Unknown)
        {
            return true;
        }

        match (expected, actual) {
            (Type::Float, Type::Int) => true,
            (Type::Int, Type::Float) => mode == HybridMode::LowLevel,
            (Type::Array(expected), Type::Array(actual)) => {
                self.types_compatible(expected, actual, mode)
            }
            _ => expected.is_compatible_with(actual),
        }
    }

    fn is_numeric(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float)
    }

    fn validate_type_annotation(
        &self,
        ty: &Type,
        type_params: &[String],
        line: usize,
    ) -> Result<(), ViceError> {
        match ty {
            Type::Data(name, args) => {
                if !type_params.contains(name)
                    && !self.data_defs.contains_key(name)
                    && !self.enum_defs.contains_key(name)
                    && ![
                        "Result",
                        "Option",
                        "Window",
                        "UiResponse",
                        "TextInputResponse",
                        "Canvas",
                        "Theme",
                        "UiEvent",
                        "Error",
                        "Handle",
                        "HttpResponse",
                        "Matrix",
                        "array",
                        "Array",
                        "map",
                        "Map",
                        "any",
                        "unknown",
                    ]
                    .contains(&name.as_str())
                {
                    return Err(self.error(
                        line,
                        &format!("Unknown type or generic parameter: {}", name),
                    ));
                }
                if self.data_defs.contains_key(name)
                    && args.is_empty()
                    && name != "Result"
                    && name != "Option"
                {
                    // It's a non-generic struct
                } else if type_params.contains(name) && !args.is_empty() {
                    return Err(self.error(
                        line,
                        &format!("Generic parameter '{}' cannot have type arguments", name),
                    ));
                }
                for arg in args {
                    self.validate_type_annotation(arg, type_params, line)?;
                }
            }
            Type::Array(inner) => self.validate_type_annotation(inner, type_params, line)?,
            Type::Map(k, v) => {
                self.validate_type_annotation(k, type_params, line)?;
                self.validate_type_annotation(v, type_params, line)?;
            }
            Type::Function {
                params,
                return_type,
            } => {
                for p in params {
                    self.validate_type_annotation(p, type_params, line)?;
                }
                self.validate_type_annotation(return_type, type_params, line)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_trait_type_annotation(
        &self,
        ty: &Type,
        type_params: &[String],
        span: Span,
    ) -> Result<(), ViceError> {
        match ty {
            Type::Data(name, args) if name == "Self" => {
                if args.is_empty() {
                    Ok(())
                } else {
                    Err(self.error_at_span(span, "Self cannot have type arguments"))
                }
            }
            Type::Data(name, args) => {
                if type_params.contains(name) && !args.is_empty() {
                    return Err(self.error_at_span(
                        span,
                        &format!("Generic parameter '{}' cannot have type arguments", name),
                    ));
                }
                self.validate_type_annotation(
                    &Type::Data(name.clone(), Vec::new()),
                    type_params,
                    span.line,
                )?;
                for arg in args {
                    self.validate_trait_type_annotation(arg, type_params, span)?;
                }
                Ok(())
            }
            Type::Array(inner) => self.validate_trait_type_annotation(inner, type_params, span),
            Type::Map(key, value) => {
                self.validate_trait_type_annotation(key, type_params, span)?;
                self.validate_trait_type_annotation(value, type_params, span)
            }
            Type::Function {
                params,
                return_type,
            } => {
                for param in params {
                    self.validate_trait_type_annotation(param, type_params, span)?;
                }
                self.validate_trait_type_annotation(return_type, type_params, span)
            }
            Type::Reference { inner, .. } => {
                self.validate_trait_type_annotation(inner, type_params, span)
            }
            _ => Ok(()),
        }
    }

    fn resolve_generics(&self, ty: &Type, type_params: &[String]) -> Type {
        match ty {
            Type::Data(name, args) => {
                if type_params.contains(name) {
                    Type::Generic(name.clone())
                } else {
                    let resolved_args = args
                        .iter()
                        .map(|a| self.resolve_generics(a, type_params))
                        .collect();
                    Type::Data(name.clone(), resolved_args)
                }
            }
            Type::Array(inner) => Type::Array(Box::new(self.resolve_generics(inner, type_params))),
            Type::Map(k, v) => Type::Map(
                Box::new(self.resolve_generics(k, type_params)),
                Box::new(self.resolve_generics(v, type_params)),
            ),
            Type::Function {
                params,
                return_type,
            } => {
                let res_params = params
                    .iter()
                    .map(|p| self.resolve_generics(p, type_params))
                    .collect();
                Type::Function {
                    params: res_params,
                    return_type: Box::new(self.resolve_generics(return_type, type_params)),
                }
            }
            Type::Reference { mutable, inner } => Type::Reference {
                mutable: *mutable,
                inner: Box::new(self.resolve_generics(inner, type_params)),
            },
            _ => ty.clone(),
        }
    }

    fn error(&self, line: usize, message: &str) -> ViceError {
        ViceError::at(
            ViceErrorType::Semantic,
            message,
            self.current_source.clone(),
            line,
            1,
            1,
        )
    }

    fn error_at_span(&self, span: Span, message: &str) -> ViceError {
        ViceError::at(
            ViceErrorType::Semantic,
            message,
            self.source_files.get(span.source_id).cloned(),
            span.line,
            span.column,
            span.length,
        )
    }

    fn validate_tail_recursion(&self, statements: &[Stmt], target: &str) -> Result<(), ViceError> {
        self.check_tail_stmts(statements, target)
    }

    fn check_tail_stmts(&self, statements: &[Stmt], target: &str) -> Result<(), ViceError> {
        for (i, stmt) in statements.iter().enumerate() {
            let is_last = i == statements.len() - 1;
            self.check_stmt_tailrec(stmt, target, is_last)?;
        }
        Ok(())
    }

    fn check_stmt_tailrec(
        &self,
        stmt: &Stmt,
        target: &str,
        _in_tail_position: bool,
    ) -> Result<(), ViceError> {
        match &stmt.kind {
            StmtKind::Return(Some(expr)) => {
                self.check_return_expr_tailrec(expr, target)?;
            }
            StmtKind::Return(None) => {}
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if expr_contains_call(condition, target) {
                    return Err(self.error_at_span(
                        stmt.span,
                        &format!(
                            "Function '{}' is marked with @tailrec but recursive call is in if condition",
                            target
                        ),
                    ));
                }
                self.check_tail_stmts(then_branch, target)?;
                if let Some(else_b) = else_branch {
                    self.check_tail_stmts(else_b, target)?;
                }
            }
            StmtKind::Block(stmts) => {
                self.check_tail_stmts(stmts, target)?;
            }
            _ => {
                if stmt_contains_call(stmt, target) {
                    return Err(self.error_at_span(
                        stmt.span,
                        &format!(
                            "Function '{}' is marked with @tailrec but recursive call is not in tail position",
                            target
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_return_expr_tailrec(&self, expr: &Expr, target: &str) -> Result<(), ViceError> {
        match &expr.kind {
            ExprKind::Call { name, args, .. } if name == target => {
                for arg in args {
                    if expr_contains_call(&arg.value, target) {
                        return Err(self.error_at_span(
                            arg.span,
                            &format!(
                                "Function '{}' is marked with @tailrec but recursive call is not in tail position",
                                target
                            ),
                        ));
                    }
                }
                Ok(())
            }
            _ => {
                if expr_contains_call(expr, target) {
                    return Err(self.error_at_span(
                        expr.span,
                        &format!(
                            "Function '{}' is marked with @tailrec but recursive call is not in tail position",
                            target
                        ),
                    ));
                }
                Ok(())
            }
        }
    }
}

pub fn extract_attribute_message(attr: &crate::ast::Attribute) -> Option<String> {
    for arg in &attr.args {
        if arg.name.as_deref() == Some("message")
            || arg.name.as_deref() == Some("reason")
            || arg.name.is_none()
        {
            if let crate::ast::ExprKind::String(text) = &arg.value.kind {
                return Some(text.clone());
            }
        }
    }
    None
}

pub fn extract_attribute_version(attr: &crate::ast::Attribute) -> Option<String> {
    for arg in &attr.args {
        if arg.name.as_deref() == Some("version")
            || arg.name.as_deref() == Some("since")
            || arg.name.is_none()
        {
            match &arg.value.kind {
                crate::ast::ExprKind::String(text) => return Some(text.clone()),
                crate::ast::ExprKind::Number(num) => return Some(num.clone()),
                _ => {}
            }
        }
    }
    None
}

pub fn extract_attribute_align(attr: &crate::ast::Attribute) -> Option<usize> {
    for arg in &attr.args {
        if arg.name.as_deref() == Some("bytes")
            || arg.name.as_deref() == Some("align")
            || arg.name.is_none()
        {
            if let crate::ast::ExprKind::Number(num) = &arg.value.kind {
                if let Ok(val) = num.parse::<usize>() {
                    return Some(val);
                }
            }
        }
    }
    None
}

fn count_recursive_calls(statements: &[crate::ast::Stmt], target: &str) -> usize {
    let mut count = 0;
    for stmt in statements {
        count += count_calls_in_stmt(stmt, target);
    }
    count
}

fn count_calls_in_stmt(stmt: &crate::ast::Stmt, target: &str) -> usize {
    match &stmt.kind {
        crate::ast::StmtKind::Variable { value, .. }
        | crate::ast::StmtKind::Assign { value, .. }
        | crate::ast::StmtKind::Return(Some(value))
        | crate::ast::StmtKind::Expression(value)
        | crate::ast::StmtKind::Fail { message: value } => count_calls_in_expr(value, target),
        crate::ast::StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut c =
                count_calls_in_expr(condition, target) + count_recursive_calls(then_branch, target);
            if let Some(eb) = else_branch {
                c += count_recursive_calls(eb, target);
            }
            c
        }
        crate::ast::StmtKind::While { condition, body } => {
            count_calls_in_expr(condition, target) + count_recursive_calls(body, target)
        }
        crate::ast::StmtKind::ForLoop {
            start, end, body, ..
        } => {
            count_calls_in_expr(start, target)
                + count_calls_in_expr(end, target)
                + count_recursive_calls(body, target)
        }
        crate::ast::StmtKind::ForEach {
            collection, body, ..
        } => {
            count_calls_in_expr(collection, target) + count_recursive_calls(body, target)
        }
        crate::ast::StmtKind::Block(stmts) => count_recursive_calls(stmts, target),
        crate::ast::StmtKind::MemberAssign { entity, value, .. } => {
            count_calls_in_expr(entity, target) + count_calls_in_expr(value, target)
        }
        crate::ast::StmtKind::IndexAssign {
            entity,
            index,
            value,
        } => {
            count_calls_in_expr(entity, target)
                + count_calls_in_expr(index, target)
                + count_calls_in_expr(value, target)
        }
        crate::ast::StmtKind::Match { condition, arms } => {
            let mut c = count_calls_in_expr(condition, target);
            for arm in arms {
                c += count_calls_in_stmt(&arm.body, target);
            }
            c
        }
        _ => 0,
    }
}

fn count_calls_in_expr(expr: &crate::ast::Expr, target: &str) -> usize {
    match &expr.kind {
        crate::ast::ExprKind::Call { name, args, .. } => {
            let mut c = if name == target { 1 } else { 0 };
            for arg in args {
                c += count_calls_in_expr(&arg.value, target);
            }
            c
        }
        crate::ast::ExprKind::MethodCall { entity, args, .. } => {
            let mut c = count_calls_in_expr(entity, target);
            for arg in args {
                c += count_calls_in_expr(&arg.value, target);
            }
            c
        }
        crate::ast::ExprKind::Binary { left, right, .. }
        | crate::ast::ExprKind::Logical { left, right, .. } => {
            count_calls_in_expr(left, target) + count_calls_in_expr(right, target)
        }
        crate::ast::ExprKind::Unary {
            right: operand, ..
        }
        | crate::ast::ExprKind::Try { expr: operand }
        | crate::ast::ExprKind::Unwrap { expr: operand }
        | crate::ast::ExprKind::Get {
            entity: operand, ..
        } => count_calls_in_expr(operand, target),
        crate::ast::ExprKind::Index { entity, index } => {
            count_calls_in_expr(entity, target) + count_calls_in_expr(index, target)
        }
        crate::ast::ExprKind::Array(elements)
        | crate::ast::ExprKind::InterpolatedString(elements) => {
            elements.iter().map(|e| count_calls_in_expr(e, target)).sum()
        }
        crate::ast::ExprKind::Map(pairs) => pairs
            .iter()
            .map(|(k, v)| count_calls_in_expr(k, target) + count_calls_in_expr(v, target))
            .sum(),
        _ => 0,
    }
}

fn expr_contains_call(expr: &crate::ast::Expr, target: &str) -> bool {
    count_calls_in_expr(expr, target) > 0
}

fn stmt_contains_call(stmt: &crate::ast::Stmt, target: &str) -> bool {
    count_calls_in_stmt(stmt, target) > 0
}

fn removed_ui_method_replacement(method: &str) -> Option<&'static str> {
    match method {
        "create" | "open_window" => Some("ui.window(...)"),
        "begin_frame" => Some("ui.frame(window)"),
        "rect" => Some("ui.draw_rect(...)"),
        "label" => Some("ui.text(...)"),
        "image" => Some("ui.draw_image(...)"),
        "sprite" => Some("ui.draw_sprite(...)"),
        "clear" => Some("ui.panel(...) or ui.draw_rect(...)"),
        _ => None,
    }
}
