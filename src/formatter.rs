use crate::ast::*;
use crate::error::ViceError;
use crate::parse_source_named;
use crate::types::Type;
use std::path::Path;

pub fn format_source(source: &str, file: &Path) -> Result<String, ViceError> {
    let comments = extract_comments(source);
    let program = parse_source_named(source, file)?;
    Ok(restore_comments(format_program(&program), comments))
}

pub fn format_program(program: &Program) -> String {
    let mut chunks = Vec::new();

    if program.top_level.is_empty() {
        for import in &program.imports {
            chunks.push(format_import(import));
        }
        for data in &program.data_declarations {
            chunks.push(format_stmt_top_level(data));
        }
        for function in &program.functions {
            chunks.push(format_function(function));
        }
        for impl_block in &program.impls {
            chunks.push(format_impl(impl_block));
        }
    } else {
        for item in &program.top_level {
            match item {
                TopLevel::Import(import) => chunks.push(format_import(import)),
                TopLevel::Function(function) => chunks.push(format_function(function)),
                TopLevel::Data(data) => chunks.push(format_stmt_top_level(data)),
                TopLevel::EnumDef(data) => chunks.push(format_stmt_top_level(data)),
                TopLevel::TraitDef(data) => chunks.push(format_stmt_top_level(data)),
                TopLevel::Impl(impl_block) => chunks.push(format_impl(impl_block)),
            }
        }
    }

    let mut output = chunks.join("\n\n");
    output.push('\n');
    output
}

#[derive(Debug)]
struct SourceComment {
    text: String,
    placement: CommentPlacement,
}

#[derive(Debug)]
enum CommentPlacement {
    Before {
        anchor: String,
        occurrence: usize,
        before_closing_brace: bool,
    },
    Trailing {
        anchor: String,
        occurrence: usize,
    },
    End,
}

fn extract_comments(source: &str) -> Vec<SourceComment> {
    let lines = source.lines().collect::<Vec<_>>();
    let split = lines
        .iter()
        .map(|line| split_comment(line))
        .collect::<Vec<_>>();
    let normalized = split
        .iter()
        .map(|(code, _)| normalize_code(code))
        .collect::<Vec<_>>();
    let mut comments = Vec::new();

    for (line_index, (code, comment)) in split.iter().enumerate() {
        let Some(text) = comment else {
            continue;
        };
        let code_anchor = normalize_code(code);
        let placement = if !code_anchor.is_empty() {
            CommentPlacement::Trailing {
                occurrence: anchor_occurrence(&normalized, line_index, &code_anchor),
                anchor: code_anchor,
            }
        } else if let Some((anchor_index, anchor)) = normalized
            .iter()
            .enumerate()
            .skip(line_index + 1)
            .find(|(_, candidate)| !candidate.is_empty())
        {
            CommentPlacement::Before {
                occurrence: anchor_occurrence(&normalized, anchor_index, anchor),
                before_closing_brace: anchor == "}",
                anchor: anchor.clone(),
            }
        } else {
            CommentPlacement::End
        };
        comments.push(SourceComment {
            text: text.trim_end().to_string(),
            placement,
        });
    }
    comments
}

fn split_comment(line: &str) -> (&str, Option<&str>) {
    let mut in_string = false;
    let mut escaped = false;
    let chars = line.char_indices().collect::<Vec<_>>();
    for (position, (byte_index, ch)) in chars.iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if *ch == '\\' {
                escaped = true;
            } else if *ch == '"' {
                in_string = false;
            }
            continue;
        }
        if *ch == '"' {
            in_string = true;
        } else if *ch == '#'
            || (*ch == '/' && chars.get(position + 1).map(|(_, next)| *next) == Some('/'))
        {
            return (&line[..*byte_index], Some(&line[*byte_index..]));
        }
    }
    (line, None)
}

fn normalize_code(code: &str) -> String {
    let mut normalized = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in code.trim().chars() {
        if in_string {
            normalized.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            normalized.push(ch);
        } else if !ch.is_whitespace() {
            normalized.push(ch);
        }
    }
    normalized
}

fn anchor_occurrence(lines: &[String], through: usize, anchor: &str) -> usize {
    lines
        .iter()
        .take(through + 1)
        .filter(|candidate| candidate.as_str() == anchor)
        .count()
}

fn find_anchor(lines: &[String], anchor: &str, occurrence: usize) -> Option<usize> {
    let mut seen = 0;
    for (index, line) in lines.iter().enumerate() {
        if normalize_code(line) == anchor {
            seen += 1;
            if seen == occurrence {
                return Some(index);
            }
        }
    }
    None
}

fn restore_comments(formatted: String, comments: Vec<SourceComment>) -> String {
    if comments.is_empty() {
        return formatted;
    }

    let lines = formatted.lines().map(str::to_string).collect::<Vec<_>>();
    let mut before = vec![Vec::<String>::new(); lines.len()];
    let mut trailing = vec![Vec::<String>::new(); lines.len()];
    let mut end = Vec::new();

    for comment in comments {
        match comment.placement {
            CommentPlacement::Before {
                anchor,
                occurrence,
                before_closing_brace,
            } => {
                if let Some(index) = find_anchor(&lines, &anchor, occurrence) {
                    let base_indent = lines[index].chars().take_while(|ch| *ch == ' ').count();
                    let indent = base_indent + usize::from(before_closing_brace) * 4;
                    before[index].push(format!("{}{}", " ".repeat(indent), comment.text));
                } else {
                    end.push(comment.text);
                }
            }
            CommentPlacement::Trailing { anchor, occurrence } => {
                if let Some(index) = find_anchor(&lines, &anchor, occurrence) {
                    trailing[index].push(comment.text);
                } else {
                    end.push(comment.text);
                }
            }
            CommentPlacement::End => end.push(comment.text),
        }
    }

    let mut output = Vec::new();
    for (index, line) in lines.into_iter().enumerate() {
        output.append(&mut before[index]);
        if trailing[index].is_empty() {
            output.push(line);
        } else {
            output.push(format!("{} {}", line, trailing[index].join(" ")));
        }
    }
    output.extend(end);
    format!("{}\n", output.join("\n"))
}

fn format_import(import: &Import) -> String {
    format!("use \"{}\";", escape_string(&import.path))
}

fn format_stmt_top_level(stmt: &Stmt) -> String {
    match &stmt.kind {
        StmtKind::DataDeclaration { .. } => format_statement(stmt, 0),
        _ => format_statement(stmt, 0),
    }
}

fn format_function(function: &Function) -> String {
    let mut lines = format_attributes(&function.attributes, 0);
    let params = function
        .params
        .iter()
        .map(|param| {
            let mut_prefix = if param.mutable { "mut " } else { "" };
            let type_suffix = param
                .type_annotation
                .as_ref()
                .map(|ty| format!(": {}", format_type(ty)))
                .unwrap_or_default();
            format!("{}{}{}", mut_prefix, param.name, type_suffix)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let return_type = function
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", format_type(ty)))
        .unwrap_or_default();
    lines.push(format!(
        "fn {}({}){} {{",
        function.name, params, return_type
    ));
    lines.extend(format_block_body(&function.body, 1));
    lines.push("}".to_string());
    lines.join("\n")
}

fn format_impl(impl_block: &ImplBlock) -> String {
    let type_params_str = if impl_block.type_params.is_empty() {
        String::new()
    } else {
        format!("<{}>", impl_block.type_params.join(", "))
    };
    let trait_str = if let Some(trait_name) = &impl_block.trait_name {
        format!("{} for ", format_type(trait_name))
    } else {
        String::new()
    };
    let mut lines = vec![format!(
        "impl{}{} {}{} {{",
        type_params_str,
        if type_params_str.is_empty() { "" } else { " " },
        trait_str,
        impl_block.target
    )];
    for method in &impl_block.methods {
        let method_formatted = format_function(method);
        for line in method_formatted.lines() {
            if line.is_empty() {
                lines.push(String::new());
            } else {
                lines.push(format!("    {}", line));
            }
        }
    }
    lines.push("}".to_string());
    lines.join("\n")
}

fn format_attributes(attributes: &[Attribute], indent_level: usize) -> Vec<String> {
    attributes
        .iter()
        .map(|attribute| {
            let args = if attribute.args.is_empty() {
                String::new()
            } else {
                format!("({})", format_call_args(&attribute.args))
            };
            format!("{}@{}{}", indent(indent_level), attribute.name, args)
        })
        .collect()
}

fn format_block_body(statements: &[Stmt], indent_level: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for statement in statements {
        lines.extend(
            format_statement(statement, indent_level)
                .lines()
                .map(str::to_string),
        );
    }
    lines
}

fn format_statement(stmt: &Stmt, indent_level: usize) -> String {
    let pad = indent(indent_level);
    match &stmt.kind {
        StmtKind::Output(expressions) => format!(
            "{}output <- {};",
            pad,
            expressions
                .iter()
                .map(format_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        StmtKind::Emit(expressions) => format!(
            "{}emit <- {};",
            pad,
            expressions
                .iter()
                .map(format_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        StmtKind::Variable {
            name,
            mutable,
            type_annotation,
            value,
        } => {
            let keyword = if *mutable { "mut" } else { "let" };
            let type_text = type_annotation
                .as_ref()
                .map(|ty| format!(": {}", format_type(ty)))
                .unwrap_or_default();
            format!(
                "{}{} {}{} => {};",
                pad,
                keyword,
                name,
                type_text,
                format_expr(value)
            )
        }
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut lines = vec![format!("{}if ({}) {{", pad, format_expr(condition))];
            lines.extend(format_block_body(then_branch, indent_level + 1));
            if let Some(else_statements) = else_branch {
                lines.push(format!("{}}} else {{", pad));
                lines.extend(format_block_body(else_statements, indent_level + 1));
                lines.push(format!("{}}}", pad));
            } else {
                lines.push(format!("{}}}", pad));
            }
            lines.join("\n")
        }
        StmtKind::Assign { name, value } => format!("{}{} = {};", pad, name, format_expr(value)),
        StmtKind::DerefAssign { pointer, value } => {
            format!("{}*{} = {};", pad, format_expr(pointer), format_expr(value))
        }
        StmtKind::While { condition, body } => {
            let mut lines = vec![format!("{}while ({}) {{", pad, format_expr(condition))];
            lines.extend(format_block_body(body, indent_level + 1));
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        }
        StmtKind::ForLoop {
            iterator,
            start,
            end,
            body,
        } => {
            let mut lines = vec![format!(
                "{}for {} in {}..{} {{",
                pad,
                iterator,
                format_expr(start),
                format_expr(end)
            )];
            lines.extend(format_block_body(body, indent_level + 1));
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        }
        StmtKind::ForEach {
            iterator,
            collection,
            body,
        } => {
            let mut lines = vec![format!(
                "{}for {} in {} {{",
                pad,
                iterator,
                format_expr(collection)
            )];
            lines.extend(format_block_body(body, indent_level + 1));
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        }
        StmtKind::Block(statements) => {
            let mut lines = vec![format!("{}{{", pad)];
            lines.extend(format_block_body(statements, indent_level + 1));
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        }
        StmtKind::Input(name) => format!("{}input -> {};", pad, name),
        StmtKind::Expression(expr) => format!("{}{};", pad, format_expr(expr)),
        StmtKind::Return(Some(expr)) => format!("{}return {};", pad, format_expr(expr)),
        StmtKind::Return(None) => format!("{}return;", pad),
        StmtKind::Break => format!("{}break;", pad),
        StmtKind::Continue => format!("{}continue;", pad),
        StmtKind::Match { condition, arms } => {
            let mut lines = vec![format!("{}match ({}) {{", pad, format_expr(condition))];
            for arm in arms {
                let patterns = arm
                    .patterns
                    .iter()
                    .map(format_expr)
                    .collect::<Vec<_>>()
                    .join(" | ");
                let body_str = match &arm.body.kind {
                    StmtKind::Expression(expr) => format_expr(expr),
                    StmtKind::Block(stmts) => {
                        let mut inner = vec!["{".to_string()];
                        inner.extend(format_block_body(stmts, indent_level + 1));
                        inner.push("}".to_string());
                        inner.join("\n")
                    }
                    _ => format_statement(&arm.body, indent_level + 1),
                };
                lines.push(format!(
                    "{}{} => {}",
                    indent(indent_level + 1),
                    patterns,
                    body_str
                ));
            }
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        }
        StmtKind::MemberAssign {
            entity,
            name,
            value,
        } => format!(
            "{}{}.{} = {};",
            pad,
            format_expr(entity),
            name,
            format_expr(value)
        ),
        StmtKind::IndexAssign {
            entity,
            index,
            value,
        } => format!(
            "{}{}[{}] = {};",
            pad,
            format_expr(entity),
            format_expr(index),
            format_expr(value)
        ),
        StmtKind::DataDeclaration {
            name,
            type_params,
            attributes,
            fields,
        } => {
            let mut lines = format_attributes(attributes, indent_level);
            let generic_str = if type_params.is_empty() {
                String::new()
            } else {
                format!("<{}>", type_params.join(", "))
            };
            lines.push(format!("{}data {}{} {{", pad, name, generic_str));
            for field in fields {
                lines.push(format!(
                    "{}{}: {} => {};",
                    indent(indent_level + 1),
                    field.name,
                    format_type(&field.type_annotation),
                    format_expr(&field.default_value)
                ));
            }
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        }
        StmtKind::EnumDef {
            name,
            variants,
            attributes,
        } => {
            let mut lines = format_attributes(attributes, indent_level);
            lines.push(format!("{}enum {} {{", pad, name));
            for variant in variants {
                lines.push(format!("{}{},", indent(indent_level + 1), variant));
            }
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        }
        StmtKind::TraitDef {
            name,
            type_params,
            attributes,
            methods,
        } => {
            let generic_str = if type_params.is_empty() {
                String::new()
            } else {
                format!("<{}>", type_params.join(", "))
            };
            let mut lines = format_attributes(attributes, indent_level);
            lines.push(format!("{}trait {}{} {{", pad, name, generic_str));
            for method in methods {
                let params = method
                    .params
                    .iter()
                    .map(|param| {
                        let mut_prefix = if param.mutable { "mut " } else { "" };
                        let type_suffix = param
                            .type_annotation
                            .as_ref()
                            .map(|ty| format!(": {}", format_type(ty)))
                            .unwrap_or_default();
                        format!("{}{}{}", mut_prefix, param.name, type_suffix)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let return_type = method
                    .return_type
                    .as_ref()
                    .map(|ty| format!(" -> {}", format_type(ty)))
                    .unwrap_or_default();
                lines.push(format!(
                    "{}fn {}({}){};",
                    indent(indent_level + 1),
                    method.name,
                    params,
                    return_type
                ));
            }
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        }
        StmtKind::Error { args, message } => {
            format!(
                "{}error({}) <- {};",
                pad,
                format_call_args(args),
                format_expr(message)
            )
        }
        StmtKind::Fail { message } => {
            format!("{}fail <- {};", pad, format_expr(message))
        }
    }
}

fn format_expr(expr: &Expr) -> String {
    format_expr_with_precedence(expr, 0)
}

fn format_expr_with_precedence(expr: &Expr, parent_precedence: u8) -> String {
    let precedence = expr_precedence(expr);
    let rendered = match &expr.kind {
        ExprKind::String(value) => format!("\"{}\"", escape_string(value)),
        ExprKind::InterpolatedString(parts) => {
            let mut formatted_parts = Vec::new();
            for part in parts {
                match &part.kind {
                    ExprKind::String(val) => formatted_parts.push(val.clone()),
                    _ => formatted_parts.push(format!("${{{}}}", format_expr(part))),
                }
            }
            format!("\"{}\"", formatted_parts.join(""))
        }
        ExprKind::Number(value) => format_number_str(value),
        ExprKind::Bool(value) => value.to_string(),
        ExprKind::Array(elements) => format!(
            "[{}]",
            elements
                .iter()
                .map(format_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExprKind::Map(elements) => {
            let elements_str = elements
                .iter()
                .map(|(k, v)| format!("{}: {}", format_expr(k), format_expr(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", elements_str)
        }
        ExprKind::Identifier(name) => name.clone(),
        ExprKind::EnumVariant {
            enum_name,
            variant_name,
        } => format!("{}.{}", enum_name, variant_name),
        ExprKind::Unary { operator, right } => format!(
            "{}{}",
            format_unary_operator(operator),
            format_expr_with_precedence(right, precedence)
        ),
        ExprKind::Binary {
            left,
            operator,
            right,
        } => format!(
            "{} {} {}",
            format_expr_with_precedence(left, precedence),
            format_binary_operator(operator),
            format_expr_with_precedence(right, precedence + 1)
        ),
        ExprKind::Logical {
            left,
            operator,
            right,
        } => format!(
            "{} {} {}",
            format_expr_with_precedence(left, precedence),
            format_logical_operator(operator),
            format_expr_with_precedence(right, precedence + 1)
        ),
        ExprKind::Index { entity, index } => format!(
            "{}[{}]",
            format_expr_with_precedence(entity, precedence),
            format_expr(index)
        ),
        ExprKind::Call {
            name,
            type_args,
            args,
        } => {
            if type_args.is_empty() {
                format!("{}({})", name, format_call_args(args))
            } else {
                let formatted_types = type_args
                    .iter()
                    .map(format_type)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "{}::<{}>({})",
                    name,
                    formatted_types,
                    format_call_args(args)
                )
            }
        }
        ExprKind::MethodCall {
            entity,
            method,
            type_args,
            args,
        } => {
            let type_args_str = if type_args.is_empty() {
                String::new()
            } else {
                format!(
                    "::<{}>",
                    type_args
                        .iter()
                        .map(format_type)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            format!(
                "{}.{}{}({})",
                format_expr_with_precedence(entity, precedence),
                method,
                type_args_str,
                format_call_args(args)
            )
        }
        ExprKind::Get { entity, name } => {
            format!(
                "{}.{}",
                format_expr_with_precedence(entity, precedence),
                name
            )
        }
        ExprKind::PropertyCheck { entity, name } => format!(
            "{} : {}",
            format_expr_with_precedence(entity, precedence),
            name
        ),
        ExprKind::Match { condition, arms } => {
            let cond = format_expr(condition);
            let arm_strs = arms
                .iter()
                .map(|arm| {
                    let pats = arm
                        .patterns
                        .iter()
                        .map(format_expr)
                        .collect::<Vec<_>>()
                        .join(" | ");
                    let body_str = match &arm.body.kind {
                        StmtKind::Expression(expr) => format_expr(expr),
                        StmtKind::Block(stmts) => {
                            let mut lines = vec!["{".to_string()];
                            lines.extend(format_block_body(stmts, 1));
                            lines.push("}".to_string());
                            lines.join("\n")
                        }
                        _ => format_statement(&arm.body, 1),
                    };
                    format!("{} => {}", pats, body_str)
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("match {} {{ {} }}", cond, arm_strs)
        }
        ExprKind::Lambda {
            params,
            return_type,
            body,
        } => {
            let param_strs = params
                .iter()
                .map(|p| {
                    let mut p_str = if p.mutable {
                        format!("mut {}", p.name)
                    } else {
                        p.name.clone()
                    };
                    if let Some(ty) = &p.type_annotation {
                        p_str = format!("{}: {}", p_str, format_type(ty));
                    }
                    p_str
                })
                .collect::<Vec<_>>()
                .join(", ");
            let body_str = format_expr(body);
            if let Some(ret) = return_type {
                format!("|{}| -> {} => {}", param_strs, format_type(ret), body_str)
            } else {
                format!("|{}| => {}", param_strs, body_str)
            }
        }
        ExprKind::Range { start, end } => {
            format!("{}..{}", format_expr(start), format_expr(end))
        }
        ExprKind::Try { expr } => {
            format!("{}?", format_expr(expr))
        }
        ExprKind::Unwrap { expr } => {
            format!("{}!", format_expr(expr))
        }
    };

    if precedence < parent_precedence {
        format!("({})", rendered)
    } else {
        rendered
    }
}

fn expr_precedence(expr: &Expr) -> u8 {
    match &expr.kind {
        ExprKind::PropertyCheck { .. } => 1,
        ExprKind::Logical {
            operator: LogicalOperator::Or,
            ..
        } => 2,
        ExprKind::Logical {
            operator: LogicalOperator::And,
            ..
        } => 3,
        ExprKind::Binary {
            operator:
                BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::Greater
                | BinaryOperator::GreaterEqual
                | BinaryOperator::Less
                | BinaryOperator::LessEqual,
            ..
        } => 4,
        ExprKind::Binary {
            operator: BinaryOperator::Add | BinaryOperator::Subtract,
            ..
        } => 5,
        ExprKind::Binary {
            operator: BinaryOperator::Multiply | BinaryOperator::Divide,
            ..
        } => 6,
        ExprKind::Unary { .. } => 7,
        ExprKind::Index { .. }
        | ExprKind::Call { .. }
        | ExprKind::MethodCall { .. }
        | ExprKind::Get { .. }
        | ExprKind::Try { .. }
        | ExprKind::Unwrap { .. } => 8,
        ExprKind::Lambda { .. } => 0,
        _ => 8,
    }
}

fn format_call_args(args: &[CallArg]) -> String {
    args.iter()
        .map(|arg| match &arg.name {
            Some(name) => format!("{} => {}", name, format_expr(&arg.value)),
            None => format_expr(&arg.value),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_binary_operator(operator: &BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Modulo => "%",
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::Greater => ">",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::Less => "<",
        BinaryOperator::LessEqual => "<=",
    }
}

fn format_logical_operator(operator: &LogicalOperator) -> &'static str {
    match operator {
        LogicalOperator::And => "&&",
        LogicalOperator::Or => "||",
    }
}

fn format_unary_operator(operator: &UnaryOperator) -> &'static str {
    match operator {
        UnaryOperator::Negate => "-",
        UnaryOperator::Not => "!",
        UnaryOperator::Reference => "&",
        UnaryOperator::Dereference => "*",
    }
}

fn format_type(ty: &Type) -> String {
    ty.to_string()
}

fn format_number_str(value: &str) -> String {
    // Preserve the original numeric literal formatting.
    // Do not normalize `5.0` to `5` — keep source representation.
    value.to_string()
}

fn escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn indent(level: usize) -> String {
    "    ".repeat(level)
}
