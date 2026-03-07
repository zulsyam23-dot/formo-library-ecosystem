pub(crate) fn render_index_html(entry: &str, state_json: &str, dev_bootstrap: &str) -> String {
    let safe_json = state_json.replace("</script>", "<\\/script>");

    format!(
        "<!doctype html>\n<html>\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n  <title>{}</title>\n  <link rel=\"stylesheet\" href=\"app.css\">\n</head>\n<body>\n  <div id=\"app\"></div>\n  <script id=\"formo-ir\" type=\"application/json\">{}</script>\n  <script>{}</script>\n  <script src=\"app.js\"></script>\n</body>\n</html>\n",
        entry, safe_json, dev_bootstrap
    )
}
