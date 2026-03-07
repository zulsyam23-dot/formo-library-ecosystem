use crate::lowering::lower_to_ir;
use formo_ir::IrProgram;
use formo_parser::parse;
use formo_resolver::resolve;
use formo_style::compile_styles;
use formo_typer::type_check;
use std::fs;

pub fn pipeline(input: &str) -> Result<IrProgram, String> {
    let source = fs::read_to_string(input).map_err(|e| format!("cannot read {input}: {e}"))?;
    let ast = parse(&source).map_err(|raw| format_parser_diag("E1100", input, &raw))?;
    let resolved = resolve(ast, input)?;
    let typed = type_check(resolved)?;
    let styled = compile_styles(typed)?;
    lower_to_ir(&styled, input)
}

fn format_parser_diag(code: &str, file: &str, raw: &str) -> String {
    if let Some((message, line, col)) = parse_parser_line_col(raw) {
        return format!("{code} {file}:{line}:{col} {message}");
    }
    format!("{code} {file}:1:1 {raw}")
}

fn parse_parser_line_col(raw: &str) -> Option<(&str, usize, usize)> {
    let marker = " at ";
    let index = raw.rfind(marker)?;
    let message = raw[..index].trim();
    let location = raw[index + marker.len()..].trim();
    let (line_raw, col_raw) = location.split_once(':')?;
    let line = line_raw.parse::<usize>().ok()?;
    let col = col_raw.parse::<usize>().ok()?;
    Some((message, line, col))
}
