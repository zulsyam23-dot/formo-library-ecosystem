use crate::style::{with_opacity, AlignMode, Edges, Flow, JustifyMode, RenderStyle, TextAlign};
use eframe::egui::{self, Color32, RichText};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct FrameDefaults {
    pub fill: Option<Color32>,
    pub border_color: Option<Color32>,
    pub border_width: Option<f32>,
    pub border_radius: Option<f32>,
    pub padding: Option<Edges>,
    pub margin: Option<Edges>,
    pub shadow: Option<egui::epaint::Shadow>,
}

pub(super) fn with_style_container(
    ui: &mut egui::Ui,
    style: RenderStyle,
    defaults: FrameDefaults,
    body: impl FnOnce(&mut egui::Ui),
) {
    if let Some(frame) = frame_from_style(style, defaults) {
        frame.show(ui, |ui| {
            apply_size(ui, style);
            body(ui);
        });
    } else {
        ui.scope(|ui| {
            apply_size(ui, style);
            body(ui);
        });
    }
}

fn frame_from_style(style: RenderStyle, defaults: FrameDefaults) -> Option<egui::Frame> {
    let mut frame = egui::Frame::none();
    let mut used = false;
    if let Some(fill) = style.background.or(defaults.fill) {
        frame.fill = with_opacity(fill, style.opacity);
        used = true;
    }
    if let Some(color) = style.border_color.or(defaults.border_color) {
        frame.stroke.color = with_opacity(color, style.opacity);
        used = true;
    }
    if let Some(width) = style.border_width.or(defaults.border_width) {
        frame.stroke.width = width.max(0.0);
        used = true;
    }
    if let Some(radius) = style.border_radius.or(defaults.border_radius) {
        frame.rounding = egui::Rounding::same(radius.max(0.0));
        used = true;
    }

    let padding = if style.has_padding {
        Some(style.padding)
    } else {
        defaults.padding
    };
    if let Some(v) = padding {
        if !v.is_zero() {
            frame.inner_margin = v.margin();
            used = true;
        }
    }
    let margin = if style.has_margin {
        Some(style.margin)
    } else {
        defaults.margin
    };
    if let Some(v) = margin {
        if !v.is_zero() {
            frame.outer_margin = v.margin();
            used = true;
        }
    }

    if let Some(mut shadow) = style.shadow.or(defaults.shadow) {
        shadow.color = with_opacity(shadow.color, style.opacity);
        frame.shadow = shadow;
        used = true;
    }
    if let Some(opacity) = style.opacity {
        frame = frame.multiply_with_opacity(opacity.clamp(0.0, 1.0));
        used = true;
    }
    if used {
        Some(frame)
    } else {
        None
    }
}

fn apply_size(ui: &mut egui::Ui, style: RenderStyle) {
    if let Some(v) = style.min_width {
        ui.set_min_width(v.max(0.0));
    }
    if let Some(v) = style.min_height {
        ui.set_min_height(v.max(0.0));
    }
    if let Some(v) = style.max_width {
        ui.set_max_width(v.max(0.0));
    }
    if let Some(v) = style.max_height {
        ui.set_max_height(v.max(0.0));
    }
    if let Some(v) = style.width {
        ui.set_width(v.max(0.0));
    }
    if let Some(v) = style.height {
        ui.set_height(v.max(0.0));
    }
}

pub(super) fn apply_gap(ui: &mut egui::Ui, gap: Option<f32>, default_gap: Option<f32>) {
    if let Some(v) = gap.or(default_gap) {
        let value = v.max(0.0);
        ui.spacing_mut().item_spacing.x = value;
        ui.spacing_mut().item_spacing.y = value;
    }
}

pub(super) fn layout_from_style(flow: Flow, style: RenderStyle) -> egui::Layout {
    let cross = match style.align {
        AlignMode::Start | AlignMode::Stretch => egui::Align::Min,
        AlignMode::Center => egui::Align::Center,
        AlignMode::End => egui::Align::Max,
    };
    // In egui, row cross-justify can over-expand children vertically unless the row has a fixed height.
    let cross_justify = match flow {
        Flow::Column => style.align == AlignMode::Stretch,
        Flow::Row => {
            style.align == AlignMode::Stretch
                && (style.height.is_some()
                    || style.min_height.is_some()
                    || style.max_height.is_some())
        }
    };
    let mut layout = match flow {
        Flow::Row => egui::Layout::left_to_right(cross),
        Flow::Column => egui::Layout::top_down(cross),
    }
    .with_main_wrap(style.wrap)
    .with_cross_justify(cross_justify);

    layout = match style.justify {
        JustifyMode::Start => layout.with_main_align(egui::Align::Min),
        JustifyMode::Center => layout.with_main_align(egui::Align::Center),
        JustifyMode::End => layout.with_main_align(egui::Align::Max),
        JustifyMode::Space => layout
            .with_main_align(egui::Align::Center)
            .with_main_justify(true),
    };
    layout
}

pub(super) fn show_text(ui: &mut egui::Ui, text: RichText, align: TextAlign) {
    match align {
        TextAlign::Start => ui.label(text),
        TextAlign::Center => ui.horizontal_centered(|ui| ui.label(text)).response,
        TextAlign::End => ui
            .with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(text)
            })
            .response,
    };
}

pub(super) fn apply_text_style(
    mut rich: RichText,
    style: RenderStyle,
    fallback_color: Option<Color32>,
    fallback_weight: Option<f32>,
    fallback_italic: bool,
) -> RichText {
    if let Some(color) = style.text_color.or(fallback_color) {
        rich = rich.color(with_opacity(color, style.opacity));
    }
    let weight = style.font_weight.or(fallback_weight);
    if weight.unwrap_or(400.0) >= 600.0 {
        rich = rich.strong();
    }
    if style.italic || fallback_italic {
        rich = rich.italics();
    }
    rich
}
