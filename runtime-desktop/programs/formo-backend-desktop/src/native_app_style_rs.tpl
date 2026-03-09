use crate::model::{FormoValue, NativeNode};
use eframe::egui::Color32;

#[derive(Debug, Clone, Copy, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub fn same(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    pub fn is_zero(self) -> bool {
        self.top == 0.0 && self.right == 0.0 && self.bottom == 0.0 && self.left == 0.0
    }

    pub fn margin(self) -> eframe::egui::Margin {
        eframe::egui::Margin {
            left: self.left.max(0.0),
            right: self.right.max(0.0),
            top: self.top.max(0.0),
            bottom: self.bottom.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignMode {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyMode {
    Start,
    Center,
    End,
    Space,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderStyle {
    pub text_color: Option<Color32>,
    pub background: Option<Color32>,
    pub border_color: Option<Color32>,
    pub border_width: Option<f32>,
    pub border_radius: Option<f32>,
    pub shadow: Option<eframe::egui::epaint::Shadow>,
    pub opacity: Option<f32>,
    pub gap: Option<f32>,
    pub font_size: Option<f32>,
    pub font_weight: Option<f32>,
    pub italic: bool,
    pub line_height: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub padding: Edges,
    pub has_padding: bool,
    pub margin: Edges,
    pub has_margin: bool,
    pub flow: Option<Flow>,
    pub align: AlignMode,
    pub justify: JustifyMode,
    pub text_align: TextAlign,
    pub overflow: Overflow,
    pub wrap: bool,
    pub display_none: bool,
    pub display_flex: bool,
}

impl RenderStyle {
    pub fn from_node(node: &NativeNode) -> Self {
        let (mut padding, mut has_padding) = style_edges(node, "padding");
        for (k, slot) in [
            ("padding-top", &mut padding.top),
            ("padding-right", &mut padding.right),
            ("padding-bottom", &mut padding.bottom),
            ("padding-left", &mut padding.left),
        ] {
            if let Some(v) = style_len(node, &[k]) {
                *slot = v;
                has_padding = true;
            }
        }

        let (mut margin, mut has_margin) = style_edges(node, "margin");
        for (k, slot) in [
            ("margin-top", &mut margin.top),
            ("margin-right", &mut margin.right),
            ("margin-bottom", &mut margin.bottom),
            ("margin-left", &mut margin.left),
        ] {
            if let Some(v) = style_len(node, &[k]) {
                *slot = v;
                has_margin = true;
            }
        }

        let (border_w, border_c) = style_value(node, &["border"])
            .and_then(parse_border_shorthand)
            .unwrap_or((None, None));

        Self {
            text_color: style_color(node, &["color"]),
            background: style_color(node, &["background", "background-color", "backgroundColor"]),
            border_color: style_color(node, &["border-color", "borderColor"]).or(border_c),
            border_width: style_len(node, &["border-width", "borderWidth"]).or(border_w),
            border_radius: style_len(node, &["border-radius", "borderRadius", "radius"]),
            shadow: style_value(node, &["box-shadow", "boxShadow"]).and_then(parse_box_shadow),
            opacity: style_number(node, &["opacity"]),
            gap: style_len(node, &["gap"]),
            font_size: style_len(node, &["font-size", "fontSize", "size"]),
            font_weight: style_text(node, &["font-weight", "fontWeight"]).and_then(parse_font_weight),
            italic: style_text(node, &["font-style", "fontStyle"])
                .map(|v| {
                    let v = normalize(v);
                    v == "italic" || v == "oblique"
                })
                .unwrap_or(false),
            line_height: style_number(node, &["line-height", "lineHeight"]),
            width: style_len(node, &["width"]),
            height: style_len(node, &["height"]),
            min_width: style_len(node, &["min-width", "minWidth"]),
            min_height: style_len(node, &["min-height", "minHeight"]),
            max_width: style_len(node, &["max-width", "maxWidth"]),
            max_height: style_len(node, &["max-height", "maxHeight"]),
            padding,
            has_padding,
            margin,
            has_margin,
            flow: style_text(node, &["flex-direction", "flexDirection"]).and_then(parse_flow),
            align: style_text(node, &["align-items", "alignItems"])
                .and_then(parse_align)
                .unwrap_or(AlignMode::Stretch),
            justify: style_text(node, &["justify-content", "justifyContent"])
                .and_then(parse_justify)
                .unwrap_or(JustifyMode::Start),
            text_align: style_text(node, &["text-align", "textAlign"])
                .and_then(parse_text_align)
                .unwrap_or(TextAlign::Start),
            overflow: style_text(node, &["overflow"])
                .and_then(parse_overflow)
                .unwrap_or(Overflow::Visible),
            wrap: style_text(node, &["flex-wrap", "flexWrap"])
                .map(|v| normalize(v) == "wrap")
                .unwrap_or(false),
            display_none: style_text(node, &["display"])
                .map(|v| normalize(v) == "none")
                .unwrap_or(false),
            display_flex: style_text(node, &["display"])
                .map(|v| {
                    let v = normalize(v);
                    v == "flex" || v == "inline-flex"
                })
                .unwrap_or(false),
        }
    }
}

pub fn with_opacity(color: Color32, opacity: Option<f32>) -> Color32 {
    if let Some(v) = opacity {
        color.linear_multiply(v.clamp(0.0, 1.0))
    } else {
        color
    }
}

pub fn parse_hex_color(raw: &str) -> Option<Color32> {
    let hex = raw.trim().strip_prefix('#')?;
    match hex.len() {
        6 => Some(Color32::from_rgb(
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        )),
        8 => Some(Color32::from_rgba_unmultiplied(
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?,
        )),
        _ => None,
    }
}

fn style_value<'a>(node: &'a NativeNode, keys: &[&str]) -> Option<&'a FormoValue> {
    keys.iter()
        .find_map(|k| node.resolved_style.get(*k))
        .or_else(|| {
            keys.iter()
                .find_map(|k| node.resolved_style.get(&to_kebab(k)))
        })
}

fn style_text<'a>(node: &'a NativeNode, keys: &[&str]) -> Option<&'a str> {
    style_value(node, keys)
        .and_then(|v| v.v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

fn style_color(node: &NativeNode, keys: &[&str]) -> Option<Color32> {
    style_value(node, keys).and_then(parse_color_value)
}

fn style_number(node: &NativeNode, keys: &[&str]) -> Option<f32> {
    style_value(node, keys).and_then(parse_number_value)
}

fn style_len(node: &NativeNode, keys: &[&str]) -> Option<f32> {
    style_value(node, keys).and_then(parse_len_px)
}

fn style_edges(node: &NativeNode, key: &str) -> (Edges, bool) {
    if let Some(v) = style_value(node, &[key]).and_then(parse_edges) {
        return (v, true);
    }
    (Edges::default(), false)
}

fn parse_color_value(value: &FormoValue) -> Option<Color32> {
    value.v.as_str().and_then(parse_color_text)
}

fn parse_number_value(value: &FormoValue) -> Option<f32> {
    if let Some(i) = value.v.as_i64() {
        return Some(i as f32);
    }
    if let Some(f) = value.v.as_f64() {
        return Some(f as f32);
    }
    if let Some(obj) = value.v.as_object() {
        return obj.get("value").and_then(|v| v.as_f64()).map(|v| v as f32);
    }
    value.v.as_str().and_then(|raw| raw.trim().parse::<f32>().ok())
}

fn parse_len_px(value: &FormoValue) -> Option<f32> {
    if let Some(i) = value.v.as_i64() {
        return Some(i as f32);
    }
    if let Some(f) = value.v.as_f64() {
        return Some(f as f32);
    }
    if let Some(obj) = value.v.as_object() {
        let raw = obj.get("value")?.as_f64()? as f32;
        let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
        return match unit {
            "px" | "dp" => Some(raw),
            _ => None,
        };
    }
    value.v.as_str().and_then(parse_len_string_px)
}

fn parse_len_string_px(raw: &str) -> Option<f32> {
    let text = raw.trim();
    if let Some(px) = text.strip_suffix("px") {
        return px.trim().parse::<f32>().ok();
    }
    if let Some(dp) = text.strip_suffix("dp") {
        return dp.trim().parse::<f32>().ok();
    }
    text.parse::<f32>().ok()
}

fn parse_edges(value: &FormoValue) -> Option<Edges> {
    if let Some(v) = parse_len_px(value) {
        return Some(Edges::same(v));
    }
    let raw = value.v.as_str()?.trim();
    let list: Vec<f32> = raw
        .split_whitespace()
        .filter_map(parse_len_string_px)
        .collect();
    match list.as_slice() {
        [a] => Some(Edges::same(*a)),
        [v, h] => Some(Edges {
            top: *v,
            right: *h,
            bottom: *v,
            left: *h,
        }),
        [t, h, b] => Some(Edges {
            top: *t,
            right: *h,
            bottom: *b,
            left: *h,
        }),
        [t, r, b, l] => Some(Edges {
            top: *t,
            right: *r,
            bottom: *b,
            left: *l,
        }),
        _ => None,
    }
}

fn parse_border_shorthand(value: &FormoValue) -> Option<(Option<f32>, Option<Color32>)> {
    let raw = value.v.as_str()?.trim();
    let mut width = None;
    let mut color = None;
    for token in split_css_tokens(raw) {
        if width.is_none() {
            width = parse_len_string_px(&token);
            if width.is_some() {
                continue;
            }
        }
        if color.is_none() {
            color = parse_color_text(&token);
        }
    }
    Some((width, color))
}

fn parse_box_shadow(value: &FormoValue) -> Option<eframe::egui::epaint::Shadow> {
    let raw = value.v.as_str()?.trim();
    let first = first_css_item(raw);
    let mut lengths = Vec::new();
    let mut color = None;
    for token in split_css_tokens(first) {
        if color.is_none() {
            color = parse_color_text(&token);
            if color.is_some() {
                continue;
            }
        }
        if let Some(v) = parse_len_string_px(&token) {
            lengths.push(v);
        }
    }
    if lengths.len() < 2 {
        return None;
    }
    Some(eframe::egui::epaint::Shadow {
        offset: eframe::egui::vec2(lengths[0], lengths[1]),
        blur: lengths.get(2).copied().unwrap_or(0.0).max(0.0),
        spread: lengths.get(3).copied().unwrap_or(0.0),
        color: color.unwrap_or_else(|| Color32::from_rgba_unmultiplied(0, 0, 0, 40)),
    })
}

fn parse_flow(raw: &str) -> Option<Flow> {
    match normalize(raw).as_str() {
        "row" => Some(Flow::Row),
        "column" => Some(Flow::Column),
        _ => None,
    }
}

fn parse_align(raw: &str) -> Option<AlignMode> {
    match normalize(raw).as_str() {
        "start" | "flex-start" | "left" | "top" => Some(AlignMode::Start),
        "center" => Some(AlignMode::Center),
        "end" | "flex-end" | "right" | "bottom" => Some(AlignMode::End),
        "stretch" => Some(AlignMode::Stretch),
        _ => None,
    }
}

fn parse_justify(raw: &str) -> Option<JustifyMode> {
    match normalize(raw).as_str() {
        "start" | "flex-start" | "left" | "top" => Some(JustifyMode::Start),
        "center" => Some(JustifyMode::Center),
        "end" | "flex-end" | "right" | "bottom" => Some(JustifyMode::End),
        "space-between" | "space-around" | "space-evenly" => Some(JustifyMode::Space),
        _ => None,
    }
}

fn parse_text_align(raw: &str) -> Option<TextAlign> {
    match normalize(raw).as_str() {
        "left" | "start" => Some(TextAlign::Start),
        "center" => Some(TextAlign::Center),
        "right" | "end" => Some(TextAlign::End),
        _ => None,
    }
}

fn parse_overflow(raw: &str) -> Option<Overflow> {
    match normalize(raw).as_str() {
        "hidden" => Some(Overflow::Hidden),
        "auto" | "scroll" => Some(Overflow::Scroll),
        "visible" => Some(Overflow::Visible),
        _ => None,
    }
}

fn parse_font_weight(raw: &str) -> Option<f32> {
    match normalize(raw).as_str() {
        "normal" => Some(400.0),
        "bold" => Some(700.0),
        "bolder" => Some(800.0),
        "lighter" => Some(300.0),
        other => other.parse::<f32>().ok(),
    }
}

fn parse_color_text(raw: &str) -> Option<Color32> {
    parse_hex_color(raw)
        .or_else(|| parse_rgb_color(raw))
        .or_else(|| parse_named_color(raw))
}

fn parse_rgb_color(raw: &str) -> Option<Color32> {
    let text = raw.trim();
    let (is_rgba, inner) = if text.starts_with("rgba(") && text.ends_with(')') {
        (true, &text[5..text.len() - 1])
    } else if text.starts_with("rgb(") && text.ends_with(')') {
        (false, &text[4..text.len() - 1])
    } else {
        return None;
    };

    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    if (!is_rgba && parts.len() != 3) || (is_rgba && parts.len() != 4) {
        return None;
    }

    let r = parse_rgb_channel(parts[0])?;
    let g = parse_rgb_channel(parts[1])?;
    let b = parse_rgb_channel(parts[2])?;
    let a = if is_rgba {
        parse_alpha_channel(parts[3])?
    } else {
        255
    };
    Some(Color32::from_rgba_unmultiplied(r, g, b, a))
}

fn parse_rgb_channel(raw: &str) -> Option<u8> {
    let text = raw.trim();
    if let Some(pct) = text.strip_suffix('%') {
        let value = pct.trim().parse::<f32>().ok()?.clamp(0.0, 100.0);
        return Some(((value / 100.0) * 255.0).round() as u8);
    }
    let value = text.parse::<f32>().ok()?.clamp(0.0, 255.0);
    Some(value.round() as u8)
}

fn parse_alpha_channel(raw: &str) -> Option<u8> {
    let text = raw.trim();
    if let Some(pct) = text.strip_suffix('%') {
        let value = pct.trim().parse::<f32>().ok()?.clamp(0.0, 100.0);
        return Some(((value / 100.0) * 255.0).round() as u8);
    }
    let value = text.parse::<f32>().ok()?;
    if value <= 1.0 {
        Some((value.clamp(0.0, 1.0) * 255.0).round() as u8)
    } else {
        Some(value.clamp(0.0, 255.0).round() as u8)
    }
}

fn parse_named_color(raw: &str) -> Option<Color32> {
    match normalize(raw).as_str() {
        "transparent" => Some(Color32::from_rgba_unmultiplied(0, 0, 0, 0)),
        "black" => Some(Color32::BLACK),
        "white" => Some(Color32::WHITE),
        "red" => Some(Color32::from_rgb(255, 0, 0)),
        "green" => Some(Color32::from_rgb(0, 128, 0)),
        "blue" => Some(Color32::from_rgb(0, 0, 255)),
        "gray" | "grey" => Some(Color32::from_gray(128)),
        _ => None,
    }
}

fn first_css_item(raw: &str) -> &str {
    let mut depth = 0usize;
    for (idx, ch) in raw.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return raw[..idx].trim(),
            _ => {}
        }
    }
    raw.trim()
}

fn split_css_tokens(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for ch in raw.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            _ if ch.is_whitespace() && depth == 0 => {
                let token = current.trim();
                if !token.is_empty() {
                    out.push(token.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let token = current.trim();
    if !token.is_empty() {
        out.push(token.to_string());
    }
    out
}

fn normalize(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn to_kebab(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_uppercase() {
            if !out.is_empty() {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}
