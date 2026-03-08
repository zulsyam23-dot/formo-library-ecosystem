use super::*;

#[test]
fn parse_logic_program_with_cross_platform_event() {
    let src = r#"
module AppController;
use "../services/session.fl" as Session;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  event startApp {
    action call Session.bootstrap;
    platform desktop { action call DesktopAdapter.sync; }
    platform web { action call WebAdapter.sync; }
  }
}
"#;
    let ast = parse(src).expect("logic should parse");
    assert_eq!(ast.units.len(), 1);
    assert_eq!(ast.units[0].events.len(), 1);
    assert!(ast.units[0].parity_ready);
}

#[test]
fn parse_logic_program_with_library_use_uri() {
    let src = r#"
module AppController;
use "lib://matimatika/core.fl" as MathCore;
logic AppController {
  event startApp {
    action emit "READY";
  }
}
"#;
    let ast = parse(src).expect("logic with library uri should parse");
    assert_eq!(ast.uses.len(), 1);
    assert_eq!(ast.uses[0].path, "lib://matimatika/core.fl");
    assert_eq!(ast.uses[0].alias, "MathCore");
}

#[test]
fn reject_invalid_library_use_uri() {
    let src = r#"
module AppController;
use "lib:///core.fl" as MathCore;
logic AppController {
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("invalid library uri should fail");
    assert!(err.contains("library path must be `lib://<library>/<module>.fl`"));
}

#[test]
fn reject_duplicate_use_alias() {
    let src = r#"
module AppController;
use "../a.fl" as Shared;
use "../b.fl" as Shared;
logic AppController { event startApp { action emit "READY"; } }
"#;
    let err = parse(src).expect_err("duplicate alias should fail");
    assert!(err.contains("duplicate use alias"));
}

#[test]
fn reject_unknown_call_alias() {
    let src = r#"
module AppController;
logic AppController { event startApp { action call MissingAlias.x; } }
"#;
    let err = parse(src).expect_err("unknown alias should fail");
    assert!(err.contains("unknown call alias"));
}

#[test]
fn reject_non_parity_logic_platform_block() {
    let src = r#"
module AppController;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  event startApp {
    action emit "STARTED";
    platform desktop { action call DesktopAdapter.sync; }
    platform web {
      action call WebAdapter.sync;
      action call WebAdapter.syncSecondary;
    }
  }
}
"#;
    let err = parse(src).expect_err("single platform should fail");
    assert!(err.contains("must define symmetric web/desktop platform actions"));
}

#[test]
fn reject_logic_event_without_global_action() {
    let src = r#"
module AppController;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  event startApp {
    platform desktop { action call DesktopAdapter.sync; }
    platform web { action call WebAdapter.sync; }
  }
}
"#;
    let err = parse(src).expect_err("logic event without global action should fail");
    assert!(err.contains("must contain at least one global action"));
}

#[test]
fn reject_service_event_with_platform_block() {
    let src = r#"
module SessionService;
use "../platform/web_adapter.fl" as WebAdapter;
service SessionService {
  event bootstrap {
    platform web { action call WebAdapter.sync; }
  }
}
"#;
    let err = parse(src).expect_err("service platform block should fail");
    assert!(err.contains("must not contain platform blocks"));
}

#[test]
fn reject_web_platform_declared_before_desktop() {
    let src = r#"
module AppController;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  event startApp {
    action emit "BOOT";
    platform web { action call WebAdapter.sync; }
    platform desktop { action call DesktopAdapter.sync; }
  }
}
"#;
    let err = parse(src).expect_err("web platform before desktop should fail");
    assert!(err.contains("desktop-first policy"));
}

#[test]
fn reject_logic_platform_non_call_action() {
    let src = r#"
module AppController;
use "../platform/web_adapter.fl" as WebAdapter;
logic AppController {
  event startApp {
    action emit "BOOT";
    platform desktop { action emit "DESKTOP_ONLY"; }
    platform web { action call WebAdapter.sync; }
  }
}
"#;
    let err = parse(src).expect_err("non-call action in logic platform block should fail");
    assert!(err.contains("only allows `action call` inside platform blocks"));
}

#[test]
fn reject_global_action_after_platform_blocks() {
    let src = r#"
module AppController;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  event startApp {
    action emit "BOOT";
    platform desktop { action call DesktopAdapter.sync; }
    platform web { action call WebAdapter.sync; }
    action emit "AFTER_PLATFORM";
  }
}
"#;
    let err = parse(src).expect_err("global action after platform block should fail");
    assert!(err.contains("must keep global actions before platform blocks"));
}

#[test]
fn reject_platform_interleaving_after_web_block() {
    let src = r#"
module AppController;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  event startApp {
    action emit "BOOT";
    platform desktop { action call DesktopAdapter.sync; }
    platform web { action call WebAdapter.sync; }
    platform desktop { action call DesktopAdapter.syncAgain; }
  }
}
"#;
    let err = parse(src).expect_err("desktop action after web block should fail");
    assert!(err.contains("must keep platform actions grouped as desktop then web"));
}

#[test]
fn reject_web_platform_calling_desktop_only_adapter() {
    let src = r#"
module AppController;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  event startApp {
    action emit "BOOT";
    platform desktop { action call DesktopAdapter.sync; }
    platform web { action call DesktopAdapter.sync; }
  }
}
"#;
    let err = parse(src).expect_err("desktop adapter inside web platform should fail");
    assert!(err.contains("desktop-only and cannot be used inside `platform web`"));
}

#[test]
fn reject_desktop_platform_calling_web_only_adapter() {
    let src = r#"
module AppController;
use "../platform/web_adapter.fl" as WebAdapter;
use "../platform/desktop_adapter.fl" as DesktopAdapter;
logic AppController {
  event startApp {
    action emit "BOOT";
    platform desktop { action call WebAdapter.sync; }
    platform web { action call WebAdapter.sync; }
  }
}
"#;
    let err = parse(src).expect_err("web adapter inside desktop platform should fail");
    assert!(err.contains("web-only and cannot be used inside `platform desktop`"));
}

#[test]
fn reject_logic_direct_browser_call() {
    let src = r#"
module AppController;
logic AppController {
  event startApp {
    action call Browser.historyPush;
  }
}
"#;
    let err = parse(src).expect_err("direct runtime alias should fail");
    assert!(err.contains("direct runtime alias `Browser`"));
}

#[test]
fn reject_adapter_set_action() {
    let src = r#"
module WebAdapter;
adapter WebAdapter {
  event syncBrowserRoute {
    action set route = "/home";
  }
}
"#;
    let err = parse(src).expect_err("adapter set action should fail");
    assert!(err.contains("only allows `action call`"));
}

#[test]
fn reject_non_lower_camel_event_name() {
    let src = r#"
module AppController;
logic AppController {
  event StartApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("event name case should fail");
    assert!(err.contains("must be lowerCamelCase"));
}
