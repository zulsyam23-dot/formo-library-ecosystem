use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const IR_VERSION: &str = "0.3.0";
pub const IR_SCHEMA_ID: &str = "https://formo.dev/schema/ir/0.3.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Web,
    Desktop,
    Multi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SourceLoc {
    pub file: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Value {
    pub t: String,
    pub v: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IrNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub style_refs: Vec<String>,
    pub children: Vec<String>,
    pub source: SourceLoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IrComponent {
    pub id: String,
    pub name: String,
    pub root_node_id: String,
    pub exports: bool,
    pub source: SourceLoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StyleSelector {
    pub component: String,
    pub part: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IrStyle {
    pub id: String,
    pub selector: StyleSelector,
    pub decls: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub level: String,
    pub message: String,
    pub source: SourceLoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IrProgram {
    pub ir_version: String,
    pub entry: String,
    pub target: Target,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    pub components: Vec<IrComponent>,
    pub nodes: Vec<IrNode>,
    pub styles: Vec<IrStyle>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct OutputFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct BackendOutput {
    pub files: Vec<OutputFile>,
}

pub trait Backend {
    fn emit(&self, ir: &IrProgram) -> Result<BackendOutput, String>;
}
