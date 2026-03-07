use formo_ir::{IrProgram, IR_SCHEMA_ID, IR_VERSION};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn schema_id_and_ir_version_are_locked() {
    let schema_path = workspace_root().join("formo-ir.schema.json");
    let raw = fs::read_to_string(&schema_path).expect("schema file should be readable");
    let schema: Value =
        serde_json::from_str(trim_utf8_bom(&raw)).expect("schema should be valid json");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(IR_SCHEMA_ID),
        "schema $id must match formo-ir crate constant"
    );

    assert_eq!(
        schema
            .get("properties")
            .and_then(|obj| obj.get("irVersion"))
            .and_then(|obj| obj.get("const"))
            .and_then(Value::as_str),
        Some(IR_VERSION),
        "schema irVersion const must match formo-ir crate constant"
    );
}

#[test]
fn golden_ir_files_follow_contract_and_graph_integrity() {
    let fixture_dir = workspace_root().join("fixtures").join("ir");
    let entries = fs::read_dir(&fixture_dir).expect("fixture dir should exist");

    let mut seen = 0usize;
    for entry in entries {
        let entry = entry.expect("fixture entry should be readable");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let ir = load_ir_program(&path);
        assert_eq!(
            ir.ir_version,
            IR_VERSION,
            "golden file {} must keep locked irVersion",
            path.display()
        );
        assert!(
            !ir.components.is_empty(),
            "golden file {} must have at least one component",
            path.display()
        );
        assert!(
            ir.components.iter().any(|comp| comp.name == ir.entry),
            "golden file {} must contain entry component {}",
            path.display(),
            ir.entry
        );

        let node_ids = ir
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<HashSet<_>>();
        assert!(
            !node_ids.is_empty(),
            "golden file {} must have at least one node",
            path.display()
        );

        for component in &ir.components {
            assert!(
                node_ids.contains(component.root_node_id.as_str()),
                "component {} root node {} must exist in {}",
                component.name,
                component.root_node_id,
                path.display()
            );
        }

        for node in &ir.nodes {
            for child_id in &node.children {
                assert!(
                    node_ids.contains(child_id.as_str()),
                    "node {} references missing child {} in {}",
                    node.id,
                    child_id,
                    path.display()
                );
            }
        }

        seen += 1;
    }

    assert!(
        seen >= 2,
        "expected at least 2 golden IR fixtures, found {seen}"
    );
}

fn load_ir_program(path: &Path) -> IrProgram {
    let raw = fs::read_to_string(path).expect("fixture should be readable");
    serde_json::from_str(trim_utf8_bom(&raw)).unwrap_or_else(|err| {
        panic!(
            "fixture {} should deserialize as IrProgram: {err}",
            path.display()
        )
    })
}

fn trim_utf8_bom(raw: &str) -> &str {
    raw.strip_prefix('\u{feff}').unwrap_or(raw)
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root should exist")
        .to_path_buf()
}
