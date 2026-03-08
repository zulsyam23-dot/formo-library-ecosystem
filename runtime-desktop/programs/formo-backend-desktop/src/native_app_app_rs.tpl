use crate::model::{NativeBundle, NativeNode};
use crate::render::render_tree;
use eframe::egui::{self, Color32};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

const FORMO_NATIVE_JSON: &str = include_str!("../../app.native.json");
const FORMO_COMPILED_ENTRY: &str = "{{ENTRY_COMPONENT}}";

pub fn run() -> Result<(), eframe::Error> {
    let bundle: NativeBundle =
        serde_json::from_str(FORMO_NATIVE_JSON).expect("app.native.json must be valid");
    let window_title = format!("Formo Native Desktop - {}", bundle.entry_component);
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        &window_title,
        native_options,
        Box::new(move |cc| {
            configure_theme(&cc.egui_ctx);
            Box::new(FormoNativeApp::new(bundle))
        }),
    )
}

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.override_text_color = Some(Color32::from_rgb(0x15, 0x15, 0x15));
    visuals.panel_fill = Color32::from_rgb(0xF6, 0xF7, 0xFB);
    visuals.window_fill = Color32::from_rgb(0xFF, 0xFF, 0xFF);
    visuals.faint_bg_color = Color32::from_rgb(0xF0, 0xF2, 0xF8);
    visuals.extreme_bg_color = Color32::from_rgb(0xEA, 0xED, 0xF6);
    visuals.window_stroke = egui::Stroke::new(1.0, Color32::from_rgb(0xD4, 0xD8, 0xE5));
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(0xD4, 0xD8, 0xE5));
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(0xF4, 0xF7, 0xFF);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(0xB8, 0xBF, 0xD8));
    visuals.widgets.active.bg_fill = Color32::from_rgb(0xE8, 0xEE, 0xFF);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0xEC, 0xF1, 0xFF);
    visuals.selection.bg_fill = Color32::from_rgb(0xD9, 0xE6, 0xFF);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.menu_margin = egui::Margin::same(10.0);
    style.spacing.indent = 14.0;
    ctx.set_style(style);
}

struct FormoNativeApp {
    bundle: NativeBundle,
    state: BTreeMap<String, JsonValue>,
    action_log: Vec<String>,
    parity_warnings: usize,
}

impl FormoNativeApp {
    fn new(bundle: NativeBundle) -> Self {
        let parity_warnings = bundle
            .diagnostics
            .iter()
            .filter(|d| d.level.eq_ignore_ascii_case("warning"))
            .count();
        if parity_warnings > 0 {
            eprintln!(
                "[formo-native] {} desktop parity warning(s) emitted:",
                parity_warnings
            );
            for d in &bundle.diagnostics {
                if d.level.eq_ignore_ascii_case("warning") {
                    eprintln!(
                        "  [{}] {} ({}:{}:{})",
                        d.code, d.message, d.source.file, d.source.line, d.source.col
                    );
                }
            }
        }

        Self {
            bundle,
            state: BTreeMap::new(),
            action_log: Vec::new(),
            parity_warnings,
        }
    }

    fn entry_root(&self) -> Option<&NativeNode> {
        if let Some(component) = self
            .bundle
            .components
            .iter()
            .find(|c| c.name == self.bundle.entry_component)
        {
            return Some(&component.root_node);
        }
        self.bundle.components.first().map(|c| &c.root_node)
    }
}

impl eframe::App for FormoNativeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let debug_enabled = std::env::var("FORMO_NATIVE_DEBUG")
            .map(|v| {
                let v = v.trim().to_ascii_lowercase();
                v == "1" || v == "true" || v == "yes" || v == "on"
            })
            .unwrap_or(false);

        if self.parity_warnings > 0 {
            egui::TopBottomPanel::top("formo-native-parity-hint")
                .exact_height(28.0)
                .show(ctx, |ui| {
                    ui.colored_label(
                        Color32::from_rgb(0x9A, 0x5B, 0x00),
                        format!(
                            "Desktop parity warnings: {} (set FORMO_NATIVE_DEBUG=1 untuk detail)",
                            self.parity_warnings
                        ),
                    );
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(root) = self.entry_root().cloned() {
                render_tree(ui, &root, &mut self.state, &mut self.action_log);
            } else {
                ui.colored_label(Color32::RED, "No entry component found in app.native.json");
            }
        });

        if debug_enabled {
            egui::TopBottomPanel::bottom("formo-native-debug")
                .resizable(true)
                .default_height(120.0)
                .show(ctx, |ui| {
                    ui.label(format!("Entry: {}", FORMO_COMPILED_ENTRY));
                    ui.collapsing("State", |ui| {
                        for (k, v) in &self.state {
                            ui.monospace(format!("{k} = {v}"));
                        }
                        if self.state.is_empty() {
                            ui.monospace("<empty>");
                        }
                    });
                    ui.collapsing("Action Log", |ui| {
                        for item in self.action_log.iter().rev().take(20) {
                            ui.monospace(item);
                        }
                        if self.action_log.is_empty() {
                            ui.monospace("<no actions>");
                        }
                    });
                    ui.collapsing("Diagnostics", |ui| {
                        for d in &self.bundle.diagnostics {
                            ui.monospace(format!(
                                "[{}:{}] {} ({}:{}:{})",
                                d.level, d.code, d.message, d.source.file, d.source.line, d.source.col
                            ));
                        }
                        if self.bundle.diagnostics.is_empty() {
                            ui.monospace("<none>");
                        }
                    });
                });
        }
    }
}
