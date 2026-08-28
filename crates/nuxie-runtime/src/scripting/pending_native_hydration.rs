// Preserved pre-integration hydration assertions; adapt to the live translated owner before validation.
#[cfg(test)]
mod hydration_atomicity_tests {
    use super::*;

    #[derive(Debug)]
    struct FailingArtboardResolver {
        calls: Rc<Cell<usize>>,
    }

    impl ScriptArtboardResolver for FailingArtboardResolver {
        fn prepare_script_artboard(
            &self,
            _source: &crate::ScriptArtboardSource,
            _parent_context: Option<&ScriptArtboardParentContext>,
        ) -> Result<Box<dyn PreparedScriptArtboard>, ScriptError> {
            self.calls.set(self.calls.get() + 1);
            Err(ScriptError::new("facade rejected the live artboard"))
        }
    }

    #[derive(Debug)]
    struct DeferredFailureRecipe {
        constructions: Rc<Cell<usize>>,
    }

    impl PreparedScriptArtboard for DeferredFailureRecipe {
        fn construct(self: Box<Self>) -> Result<Box<dyn ScriptArtboard>, ScriptError> {
            self.constructions.set(self.constructions.get() + 1);
            Err(ScriptError::new("deferred Artboard construction ran"))
        }
    }

    #[derive(Debug)]
    struct DeferredFailureResolver {
        constructions: Rc<Cell<usize>>,
    }

    impl ScriptArtboardResolver for DeferredFailureResolver {
        fn prepare_script_artboard(
            &self,
            _source: &crate::ScriptArtboardSource,
            _parent_context: Option<&ScriptArtboardParentContext>,
        ) -> Result<Box<dyn PreparedScriptArtboard>, ScriptError> {
            Ok(Box::new(DeferredFailureRecipe {
                constructions: Rc::clone(&self.constructions),
            }))
        }
    }

    struct InertScriptInstance;

    impl ScriptInstance for InertScriptInstance {
        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(false)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }

        fn script_lifetime_valid(&self) -> bool {
            false
        }
    }

    struct RetryOrderingScript {
        trace: Rc<RefCell<Vec<&'static str>>>,
    }

    impl ScriptInstance for RetryOrderingScript {
        fn set_context_view_model_chain(
            &mut self,
            _view_model: Option<ScriptViewModel>,
            _parents: Vec<Option<ScriptViewModel>>,
        ) -> Result<(), ScriptError> {
            self.trace.borrow_mut().push("context");
            Ok(())
        }

        fn prepare_init_retry(&mut self) -> Result<(), ScriptError> {
            self.trace.borrow_mut().push("recreate");
            Ok(())
        }

        fn script_lifetime_valid(&self) -> bool {
            self.trace.borrow_mut().push("guard");
            false
        }

        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(false)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[test]
    fn every_retry_owner_uses_the_shared_recreate_then_guard_boundary() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let handle = RuntimeScriptInstanceHandle::new(Box::new(RetryOrderingScript {
            trace: Rc::clone(&trace),
        }));
        let context = ScriptListenerActionHydration::new(None, Vec::new());
        assert!(
            !install_context_recreate_and_guard_script_lifetime(&handle, &context, &mut None)
                .expect("retry boundary")
        );
        assert_eq!(&*trace.borrow(), &["context", "recreate", "guard"]);

        let owners = [
            include_str!("state_machine/state_machine_instance/state_machine_instance.rs"),
            include_str!("state_machine/state_machine_instance/data_converter_group.rs"),
            include_str!("scripted_interpolator.rs"),
        ];
        let owner_calls = owners
            .iter()
            .map(|source| {
                source
                    .matches("install_context_recreate_and_guard_script_lifetime(")
                    .count()
            })
            .sum::<usize>();
        assert_eq!(
            owner_calls, 5,
            "the complete retry-owner census must stay on the shared boundary"
        );
        assert_eq!(
            owners
                .iter()
                .map(|source| source.matches(".prepare_init_retry").count())
                .sum::<usize>(),
            0,
            "no retry sibling may reacquire recipes before the shared lifetime guard"
        );
    }

    #[derive(Debug)]
    struct UnresolvedViewModelResolver;

    impl ScriptViewModelInputResolver for UnresolvedViewModelResolver {
        fn resolve_script_view_model(
            &self,
            _input_global_id: u32,
            _path: &crate::ScriptInputViewModelPropertyPath,
        ) -> Result<Option<ScriptViewModel>, ScriptError> {
            Err(ScriptError::new("unresolved ViewModel prerequisite"))
        }
    }

    struct SetterCountingScriptInstance {
        calls: Rc<Cell<usize>>,
    }

    impl ScriptInstance for SetterCountingScriptInstance {
        fn set_context_view_model(
            &mut self,
            _view_model: Option<ScriptViewModel>,
        ) -> Result<(), ScriptError> {
            self.calls.set(self.calls.get() + 1);
            Ok(())
        }

        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(false)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            self.calls.set(self.calls.get() + 1);
            Ok(())
        }
    }

    #[test]
    fn artboard_facade_failure_precedes_every_hydration_write() {
        let calls = Rc::new(Cell::new(0));
        let hydration = ScriptListenerActionHydration::new(
            None,
            vec![
                ScriptListenerInputHydration::Value {
                    name: ScriptCoreString::from("before"),
                    value: ScriptValue::Number(7.0),
                },
                ScriptListenerInputHydration::Artboard {
                    name: ScriptCoreString::from("panel"),
                    source: crate::ScriptArtboardSource::File(0),
                    resolver: Rc::new(FailingArtboardResolver {
                        calls: Rc::clone(&calls),
                    }),
                    parent_context: None,
                },
            ],
        );

        let error = match hydration.preflight_artboards() {
            Ok(_) => panic!("facade failure must reject the complete batch"),
            Err(error) => error,
        };
        assert_eq!(error.message(), "facade rejected the live artboard");
        assert_eq!(calls.get(), 1);
        // Preflight owns no ScriptInstance parameter. Therefore the staged
        // scalar preceding the failed artboard cannot have reached a table.
    }

    #[test]
    fn empty_live_occurrence_is_rejected_in_the_first_validation_loop() {
        let resolver_calls = Rc::new(Cell::new(0));
        let hydration = ScriptListenerActionHydration::new(
            None,
            vec![ScriptListenerInputHydration::Artboard {
                name: ScriptCoreString::from("panel"),
                source: crate::ScriptArtboardSource::Live(crate::RuntimeBindableArtboard::new(
                    "empty live source",
                )),
                resolver: Rc::new(FailingArtboardResolver {
                    calls: Rc::clone(&resolver_calls),
                }),
                parent_context: None,
            }],
        );

        let error = match hydration.preflight_artboards() {
            Ok(_) => panic!("an absent concrete occurrence must fail preflight"),
            Err(error) => error,
        };
        assert_eq!(
            error.message(),
            "live scripted artboard source is unavailable"
        );
        assert_eq!(
            resolver_calls.get(),
            0,
            "the shared prerequisite guard runs before facade preparation"
        );
    }

    #[test]
    fn unresolved_view_model_preflight_precedes_public_apply_setters() {
        let setter_calls = Rc::new(Cell::new(0));
        let hydration = ScriptListenerActionHydration::new(
            None,
            vec![
                ScriptListenerInputHydration::Value {
                    name: ScriptCoreString::from("before"),
                    value: ScriptValue::Number(7.0),
                },
                ScriptListenerInputHydration::ViewModel {
                    name: ScriptCoreString::from("child"),
                    input_global_id: 42,
                    path: crate::ScriptInputViewModelPropertyPath {
                        path_ids: vec![1],
                        resolved_path_ids: vec![1],
                        is_relative: false,
                    },
                    resolver: Rc::new(UnresolvedViewModelResolver),
                },
            ],
        );
        let mut instance = SetterCountingScriptInstance {
            calls: Rc::clone(&setter_calls),
        };

        let error = hydration
            .apply(&mut instance, &mut NoopScriptHost)
            .expect_err("an unresolved ViewModel rejects public apply");

        assert_eq!(error.message(), "unresolved ViewModel prerequisite");
        assert_eq!(
            setter_calls.get(),
            0,
            "phase one rejects before Context or the earlier scalar setter"
        );
    }

    #[test]
    fn inert_script_lifetime_returns_before_deferred_artboard_construction() {
        let constructions = Rc::new(Cell::new(0));
        let hydration = ScriptListenerActionHydration::new(
            None,
            vec![ScriptListenerInputHydration::Artboard {
                name: ScriptCoreString::from("panel"),
                source: crate::ScriptArtboardSource::File(0),
                resolver: Rc::new(DeferredFailureResolver {
                    constructions: Rc::clone(&constructions),
                }),
                parent_context: None,
            }],
        )
        .preflight_artboards()
        .expect("immutable File prerequisite is valid");

        hydration
            .apply_inputs(&mut InertScriptInstance, &mut NoopScriptHost)
            .expect("pinned inert setter returns without construction failure");
        assert_eq!(constructions.get(), 0);
    }
}
