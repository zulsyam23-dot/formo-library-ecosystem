#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicProgram {
    pub raw: String,
    pub module: String,
    pub uses: Vec<LogicUse>,
    pub units: Vec<LogicUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicUse {
    pub path: String,
    pub alias: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicUnitKind {
    Logic,
    Service,
    Contract,
    Adapter,
}

impl LogicUnitKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            LogicUnitKind::Logic => "logic",
            LogicUnitKind::Service => "service",
            LogicUnitKind::Contract => "contract",
            LogicUnitKind::Adapter => "adapter",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicScope {
    Global,
    Web,
    Desktop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicActionKind {
    Call,
    Set,
    Emit,
    Throw,
    Break,
    Continue,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicSetValueHint {
    BoolLiteral,
    StringLiteral,
    IntLiteral,
    FloatLiteral,
    Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicSetOperand {
    StateRef(String),
    BoolLiteral(bool),
    StringLiteral(String),
    IntLiteral(String),
    FloatLiteral(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicSetOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicSetExprToken {
    Operand(LogicSetOperand),
    Operator(LogicSetOperator),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicAction {
    pub kind: LogicActionKind,
    pub scope: LogicScope,
    pub target: Option<String>,
    pub set_value_hint: Option<LogicSetValueHint>,
    pub set_operands: Vec<LogicSetOperand>,
    pub set_operators: Vec<LogicSetOperator>,
    pub set_expression_rpn: Vec<LogicSetExprToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicStateField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicEvent {
    pub name: String,
    pub actions: Vec<LogicAction>,
    pub if_count: usize,
    pub for_count: usize,
    pub while_count: usize,
    pub match_count: usize,
    pub try_count: usize,
    pub catch_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicUnit {
    pub kind: LogicUnitKind,
    pub name: String,
    pub state_fields: Vec<LogicStateField>,
    pub events: Vec<LogicEvent>,
    pub platforms: Vec<String>,
    pub parity_ready: bool,
    pub state_field_count: usize,
    pub typed_state_field_count: usize,
    pub function_count: usize,
    pub typed_function_count: usize,
    pub returning_function_count: usize,
    pub enum_count: usize,
    pub enum_variant_count: usize,
    pub struct_count: usize,
    pub typed_struct_count: usize,
    pub struct_field_count: usize,
    pub type_alias_count: usize,
    pub qualified_type_alias_count: usize,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContract {
    pub module: String,
    pub units: Vec<RuntimeUnitContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeUnitContract {
    pub kind: String,
    pub name: String,
    pub parity_ready: bool,
    pub state_field_count: usize,
    pub typed_state_field_count: usize,
    pub function_count: usize,
    pub typed_function_count: usize,
    pub returning_function_count: usize,
    pub enum_count: usize,
    pub enum_variant_count: usize,
    pub struct_count: usize,
    pub typed_struct_count: usize,
    pub struct_field_count: usize,
    pub type_alias_count: usize,
    pub qualified_type_alias_count: usize,
    pub events: Vec<RuntimeEventContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEventContract {
    pub name: String,
    pub global_calls: Vec<String>,
    pub web_calls: Vec<String>,
    pub desktop_calls: Vec<String>,
    pub set_count: usize,
    pub emit_count: usize,
    pub throw_count: usize,
    pub break_count: usize,
    pub continue_count: usize,
    pub return_count: usize,
    pub total_actions: usize,
    pub if_count: usize,
    pub for_count: usize,
    pub while_count: usize,
    pub match_count: usize,
    pub try_count: usize,
    pub catch_count: usize,
}
