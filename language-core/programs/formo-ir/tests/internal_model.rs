use formo_ir::internal::InternalProgram;
use formo_ir::{
    IR_VERSION, IrComponent, IrNode, IrProgram, IrStyle, SourceLoc, StyleSelector, Target, Value,
};
use std::collections::BTreeMap;

#[test]
fn internal_roundtrip_preserves_public_ir_shape() {
    let input = sample_ir();
    let internal = InternalProgram::try_from(&input).expect("internal conversion should succeed");
    let output = internal.into_public();

    assert_eq!(output.ir_version, input.ir_version);
    assert_eq!(output.entry, input.entry);
    assert_eq!(output.components.len(), input.components.len());
    assert_eq!(output.nodes.len(), input.nodes.len());
    assert_eq!(output.styles.len(), input.styles.len());
}

#[test]
fn internal_conversion_rejects_duplicate_node_ids() {
    let mut input = sample_ir();
    input.nodes.push(input.nodes[0].clone());

    let err = InternalProgram::try_from(&input).expect_err("duplicate node id must fail");
    assert!(err.contains("duplicate node id `n_root`"));
}

fn sample_ir() -> IrProgram {
    IrProgram {
        ir_version: IR_VERSION.to_string(),
        entry: "App".to_string(),
        target: Target::Web,
        tokens: BTreeMap::new(),
        components: vec![IrComponent {
            id: "c_app".to_string(),
            name: "App".to_string(),
            root_node_id: "n_root".to_string(),
            exports: true,
            source: loc(),
        }],
        nodes: vec![
            IrNode {
                id: "n_root".to_string(),
                kind: "element".to_string(),
                name: "Page".to_string(),
                props: BTreeMap::new(),
                style_refs: vec![],
                children: vec!["n_text".to_string()],
                source: loc(),
            },
            IrNode {
                id: "n_text".to_string(),
                kind: "element".to_string(),
                name: "Text".to_string(),
                props: BTreeMap::from([(
                    "value".to_string(),
                    Value {
                        t: "string".to_string(),
                        v: serde_json::Value::String("hello".to_string()),
                    },
                )]),
                style_refs: vec![],
                children: vec![],
                source: loc(),
            },
        ],
        styles: vec![IrStyle {
            id: "style_page".to_string(),
            selector: StyleSelector {
                component: "Page".to_string(),
                part: "root".to_string(),
            },
            decls: BTreeMap::new(),
        }],
        diagnostics: vec![],
    }
}

fn loc() -> SourceLoc {
    SourceLoc {
        file: "main.fm".to_string(),
        line: 1,
        col: 1,
    }
}
