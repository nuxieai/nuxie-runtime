pub const NONE: u32 = 0;
pub const DEPENDENTS: u32 = 1;
pub const BINDINGS: u32 = 2;
pub const BINDINGS_TARGET: u32 = 4;
pub trait ContainerDataBind {
    fn unbind(&mut self);
    fn bind_from_context(&mut self, context: *mut ());
    fn advance(&mut self, elapsed: f32) -> bool;
    fn to_source(&self) -> bool;
    fn target_supports_push(&self) -> bool;
    fn in_persisting_list(&self) -> bool;
    fn set_in_persisting_list(&mut self, value: bool);
    fn in_dirty_list(&self) -> bool;
    fn set_in_dirty_list(&mut self, value: bool);
    fn set_container(&mut self, value: Option<*mut DataBindContainer>);
    fn dirt(&self) -> u32;
    fn set_dirt(&mut self, value: u32);
    fn update_dependents(&mut self);
    fn source_to_target_runs_first(&self) -> bool;
    fn update_source_binding(&mut self);
    fn update(&mut self, dirt: u32);
    fn can_skip(&self) -> bool;
}
#[derive(Default)]
pub struct DataBindContainer {
    data_binds: Vec<*mut dyn ContainerDataBind>,
    persisting: Vec<*mut dyn ContainerDataBind>,
    dirty_to_source: Vec<*mut dyn ContainerDataBind>,
    pending_dirty_to_source: Vec<*mut dyn ContainerDataBind>,
    dirty: Vec<*mut dyn ContainerDataBind>,
    pending_dirty: Vec<*mut dyn ContainerDataBind>,
    pending_additions: Vec<*mut dyn ContainerDataBind>,
    pending_removals: Vec<*mut dyn ContainerDataBind>,
    data_context: Option<*mut ()>,
    is_processing: bool,
}
impl DataBindContainer {
    pub fn delete_data_binds(&mut self) {
        for bind in self.data_binds.drain(..) {
            unsafe {
                drop(Box::from_raw(bind));
            }
        }
    }

    pub fn unbind_data_binds(&mut self) {
        for bind in &self.data_binds {
            unsafe {
                (&mut **bind).unbind();
            }
        }
        self.data_context = None
    }
    pub fn bind_data_binds_from_context(&mut self, context: *mut ()) {
        for bind in &self.data_binds {
            unsafe {
                (&mut **bind).bind_from_context(context);
            }
        }
        self.data_context = Some(context)
    }
    pub fn advance_data_binds(&mut self, elapsed: f32) -> bool {
        let mut updated = false;
        for bind in &self.data_binds {
            if unsafe { (&mut **bind).advance(elapsed) } {
                updated = true;
            }
        }
        updated
    }
    fn erase(list: &mut Vec<*mut dyn ContainerDataBind>, bind: *mut dyn ContainerDataBind) {
        list.retain(|item| !core::ptr::addr_eq(*item, bind));
    }
    pub fn remove_data_bind(&mut self, bind: *mut dyn ContainerDataBind) {
        if self.is_processing {
            self.pending_removals.push(bind);
            return;
        }
        Self::erase(&mut self.data_binds, bind);
        unsafe {
            if (&*bind).in_persisting_list() {
                Self::erase(&mut self.persisting, bind);
                (&mut *bind).set_in_persisting_list(false);
            }
            if (&*bind).in_dirty_list() {
                Self::erase(&mut self.dirty_to_source, bind);
                Self::erase(&mut self.pending_dirty_to_source, bind);
                Self::erase(&mut self.dirty, bind);
                Self::erase(&mut self.pending_dirty, bind);
                (&mut *bind).set_in_dirty_list(false);
            }
            (&mut *bind).set_container(None);
        }
    }
    pub fn add_data_bind(&mut self, bind: *mut dyn ContainerDataBind) {
        if self.is_processing {
            self.pending_additions.push(bind);
            return;
        }
        self.data_binds.push(bind);
        unsafe {
            if (&*bind).to_source() && !(&*bind).target_supports_push() {
                self.persisting.push(bind);
                (&mut *bind).set_in_persisting_list(true);
            }
            (&mut *bind).set_container(Some(self as *mut Self));
            if self.data_context.is_some() {
                (&mut *bind).bind_from_context(self.data_context.unwrap());
                self.update_data_bind(bind, true);
            }
        }
    }
    fn update_data_bind(&mut self, bind: *mut dyn ContainerDataBind, apply_target_to_source: bool) {
        unsafe {
            let bind = &mut *bind;
            let dirt = bind.dirt();
            if dirt & DEPENDENTS == DEPENDENTS {
                bind.update_dependents();
            }
            let wants = apply_target_to_source
                && (bind.in_persisting_list() || dirt & BINDINGS_TARGET == BINDINGS_TARGET);
            if wants && !bind.source_to_target_runs_first() {
                bind.update_source_binding();
            }
            if dirt != NONE {
                bind.set_dirt(NONE);
                bind.update(dirt);
            }
            if wants && bind.source_to_target_runs_first() {
                bind.update_source_binding();
            }
        }
    }
    pub fn update_data_binds(&mut self, apply_target_to_source: bool) {
        if self.is_processing {
            return;
        }
        if self.persisting.is_empty() && self.dirty_to_source.is_empty() && self.dirty.is_empty() {
            return;
        }
        self.is_processing = true;
        for bind in self.persisting.clone() {
            if !unsafe { (&*bind).can_skip() } {
                self.update_data_bind(bind, apply_target_to_source);
            }
        }
        for bind in self.dirty_to_source.clone() {
            unsafe {
                (&mut *bind).set_in_dirty_list(false);
            }
            self.update_data_bind(bind, apply_target_to_source);
        }
        for bind in self.dirty.clone() {
            unsafe {
                (&mut *bind).set_in_dirty_list(false);
            }
            self.update_data_bind(bind, apply_target_to_source);
        }
        self.dirty_to_source.clear();
        self.dirty.clear();
        if !self.pending_dirty_to_source.is_empty() {
            core::mem::swap(&mut self.dirty_to_source, &mut self.pending_dirty_to_source);
        }
        if !self.pending_dirty.is_empty() {
            core::mem::swap(&mut self.dirty, &mut self.pending_dirty);
        }
        self.is_processing = false;
        for bind in core::mem::take(&mut self.pending_additions) {
            self.add_data_bind(bind);
        }
        for bind in core::mem::take(&mut self.pending_removals) {
            self.remove_data_bind(bind);
        }
    }
    pub fn sort_data_binds(&mut self) {
        let mut to_source = 0;
        for index in 0..self.data_binds.len() {
            if unsafe { (&*self.data_binds[index]).to_source() } {
                if index != to_source {
                    self.data_binds.swap(to_source, index);
                }
                to_source += 1;
            }
        }
    }
    pub fn add_dirty_data_bind(&mut self, bind: *mut dyn ContainerDataBind) {
        unsafe {
            if (&*bind).to_source() && (&*bind).in_persisting_list() {
                return;
            }
            if (&*bind).in_dirty_list() {
                return;
            }
            let list = if (&*bind).to_source() {
                if self.is_processing {
                    &mut self.pending_dirty_to_source
                } else {
                    &mut self.dirty_to_source
                }
            } else if self.is_processing {
                &mut self.pending_dirty
            } else {
                &mut self.dirty
            };
            list.push(bind);
            (&mut *bind).set_in_dirty_list(true);
        }
    }
    pub fn data_binds(&self) -> Vec<*mut dyn ContainerDataBind> {
        self.data_binds.clone()
    }

    pub fn rebind(&mut self) {}

    pub fn relink_data_context(&mut self) {}

    pub fn rebuild_data_bind(&mut self, _data_bind: *mut dyn ContainerDataBind) {}
}
