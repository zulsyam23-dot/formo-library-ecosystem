use crate::ast::{LogicEvent, LogicScope};

pub(crate) fn event_platform_action_counts(event: &LogicEvent) -> (usize, usize) {
    let mut web_actions = 0usize;
    let mut desktop_actions = 0usize;
    for action in &event.actions {
        match action.scope {
            LogicScope::Web => web_actions += 1,
            LogicScope::Desktop => desktop_actions += 1,
            LogicScope::Global => {}
        }
    }
    (web_actions, desktop_actions)
}

pub(crate) fn is_symmetric_platform_actions(web_actions: usize, desktop_actions: usize) -> bool {
    (web_actions == 0 && desktop_actions == 0)
        || (web_actions > 0 && desktop_actions > 0 && web_actions == desktop_actions)
}
