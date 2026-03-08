mod analyzer;
mod ast;
mod contract;
mod parity;
mod parser;
mod utils;
mod validator;

pub use ast::*;

pub fn parse(source: &str) -> Result<LogicProgram, String> {
    parser::parse(source)
}

pub fn runtime_contract(program: &LogicProgram) -> RuntimeContract {
    contract::runtime_contract(program)
}

#[cfg(test)]
mod tests;
