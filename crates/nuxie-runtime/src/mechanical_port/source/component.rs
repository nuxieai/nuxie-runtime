use crate::mechanical_port::source::{
    artboard::Artboard,
    component_dirt::ComponentDirt,
    container_component::ContainerComponent,
    core::CoreHandle,
    core_context::CoreContext,
    dependency_helper::{DependencyHelper, DirtDependent},
    generated::{
        artboard_base::ArtboardBase,
        component_base::{ComponentBase, ComponentBaseCallbacks},
        container_component_base::ContainerComponentBase,
        core_registry::CoreCapabilities,
    },
    importers::{artboard_importer::ArtboardImporter, import_stack::ImportStack},
    math::vec2d::Vec2D,
    status_code::StatusCode,
};

pub struct Component {
    pub base: ComponentBase,
    dependency_helper: DependencyHelper<Component>,
    parent: Option<CoreHandle>,
    graph_order: u32,
    artboard: Option<CoreHandle>,
    collapsables: Vec<CoreHandle>,
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
            collapsables: Vec::new(),
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
    pub fn dependency_root(&self) -> Option<CoreHandle> {
        self.artboard.clone()
    }

    pub fn artboard_handle(&self) -> Option<CoreHandle> {
        self.artboard.clone()
    }

    pub fn with_artboard<R>(&self, use_artboard: impl FnOnce(&Artboard) -> R) -> Option<R> {
        self.artboard
            .as_ref()?
            .with_downcast::<Artboard, _>(use_artboard)
    }

    pub fn with_artboard_mut<R>(&self, use_artboard: impl FnOnce(&mut Artboard) -> R) -> Option<R> {
        self.artboard
            .as_ref()?
            .with_downcast_mut::<Artboard, _>(use_artboard)
    }

    pub fn parent_handle(&self) -> Option<CoreHandle> {
        self.parent.clone()
    }

    pub fn with_parent<R>(&self, use_parent: impl FnOnce(&ContainerComponent) -> R) -> Option<R> {
        self.parent
            .as_ref()?
            .with(|parent| parent.as_container_component().map(use_parent))?
    }

    pub fn with_parent_mut<R>(
        &self,
        use_parent: impl FnOnce(&mut ContainerComponent) -> R,
    ) -> Option<R> {
        self.parent
            .as_ref()?
            .with_mut(|parent| parent.as_container_component_mut().map(use_parent))?
    }

    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        context
            .resolve(self.base.parent_id())
            .is_some_and(|object| object.is_type_of(ContainerComponentBase::TYPE_KEY))
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        self.artboard = context.resolve_handle(0);
        if self.artboard.as_ref() == Some(&this) {
            return StatusCode::Ok;
        }
        self.parent = context
            .resolve(self.base.parent_id())
            .filter(|object| object.is_type_of(ContainerComponentBase::TYPE_KEY));
        let Some(parent) = self.parent.as_ref() else {
            return StatusCode::MissingObject;
        };
        let added = parent
            .with_mut(|parent| {
                parent
                    .as_container_component_mut()
                    .map(|parent| parent.add_child(this))
            })
            .flatten()
            .is_some();
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }

    pub fn add_collapsable(&mut self, collapsable: CoreHandle) {
        if !self.collapsables.contains(&collapsable) {
            self.collapsables.push(collapsable.clone());
            collapsable.with_mut(|collapsable| {
                if let Some(collapsable) = collapsable.as_data_bind_mut() {
                    collapsable.collapse(self.is_collapsed());
                }
            });
        }
    }

    pub fn build_dependencies(&mut self) {}

    pub fn on_dirty(&mut self, _dirt: ComponentDirt) {}

    pub fn update(&mut self, _value: ComponentDirt) {}

    pub fn graph_order(&self) -> u32 {
        self.graph_order
    }

    pub fn set_graph_order(&mut self, value: u32) {
        self.graph_order = value;
    }

    pub fn dirt(&self) -> ComponentDirt {
        self.dirt
    }

    pub fn set_dirt(&mut self, value: ComponentDirt) {
        self.dirt = value;
    }

    /// Mutate only retained dirt state. The central virtual action sequences
    /// callbacks, Artboard notification, and recursion after this borrow ends.
    pub fn add_dirt_state(&mut self, value: ComponentDirt) -> Option<ComponentDirt> {
        if self.dirt.contains(value) {
            return None;
        }
        self.dirt |= value;
        Some(self.dirt)
    }

    /// Mutate only the retained collapsed bit. Most-derived propagation is
    /// sequenced by the central virtual action after this borrow ends.
    pub fn collapse_state(&mut self, value: bool) -> Option<ComponentDirt> {
        if self.is_collapsed() == value {
            return None;
        }
        if value {
            self.dirt |= ComponentDirt::COLLAPSED;
        } else {
            self.dirt &= !ComponentDirt::COLLAPSED;
        }
        Some(self.dirt)
    }

    pub fn collapsables_snapshot(&self) -> Vec<CoreHandle> {
        self.collapsables.clone()
    }

    pub fn dependents_snapshot(&self) -> Vec<CoreHandle> {
        self.dependency_helper.dependents().to_vec()
    }

    pub fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        CoreCapabilities::component_add_dirt(self, value, recurse)
    }

    pub fn has_dirt(&self, flag: ComponentDirt) -> bool {
        self.dirt.contains(flag)
    }

    pub fn has_dirt_in(value: ComponentDirt, flag: ComponentDirt) -> bool {
        (value & flag) != ComponentDirt::NONE
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        if self.base.base.is_type_of(ArtboardBase::TYPE_KEY) {
            return self.base.base.import(import_stack);
        }
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };

        let Some(artboard_importer) =
            import_stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        artboard_importer.add_component(Some(this));
        self.base.base.import(import_stack)
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        CoreCapabilities::component_collapse(self, value)
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
        self.parent
            .as_ref()
            .and_then(|parent| {
                parent
                    .with_mut(|parent| {
                        parent.component_hit_test_point(position, skip_on_unclipped, false)
                    })
                    .flatten()
            })
            .unwrap_or(true)
    }

    pub fn dependents(&self) -> &[CoreHandle] {
        self.dependency_helper.dependents()
    }

    pub(crate) fn update_collapsables(&mut self) {
        let collapsed = self.is_collapsed();
        for collapsable in self.collapsables.iter().cloned() {
            collapsable.with_mut(|collapsable| {
                if let Some(collapsable) = collapsable.as_data_bind_mut() {
                    collapsable.collapse(collapsed);
                }
            });
        }
    }

    pub fn add_dependent(&mut self, dependent: CoreHandle) {
        self.dependency_helper.add_dependent(dependent);
    }

    pub fn remove_dependent(&mut self, dependent: &CoreHandle) {
        self.dependency_helper.remove_dependent(dependent);
    }
}

impl DirtDependent for Component {
    fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) {
        CoreCapabilities::component_add_dirt(self, value, recurse);
    }
}

impl std::ops::Deref for Component {
    type Target = ComponentBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Component {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
