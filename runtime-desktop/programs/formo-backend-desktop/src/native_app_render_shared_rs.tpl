use crate::model::NativeNode;
use crate::style::{style_attr, widget_class};

pub(super) fn node_class(node: &NativeNode, extra: &str) -> String {
    let base = widget_class(&node.widget);
    if extra.trim().is_empty() {
        format!("formo-node {base}")
    } else {
        format!("formo-node {base} {extra}")
    }
}

pub(super) fn node_style(node: &NativeNode) -> String {
    style_attr(node)
}

pub(super) fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(super) fn escape_attr(raw: &str) -> String {
    escape_html(raw).replace('"', "&quot;")
}
