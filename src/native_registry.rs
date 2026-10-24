use crate::types::Type;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NativeArg {
    pub name: &'static str,
    pub ty: Type,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct NativeFuncDef {
    pub name: &'static str,
    pub args: Vec<NativeArg>,
    pub return_type: Type,
    pub varargs: bool,
}

#[derive(Debug, Clone)]
pub struct NativeRegistry {
    pub modules: HashMap<&'static str, HashMap<&'static str, NativeFuncDef>>,
}

impl Default for NativeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modules: HashMap::new(),
        };
        registry.init_all();
        registry
    }

    fn add_func(&mut self, module: &'static str, def: NativeFuncDef) {
        self.modules
            .entry(module)
            .or_default()
            .insert(def.name, def);
    }

    fn init_all(&mut self) {
        // --- UI ---
        for (name, return_type) in [
            ("window", Type::Data("Window".to_string(), vec![])),
            ("frame", Type::Bool),
            ("set_theme", Type::Bool),
            ("push_style", Type::Bool),
            ("pop_style", Type::Bool),
            ("panel", Type::Bool),
            ("end_panel", Type::Bool),
            ("row", Type::Bool),
            ("column", Type::Bool),
            ("end", Type::Bool),
            ("spacer", Type::Bool),
            ("separator", Type::Bool),
            ("scroll_area", Type::Bool),
            ("end_scroll", Type::Bool),
            ("text", Type::Bool),
            ("heading", Type::Bool),
            ("mono", Type::Bool),
            ("badge", Type::Bool),
            ("button", Type::Data("UiResponse".to_string(), vec![])),
            ("checkbox", Type::Data("UiResponse".to_string(), vec![])),
            (
                "input_text",
                Type::Data("TextInputResponse".to_string(), vec![]),
            ),
            (
                "text_area",
                Type::Data("TextInputResponse".to_string(), vec![]),
            ),
            ("slider", Type::Data("UiResponse".to_string(), vec![])),
            ("select", Type::Data("UiResponse".to_string(), vec![])),
            ("progress", Type::Bool),
            ("list_item", Type::Bool),
            ("begin_table", Type::Bool),
            ("table_row", Type::Bool),
            ("table_cell", Type::Bool),
            ("end_table", Type::Bool),
            ("canvas", Type::Data("Canvas".to_string(), vec![])),
            ("draw_rect", Type::Bool),
            ("draw_line", Type::Bool),
            ("draw_circle", Type::Bool),
            ("draw_text", Type::Bool),
            ("draw_image", Type::Bool),
            ("draw_sprite", Type::Bool),
            ("menu_bar", Type::Bool),
            ("menu", Type::Bool),
            ("menu_item", Type::Data("UiResponse".to_string(), vec![])),
            ("status_bar", Type::Bool),
            ("tooltip", Type::Bool),
            ("is_key_pressed", Type::Bool),
            ("mouse_pos", Type::Array(Box::new(Type::Float))),
        ] {
            self.add_func(
                "ui",
                NativeFuncDef {
                    name,
                    args: vec![],
                    return_type,
                    varargs: true,
                },
            );
        }

        // --- Game2D ---
        for name in [
            "begin",
            "end",
            "rect",
            "text",
            "text_cached",
            "image",
            "sprite",
            "tilemap",
            "set_layer",
            "set_camera",
        ] {
            self.add_func(
                "game2d",
                NativeFuncDef {
                    name,
                    args: vec![],
                    return_type: Type::Bool,
                    varargs: true,
                },
            );
        }

        // --- FS ---
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "read_text",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "write_text",
                args: vec![
                    NativeArg {
                        name: "path",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "content",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "try_read_text",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Data(
                    "Result".to_string(),
                    vec![Type::String, Type::Data("Error".to_string(), vec![])],
                ),
                varargs: false,
            },
        );
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "try_write_text",
                args: vec![
                    NativeArg {
                        name: "path",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "content",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Data(
                    "Result".to_string(),
                    vec![Type::Bool, Type::Data("Error".to_string(), vec![])],
                ),
                varargs: false,
            },
        );
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "read_json",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "write_json",
                args: vec![
                    NativeArg {
                        name: "path",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "value",
                        ty: Type::Unknown,
                        optional: false,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "append_text",
                args: vec![
                    NativeArg {
                        name: "path",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "content",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "list_dir",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Array(Box::new(Type::String)),
                varargs: false,
            },
        );
        self.add_func(
            "fs",
            NativeFuncDef {
                name: "create_dir",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Bool,
                varargs: false,
            },
        );

        // --- STRING ---
        for method in ["contains", "starts_with", "ends_with"] {
            self.add_func(
                "string",
                NativeFuncDef {
                    name: method,
                    args: vec![
                        NativeArg {
                            name: "s",
                            ty: Type::String,
                            optional: false,
                        },
                        NativeArg {
                            name: "sub",
                            ty: Type::String,
                            optional: false,
                        },
                    ],
                    return_type: Type::Bool,
                    varargs: false,
                },
            );
        }
        for method in ["trim", "trim_start", "trim_end", "upper", "lower", "to_upper", "to_lower", "rot13", "slugify", "hex_encode", "hex_decode"] {
            self.add_func(
                "string",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "s",
                        ty: Type::String,
                        optional: false,
                    }],
                    return_type: Type::String,
                    varargs: false,
                },
            );
        }
        for method in ["from_int", "from_float", "from_bool", "to_string"] {
            self.add_func(
                "string",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "val",
                        ty: Type::Unknown,
                        optional: false,
                    }],
                    return_type: Type::String,
                    varargs: false,
                },
            );
        }
        for method in ["to_int", "parse_int"] {
            self.add_func(
                "string",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "s",
                        ty: Type::String,
                        optional: false,
                    }],
                    return_type: Type::Int,
                    varargs: false,
                },
            );
        }
        for method in ["to_float", "parse_float"] {
            self.add_func(
                "string",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "s",
                        ty: Type::String,
                        optional: false,
                    }],
                    return_type: Type::Float,
                    varargs: false,
                },
            );
        }
        for method in ["levenshtein", "hamming"] {
            self.add_func(
                "string",
                NativeFuncDef {
                    name: method,
                    args: vec![
                        NativeArg { name: "s1", ty: Type::String, optional: false },
                        NativeArg { name: "s2", ty: Type::String, optional: false },
                    ],
                    return_type: Type::Int,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "string",
            NativeFuncDef {
                name: "truncate",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "max_len", ty: Type::Int, optional: false },
                    NativeArg { name: "ellipsis", ty: Type::String, optional: true },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        for m in ["is_ascii", "is_alphanumeric"] {
            self.add_func(
                "string",
                NativeFuncDef {
                    name: m,
                    args: vec![NativeArg { name: "s", ty: Type::String, optional: false }],
                    return_type: Type::Bool,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "string",
            NativeFuncDef {
                name: "count_occurrences",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "sub", ty: Type::String, optional: false },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "pad_zero",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "width", ty: Type::Int, optional: false },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        for m in ["lines_count", "words_count"] {
            self.add_func(
                "string",
                NativeFuncDef {
                    name: m,
                    args: vec![NativeArg { name: "s", ty: Type::String, optional: false }],
                    return_type: Type::Int,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "string",
            NativeFuncDef {
                name: "lines",
                args: vec![NativeArg {
                    name: "s",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Array(Box::new(Type::String)),
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "len",
                args: vec![NativeArg {
                    name: "s",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "split",
                args: vec![
                    NativeArg {
                        name: "s",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "delim",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Array(Box::new(Type::String)),
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "slice",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "start", ty: Type::Int, optional: false },
                    NativeArg { name: "end", ty: Type::Int, optional: false },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "reverse",
                args: vec![NativeArg { name: "s", ty: Type::String, optional: false }],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "repeat",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "count", ty: Type::Int, optional: false },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "char_at",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "index", ty: Type::Int, optional: false },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "count",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "needle", ty: Type::String, optional: false },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "capitalize",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "index_of",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "sub", ty: Type::String, optional: false },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "last_index_of",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "sub", ty: Type::String, optional: false },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "pad_start",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "target_len", ty: Type::Int, optional: false },
                    NativeArg { name: "pad", ty: Type::String, optional: false },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "pad_end",
                args: vec![
                    NativeArg { name: "s", ty: Type::String, optional: false },
                    NativeArg { name: "target_len", ty: Type::Int, optional: false },
                    NativeArg { name: "pad", ty: Type::String, optional: false },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "is_numeric",
                args: vec![NativeArg { name: "s", ty: Type::String, optional: false }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "is_alpha",
                args: vec![NativeArg { name: "s", ty: Type::String, optional: false }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "string",
            NativeFuncDef {
                name: "words",
                args: vec![NativeArg { name: "s", ty: Type::String, optional: false }],
                return_type: Type::Array(Box::new(Type::String)),
                varargs: false,
            },
        );

        // --- ARRAY ---
        for method in ["push", "append"] {
            self.add_func(
                "array",
                NativeFuncDef {
                    name: method,
                    args: vec![
                        NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                        NativeArg { name: "val", ty: Type::Unknown, optional: false },
                    ],
                    return_type: Type::Array(Box::new(Type::Unknown)),
                    varargs: false,
                },
            );
        }
        self.add_func(
            "array",
            NativeFuncDef {
                name: "pop",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "len",
                args: vec![NativeArg {
                    name: "arr",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::Int,
                varargs: false,
            },
        );
        for method in ["first", "last", "sort", "reverse"] {
            self.add_func(
                "array",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "arr",
                        ty: Type::Unknown,
                        optional: false,
                    }],
                    return_type: Type::Unknown,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "array",
            NativeFuncDef {
                name: "contains",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "val", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "index_of",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "val", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "slice",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "start", ty: Type::Int, optional: false },
                    NativeArg { name: "end", ty: Type::Int, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "concat",
                args: vec![
                    NativeArg { name: "a", ty: Type::Unknown, optional: false },
                    NativeArg { name: "b", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "join",
                args: vec![
                    NativeArg {
                        name: "arr",
                        ty: Type::Unknown,
                        optional: false,
                    },
                    NativeArg {
                        name: "delim",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "remove_at",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "index", ty: Type::Int, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "insert",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "index", ty: Type::Int, optional: false },
                    NativeArg { name: "element", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "fill",
                args: vec![
                    NativeArg { name: "value", ty: Type::Unknown, optional: false },
                    NativeArg { name: "count", ty: Type::Int, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "unique",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "chunk",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "size", ty: Type::Int, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "flatten",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "sum",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        for method in ["min", "max"] {
            self.add_func(
                "array",
                NativeFuncDef {
                    name: method,
                    args: vec![
                        NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    ],
                    return_type: Type::Unknown,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "array",
            NativeFuncDef {
                name: "zip",
                args: vec![
                    NativeArg { name: "a", ty: Type::Unknown, optional: false },
                    NativeArg { name: "b", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "product",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "dedup",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "binary_search",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "target", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "rotate",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "count", ty: Type::Int, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "repeat",
                args: vec![
                    NativeArg { name: "val", ty: Type::Unknown, optional: false },
                    NativeArg { name: "count", ty: Type::Int, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "windows",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "size", ty: Type::Int, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Array(Box::new(Type::Unknown)))),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "frequencies",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Map(Box::new(Type::String), Box::new(Type::Int)),
                varargs: false,
            },
        );
        for m in ["is_sorted", "all_unique"] {
            self.add_func(
                "array",
                NativeFuncDef {
                    name: m,
                    args: vec![NativeArg { name: "arr", ty: Type::Unknown, optional: false }],
                    return_type: Type::Bool,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "array",
            NativeFuncDef {
                name: "find_index",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "target", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "clamp_elements",
                args: vec![
                    NativeArg { name: "arr", ty: Type::Unknown, optional: false },
                    NativeArg { name: "min", ty: Type::Unknown, optional: false },
                    NativeArg { name: "max", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "array",
            NativeFuncDef {
                name: "compact",
                args: vec![NativeArg { name: "arr", ty: Type::Unknown, optional: false }],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        for method in ["diff", "intersect"] {
            self.add_func(
                "array",
                NativeFuncDef {
                    name: method,
                    args: vec![
                        NativeArg { name: "a", ty: Type::Unknown, optional: false },
                        NativeArg { name: "b", ty: Type::Unknown, optional: false },
                    ],
                    return_type: Type::Array(Box::new(Type::Unknown)),
                    varargs: false,
                },
            );
        }

        // --- MATH ---
        for method in ["floor", "ceil", "round"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "val",
                        ty: Type::Float,
                        optional: false,
                    }],
                    return_type: Type::Int,
                    varargs: false,
                },
            );
        }
        for method in ["sin", "cos", "tan", "sqrt", "cosh", "sinh", "tanh"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "val",
                        ty: Type::Float,
                        optional: false,
                    }],
                    return_type: Type::Float,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "math",
            NativeFuncDef {
                name: "pow",
                args: vec![
                    NativeArg {
                        name: "base",
                        ty: Type::Float,
                        optional: false,
                    },
                    NativeArg {
                        name: "exponent",
                        ty: Type::Float,
                        optional: false,
                    },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "random",
                args: vec![],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "abs",
                args: vec![NativeArg {
                    name: "val",
                    ty: Type::Unknown,
                    optional: false,
                }], // Accepts int or float
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        for method in ["min", "max"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: method,
                    args: vec![
                        NativeArg {
                            name: "a",
                            ty: Type::Unknown,
                            optional: false,
                        },
                        NativeArg {
                            name: "b",
                            ty: Type::Unknown,
                            optional: false,
                        },
                    ],
                    return_type: Type::Unknown,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "math",
            NativeFuncDef {
                name: "clamp",
                args: vec![
                    NativeArg {
                        name: "val",
                        ty: Type::Unknown,
                        optional: false,
                    },
                    NativeArg {
                        name: "min",
                        ty: Type::Unknown,
                        optional: false,
                    },
                    NativeArg {
                        name: "max",
                        ty: Type::Unknown,
                        optional: false,
                    },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "sign",
                args: vec![NativeArg {
                    name: "val",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::Int,
                varargs: false,
            },
        );
        for method in ["log", "log10", "log2"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "val",
                        ty: Type::Float,
                        optional: false,
                    }],
                    return_type: Type::Float,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "math",
            NativeFuncDef {
                name: "hypot",
                args: vec![
                    NativeArg {
                        name: "x",
                        ty: Type::Float,
                        optional: false,
                    },
                    NativeArg {
                        name: "y",
                        ty: Type::Float,
                        optional: false,
                    },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "atan2",
                args: vec![
                    NativeArg {
                        name: "y",
                        ty: Type::Float,
                        optional: false,
                    },
                    NativeArg {
                        name: "x",
                        ty: Type::Float,
                        optional: false,
                    },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        for method in ["asin", "acos"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "val",
                        ty: Type::Float,
                        optional: false,
                    }],
                    return_type: Type::Float,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "math",
            NativeFuncDef {
                name: "pi",
                args: vec![],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "e",
                args: vec![],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "deg",
                args: vec![NativeArg {
                    name: "radians",
                    ty: Type::Float,
                    optional: false,
                }],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "rad",
                args: vec![NativeArg {
                    name: "degrees",
                    ty: Type::Float,
                    optional: false,
                }],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "signum",
                args: vec![NativeArg {
                    name: "val",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        for method in ["trunc", "fract", "cbrt"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "val",
                        ty: Type::Float,
                        optional: false,
                    }],
                    return_type: Type::Float,
                    varargs: false,
                },
            );
        }
        for method in ["gcd", "lcm"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: method,
                    args: vec![
                        NativeArg {
                            name: "a",
                            ty: Type::Int,
                            optional: false,
                        },
                        NativeArg {
                            name: "b",
                            ty: Type::Int,
                            optional: false,
                        },
                    ],
                    return_type: Type::Int,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "math",
            NativeFuncDef {
                name: "is_nan",
                args: vec![NativeArg {
                    name: "val",
                    ty: Type::Float,
                    optional: false,
                }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "is_finite",
                args: vec![NativeArg {
                    name: "val",
                    ty: Type::Float,
                    optional: false,
                }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "factorial",
                args: vec![NativeArg {
                    name: "n",
                    ty: Type::Int,
                    optional: false,
                }],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "round_to",
                args: vec![
                    NativeArg {
                        name: "val",
                        ty: Type::Float,
                        optional: false,
                    },
                    NativeArg {
                        name: "decimals",
                        ty: Type::Int,
                        optional: false,
                    },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "lerp",
                args: vec![
                    NativeArg { name: "a", ty: Type::Float, optional: false },
                    NativeArg { name: "b", ty: Type::Float, optional: false },
                    NativeArg { name: "t", ty: Type::Float, optional: false },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "exp",
                args: vec![NativeArg { name: "val", ty: Type::Float, optional: false }],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "is_even",
                args: vec![NativeArg { name: "n", ty: Type::Int, optional: false }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "is_odd",
                args: vec![NativeArg { name: "n", ty: Type::Int, optional: false }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "is_prime",
                args: vec![NativeArg { name: "n", ty: Type::Int, optional: false }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "is_square",
                args: vec![NativeArg { name: "n", ty: Type::Int, optional: false }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "is_power_of_two",
                args: vec![NativeArg { name: "n", ty: Type::Int, optional: false }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        for bit_op in ["count_ones", "leading_zeros", "trailing_zeros", "next_power_of_two"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: bit_op,
                    args: vec![NativeArg { name: "n", ty: Type::Int, optional: false }],
                    return_type: Type::Int,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "math",
            NativeFuncDef {
                name: "clamp01",
                args: vec![NativeArg { name: "val", ty: Type::Unknown, optional: false }],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "lerp_inv",
                args: vec![
                    NativeArg { name: "a", ty: Type::Float, optional: false },
                    NativeArg { name: "b", ty: Type::Float, optional: false },
                    NativeArg { name: "v", ty: Type::Float, optional: false },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "remap",
                args: vec![
                    NativeArg { name: "v", ty: Type::Float, optional: false },
                    NativeArg { name: "in_min", ty: Type::Float, optional: false },
                    NativeArg { name: "in_max", ty: Type::Float, optional: false },
                    NativeArg { name: "out_min", ty: Type::Float, optional: false },
                    NativeArg { name: "out_max", ty: Type::Float, optional: false },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "midpoint",
                args: vec![
                    NativeArg { name: "a", ty: Type::Unknown, optional: false },
                    NativeArg { name: "b", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "approx_eq",
                args: vec![
                    NativeArg { name: "a", ty: Type::Float, optional: false },
                    NativeArg { name: "b", ty: Type::Float, optional: false },
                    NativeArg { name: "eps", ty: Type::Float, optional: true },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "hypot3",
                args: vec![
                    NativeArg { name: "x", ty: Type::Float, optional: false },
                    NativeArg { name: "y", ty: Type::Float, optional: false },
                    NativeArg { name: "z", ty: Type::Float, optional: false },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "clamp_int",
                args: vec![
                    NativeArg { name: "v", ty: Type::Int, optional: false },
                    NativeArg { name: "min", ty: Type::Int, optional: false },
                    NativeArg { name: "max", ty: Type::Int, optional: false },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        for bitwise in ["bitwise_and", "bitwise_or", "bitwise_xor", "bitwise_shl", "bitwise_shr"] {
            self.add_func(
                "math",
                NativeFuncDef {
                    name: bitwise,
                    args: vec![
                        NativeArg { name: "a", ty: Type::Int, optional: false },
                        NativeArg { name: "b", ty: Type::Int, optional: false },
                    ],
                    return_type: Type::Int,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "math",
            NativeFuncDef {
                name: "bitwise_not",
                args: vec![NativeArg { name: "a", ty: Type::Int, optional: false }],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "wrap",
                args: vec![
                    NativeArg { name: "val", ty: Type::Unknown, optional: false },
                    NativeArg { name: "min", ty: Type::Unknown, optional: false },
                    NativeArg { name: "max", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "smoothstep",
                args: vec![
                    NativeArg { name: "edge0", ty: Type::Float, optional: false },
                    NativeArg { name: "edge1", ty: Type::Float, optional: false },
                    NativeArg { name: "x", ty: Type::Float, optional: false },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "deg_to_rad",
                args: vec![NativeArg { name: "deg", ty: Type::Float, optional: false }],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "rad_to_deg",
                args: vec![NativeArg { name: "rad", ty: Type::Float, optional: false }],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "copysign",
                args: vec![
                    NativeArg { name: "x", ty: Type::Float, optional: false },
                    NativeArg { name: "y", ty: Type::Float, optional: false },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "math",
            NativeFuncDef {
                name: "dist",
                args: vec![
                    NativeArg { name: "x1", ty: Type::Float, optional: false },
                    NativeArg { name: "y1", ty: Type::Float, optional: false },
                    NativeArg { name: "x2", ty: Type::Float, optional: false },
                    NativeArg { name: "y2", ty: Type::Float, optional: false },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );

        // --- RANDOM ---
        self.add_func(
            "random",
            NativeFuncDef {
                name: "int",
                args: vec![
                    NativeArg {
                        name: "min",
                        ty: Type::Unknown,
                        optional: false,
                    },
                    NativeArg {
                        name: "max",
                        ty: Type::Unknown,
                        optional: false,
                    },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "random",
            NativeFuncDef {
                name: "float",
                args: vec![],
                return_type: Type::Float,
                varargs: false,
            },
        );

        // --- JSON ---
        self.add_func(
            "json",
            NativeFuncDef {
                name: "parse",
                args: vec![NativeArg {
                    name: "s",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "json",
            NativeFuncDef {
                name: "stringify",
                args: vec![NativeArg {
                    name: "val",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "json",
            NativeFuncDef {
                name: "try_parse",
                args: vec![NativeArg {
                    name: "s",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Data(
                    "Result".to_string(),
                    vec![Type::Unknown, Type::Data("Error".to_string(), vec![])],
                ),
                varargs: false,
            },
        );

        // --- PATH ---
        self.add_func(
            "path",
            NativeFuncDef {
                name: "join",
                args: vec![
                    NativeArg {
                        name: "a",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "b",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        for method in [
            "dirname",
            "basename",
            "extension",
            "exists",
            "is_file",
            "is_dir",
        ] {
            self.add_func(
                "path",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "p",
                        ty: Type::String,
                        optional: false,
                    }],
                    return_type: if matches!(method, "exists" | "is_file" | "is_dir") {
                        Type::Bool
                    } else {
                        Type::String
                    },
                    varargs: false,
                },
            );
        }

        // --- ENV ---
        self.add_func(
            "env",
            NativeFuncDef {
                name: "get",
                args: vec![NativeArg {
                    name: "k",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Data("Option".to_string(), vec![Type::String]),
                varargs: false,
            },
        );
        self.add_func(
            "env",
            NativeFuncDef {
                name: "has",
                args: vec![NativeArg {
                    name: "k",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "env",
            NativeFuncDef {
                name: "set",
                args: vec![
                    NativeArg {
                        name: "k",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "v",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );

        // --- PROCESS ---
        self.add_func(
            "process",
            NativeFuncDef {
                name: "args",
                args: vec![],
                return_type: Type::Array(Box::new(Type::String)),
                varargs: false,
            },
        );
        self.add_func(
            "process",
            NativeFuncDef {
                name: "cwd",
                args: vec![],
                return_type: Type::String,
                varargs: false,
            },
        );

        // --- RESULT / OPTION / CLI ---
        self.add_func(
            "result",
            NativeFuncDef {
                name: "ok",
                args: vec![NativeArg {
                    name: "value",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::Data("Result".to_string(), vec![Type::Unknown, Type::Unknown]),
                varargs: false,
            },
        );
        self.add_func(
            "result",
            NativeFuncDef {
                name: "err",
                args: vec![
                    NativeArg {
                        name: "message",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "code",
                        ty: Type::Int,
                        optional: false,
                    },
                ],
                return_type: Type::Data(
                    "Result".to_string(),
                    vec![Type::Unknown, Type::Data("Error".to_string(), vec![])],
                ),
                varargs: false,
            },
        );
        self.add_func(
            "option",
            NativeFuncDef {
                name: "some",
                args: vec![NativeArg {
                    name: "value",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::Data("Option".to_string(), vec![Type::Unknown]),
                varargs: false,
            },
        );
        self.add_func(
            "option",
            NativeFuncDef {
                name: "none",
                args: vec![],
                return_type: Type::Data("Option".to_string(), vec![Type::Unknown]),
                varargs: false,
            },
        );
        self.add_func(
            "cli",
            NativeFuncDef {
                name: "flag",
                args: vec![
                    NativeArg {
                        name: "args",
                        ty: Type::Array(Box::new(Type::String)),
                        optional: false,
                    },
                    NativeArg {
                        name: "name",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "cli",
            NativeFuncDef {
                name: "option",
                args: vec![
                    NativeArg {
                        name: "args",
                        ty: Type::Array(Box::new(Type::String)),
                        optional: false,
                    },
                    NativeArg {
                        name: "name",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Data("Option".to_string(), vec![Type::String]),
                varargs: false,
            },
        );

        // --- TIME ---
        self.add_func(
            "time",
            NativeFuncDef {
                name: "now_ms",
                args: vec![],
                return_type: Type::Int,
                varargs: false,
            },
        );

        // --- ENGINE ---
        self.add_func(
            "engine",
            NativeFuncDef {
                name: "load_scene",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Data("Level".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "engine",
            NativeFuncDef {
                name: "save_scene",
                args: vec![
                    NativeArg {
                        name: "path",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "value",
                        ty: Type::Unknown,
                        optional: false,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "engine",
            NativeFuncDef {
                name: "load_layout",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "engine",
            NativeFuncDef {
                name: "save_layout",
                args: vec![
                    NativeArg {
                        name: "path",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "value",
                        ty: Type::Unknown,
                        optional: false,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );

        // --- AUDIO ---
        self.add_func(
            "audio",
            NativeFuncDef {
                name: "play_sound",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "audio",
            NativeFuncDef {
                name: "play_music",
                args: vec![
                    NativeArg {
                        name: "path",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "volume",
                        ty: Type::Float,
                        optional: true,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );

        // --- MAP ---
        self.add_func(
            "map",
            NativeFuncDef {
                name: "len",
                args: vec![NativeArg { name: "m", ty: Type::Unknown, optional: false }],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "is_empty",
                args: vec![NativeArg { name: "m", ty: Type::Unknown, optional: false }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "contains_key",
                args: vec![
                    NativeArg { name: "m", ty: Type::Unknown, optional: false },
                    NativeArg { name: "key", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "keys",
                args: vec![NativeArg { name: "m", ty: Type::Unknown, optional: false }],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "values",
                args: vec![NativeArg { name: "m", ty: Type::Unknown, optional: false }],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "get",
                args: vec![
                    NativeArg { name: "m", ty: Type::Unknown, optional: false },
                    NativeArg { name: "key", ty: Type::Unknown, optional: false },
                    NativeArg { name: "default", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "set",
                args: vec![
                    NativeArg { name: "m", ty: Type::Unknown, optional: false },
                    NativeArg { name: "key", ty: Type::Unknown, optional: false },
                    NativeArg { name: "val", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "remove",
                args: vec![
                    NativeArg { name: "m", ty: Type::Unknown, optional: false },
                    NativeArg { name: "key", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "merge",
                args: vec![
                    NativeArg { name: "a", ty: Type::Unknown, optional: false },
                    NativeArg { name: "b", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "clear",
                args: vec![NativeArg { name: "m", ty: Type::Unknown, optional: false }],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "has_value",
                args: vec![
                    NativeArg { name: "m", ty: Type::Unknown, optional: false },
                    NativeArg { name: "val", ty: Type::Unknown, optional: false },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "map",
            NativeFuncDef {
                name: "invert",
                args: vec![NativeArg { name: "m", ty: Type::Unknown, optional: false }],
                return_type: Type::Unknown,
                varargs: false,
            },
        );

        // --- HASH ---
        for method in ["fnv1a", "djb2", "adler32", "crc32", "murmur3"] {
            self.add_func(
                "hash",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "s",
                        ty: Type::String,
                        optional: false,
                    }],
                    return_type: Type::Int,
                    varargs: false,
                },
            );
        }
        self.add_func(
            "hash",
            NativeFuncDef {
                name: "xxhash32",
                args: vec![
                    NativeArg {
                        name: "s",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "seed",
                        ty: Type::Int,
                        optional: true,
                    },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "hash",
            NativeFuncDef {
                name: "rot13",
                args: vec![NativeArg {
                    name: "s",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::String,
                varargs: false,
            },
        );
        for method in ["hex_encode", "hex_decode"] {
            self.add_func(
                "hash",
                NativeFuncDef {
                    name: method,
                    args: vec![NativeArg {
                        name: "s",
                        ty: Type::String,
                        optional: false,
                    }],
                    return_type: Type::String,
                    varargs: false,
                },
            );
        }

        // --- FFI ---
        self.add_func(
            "ffi",
            NativeFuncDef {
                name: "load",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Data("Handle".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "ffi",
            NativeFuncDef {
                name: "free",
                args: vec![NativeArg {
                    name: "handle",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "ffi",
            NativeFuncDef {
                name: "call",
                args: vec![
                    NativeArg {
                        name: "handle",
                        ty: Type::Unknown,
                        optional: false,
                    },
                    NativeArg {
                        name: "symbol",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "args",
                        ty: Type::Array(Box::new(Type::Unknown)),
                        optional: false,
                    },
                    NativeArg {
                        name: "return_type",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );

        // --- HTTP ---
        self.add_func(
            "http",
            NativeFuncDef {
                name: "get",
                args: vec![
                    NativeArg {
                        name: "url",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "headers",
                        ty: Type::Map(Box::new(Type::String), Box::new(Type::String)),
                        optional: true,
                    },
                ],
                return_type: Type::Data("HttpResponse".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "http",
            NativeFuncDef {
                name: "post",
                args: vec![
                    NativeArg {
                        name: "url",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "body",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "headers",
                        ty: Type::Map(Box::new(Type::String), Box::new(Type::String)),
                        optional: true,
                    },
                ],
                return_type: Type::Data("HttpResponse".to_string(), vec![]),
                varargs: false,
            },
        );

        // --- MATRIX ---
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "zeros",
                args: vec![
                    NativeArg {
                        name: "rows",
                        ty: Type::Int,
                        optional: false,
                    },
                    NativeArg {
                        name: "cols",
                        ty: Type::Int,
                        optional: false,
                    },
                ],
                return_type: Type::Data("Matrix".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "from_array",
                args: vec![
                    NativeArg {
                        name: "data",
                        ty: Type::Array(Box::new(Type::Float)),
                        optional: false,
                    },
                    NativeArg {
                        name: "rows",
                        ty: Type::Int,
                        optional: false,
                    },
                    NativeArg {
                        name: "cols",
                        ty: Type::Int,
                        optional: false,
                    },
                ],
                return_type: Type::Data("Matrix".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "to_array",
                args: vec![NativeArg {
                    name: "matrix",
                    ty: Type::Data("Matrix".to_string(), vec![]),
                    optional: false,
                }],
                return_type: Type::Array(Box::new(Type::Float)),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "shape",
                args: vec![NativeArg {
                    name: "matrix",
                    ty: Type::Data("Matrix".to_string(), vec![]),
                    optional: false,
                }],
                return_type: Type::Array(Box::new(Type::Int)),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "add",
                args: vec![
                    NativeArg {
                        name: "m1",
                        ty: Type::Data("Matrix".to_string(), vec![]),
                        optional: false,
                    },
                    NativeArg {
                        name: "m2",
                        ty: Type::Data("Matrix".to_string(), vec![]),
                        optional: false,
                    },
                ],
                return_type: Type::Data("Matrix".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "sub",
                args: vec![
                    NativeArg {
                        name: "m1",
                        ty: Type::Data("Matrix".to_string(), vec![]),
                        optional: false,
                    },
                    NativeArg {
                        name: "m2",
                        ty: Type::Data("Matrix".to_string(), vec![]),
                        optional: false,
                    },
                ],
                return_type: Type::Data("Matrix".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "mul",
                args: vec![
                    NativeArg {
                        name: "m1",
                        ty: Type::Data("Matrix".to_string(), vec![]),
                        optional: false,
                    },
                    NativeArg {
                        name: "m2",
                        ty: Type::Data("Matrix".to_string(), vec![]),
                        optional: false,
                    },
                ],
                return_type: Type::Data("Matrix".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "scale",
                args: vec![
                    NativeArg {
                        name: "matrix",
                        ty: Type::Data("Matrix".to_string(), vec![]),
                        optional: false,
                    },
                    NativeArg {
                        name: "factor",
                        ty: Type::Float,
                        optional: false,
                    },
                ],
                return_type: Type::Data("Matrix".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "transpose",
                args: vec![NativeArg {
                    name: "matrix",
                    ty: Type::Data("Matrix".to_string(), vec![]),
                    optional: false,
                }],
                return_type: Type::Data("Matrix".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "dot",
                args: vec![
                    NativeArg {
                        name: "v1",
                        ty: Type::Array(Box::new(Type::Float)),
                        optional: false,
                    },
                    NativeArg {
                        name: "v2",
                        ty: Type::Array(Box::new(Type::Float)),
                        optional: false,
                    },
                ],
                return_type: Type::Float,
                varargs: false,
            },
        );
        self.add_func(
            "matrix",
            NativeFuncDef {
                name: "sum",
                args: vec![NativeArg {
                    name: "matrix",
                    ty: Type::Data("Matrix".to_string(), vec![]),
                    optional: false,
                }],
                return_type: Type::Float,
                varargs: false,
            },
        );

        // --- CRYPTO ---
        self.add_func(
            "crypto",
            NativeFuncDef {
                name: "sha256",
                args: vec![NativeArg {
                    name: "text",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "crypto",
            NativeFuncDef {
                name: "sha512",
                args: vec![NativeArg {
                    name: "text",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "crypto",
            NativeFuncDef {
                name: "base64_encode",
                args: vec![NativeArg {
                    name: "text",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "crypto",
            NativeFuncDef {
                name: "base64_decode",
                args: vec![NativeArg {
                    name: "b64_string",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "crypto",
            NativeFuncDef {
                name: "random_bytes",
                args: vec![NativeArg {
                    name: "count",
                    ty: Type::Int,
                    optional: false,
                }],
                return_type: Type::Array(Box::new(Type::Int)),
                varargs: false,
            },
        );

        // --- REGEX ---
        self.add_func(
            "regex",
            NativeFuncDef {
                name: "is_match",
                args: vec![
                    NativeArg {
                        name: "pattern",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "text",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "regex",
            NativeFuncDef {
                name: "find",
                args: vec![
                    NativeArg {
                        name: "pattern",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "text",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Unknown,
                varargs: false,
            },
        );
        self.add_func(
            "regex",
            NativeFuncDef {
                name: "find_all",
                args: vec![
                    NativeArg {
                        name: "pattern",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "text",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Array(Box::new(Type::String)),
                varargs: false,
            },
        );
        self.add_func(
            "regex",
            NativeFuncDef {
                name: "replace",
                args: vec![
                    NativeArg {
                        name: "pattern",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "text",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "replacement",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::String,
                varargs: false,
            },
        );
        self.add_func(
            "regex",
            NativeFuncDef {
                name: "split",
                args: vec![
                    NativeArg {
                        name: "pattern",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "text",
                        ty: Type::String,
                        optional: false,
                    },
                ],
                return_type: Type::Array(Box::new(Type::String)),
                varargs: false,
            },
        );

        // --- SQLITE ---
        self.add_func(
            "sqlite",
            NativeFuncDef {
                name: "open",
                args: vec![NativeArg {
                    name: "path",
                    ty: Type::String,
                    optional: false,
                }],
                return_type: Type::Data("Handle".to_string(), vec![]),
                varargs: false,
            },
        );
        self.add_func(
            "sqlite",
            NativeFuncDef {
                name: "close",
                args: vec![NativeArg {
                    name: "handle",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::Bool,
                varargs: false,
            },
        );
        self.add_func(
            "sqlite",
            NativeFuncDef {
                name: "execute",
                args: vec![
                    NativeArg {
                        name: "handle",
                        ty: Type::Unknown,
                        optional: false,
                    },
                    NativeArg {
                        name: "sql",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "params",
                        ty: Type::Array(Box::new(Type::Unknown)),
                        optional: true,
                    },
                ],
                return_type: Type::Int,
                varargs: false,
            },
        );
        self.add_func(
            "sqlite",
            NativeFuncDef {
                name: "query",
                args: vec![
                    NativeArg {
                        name: "handle",
                        ty: Type::Unknown,
                        optional: false,
                    },
                    NativeArg {
                        name: "sql",
                        ty: Type::String,
                        optional: false,
                    },
                    NativeArg {
                        name: "params",
                        ty: Type::Array(Box::new(Type::Unknown)),
                        optional: true,
                    },
                ],
                return_type: Type::Array(Box::new(Type::Unknown)),
                varargs: false,
            },
        );
        self.add_func(
            "sqlite",
            NativeFuncDef {
                name: "last_insert_rowid",
                args: vec![NativeArg {
                    name: "handle",
                    ty: Type::Unknown,
                    optional: false,
                }],
                return_type: Type::Int,
                varargs: false,
            },
        );
    }

    pub fn get_method(&self, module: &str, method: &str) -> Option<&NativeFuncDef> {
        self.modules.get(module).and_then(|m| m.get(method))
    }
}
