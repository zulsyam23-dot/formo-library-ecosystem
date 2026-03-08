use crate::ast::{
    LogicActionKind, LogicScope, LogicUnit, RuntimeContract, RuntimeEventContract,
    RuntimeUnitContract,
};

pub(crate) fn runtime_contract(units_src_module: &crate::ast::LogicProgram) -> RuntimeContract {
    let mut units = Vec::new();
    for unit in &units_src_module.units {
        units.push(build_unit_contract(unit));
    }
    RuntimeContract {
        module: units_src_module.module.clone(),
        units,
    }
}

fn build_unit_contract(unit: &LogicUnit) -> RuntimeUnitContract {
    let mut event_contracts = Vec::new();
    for event in &unit.events {
        let mut global_calls = Vec::new();
        let mut web_calls = Vec::new();
        let mut desktop_calls = Vec::new();
        let mut set_count = 0usize;
        let mut emit_count = 0usize;
        let mut throw_count = 0usize;
        let mut break_count = 0usize;
        let mut continue_count = 0usize;
        let mut return_count = 0usize;
        for action in &event.actions {
            match action.kind {
                LogicActionKind::Call => {
                    let target = action.target.clone().unwrap_or_default();
                    match action.scope {
                        LogicScope::Global => global_calls.push(target),
                        LogicScope::Web => web_calls.push(target),
                        LogicScope::Desktop => desktop_calls.push(target),
                    }
                }
                LogicActionKind::Set => set_count += 1,
                LogicActionKind::Emit => emit_count += 1,
                LogicActionKind::Throw => throw_count += 1,
                LogicActionKind::Break => break_count += 1,
                LogicActionKind::Continue => continue_count += 1,
                LogicActionKind::Return => return_count += 1,
            }
        }
        event_contracts.push(RuntimeEventContract {
            name: event.name.clone(),
            global_calls,
            web_calls,
            desktop_calls,
            set_count,
            emit_count,
            throw_count,
            break_count,
            continue_count,
            return_count,
            total_actions: event.actions.len(),
            if_count: event.if_count,
            for_count: event.for_count,
            while_count: event.while_count,
            match_count: event.match_count,
            try_count: event.try_count,
            catch_count: event.catch_count,
        });
    }

    RuntimeUnitContract {
        kind: unit.kind.as_str().to_string(),
        name: unit.name.clone(),
        parity_ready: unit.parity_ready,
        state_field_count: unit.state_field_count,
        typed_state_field_count: unit.typed_state_field_count,
        function_count: unit.function_count,
        typed_function_count: unit.typed_function_count,
        returning_function_count: unit.returning_function_count,
        enum_count: unit.enum_count,
        enum_variant_count: unit.enum_variant_count,
        struct_count: unit.struct_count,
        typed_struct_count: unit.typed_struct_count,
        struct_field_count: unit.struct_field_count,
        type_alias_count: unit.type_alias_count,
        qualified_type_alias_count: unit.qualified_type_alias_count,
        events: event_contracts,
    }
}
