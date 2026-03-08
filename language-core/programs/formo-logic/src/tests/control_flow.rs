use super::*;

#[test]
fn parse_break_continue_and_return_actions() {
    let src = r#"
module SessionService;
service SessionService {
  state {
    retries: int = 0;
  }
  event bootstrap {
    action set retries = 0;
    while shouldRetry {
      action call Runtime.sleep;
      if shouldStop {
        action break;
      }
      if shouldSkip {
        action continue;
      }
      action set retries = retries + 1;
    }
    action return;
  }
}
"#;
    let ast = parse(src).expect("break/continue/return should parse");
    let contract = runtime_contract(&ast);
    let event = &contract.units[0].events[0];
    assert_eq!(event.break_count, 1);
    assert_eq!(event.continue_count, 1);
    assert_eq!(event.return_count, 1);
}

#[test]
fn reject_empty_if_block() {
    let src = r#"
module SessionService;
service SessionService {
  event bootstrap {
    action emit "BOOT";
    if hasCachedSession { }
  }
}
"#;
    let err = parse(src).expect_err("empty if block should fail");
    assert!(err.contains("`if` block in event `bootstrap` must contain at least one `action`"));
}

#[test]
fn reject_break_outside_loop() {
    let src = r#"
module SessionService;
service SessionService {
  event bootstrap {
    action emit "BOOT";
    action break;
  }
}
"#;
    let err = parse(src).expect_err("break outside loop should fail");
    assert!(err.contains("must be inside `for` or `while` block"));
}

#[test]
fn reject_continue_outside_loop() {
    let src = r#"
module SessionService;
service SessionService {
  event bootstrap {
    action emit "BOOT";
    action continue;
  }
}
"#;
    let err = parse(src).expect_err("continue outside loop should fail");
    assert!(err.contains("must be inside `for` or `while` block"));
}

#[test]
fn reject_throw_outside_try_catch() {
    let src = r#"
module SessionService;
service SessionService {
  state {
    ready: bool = true;
  }
  event bootstrap {
    action set ready = true;
    action throw "FAILED";
  }
}
"#;
    let err = parse(src).expect_err("throw outside try/catch should fail");
    assert!(err.contains("must be inside `try` or `catch` block"));
}

#[test]
fn reject_catch_without_try() {
    let src = r#"
module SessionService;
service SessionService {
  state {
    ready: bool = true;
  }
  event bootstrap {
    action set ready = true;
    catch {
      action emit "RECOVERED";
    }
  }
}
"#;
    let err = parse(src).expect_err("catch without try should fail");
    assert!(err.contains("must follow a `try` block"));
}

#[test]
fn reject_action_after_return() {
    let src = r#"
module SessionService;
service SessionService {
  event bootstrap {
    action return;
    action emit "AFTER";
  }
}
"#;
    let err = parse(src).expect_err("action after return should fail");
    assert!(err.contains("must be the last action"));
}
