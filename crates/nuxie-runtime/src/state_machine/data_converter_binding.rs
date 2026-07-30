//! Authored-order `StateMachineInstance` DataBind/converter binding plan.
//!
//! Pinned C++ executes one outer `DataBindContext::bindFromContext` and its
//! complete converter virtual call before it advances to the next authored
//! outer DataBind (`data_bind_container.cpp:25-35`;
//! `data_bind_context.cpp:56-89`). Scripted converter reinitialization crosses
//! the facade/runtime boundary, so Rust exposes the same call stack as an
//! immutable operation stream and reacquires the concrete occurrence for each
//! step.

use crate::data_bind_graph::RuntimeDataBindGraph;
use crate::data_converter::{RuntimeDataConverterBindStep, runtime_data_converter_bind_steps};

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeStateMachineDataConverterBindStep {
    BindOuter {
        data_bind_index: usize,
    },
    BindConverter {
        data_bind_index: usize,
        converter_path: Vec<usize>,
    },
    Rehydrate {
        data_bind_index: usize,
        converter_path: Vec<usize>,
        converter_global_id: u32,
        inits: bool,
    },
    RebindFinalInput {
        data_bind_index: usize,
        converter_path: Vec<usize>,
        converter_input_index: usize,
        inner_data_bind_index: usize,
    },
    FinalizeOuter {
        data_bind_index: usize,
    },
}

pub(crate) fn runtime_state_machine_data_converter_bind_steps(
    graph: &RuntimeDataBindGraph,
) -> Vec<RuntimeStateMachineDataConverterBindStep> {
    let mut steps = Vec::new();
    for binding in &graph.default_view_model_bindings {
        let Some(source) = graph.sources.get(binding.source.0) else {
            continue;
        };
        if !source.context_bindable {
            continue;
        }
        steps.push(RuntimeStateMachineDataConverterBindStep::BindOuter {
            data_bind_index: binding.data_bind_index,
        });
        if let Some(converter) = source.converter.as_ref() {
            steps.extend(
                runtime_data_converter_bind_steps(converter)
                    .into_iter()
                    .map(|step| match step {
                        RuntimeDataConverterBindStep::BindOwn { path } => {
                            RuntimeStateMachineDataConverterBindStep::BindConverter {
                                data_bind_index: binding.data_bind_index,
                                converter_path: path,
                            }
                        }
                        RuntimeDataConverterBindStep::Rehydrate {
                            path,
                            converter_global_id,
                            inits,
                        } => RuntimeStateMachineDataConverterBindStep::Rehydrate {
                            data_bind_index: binding.data_bind_index,
                            converter_path: path,
                            converter_global_id,
                            inits,
                        },
                        RuntimeDataConverterBindStep::RebindFinalInput {
                            path,
                            input_index,
                            data_bind_index,
                        } => RuntimeStateMachineDataConverterBindStep::RebindFinalInput {
                            data_bind_index: binding.data_bind_index,
                            converter_path: path,
                            converter_input_index: input_index,
                            inner_data_bind_index: data_bind_index,
                        },
                    }),
            );
        }
        steps.push(RuntimeStateMachineDataConverterBindStep::FinalizeOuter {
            data_bind_index: binding.data_bind_index,
        });
    }
    steps
}
