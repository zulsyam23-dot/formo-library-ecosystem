pub(crate) fn starts_uppercase(value: &str) -> bool {
    value
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
}

pub(crate) fn is_lower_camel_case(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric())
}

pub(crate) fn is_member_path(value: &str) -> bool {
    value
        .split('.')
        .all(|segment| !segment.is_empty() && is_lower_camel_case(segment))
}

pub(crate) fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

pub(crate) fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
