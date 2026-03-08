use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeBundle {
    pub entry_component: String,
    pub components: Vec<NativeComponent>,
    #[serde(default)]
    #[allow(dead_code)]
    pub tokens: BTreeMap<String, FormoValue>,
    #[serde(default)]
    pub diagnostics: Vec<NativeDiagnostic>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeComponent {
    pub name: String,
    pub root_node: NativeNode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeNode {
    pub id: String,
    pub widget: String,
    #[serde(default)]
    pub props: BTreeMap<String, FormoValue>,
    #[serde(default)]
    pub resolved_style: BTreeMap<String, FormoValue>,
    #[serde(default)]
    pub children: Vec<NativeNode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeDiagnostic {
    pub code: String,
    pub level: String,
    pub message: String,
    pub source: NativeSourceLoc,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeSourceLoc {
    pub file: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormoValue {
    #[allow(dead_code)]
    pub t: String,
    pub v: JsonValue,
}
