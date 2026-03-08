use super::*;

#[test]
fn runtime_contract_contains_call_buckets_per_scope() {
    let src = r#"
module AppController;
use "../services/session.fl" as Session;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  state {
    ready: bool = false;
  }
  event startApp {
    action call Session.bootstrap;
    action set ready = true;
    platform desktop { action call DesktopAdapter.sync; }
    platform web { action call WebAdapter.sync; }
  }
}
"#;
    let ast = parse(src).expect("logic parse should pass");
    let contract = runtime_contract(&ast);
    let event = &contract.units[0].events[0];
    assert_eq!(event.global_calls, vec!["Session.bootstrap".to_string()]);
    assert_eq!(event.web_calls, vec!["WebAdapter.sync".to_string()]);
    assert_eq!(event.desktop_calls, vec!["DesktopAdapter.sync".to_string()]);
    assert_eq!(event.set_count, 1);
    assert_eq!(event.break_count, 0);
    assert_eq!(event.continue_count, 0);
    assert_eq!(event.return_count, 0);
    assert_eq!(event.throw_count, 0);
    assert_eq!(event.if_count, 0);
    assert_eq!(event.for_count, 0);
    assert_eq!(event.while_count, 0);
    assert_eq!(event.match_count, 0);
    assert_eq!(event.try_count, 0);
    assert_eq!(event.catch_count, 0);
}
