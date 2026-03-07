#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCodeEntry {
    pub code: &'static str,
    pub category: &'static str,
    pub summary: &'static str,
}

pub const COMPONENT_EMPTY_ROOT: &str = "E2001";
pub const COMPONENT_MULTI_ROOT: &str = "E2002";

pub const NODE_NAME_UPPERCASE: &str = "E2101";
pub const NODE_UNKNOWN: &str = "E2102";
pub const ATTR_DUPLICATE: &str = "E2103";

pub const STYLE_SLOT_ATTR_FORBIDDEN: &str = "E2221";
pub const STYLE_EMPTY: &str = "E2222";
pub const STYLE_INVALID_TYPE: &str = "E2223";

pub const BUILTIN_UNKNOWN_PROP: &str = "E2250";
pub const BUILTIN_INVALID_PROP_TYPE: &str = "E2251";
pub const BUILTIN_CHILDREN_FORBIDDEN: &str = "E2252";
pub const BUILTIN_REQUIRED_PROP_MISSING: &str = "E2253";

pub const CUSTOM_REQUIRED_PROP_MISSING: &str = "E2301";
pub const CUSTOM_UNKNOWN_PROP: &str = "E2302";
pub const CUSTOM_PROP_TYPE_MISMATCH: &str = "E2303";
pub const CUSTOM_INLINE_CHILDREN_FORBIDDEN: &str = "E2304";

pub const COMPONENT_DUPLICATE_PARAM: &str = "E2401";

pub const REGISTRY: &[ErrorCodeEntry] = &[
    ErrorCodeEntry {
        code: COMPONENT_EMPTY_ROOT,
        category: "component",
        summary: "Component must have at least one root node",
    },
    ErrorCodeEntry {
        code: COMPONENT_MULTI_ROOT,
        category: "component",
        summary: "Component must have exactly one root node",
    },
    ErrorCodeEntry {
        code: NODE_NAME_UPPERCASE,
        category: "node",
        summary: "Node name must start with uppercase letter",
    },
    ErrorCodeEntry {
        code: NODE_UNKNOWN,
        category: "node",
        summary: "Node is unknown (not built-in and not registered component)",
    },
    ErrorCodeEntry {
        code: ATTR_DUPLICATE,
        category: "attribute",
        summary: "Duplicate attribute on node",
    },
    ErrorCodeEntry {
        code: STYLE_SLOT_ATTR_FORBIDDEN,
        category: "style",
        summary: "Slot node does not accept style attribute",
    },
    ErrorCodeEntry {
        code: STYLE_EMPTY,
        category: "style",
        summary: "Style attribute cannot be empty",
    },
    ErrorCodeEntry {
        code: STYLE_INVALID_TYPE,
        category: "style",
        summary: "Style attribute type is invalid",
    },
    ErrorCodeEntry {
        code: BUILTIN_UNKNOWN_PROP,
        category: "builtin",
        summary: "Unknown prop on built-in component",
    },
    ErrorCodeEntry {
        code: BUILTIN_INVALID_PROP_TYPE,
        category: "builtin",
        summary: "Built-in prop type mismatch",
    },
    ErrorCodeEntry {
        code: BUILTIN_CHILDREN_FORBIDDEN,
        category: "builtin",
        summary: "Built-in node does not allow children",
    },
    ErrorCodeEntry {
        code: BUILTIN_REQUIRED_PROP_MISSING,
        category: "builtin",
        summary: "Required built-in prop is missing",
    },
    ErrorCodeEntry {
        code: CUSTOM_REQUIRED_PROP_MISSING,
        category: "custom",
        summary: "Required custom component prop is missing",
    },
    ErrorCodeEntry {
        code: CUSTOM_UNKNOWN_PROP,
        category: "custom",
        summary: "Unknown prop on custom component call",
    },
    ErrorCodeEntry {
        code: CUSTOM_PROP_TYPE_MISMATCH,
        category: "custom",
        summary: "Custom component prop type mismatch",
    },
    ErrorCodeEntry {
        code: CUSTOM_INLINE_CHILDREN_FORBIDDEN,
        category: "custom",
        summary: "Inline children forbidden when component has no Slot",
    },
    ErrorCodeEntry {
        code: COMPONENT_DUPLICATE_PARAM,
        category: "component",
        summary: "Duplicate parameter in component signature",
    },
];

pub fn find(code: &str) -> Option<&'static ErrorCodeEntry> {
    REGISTRY.iter().find(|entry| entry.code == code)
}

pub fn is_registered(code: &str) -> bool {
    find(code).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_codes_are_unique() {
        let mut seen = HashSet::new();
        for entry in REGISTRY {
            assert!(
                seen.insert(entry.code),
                "duplicate error code in registry: {}",
                entry.code
            );
        }
    }

    #[test]
    fn all_public_codes_are_registered() {
        let expected = [
            COMPONENT_EMPTY_ROOT,
            COMPONENT_MULTI_ROOT,
            NODE_NAME_UPPERCASE,
            NODE_UNKNOWN,
            ATTR_DUPLICATE,
            STYLE_SLOT_ATTR_FORBIDDEN,
            STYLE_EMPTY,
            STYLE_INVALID_TYPE,
            BUILTIN_UNKNOWN_PROP,
            BUILTIN_INVALID_PROP_TYPE,
            BUILTIN_CHILDREN_FORBIDDEN,
            BUILTIN_REQUIRED_PROP_MISSING,
            CUSTOM_REQUIRED_PROP_MISSING,
            CUSTOM_UNKNOWN_PROP,
            CUSTOM_PROP_TYPE_MISMATCH,
            CUSTOM_INLINE_CHILDREN_FORBIDDEN,
            COMPONENT_DUPLICATE_PARAM,
        ];

        for code in expected {
            assert!(
                is_registered(code),
                "code {code} should be present in registry"
            );
        }
    }
}
