pub(crate) fn format_style_diag(
    code: &str,
    file: &str,
    line: usize,
    col: usize,
    message: &str,
) -> String {
    format!("{code} {file}:{line}:{col} {message}")
}
