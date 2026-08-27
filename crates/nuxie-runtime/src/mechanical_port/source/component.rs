use std::ptr::NonNull;

use crate::mechanical_port::source::{
    artboard::Artboard,
    component_dirt::ComponentDirt,
    container_component::ContainerComponent,
    core_context::CoreContext,
    data_bind::data_bind::DataBind,
    dependency_helper::{DependencyHelper, DirtDependent},
    generated::{
        artboard_base::ArtboardBase,
        component_base::{ComponentBase, ComponentBaseCallbacks},
    },
    importers::{artboard_importer::ArtboardImporter, import_stack::ImportStack},
    lazy_vector::LazyVector,
    math::vec2d::Vec2D,
    status_code::StatusCode,
};

pub struct Component {
    pub base: ComponentBase,
    dependency_helper: DependencyHelper<Component>,
    parent: Option<*mut ContainerComponent>,
    graph_order: u32,
    artboard: Option<*mut Artboard>,
    collapsables: LazyVector<*mut DataBind>,
    dirt: ComponentDirt,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            base: ComponentBase::default(),
            dependency_helper: DependencyHelper::default(),
            parent: None,
            graph_order: 0,
            artboard: None,
            collapsables: LazyVector::default(),
            dirt: ComponentDirt::FILTHY,
        }
    }
}

impl ComponentBaseCallbacks for Component {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}

impl Component {
    pub fn dependency_root(&self) -> Option<*mut Artboard> {
        self.artboard
    }

    pub fn artboard(&self) -> Option<&Artboard> {
        self.artboard.map(|artboard| unsafe { &*artboard })
    }

    pub fn artboard_mut(&mut self) -> Option<&mut Artboard> {
        self.artboard.map(|artboard| unsafe { &mut *artboard })
    }

    pub fn parent(&self) -> Option<&ContainerComponent> {
        self.parent.map(|parent| unsafe { &*parent })
    }

    pub fn parent_mut(&mut self) -> Option<&mut ContainerComponent> {
        self.parent.map(|parent| unsafe { &mut *parent })
    }

    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        context
            .resolve(self.base.parent_id())
            .is_some_and(|object| object.is_container_component())
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.artboard = context
            .artboard_mut()
            .map(|artboard| artboard as *mut Artboard);
        if self
            .artboard
            .is_some_and(|artboard| std::ptr::eq(self, artboard.cast::<Component>()))
        {
            return StatusCode::Ok;
        }
        self.parent = context
            .resolve(self.base.parent_id())
            .and_then(|object| object.as_container_component_mut())
            .map(|parent| parent as *mut ContainerComponent);
        let Some(parent) = self.parent else {
            return StatusCode::MissingObject;
        };
        unsafe { &mut *parent }.add_child(self as *mut Component);
        StatusCode::Ok
    }

    pub fn add_collapsable(&mut self, collapsable: *mut DataBind) {
        let size_before = self.collapsables.size();
        self.collapsables.push_unique(collapsable);
        if self.collapsables.size() != size_before {
            unsafe { &mut *collapsable }.collapse(self.is_collapsed());
        }
    }

    pub fn build_dependencies(&mut self) {}

    pub fn on_dirty(&mut self, _dirt: ComponentDirt) {}

    pub fn update(&mut self, _value: ComponentDirt) {}

    pub fn graph_order(&self) -> u32 {
        self.graph_order
    }

    pub fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        if self.dirt.contains(value) {
            return false;
        }

        self.dirt |= value;
        self.on_dirty(self.dirt);
        if let Some(artboard) = self.artboard_mut() {
            artboard.on_component_dirty(self);
        }

        if recurse {
            self.dependency_helper.add_dirt_to_dependents(value);
        }
        true
    }

    pub fn has_dirt(&self, flag: ComponentDirt) -> bool {
        self.dirt.contains(flag)
    }

    pub fn has_dirt_in(value: ComponentDirt, flag: ComponentDirt) -> bool {
        (value & flag) != ComponentDirt::NONE
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        if self.base.base.is_type_of(ArtboardBase::TYPE_KEY) {
            let artboard = unsafe { &mut *(self as *mut Component).cast::<Artboard>() };
            assert!(artboard.objects().is_empty());
            artboard.add_object(Some(NonNull::from(&mut self.base.base)));
            return self.base.base.import(import_stack);
        }

        let Some(artboard_importer) =
            import_stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        artboard_importer.add_component(Some(NonNull::from(&mut self.base.base)));
        self.base.base.import(import_stack)
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        if self.dirt.contains(ComponentDirt::COLLAPSED) == value {
            return false;
        }
        if value {
            self.dirt |= ComponentDirt::COLLAPSED;
        } else {
            self.dirt &= !ComponentDirt::COLLAPSED;
        }
        self.on_dirty(self.dirt);
        if let Some(artboard) = self.artboard_mut() {
            artboard.on_component_dirty(self);
        }
        self.update_collapsables();
        true
    }

    pub fn is_collapsed(&self) -> bool {
        self.dirt.contains(ComponentDirt::COLLAPSED)
    }

    pub fn hit_test_point(
        &self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        _is_primary_hit: bool,
    ) -> bool {
        if let Some(parent) = self.parent() {
            return parent.hit_test_point(position, skip_on_unclipped, false);
        }
        true
    }

    pub fn dependents(&self) -> &[*mut Component] {
        self.dependency_helper.dependents()
    }

    fn update_collapsables(&mut self) {
        let collapsed = self.is_collapsed();
        for collapsable in self.collapsables.view().iter().copied() {
            unsafe { &mut *collapsable }.collapse(collapsed);
        }
    }
}

impl DirtDependent for Component {
    fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) {
        Component::add_dirt(self, value, recurse);
    }
}
