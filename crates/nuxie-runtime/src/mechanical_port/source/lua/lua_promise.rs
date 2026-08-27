#![cfg(feature = "rive_scripting")]

use crate::mechanical_port::source::lua::rive_lua_libs::*;

fn promise_cleanup_links(state: &mut LuaState, promise: &mut ScriptedPromise) {
    if promise.parent_ref != LUA_NOREF {
        state.unref(promise.parent_ref);
        promise.parent_ref = LUA_NOREF;
    }
    for reference in promise.consumer_refs.drain(..) {
        if reference != LUA_NOREF {
            state.unref(reference);
        }
    }
    if promise.on_cancel_ref != LUA_NOREF {
        state.unref(promise.on_cancel_ref);
        promise.on_cancel_ref = LUA_NOREF;
    }
}

impl ScriptedPromise {
    pub fn new(main_thread: *mut LuaState) -> Self {
        Self {
            state: main_thread,
            promise_state: PromiseState::Pending,
            result_ref: LUA_NOREF,
            then_callbacks: Vec::new(),
            finally_callbacks: Vec::new(),
            parent_ref: LUA_NOREF,
            consumer_refs: Vec::new(),
            on_cancel_ref: LUA_NOREF,
        }
    }

    pub fn resolve(&mut self, state: &mut LuaState, value_index: i32) {
        if self.promise_state != PromiseState::Pending {
            return;
        }
        let inner = state.to_rive_optional::<ScriptedPromise>(value_index, true);
        if let Some(inner) = inner.filter(|inner| !std::ptr::eq(*inner, self)) {
            if inner.is_fulfilled() {
                state.raw_get_i(LUA_REGISTRY_INDEX, inner.result_ref());
                self.resolve(state, state.top());
                state.pop(1);
                return;
            }
            if inner.is_rejected() {
                state.raw_get_i(LUA_REGISTRY_INDEX, inner.result_ref());
                self.reject(state, state.top());
                state.pop(1);
                return;
            }
            if inner.is_cancelled() {
                self.cancel(state);
                return;
            }
            assert!(
                false,
                "resolve() with pending inner promise requires selfRef — use the overload"
            );
            return;
        }
        self.promise_state = PromiseState::Fulfilled;
        state.push_value(value_index);
        self.result_ref = state.reference(-1);
        state.pop(1);
        promise_notify_callbacks(state, self);
        promise_cleanup_links(state, self);
    }

    pub fn resolve_with_self_ref(&mut self, state: &mut LuaState, value_index: i32, self_ref: i32) {
        if self.promise_state != PromiseState::Pending {
            return;
        }
        let inner_pointer = state
            .to_rive_optional::<ScriptedPromise>(value_index, true)
            .map(|inner| inner as *mut ScriptedPromise);
        if let Some(inner_pointer) = inner_pointer.filter(|inner| !std::ptr::eq(*inner, self)) {
            let inner = unsafe { &mut *inner_pointer };
            if inner.is_fulfilled() {
                state.raw_get_i(LUA_REGISTRY_INDEX, inner.result_ref());
                self.resolve(state, state.top());
                state.pop(1);
                return;
            }
            if inner.is_rejected() {
                state.raw_get_i(LUA_REGISTRY_INDEX, inner.result_ref());
                self.reject(state, state.top());
                state.pop(1);
                return;
            }
            if inner.is_cancelled() {
                self.cancel(state);
                return;
            }
            if self.parent_ref != LUA_NOREF {
                state.unref(self.parent_ref);
            }
            state.push_value(value_index);
            self.parent_ref = state.reference(-1);
            state.pop(1);

            state.raw_get_i(LUA_REGISTRY_INDEX, self_ref);
            let consumer_ref = state.reference(-1);
            state.pop(1);
            state.raw_get_i(LUA_REGISTRY_INDEX, self_ref);
            let chained_ref = state.reference(-1);
            state.pop(1);

            inner.consumer_refs.push(consumer_ref);
            inner.then_callbacks.push(ThenCallback {
                chained_promise_ref: chained_ref,
                ..ThenCallback::default()
            });
            return;
        }
        self.promise_state = PromiseState::Fulfilled;
        state.push_value(value_index);
        self.result_ref = state.reference(-1);
        state.pop(1);
        promise_notify_callbacks(state, self);
        promise_cleanup_links(state, self);
    }

    pub fn reject(&mut self, state: &mut LuaState, error_index: i32) {
        if self.promise_state != PromiseState::Pending {
            return;
        }
        self.promise_state = PromiseState::Rejected;
        state.push_value(error_index);
        self.result_ref = state.reference(-1);
        state.pop(1);
        promise_notify_callbacks(state, self);
        promise_cleanup_links(state, self);
    }

    pub fn cancel(&mut self, state: &mut LuaState) {
        if self.promise_state != PromiseState::Pending {
            return;
        }
        self.promise_state = PromiseState::Cancelled;

        if self.on_cancel_ref != LUA_NOREF {
            state.raw_get_i(LUA_REGISTRY_INDEX, self.on_cancel_ref);
            if state.pcall(0, 0, 0) != LUA_OK {
                state.pop(1);
            }
        }

        let consumer_refs = self.consumer_refs.clone();
        for reference in consumer_refs {
            if reference != LUA_NOREF {
                state.raw_get_i(LUA_REGISTRY_INDEX, reference);
                let consumer = state
                    .to_rive_optional::<ScriptedPromise>(-1, true)
                    .map(|promise| promise as *mut ScriptedPromise);
                state.pop(1);
                if let Some(consumer) = consumer {
                    let consumer = unsafe { &mut *consumer };
                    if consumer.is_pending() {
                        consumer.cancel(state);
                    }
                }
            }
        }

        if self.parent_ref != LUA_NOREF {
            state.raw_get_i(LUA_REGISTRY_INDEX, self.parent_ref);
            let parent = state
                .to_rive_optional::<ScriptedPromise>(-1, true)
                .map(|promise| promise as *mut ScriptedPromise);
            state.pop(1);
            if let Some(parent) = parent {
                let parent = unsafe { &mut *parent };
                if parent.is_pending() {
                    let mut all_cancelled = true;
                    for &consumer_ref in &parent.consumer_refs {
                        if consumer_ref != LUA_NOREF {
                            state.raw_get_i(LUA_REGISTRY_INDEX, consumer_ref);
                            let consumer = state
                                .to_rive_optional::<ScriptedPromise>(-1, true)
                                .map(|promise| promise as *mut ScriptedPromise);
                            state.pop(1);
                            if let Some(consumer) = consumer {
                                if !unsafe { &*consumer }.is_cancelled() {
                                    all_cancelled = false;
                                    break;
                                }
                            }
                        }
                    }
                    if all_cancelled {
                        parent.cancel(state);
                    }
                }
            }
        }

        for callback in self.then_callbacks.drain(..) {
            if callback.cancel_ref != LUA_NOREF {
                state.raw_get_i(LUA_REGISTRY_INDEX, callback.cancel_ref);
                if state.pcall(0, 0, 0) != LUA_OK {
                    state.pop(1);
                }
                state.unref(callback.cancel_ref);
            }
            if callback.success_ref != LUA_NOREF {
                state.unref(callback.success_ref);
            }
            if callback.failure_ref != LUA_NOREF {
                state.unref(callback.failure_ref);
            }
            if callback.chained_promise_ref != LUA_NOREF {
                state.unref(callback.chained_promise_ref);
            }
        }
        for callback in self.finally_callbacks.drain(..) {
            if callback.callback_ref != LUA_NOREF {
                state.unref(callback.callback_ref);
            }
            if callback.chained_promise_ref != LUA_NOREF {
                state.unref(callback.chained_promise_ref);
            }
        }
        promise_cleanup_links(state, self);
    }

    pub fn is_fulfilled(&self) -> bool {
        self.promise_state == PromiseState::Fulfilled
    }

    pub fn is_rejected(&self) -> bool {
        self.promise_state == PromiseState::Rejected
    }

    pub fn is_cancelled(&self) -> bool {
        self.promise_state == PromiseState::Cancelled
    }

    pub fn is_pending(&self) -> bool {
        self.promise_state == PromiseState::Pending
    }

    pub fn result_ref(&self) -> i32 {
        self.result_ref
    }
}

impl Drop for ScriptedPromise {
    fn drop(&mut self) {
        if self.state.is_null() {
            return;
        }
        let state = unsafe { &mut *self.state };
        if self.result_ref != LUA_NOREF {
            state.unref(self.result_ref);
        }
        for callback in self.then_callbacks.drain(..) {
            for reference in [
                callback.success_ref,
                callback.failure_ref,
                callback.chained_promise_ref,
                callback.cancel_ref,
            ] {
                if reference != LUA_NOREF {
                    state.unref(reference);
                }
            }
        }
        for callback in self.finally_callbacks.drain(..) {
            for reference in [callback.callback_ref, callback.chained_promise_ref] {
                if reference != LUA_NOREF {
                    state.unref(reference);
                }
            }
        }
        if self.parent_ref != LUA_NOREF {
            state.unref(self.parent_ref);
        }
        for reference in self.consumer_refs.drain(..) {
            if reference != LUA_NOREF {
                state.unref(reference);
            }
        }
        if self.on_cancel_ref != LUA_NOREF {
            state.unref(self.on_cancel_ref);
        }
    }
}

fn promise_notify_callbacks(state: &mut LuaState, promise: &mut ScriptedPromise) {
    if promise.is_cancelled() {
        return;
    }
    let fulfilled = promise.is_fulfilled();
    state.raw_get_i(LUA_REGISTRY_INDEX, promise.result_ref());
    let result_index = state.top();

    for callback in &promise.then_callbacks {
        let mut chained = None;
        if callback.chained_promise_ref != LUA_NOREF {
            state.raw_get_i(LUA_REGISTRY_INDEX, callback.chained_promise_ref);
            chained = state
                .to_rive_optional::<ScriptedPromise>(-1, true)
                .map(|promise| promise as *mut ScriptedPromise);
            state.pop(1);
            if chained
                .map(|promise| unsafe { &*promise }.is_cancelled())
                .unwrap_or(false)
            {
                chained = None;
            }
        }
        let handler_ref = if fulfilled {
            callback.success_ref
        } else {
            callback.failure_ref
        };
        if handler_ref != LUA_NOREF {
            state.raw_get_i(LUA_REGISTRY_INDEX, handler_ref);
            state.push_value(result_index);
            let status = state.pcall(1, 1, 0);
            if status == LUA_OK {
                if let Some(chained) = chained {
                    unsafe { &mut *chained }.resolve_with_self_ref(
                        state,
                        state.top(),
                        callback.chained_promise_ref,
                    );
                }
                state.pop(1);
            } else {
                if let Some(chained) = chained {
                    unsafe { &mut *chained }.reject(state, state.top());
                }
                state.pop(1);
            }
        } else if let Some(chained) = chained {
            if fulfilled {
                unsafe { &mut *chained }.resolve_with_self_ref(
                    state,
                    result_index,
                    callback.chained_promise_ref,
                );
            } else {
                unsafe { &mut *chained }.reject(state, result_index);
            }
        }
        for reference in [
            callback.success_ref,
            callback.failure_ref,
            callback.chained_promise_ref,
            callback.cancel_ref,
        ] {
            if reference != LUA_NOREF {
                state.unref(reference);
            }
        }
    }

    for callback in &promise.finally_callbacks {
        let mut chained = None;
        if callback.chained_promise_ref != LUA_NOREF {
            state.raw_get_i(LUA_REGISTRY_INDEX, callback.chained_promise_ref);
            chained = state
                .to_rive_optional::<ScriptedPromise>(-1, true)
                .map(|promise| promise as *mut ScriptedPromise);
            state.pop(1);
            if chained
                .map(|promise| unsafe { &*promise }.is_cancelled())
                .unwrap_or(false)
            {
                chained = None;
            }
        }
        if callback.callback_ref != LUA_NOREF {
            state.raw_get_i(LUA_REGISTRY_INDEX, callback.callback_ref);
            if state.pcall(0, 0, 0) != LUA_OK {
                if let Some(chained) = chained {
                    unsafe { &mut *chained }.reject(state, state.top());
                }
                state.pop(1);
                state.unref(callback.callback_ref);
                if callback.chained_promise_ref != LUA_NOREF {
                    state.unref(callback.chained_promise_ref);
                }
                continue;
            }
            state.unref(callback.callback_ref);
        }
        if let Some(chained) = chained {
            if fulfilled {
                unsafe { &mut *chained }.resolve(state, result_index);
            } else {
                unsafe { &mut *chained }.reject(state, result_index);
            }
        }
        if callback.chained_promise_ref != LUA_NOREF {
            state.unref(callback.chained_promise_ref);
        }
    }
    state.pop(1);
    promise.then_callbacks.clear();
    promise.finally_callbacks.clear();
}

fn promise_wire_links(
    state: &mut LuaState,
    source: &mut ScriptedPromise,
    source_index: i32,
    chained: &mut ScriptedPromise,
) {
    state.push_value(-1);
    let consumer_ref = state.reference(-1);
    state.pop(1);
    source.consumer_refs.push(consumer_ref);
    state.push_value(source_index);
    chained.parent_ref = state.reference(-1);
    state.pop(1);
}

fn promise_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    let promise = state.to_rive_mut::<ScriptedPromise>(1) as *mut ScriptedPromise;
    match atom {
        LuaAtoms::AndThen | LuaAtoms::Catch | LuaAtoms::Finally => {
            let mut success_ref = LUA_NOREF;
            let mut failure_ref = LUA_NOREF;
            let mut finally_ref = LUA_NOREF;
            if atom == LuaAtoms::AndThen && state.is_function(2) {
                state.push_value(2);
                success_ref = state.reference(-1);
                state.pop(1);
            }
            if ((atom == LuaAtoms::AndThen && state.is_function(3))
                || (atom == LuaAtoms::Catch && state.is_function(2)))
            {
                let index = if atom == LuaAtoms::AndThen { 3 } else { 2 };
                state.push_value(index);
                failure_ref = state.reference(-1);
                state.pop(1);
            }
            if atom == LuaAtoms::Finally && state.is_function(2) {
                state.push_value(2);
                finally_ref = state.reference(-1);
                state.pop(1);
            }
            state.new_rive(ScriptedPromise::new(state.main_thread()));
            let chained = state.to_rive_mut::<ScriptedPromise>(-1) as *mut ScriptedPromise;
            state.push_value(-1);
            let chained_ref = state.reference(-1);
            state.pop(1);
            unsafe { promise_wire_links(state, &mut *promise, 1, &mut *chained) };
            if atom == LuaAtoms::Finally {
                unsafe { &mut *promise }
                    .finally_callbacks
                    .push(FinallyCallback {
                        callback_ref: finally_ref,
                        chained_promise_ref: chained_ref,
                    });
            } else {
                unsafe { &mut *promise }.then_callbacks.push(ThenCallback {
                    success_ref,
                    failure_ref,
                    chained_promise_ref: chained_ref,
                    cancel_ref: LUA_NOREF,
                });
            }
            if !unsafe { &*promise }.is_pending() {
                promise_notify_callbacks(state, unsafe { &mut *promise });
            }
            1
        }
        LuaAtoms::Cancel => {
            unsafe { &mut *promise }.cancel(state);
            state.push_value(1);
            1
        }
        LuaAtoms::OnCancel => {
            if state.is_function(2) {
                let promise = unsafe { &mut *promise };
                if promise.on_cancel_ref != LUA_NOREF {
                    state.unref(promise.on_cancel_ref);
                }
                state.push_value(2);
                promise.on_cancel_ref = state.reference(-1);
                state.pop(1);
            }
            state.push_value(1);
            1
        }
        LuaAtoms::GetStatus => {
            state.push_string(match unsafe { &*promise }.promise_state {
                PromiseState::Pending => "Pending",
                PromiseState::Fulfilled => "Fulfilled",
                PromiseState::Rejected => "Rejected",
                PromiseState::Cancelled => "Cancelled",
            });
            1
        }
        _ => state.error(format!(
            "'{}' is not a valid method of Promise",
            name.unwrap_or_default()
        )),
    }
}

fn promise_static_resolve(state: &mut LuaState) -> i32 {
    state.new_rive(ScriptedPromise::new(state.main_thread()));
    let promise_index = state.top();
    let promise = state.to_rive_mut::<ScriptedPromise>(promise_index) as *mut ScriptedPromise;
    if promise_index >= 2 {
        state.push_value(promise_index);
        let self_ref = state.reference(-1);
        state.pop(1);
        unsafe { &mut *promise }.resolve_with_self_ref(state, 1, self_ref);
        state.unref(self_ref);
    } else {
        state.push_nil();
        unsafe { &mut *promise }.resolve(state, state.top());
        state.pop(1);
    }
    1
}

fn promise_static_reject(state: &mut LuaState) -> i32 {
    state.new_rive(ScriptedPromise::new(state.main_thread()));
    let promise = state.to_rive_mut::<ScriptedPromise>(-1) as *mut ScriptedPromise;
    if state.top() >= 2 {
        unsafe { &mut *promise }.reject(state, 1);
    } else {
        state.push_string("rejected");
        unsafe { &mut *promise }.reject(state, state.top());
        state.pop(1);
    }
    1
}

fn promise_executor_resolve(state: &mut LuaState) -> i32 {
    let promise = state
        .to_rive_optional::<ScriptedPromise>(state.upvalue_index(1), true)
        .map(|promise| promise as *mut ScriptedPromise);
    if let Some(promise) = promise.filter(|promise| unsafe { &**promise }.is_pending()) {
        if state.top() >= 1 {
            state.push_value(state.upvalue_index(1));
            let self_ref = state.reference(-1);
            state.pop(1);
            unsafe { &mut *promise }.resolve_with_self_ref(state, 1, self_ref);
            state.unref(self_ref);
        } else {
            state.push_nil();
            unsafe { &mut *promise }.resolve(state, state.top());
            state.pop(1);
        }
    }
    0
}

fn promise_executor_reject(state: &mut LuaState) -> i32 {
    let promise = state
        .to_rive_optional::<ScriptedPromise>(state.upvalue_index(1), true)
        .map(|promise| promise as *mut ScriptedPromise);
    if let Some(promise) = promise.filter(|promise| unsafe { &**promise }.is_pending()) {
        if state.top() >= 1 {
            unsafe { &mut *promise }.reject(state, 1);
        } else {
            state.push_string("rejected");
            unsafe { &mut *promise }.reject(state, state.top());
            state.pop(1);
        }
    }
    0
}

fn promise_executor_on_cancel(state: &mut LuaState) -> i32 {
    let promise = state
        .to_rive_optional::<ScriptedPromise>(state.upvalue_index(1), true)
        .map(|promise| promise as *mut ScriptedPromise);
    if let Some(promise) = promise.filter(|_| state.is_function(1)) {
        let promise = unsafe { &mut *promise };
        if promise.on_cancel_ref != LUA_NOREF {
            state.unref(promise.on_cancel_ref);
        }
        state.push_value(1);
        promise.on_cancel_ref = state.reference(-1);
        state.pop(1);
    }
    0
}

fn promise_static_new(state: &mut LuaState) -> i32 {
    state.check_type(1, LuaType::Function);
    state.new_rive(ScriptedPromise::new(state.main_thread()));
    let promise_index = state.top();
    let promise = state.to_rive_mut::<ScriptedPromise>(promise_index) as *mut ScriptedPromise;
    state.push_value(promise_index);
    state.push_closure(promise_executor_resolve, 1);
    let resolve_index = state.top();
    state.push_value(promise_index);
    state.push_closure(promise_executor_reject, 1);
    let reject_index = state.top();
    state.push_value(promise_index);
    state.push_closure(promise_executor_on_cancel, 1);
    let on_cancel_index = state.top();
    state.push_value(1);
    state.push_value(resolve_index);
    state.push_value(reject_index);
    state.push_value(on_cancel_index);
    if state.pcall(3, 0, 0) != LUA_OK {
        if unsafe { &*promise }.is_pending() {
            unsafe { &mut *promise }.reject(state, state.top());
        }
        state.pop(1);
    }
    state.push_value(promise_index);
    1
}

fn promise_all_success(state: &mut LuaState) -> i32 {
    let index = state.to_integer(state.upvalue_index(1)) as i32;
    state.push_value(state.upvalue_index(2));
    let state_index = state.top();
    state.get_field(state_index, "done");
    if state.to_boolean(-1) {
        state.pop(2);
        return 0;
    }
    state.pop(1);
    state.get_field(state_index, "results");
    state.push_value(1);
    state.raw_set_i(-2, index);
    state.pop(1);
    state.get_field(state_index, "remaining");
    let remaining = state.to_integer(-1) as i32 - 1;
    state.pop(1);
    state.push_integer(remaining as i64);
    state.set_field(state_index, "remaining");
    if remaining == 0 {
        state.push_boolean(true);
        state.set_field(state_index, "done");
        let result = state
            .to_rive_optional::<ScriptedPromise>(state.upvalue_index(3), true)
            .map(|promise| promise as *mut ScriptedPromise);
        if let Some(result) = result.filter(|promise| unsafe { &**promise }.is_pending()) {
            state.get_field(state_index, "results");
            unsafe { &mut *result }.resolve(state, state.top());
            state.pop(1);
        }
        state.push_value(state.upvalue_index(4));
        let table = state.top();
        let count = state.object_len(table) as i32;
        for index in 1..=count {
            state.raw_get_i(table, index);
            let reference = state.to_integer(-1) as i32;
            state.pop(1);
            if reference != LUA_NOREF {
                state.unref(reference);
            }
        }
        state.pop(1);
    }
    state.pop(1);
    0
}

fn promise_all_failure(state: &mut LuaState) -> i32 {
    state.push_value(state.upvalue_index(1));
    let state_index = state.top();
    state.get_field(state_index, "done");
    if state.to_boolean(-1) {
        state.pop(2);
        return 0;
    }
    state.pop(1);
    state.push_boolean(true);
    state.set_field(state_index, "done");
    let result = state
        .to_rive_optional::<ScriptedPromise>(state.upvalue_index(2), true)
        .map(|promise| promise as *mut ScriptedPromise);
    if let Some(result) = result.filter(|promise| unsafe { &**promise }.is_pending()) {
        unsafe { &mut *result }.reject(state, 1);
    }
    cancel_promise_ref_table(state, state.upvalue_index(3));
    state.pop(1);
    0
}

fn cancel_promise_ref_table(state: &mut LuaState, table_index: i32) {
    state.push_value(table_index);
    let table = state.top();
    let count = state.object_len(table) as i32;
    for index in 1..=count {
        state.raw_get_i(table, index);
        let reference = state.to_integer(-1) as i32;
        state.pop(1);
        if reference != LUA_NOREF {
            state.raw_get_i(LUA_REGISTRY_INDEX, reference);
            let chained = state
                .to_rive_optional::<ScriptedPromise>(-1, true)
                .map(|promise| promise as *mut ScriptedPromise);
            state.pop(1);
            if let Some(chained) = chained.filter(|promise| unsafe { &**promise }.is_pending()) {
                unsafe { &mut *chained }.cancel(state);
            }
            state.unref(reference);
        }
    }
    state.pop(1);
}

fn promise_all_cancel(state: &mut LuaState) -> i32 {
    cancel_promise_ref_table(state, state.upvalue_index(1));
    0
}

fn promise_static_all(state: &mut LuaState) -> i32 {
    state.check_type(1, LuaType::Table);
    let count = state.object_len(1) as i32;
    state.new_rive(ScriptedPromise::new(state.main_thread()));
    let result_index = state.top();
    let result = state.to_rive_mut::<ScriptedPromise>(result_index) as *mut ScriptedPromise;
    if count == 0 {
        state.new_table();
        unsafe { &mut *result }.resolve(state, state.top());
        state.pop(1);
        state.push_value(result_index);
        return 1;
    }
    state.new_table();
    let shared_state_index = state.top();
    state.push_integer(count as i64);
    state.set_field(shared_state_index, "remaining");
    state.new_table();
    state.set_field(shared_state_index, "results");
    state.push_boolean(false);
    state.set_field(shared_state_index, "done");
    state.new_table();
    let chained_refs_index = state.top();

    for index in 1..=count {
        state.raw_get_i(1, index);
        let input = state
            .to_rive_optional::<ScriptedPromise>(-1, true)
            .map(|promise| promise as *mut ScriptedPromise);
        let input_index = state.top();
        let Some(input) = input else {
            state.pop(1);
            return state.error(format!("Promise.all: element {} is not a Promise", index));
        };
        state.new_rive(ScriptedPromise::new(state.main_thread()));
        let chained_index = state.top();
        let chained = state.to_rive_mut::<ScriptedPromise>(chained_index) as *mut ScriptedPromise;
        unsafe { promise_wire_links(state, &mut *input, input_index, &mut *chained) };
        state.push_value(chained_index);
        let chained_ref = state.reference(-1);
        state.pop(1);
        state.push_value(chained_index);
        let cancel_collection_ref = state.reference(-1);
        state.pop(1);
        state.push_integer(cancel_collection_ref as i64);
        state.raw_set_i(chained_refs_index, index);

        state.push_integer(index as i64);
        state.push_value(shared_state_index);
        state.push_value(result_index);
        state.push_value(chained_refs_index);
        state.push_closure(promise_all_success, 4);
        let success_ref = state.reference(-1);
        state.pop(1);

        state.push_value(shared_state_index);
        state.push_value(result_index);
        state.push_value(chained_refs_index);
        state.push_closure(promise_all_failure, 3);
        let failure_ref = state.reference(-1);
        state.pop(1);

        unsafe { &mut *input }.then_callbacks.push(ThenCallback {
            success_ref,
            failure_ref,
            chained_promise_ref: chained_ref,
            cancel_ref: LUA_NOREF,
        });
        if !unsafe { &*input }.is_pending() {
            promise_notify_callbacks(state, unsafe { &mut *input });
        }
        state.pop(2);
    }

    state.push_value(chained_refs_index);
    state.push_closure(promise_all_cancel, 1);
    unsafe { &mut *result }.on_cancel_ref = state.reference(-1);
    state.pop(1);
    state.pop(2);
    state.push_value(result_index);
    1
}

fn promise_table_index(state: &mut LuaState) -> i32 {
    match state.check_string(2).as_str() {
        "resolve" => state.push_named_function(promise_static_resolve, "Promise.resolve"),
        "reject" => state.push_named_function(promise_static_reject, "Promise.reject"),
        "all" => state.push_named_function(promise_static_all, "Promise.all"),
        "new" => state.push_named_function(promise_static_new, "Promise.new"),
        key => return state.error(format!("'{}' is not a valid member of Promise", key)),
    }
    1
}

fn lua_await(state: &mut LuaState) -> i32 {
    let promise = state.to_rive::<ScriptedPromise>(1);
    if !state.is_yieldable() {
        return state.error("await() must be called inside async()");
    }
    if promise.is_fulfilled() {
        state.push_boolean(true);
        state.raw_get_i(LUA_REGISTRY_INDEX, promise.result_ref());
        return 2;
    }
    if promise.is_rejected() {
        state.push_boolean(false);
        state.raw_get_i(LUA_REGISTRY_INDEX, promise.result_ref());
        return 2;
    }
    if promise.is_cancelled() {
        state.push_boolean(false);
        state.push_string("Promise was cancelled");
        return 2;
    }
    state.push_value(1);
    state.yield_values(1)
}

fn lua_async(state: &mut LuaState) -> i32 {
    state.check_type(1, LuaType::Function);
    state.new_rive(ScriptedPromise::new(state.main_thread()));
    let promise_index = state.top();
    let coroutine = state.new_thread();
    unsafe { &mut *coroutine }.set_thread_data(state.thread_data_raw());
    let coroutine_index = state.top();
    state.push_value(1);
    state.move_values(unsafe { &mut *coroutine }, 1);
    let status = unsafe { &mut *coroutine }.resume(state, 0);
    handle_coroutine_completion(state, coroutine, status, coroutine_index, promise_index);
    state.remove(coroutine_index);
    state.push_value(promise_index);
    1
}

fn async_success(state: &mut LuaState) -> i32 {
    let coroutine = state.to_thread(state.upvalue_index(1));
    unsafe { &mut *coroutine }.push_boolean(true);
    state.push_value(1);
    state.move_values(unsafe { &mut *coroutine }, 1);
    let status = unsafe { &mut *coroutine }.resume(state, 2);
    state.push_value(state.upvalue_index(1));
    let coroutine_index = state.top();
    state.push_value(state.upvalue_index(2));
    let async_promise_index = state.top();
    handle_coroutine_completion(
        state,
        coroutine,
        status,
        coroutine_index,
        async_promise_index,
    );
    state.pop(2);
    0
}

fn async_failure(state: &mut LuaState) -> i32 {
    let coroutine = state.to_thread(state.upvalue_index(1));
    unsafe { &mut *coroutine }.push_boolean(false);
    state.push_value(1);
    state.move_values(unsafe { &mut *coroutine }, 1);
    let status = unsafe { &mut *coroutine }.resume(state, 2);
    state.push_value(state.upvalue_index(1));
    let coroutine_index = state.top();
    state.push_value(state.upvalue_index(2));
    let async_promise_index = state.top();
    handle_coroutine_completion(
        state,
        coroutine,
        status,
        coroutine_index,
        async_promise_index,
    );
    state.pop(2);
    0
}

fn async_cancel(state: &mut LuaState) -> i32 {
    let coroutine = state.to_thread(state.upvalue_index(1));
    let promise = state
        .to_rive_optional::<ScriptedPromise>(state.upvalue_index(2), true)
        .map(|promise| promise as *mut ScriptedPromise);
    if !coroutine.is_null() {
        unsafe { &mut *coroutine }.reset_thread();
    }
    if let Some(promise) = promise.filter(|promise| unsafe { &**promise }.is_pending()) {
        state.push_string("Promise was cancelled");
        unsafe { &mut *promise }.reject(state, state.top());
        state.pop(1);
    }
    0
}

fn handle_coroutine_completion(
    state: &mut LuaState,
    coroutine: *mut LuaState,
    status: i32,
    coroutine_index: i32,
    async_promise_index: i32,
) {
    let async_promise =
        state.to_rive_mut::<ScriptedPromise>(async_promise_index) as *mut ScriptedPromise;
    if status == LUA_OK {
        if unsafe { &*coroutine }.top() > 0 {
            unsafe { &mut *coroutine }.move_values(state, 1);
            state.push_value(async_promise_index);
            let self_ref = state.reference(-1);
            state.pop(1);
            unsafe { &mut *async_promise }.resolve_with_self_ref(state, state.top(), self_ref);
            state.unref(self_ref);
            state.pop(1);
        } else {
            state.push_nil();
            unsafe { &mut *async_promise }.resolve(state, state.top());
            state.pop(1);
        }
    } else if status == LUA_YIELD {
        if unsafe { &*coroutine }.top() < 1 {
            state.error("async: coroutine yielded without a promise");
            return;
        }
        let awaited = unsafe { &mut *coroutine }
            .to_rive_optional::<ScriptedPromise>(-1, true)
            .map(|promise| promise as *mut ScriptedPromise);
        let Some(awaited) = awaited else {
            state.error("async: await() argument is not a Promise");
            return;
        };
        unsafe { &mut *coroutine }.pop(1);

        state.push_value(coroutine_index);
        state.push_value(async_promise_index);
        state.push_closure(async_success, 2);
        let success_ref = state.reference(-1);
        state.pop(1);
        state.push_value(coroutine_index);
        state.push_value(async_promise_index);
        state.push_closure(async_failure, 2);
        let failure_ref = state.reference(-1);
        state.pop(1);
        state.push_value(coroutine_index);
        state.push_value(async_promise_index);
        state.push_closure(async_cancel, 2);
        let cancel_ref = state.reference(-1);
        state.pop(1);
        unsafe { &mut *awaited }.then_callbacks.push(ThenCallback {
            success_ref,
            failure_ref,
            chained_promise_ref: LUA_NOREF,
            cancel_ref,
        });
        if !unsafe { &*awaited }.is_pending() {
            promise_notify_callbacks(state, unsafe { &mut *awaited });
        }
    } else if unsafe { &*coroutine }.top() > 0 {
        unsafe { &mut *coroutine }.move_values(state, 1);
        unsafe { &mut *async_promise }.reject(state, state.top());
        state.pop(1);
    } else {
        state.push_string("async function errored");
        unsafe { &mut *async_promise }.reject(state, state.top());
        state.pop(1);
    }
}

pub fn luaopen_rive_promise(state: &mut LuaState) -> i32 {
    state.register_rive::<ScriptedPromise>();
    state.push_function(promise_namecall);
    state.set_field(-2, "__namecall");
    state.set_readonly(-1, true);
    state.pop(1);

    state.new_table();
    state.new_table();
    state.push_function(promise_table_index);
    state.set_field(-2, "__index");
    state.set_metatable(-2);
    state.set_global("Promise");

    state.push_named_function(lua_await, "await");
    state.set_global("await");
    state.push_named_function(lua_async, "async");
    state.set_global("async");
    0
}
