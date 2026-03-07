const APP_JS_CHUNKS: &[&str] = &[
    include_str!("runtime/app/00_prelude.js"),
    include_str!("runtime/app/10_render_core.js"),
    include_str!("runtime/app/20_node_render_layout.js"),
    include_str!("runtime/app/21_node_render_button.js"),
    include_str!("runtime/app/22_node_render_input.js"),
    include_str!("runtime/app/22_node_render_toggles.js"),
    include_str!("runtime/app/23_node_render_modal.js"),
    include_str!("runtime/app/24_node_render_if_for.js"),
    include_str!("runtime/app/25_node_render_finalize.js"),
    include_str!("runtime/app/30_for_usage.js"),
    include_str!("runtime/app/31_for_binding.js"),
    include_str!("runtime/app/32_for_render.js"),
    include_str!("runtime/app/40_prop_record_string.js"),
    include_str!("runtime/app/40_prop_bool_len.js"),
    include_str!("runtime/app/40_prop_list.js"),
    include_str!("runtime/app/41_prop_scope.js"),
    include_str!("runtime/app/42_prop_utils.js"),
    include_str!("runtime/app/50_actions_state.js"),
    include_str!("runtime/app/60_focus_dom.js"),
];

pub(crate) fn app_js() -> String {
    APP_JS_CHUNKS.join("\n\n")
}

pub(crate) fn dev_bootstrap_script() -> &'static str {
    include_str!("runtime/dev_bootstrap.js")
}
