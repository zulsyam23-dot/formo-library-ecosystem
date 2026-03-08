#![allow(dead_code)]

use std::collections::BTreeMap;

pub const FORMO_IR_JSON: &str = include_str!("app.ir.json");
pub const FORMO_NATIVE_JSON: &str = include_str!("app.native.json");
pub const FORMO_ENTRY_COMPONENT: &str = "{{ENTRY_COMPONENT}}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormoDesktopAction {
    pub name: String,
    pub node_id: String,
    pub node_name: String,
    pub payload_json: String,
    pub scope_json: String,
    pub state_json: String,
}

pub trait FormoDesktopHost {
    fn invoke_action(&mut self, action: FormoDesktopAction);
}

#[derive(Debug, Clone, Default)]
pub struct FormoDesktopState {
    values: BTreeMap<String, String>,
}

impl FormoDesktopState {
    pub fn set_state_patch(&mut self, patch: BTreeMap<String, String>) {
        for (key, value) in patch {
            self.values.insert(key, value);
        }
    }

    pub fn replace_state(&mut self, next: BTreeMap<String, String>) {
        self.values = next;
    }

    pub fn as_map(&self) -> &BTreeMap<String, String> {
        &self.values
    }
}
