use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PropKind {
    String,
    Bool,
    BoolOrStateBool,
    Int,
    Float,
    Len,
    Color,
    StringOrIdent,
    ListSource,
    StateString,
    StateBool,
    ActionVoid,
    ActionString,
    ActionBool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LocalValueKind {
    Unknown,
    String,
    Bool,
    Int,
    Float,
    Len,
    Color,
}

pub(crate) type LocalScope = HashMap<String, LocalValueKind>;

#[derive(Clone, Copy)]
pub(crate) struct PropRule {
    pub(crate) name: &'static str,
    pub(crate) kind: PropKind,
    pub(crate) required: bool,
}

const WINDOW_RULES: [PropRule; 6] = [
    PropRule {
        name: "title",
        kind: PropKind::String,
        required: true,
    },
    PropRule {
        name: "width",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "height",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "minWidth",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "minHeight",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "resizable",
        kind: PropKind::Bool,
        required: false,
    },
];

const PAGE_RULES: [PropRule; 3] = [
    PropRule {
        name: "id",
        kind: PropKind::String,
        required: false,
    },
    PropRule {
        name: "padding",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "scroll",
        kind: PropKind::StringOrIdent,
        required: false,
    },
];

const ROW_RULES: [PropRule; 4] = [
    PropRule {
        name: "gap",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "align",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "justify",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "wrap",
        kind: PropKind::Bool,
        required: false,
    },
];

const COLUMN_RULES: [PropRule; 3] = [
    PropRule {
        name: "gap",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "align",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "justify",
        kind: PropKind::StringOrIdent,
        required: false,
    },
];

const STACK_RULES: [PropRule; 1] = [PropRule {
    name: "align",
    kind: PropKind::StringOrIdent,
    required: false,
}];

const CARD_RULES: [PropRule; 3] = [
    PropRule {
        name: "variant",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "padding",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "radius",
        kind: PropKind::Len,
        required: false,
    },
];

const TEXT_RULES: [PropRule; 6] = [
    PropRule {
        name: "value",
        kind: PropKind::String,
        required: true,
    },
    PropRule {
        name: "variant",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "color",
        kind: PropKind::Color,
        required: false,
    },
    PropRule {
        name: "align",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "maxLines",
        kind: PropKind::Int,
        required: false,
    },
    PropRule {
        name: "ellipsis",
        kind: PropKind::Bool,
        required: false,
    },
];

const IMAGE_RULES: [PropRule; 5] = [
    PropRule {
        name: "src",
        kind: PropKind::StringOrIdent,
        required: true,
    },
    PropRule {
        name: "alt",
        kind: PropKind::String,
        required: false,
    },
    PropRule {
        name: "fit",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "width",
        kind: PropKind::Len,
        required: false,
    },
    PropRule {
        name: "height",
        kind: PropKind::Len,
        required: false,
    },
];

const BUTTON_RULES: [PropRule; 5] = [
    PropRule {
        name: "label",
        kind: PropKind::String,
        required: true,
    },
    PropRule {
        name: "onPress",
        kind: PropKind::ActionVoid,
        required: true,
    },
    PropRule {
        name: "variant",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "disabled",
        kind: PropKind::Bool,
        required: false,
    },
    PropRule {
        name: "leadingIcon",
        kind: PropKind::StringOrIdent,
        required: false,
    },
];

const INPUT_RULES: [PropRule; 5] = [
    PropRule {
        name: "value",
        kind: PropKind::StateString,
        required: true,
    },
    PropRule {
        name: "onChange",
        kind: PropKind::ActionString,
        required: true,
    },
    PropRule {
        name: "placeholder",
        kind: PropKind::String,
        required: false,
    },
    PropRule {
        name: "inputType",
        kind: PropKind::StringOrIdent,
        required: false,
    },
    PropRule {
        name: "disabled",
        kind: PropKind::Bool,
        required: false,
    },
];

const CHECKBOX_RULES: [PropRule; 4] = [
    PropRule {
        name: "checked",
        kind: PropKind::StateBool,
        required: true,
    },
    PropRule {
        name: "onChange",
        kind: PropKind::ActionBool,
        required: true,
    },
    PropRule {
        name: "label",
        kind: PropKind::String,
        required: false,
    },
    PropRule {
        name: "disabled",
        kind: PropKind::Bool,
        required: false,
    },
];

const SWITCH_RULES: [PropRule; 3] = [
    PropRule {
        name: "checked",
        kind: PropKind::StateBool,
        required: true,
    },
    PropRule {
        name: "onChange",
        kind: PropKind::ActionBool,
        required: true,
    },
    PropRule {
        name: "disabled",
        kind: PropKind::Bool,
        required: false,
    },
];

const SCROLL_RULES: [PropRule; 1] = [PropRule {
    name: "axis",
    kind: PropKind::StringOrIdent,
    required: false,
}];

const SPACER_RULES: [PropRule; 1] = [PropRule {
    name: "size",
    kind: PropKind::Len,
    required: true,
}];

const MODAL_RULES: [PropRule; 2] = [
    PropRule {
        name: "open",
        kind: PropKind::StateBool,
        required: true,
    },
    PropRule {
        name: "onClose",
        kind: PropKind::ActionVoid,
        required: true,
    },
];

const IF_RULES: [PropRule; 1] = [PropRule {
    name: "when",
    kind: PropKind::BoolOrStateBool,
    required: true,
}];

const FOR_RULES: [PropRule; 2] = [
    PropRule {
        name: "each",
        kind: PropKind::ListSource,
        required: true,
    },
    PropRule {
        name: "as",
        kind: PropKind::StringOrIdent,
        required: true,
    },
];

pub(crate) fn builtin_prop_rules(name: &str) -> &'static [PropRule] {
    match name {
        "Window" => &WINDOW_RULES,
        "Page" => &PAGE_RULES,
        "Row" => &ROW_RULES,
        "Column" => &COLUMN_RULES,
        "Stack" => &STACK_RULES,
        "Card" => &CARD_RULES,
        "Text" => &TEXT_RULES,
        "Image" => &IMAGE_RULES,
        "Button" => &BUTTON_RULES,
        "Input" => &INPUT_RULES,
        "Checkbox" => &CHECKBOX_RULES,
        "Switch" => &SWITCH_RULES,
        "Scroll" => &SCROLL_RULES,
        "Spacer" => &SPACER_RULES,
        "Modal" => &MODAL_RULES,
        "If" => &IF_RULES,
        "For" => &FOR_RULES,
        "Slot" => &[],
        _ => &[],
    }
}

pub(crate) fn builtin_allows_children(name: &str) -> bool {
    matches!(
        name,
        "Window" | "Page" | "Row" | "Column" | "Stack" | "Card" | "Scroll" | "Modal" | "If" | "For"
    )
}

pub(crate) fn prop_kind_name(kind: PropKind) -> &'static str {
    match kind {
        PropKind::String => "string",
        PropKind::Bool => "bool",
        PropKind::BoolOrStateBool => "bool or state<bool>",
        PropKind::Int => "int",
        PropKind::Float => "float",
        PropKind::Len => "len",
        PropKind::Color => "color",
        PropKind::StringOrIdent => "string or identifier",
        PropKind::ListSource => "list source identifier or list expression",
        PropKind::StateString => "state<string>",
        PropKind::StateBool => "state<bool>",
        PropKind::ActionVoid => "action<void>",
        PropKind::ActionString => "action<string>",
        PropKind::ActionBool => "action<bool>",
    }
}

pub(crate) fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "Window"
            | "Page"
            | "Row"
            | "Column"
            | "Stack"
            | "Card"
            | "Text"
            | "Image"
            | "Button"
            | "Input"
            | "Checkbox"
            | "Switch"
            | "Scroll"
            | "Spacer"
            | "Modal"
            | "If"
            | "For"
            | "Slot"
    )
}
