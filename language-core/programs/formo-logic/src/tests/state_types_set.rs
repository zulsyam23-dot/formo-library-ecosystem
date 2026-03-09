use super::*;

#[test]
fn parse_standard_control_flow_keywords() {
    let src = r#"
module SessionService;
service SessionService {
  state {
    retries: int = 0;
  }
  event bootstrap {
    action set retries = 0;
    if hasCachedSession {
      action emit "SESSION_RESTORED";
    }
    for step in preloadSteps {
      action call Runtime.trace;
    }
    while shouldRetry {
      action call Runtime.sleep;
    }
    match mode {
      action emit "READY";
    }
    }
}
"#;
    let ast = parse(src).expect("control flow in event should parse");
    let event = &ast.units[0].events[0];
    assert_eq!(event.if_count, 1);
    assert_eq!(event.for_count, 1);
    assert_eq!(event.while_count, 1);
    assert_eq!(event.match_count, 1);
}

#[test]
fn parse_function_enum_and_try_catch_throw() {
    let src = r#"
module AppController;
logic AppController {
  enum SessionMode { Guest, Member, Admin }
  struct SessionState { userId: String, retryCount: Int }
  type RouteName = ui.Route;
  function resolveRoute(mode: SessionMode, retries: Int) -> String {
    return mode;
  }
  event startApp {
    action call Runtime.trace;
    try {
      action throw "FAILED_START";
    }
    catch {
      action emit "RECOVERED";
    }
  }
}
"#;
    let ast = parse(src).expect("function/enum/try-catch should parse");
    let unit = &ast.units[0];
    assert_eq!(unit.function_count, 1);
    assert_eq!(unit.typed_function_count, 1);
    assert_eq!(unit.returning_function_count, 1);
    assert_eq!(unit.enum_count, 1);
    assert_eq!(unit.struct_count, 1);
    assert_eq!(unit.typed_struct_count, 1);
    assert_eq!(unit.struct_field_count, 2);
    assert_eq!(unit.state_field_count, 0);
    assert_eq!(unit.typed_state_field_count, 0);
    assert_eq!(unit.type_alias_count, 1);
    assert_eq!(unit.qualified_type_alias_count, 1);
    let contract = runtime_contract(&ast);
    let event = &contract.units[0].events[0];
    assert_eq!(event.try_count, 1);
    assert_eq!(event.catch_count, 1);
    assert_eq!(event.throw_count, 1);
}

#[test]
fn parse_state_block_with_typed_fields() {
    let src = r#"
module AppController;
logic AppController {
  state {
    isReady: bool = false;
    activeRoute: string = "/home";
    retries: int = 0;
    timeoutSec: float = 0.5;
  }
  event startApp {
    action emit "READY";
  }
}
"#;
    let ast = parse(src).expect("state block should parse");
    let unit = &ast.units[0];
    assert_eq!(unit.state_field_count, 4);
    assert_eq!(unit.typed_state_field_count, 4);
}

#[test]
fn reject_state_field_without_type() {
    let src = r#"
module AppController;
logic AppController {
  state {
    isReady = false;
  }
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("state field without type should fail");
    assert!(err.contains("must declare type with `:`"));
}

#[test]
fn reject_state_field_without_initializer() {
    let src = r#"
module AppController;
logic AppController {
  state {
    isReady: bool;
  }
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("state field without initializer should fail");
    assert!(err.contains("must define initializer with `=`"));
}

#[test]
fn reject_state_field_with_invalid_bool_initializer() {
    let src = r#"
module AppController;
logic AppController {
  state {
    isReady: bool = "yes";
  }
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("invalid bool initializer should fail");
    assert!(err.contains("must initialize with `true` or `false`"));
}

#[test]
fn reject_state_field_non_lower_camel_name() {
    let src = r#"
module AppController;
logic AppController {
  state {
    IsReady: bool = true;
  }
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("state field naming should fail");
    assert!(err.contains("must be lowerCamelCase"));
}

#[test]
fn reject_duplicate_state_field() {
    let src = r#"
module AppController;
logic AppController {
  state {
    isReady: bool = true;
    isReady: bool = false;
  }
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("duplicate state field should fail");
    assert!(err.contains("duplicate state field"));
}

#[test]
fn reject_set_unknown_state_field() {
    let src = r#"
module AppController;
logic AppController {
  state {
    isReady: bool = false;
  }
  event startApp {
    action set activeRoute = "/home";
  }
}
"#;
    let err = parse(src).expect_err("set unknown state field should fail");
    assert!(err.contains("unknown state field"));
}

#[test]
fn reject_set_without_semicolon() {
    let src = r#"
module AppController;
logic AppController {
  state {
    isReady: bool = false;
  }
  event startApp {
    action set isReady = true
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("set without semicolon should fail");
    assert!(err.contains("must terminate with `;`"));
}

#[test]
fn reject_set_type_mismatch_bool_with_string_literal() {
    let src = r#"
module AppController;
logic AppController {
  state {
    isReady: bool = false;
  }
  event startApp {
    action set isReady = "yes";
  }
}
"#;
    let err = parse(src).expect_err("bool set with string literal should fail");
    assert!(err.contains("type mismatch in `action set isReady`"));
}

#[test]
fn reject_set_type_mismatch_string_with_number_literal() {
    let src = r#"
module AppController;
logic AppController {
  state {
    activeRoute: string = "/home";
  }
  event startApp {
    action set activeRoute = 10;
  }
}
"#;
    let err = parse(src).expect_err("string set with number literal should fail");
    assert!(err.contains("type mismatch in `action set activeRoute`"));
}

#[test]
fn reject_set_type_mismatch_int_with_float_literal() {
    let src = r#"
module AppController;
logic AppController {
  state {
    retries: int = 0;
  }
  event startApp {
    action set retries = 0.5;
  }
}
"#;
    let err = parse(src).expect_err("int set with float literal should fail");
    assert!(err.contains("type mismatch in `action set retries`"));
}

#[test]
fn reject_set_expression_with_unknown_state_reference() {
    let src = r#"
module AppController;
logic AppController {
  state {
    retries: int = 0;
  }
  event startApp {
    action set retries = missingCounter + 1;
  }
}
"#;
    let err = parse(src).expect_err("unknown state reference in set expression should fail");
    assert!(err.contains("unknown state reference `missingCounter`"));
}

#[test]
fn reject_set_expression_with_incompatible_state_reference_type() {
    let src = r#"
module AppController;
logic AppController {
  state {
    retries: int = 0;
    activeRoute: string = "/home";
  }
  event startApp {
    action set retries = activeRoute;
  }
}
"#;
    let err = parse(src).expect_err("incompatible state reference type should fail");
    assert!(err.contains("operand type `string`"));
}

#[test]
fn parse_set_bool_from_comparison_expression() {
    let src = r#"
module AppController;
logic AppController {
  state {
    retries: int = 0;
    isReady: bool = false;
  }
  event startApp {
    action set isReady = retries > 0;
    action emit "READY";
  }
}
"#;
    let ast = parse(src).expect("comparison expression should be valid bool assignment");
    let event = &ast.units[0].events[0];
    assert_eq!(event.actions.len(), 2);
}

#[test]
fn parse_set_bool_from_logical_expression() {
    let src = r#"
module AppController;
logic AppController {
  state {
    hasSession: bool = false;
    isReady: bool = false;
  }
  event startApp {
    action set isReady = hasSession || false;
    action emit "READY";
  }
}
"#;
    let ast = parse(src).expect("logical expression should be valid bool assignment");
    let event = &ast.units[0].events[0];
    assert_eq!(event.actions.len(), 2);
}

#[test]
fn reject_set_relational_expression_with_non_numeric_operands() {
    let src = r#"
module AppController;
logic AppController {
  state {
    activeRoute: string = "/home";
    isReady: bool = false;
  }
  event startApp {
    action set isReady = activeRoute > "a";
  }
}
"#;
    let err = parse(src).expect_err("relational expression on string should fail");
    assert!(err.contains("relational operators require numeric operands"));
}

#[test]
fn reject_set_arithmetic_expression_with_bool_operand() {
    let src = r#"
module AppController;
logic AppController {
  state {
    retries: int = 0;
    isReady: bool = false;
  }
  event startApp {
    action set retries = retries + isReady;
  }
}
"#;
    let err = parse(src).expect_err("arithmetic expression with bool should fail");
    assert!(err.contains("arithmetic operators require numeric operands"));
}

#[test]
fn parse_set_expression_rpn_preserves_operator_precedence() {
    let src = r#"
module AppController;
logic AppController {
  state {
    count: int = 0;
  }
  event increment {
    action set count = count + 1 * 2;
  }
}
"#;

    let ast = parse(src).expect("expression with precedence should parse");
    let action = &ast.units[0].events[0].actions[0];
    assert_eq!(
        action.set_expression_rpn,
        vec![
            LogicSetExprToken::Operand(LogicSetOperand::StateRef("count".to_string())),
            LogicSetExprToken::Operand(LogicSetOperand::IntLiteral("1".to_string())),
            LogicSetExprToken::Operand(LogicSetOperand::IntLiteral("2".to_string())),
            LogicSetExprToken::Operator(LogicSetOperator::Mul),
            LogicSetExprToken::Operator(LogicSetOperator::Add),
        ]
    );
}

#[test]
fn parse_set_expression_rpn_preserves_parentheses_order() {
    let src = r#"
module AppController;
logic AppController {
  state {
    count: int = 0;
  }
  event weighted {
    action set count = (count + 1) * 2;
  }
}
"#;

    let ast = parse(src).expect("expression with parentheses should parse");
    let action = &ast.units[0].events[0].actions[0];
    assert_eq!(
        action.set_expression_rpn,
        vec![
            LogicSetExprToken::Operand(LogicSetOperand::StateRef("count".to_string())),
            LogicSetExprToken::Operand(LogicSetOperand::IntLiteral("1".to_string())),
            LogicSetExprToken::Operator(LogicSetOperator::Add),
            LogicSetExprToken::Operand(LogicSetOperand::IntLiteral("2".to_string())),
            LogicSetExprToken::Operator(LogicSetOperator::Mul),
        ]
    );
}

#[test]
fn reject_type_alias_without_pascal_case() {
    let src = r#"
module AppController;
logic AppController {
  type routeName = String;
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("type alias name case should fail");
    assert!(err.contains("must be PascalCase"));
}

#[test]
fn reject_struct_field_without_type() {
    let src = r#"
module AppController;
logic AppController {
  struct SessionState { userId }
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("struct field without type should fail");
    assert!(err.contains("must declare type with `:`"));
}

#[test]
fn reject_function_parameter_without_type() {
    let src = r#"
module AppController;
logic AppController {
  function resolveRoute(mode) {
    return mode;
  }
  event startApp {
    action emit "READY";
  }
}
"#;
    let err = parse(src).expect_err("function params without type should fail");
    assert!(err.contains("must declare type with `:`"));
}
