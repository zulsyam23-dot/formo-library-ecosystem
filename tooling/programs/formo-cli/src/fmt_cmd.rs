use crate::args::FmtArgs;
use crate::error::CliError;
use formo_parser::{parse, AstAttr, AstComponent, AstNode, AstParam, AstProgram, AstValue};
use std::fs;
use std::path::Path;

pub fn run_fmt(args: &FmtArgs) -> Result<(), CliError> {
    let path = Path::new(&args.input);
    let source = fs::read_to_string(path)
        .map_err(|e| CliError::new(format!("cannot read {}: {e}", args.input)))?;
    let ast = parse(&source).map_err(CliError::new)?;
    let formatted = format_program(&ast);

    if args.stdout {
        print!("{formatted}");
        return if args.check && normalize_newlines(&source) != formatted {
            Err(CliError::printed(format!(
                "fmt check failed: {}",
                args.input
            )))
        } else {
            Ok(())
        };
    }

    if args.check {
        if normalize_newlines(&source) == formatted {
            println!("fmt check ok: {}", args.input);
            Ok(())
        } else {
            println!("fmt check failed: {}", args.input);
            Err(CliError::printed("formatting differs"))
        }
    } else {
        if normalize_newlines(&source) != formatted {
            fs::write(path, formatted.as_bytes())
                .map_err(|e| CliError::new(format!("cannot write {}: {e}", args.input)))?;
            println!("fmt wrote: {}", args.input);
        } else {
            println!("fmt unchanged: {}", args.input);
        }
        Ok(())
    }
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n")
}

fn format_program(program: &AstProgram) -> String {
    let mut out = String::new();

    for import in &program.imports {
        out.push_str("import \"");
        out.push_str(&escape_string(&import.path));
        out.push('"');
        if let Some(alias) = &import.alias {
            out.push_str(" as ");
            out.push_str(alias);
        }
        out.push_str(";\n");
    }

    if !program.imports.is_empty() && !program.components.is_empty() {
        out.push('\n');
    }

    for (idx, component) in program.components.iter().enumerate() {
        format_component(&mut out, component);
        if idx + 1 < program.components.len() {
            out.push('\n');
        }
    }

    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn format_component(out: &mut String, component: &AstComponent) {
    out.push_str("component ");
    out.push_str(&component.name);
    out.push('(');
    format_params(out, &component.params);
    out.push_str(") {\n");

    for node in &component.nodes {
        format_node(out, node, 1);
    }

    out.push_str("}\n");
}

fn format_params(out: &mut String, params: &[AstParam]) {
    for (idx, param) in params.iter().enumerate() {
        out.push_str(&param.name);
        if param.optional {
            out.push('?');
        }
        if let Some(ty) = &param.ty {
            out.push_str(": ");
            out.push_str(ty);
        }
        if idx + 1 < params.len() {
            out.push_str(", ");
        }
    }
}

fn format_node(out: &mut String, node: &AstNode, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    out.push_str(&indent);
    out.push('<');
    out.push_str(&node.name);

    if !node.attributes.is_empty() {
        out.push(' ');
        format_attributes(out, &node.attributes);
    }

    if node.children.is_empty() {
        out.push_str("/>\n");
        return;
    }

    out.push_str(">\n");
    for child in &node.children {
        format_node(out, child, indent_level + 1);
    }
    out.push_str(&indent);
    out.push_str("</");
    out.push_str(&node.name);
    out.push_str(">\n");
}

fn format_attributes(out: &mut String, attrs: &[AstAttr]) {
    for (idx, attr) in attrs.iter().enumerate() {
        out.push_str(&attr.name);
        out.push('=');
        format_value(out, &attr.value);
        if idx + 1 < attrs.len() {
            out.push(' ');
        }
    }
}

fn format_value(out: &mut String, value: &AstValue) {
    match value {
        AstValue::String(s) => {
            out.push('"');
            out.push_str(&escape_string(s));
            out.push('"');
        }
        AstValue::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
        AstValue::Int(v) => out.push_str(&v.to_string()),
        AstValue::Float(v) => out.push_str(&format_float(*v)),
        AstValue::Identifier(v) => out.push_str(v),
        AstValue::List(items) => {
            out.push('[');
            for (idx, item) in items.iter().enumerate() {
                format_value(out, item);
                if idx + 1 < items.len() {
                    out.push_str(", ");
                }
            }
            out.push(']');
        }
        AstValue::Object(entries) => {
            out.push('{');
            for (idx, (key, val)) in entries.iter().enumerate() {
                out.push_str(key);
                out.push_str(": ");
                format_value(out, val);
                if idx + 1 < entries.len() {
                    out.push_str(", ");
                }
            }
            out.push('}');
        }
    }
}

fn format_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn escape_string(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
