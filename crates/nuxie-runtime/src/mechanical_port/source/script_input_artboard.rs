use std::ptr::NonNull;

use crate::mechanical_port::source::{
    artboard_referencer::{ArtboardReferencer, ArtboardReferencerBehavior, CoreArtboardReferencer},
    assets::script_asset::{ScriptInput, ScriptInputBehavior},
    core::Core,
    core_context::CoreContext,
    custom_property::CustomProperty,
    file::File,
    generated::{
        backboard_base::BackboardBase, script_input_artboard_base::ScriptInputArtboardBase,
        script_input_artboard_base::ScriptInputArtboardBaseCallbacks,
        scripted::scripted_drawable_base::ScriptedDrawableBase,
    },
    importers::{
        backboard_importer::BackboardImporter, import_stack::ImportStack,
        scripted_object_importer::ScriptedObjectImporter,
    },
    status_code::StatusCode,
    viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
};

pub struct ScriptInputArtboard {
    pub base: ScriptInputArtboardBase,
    script_input: ScriptInput,
    artboard_referencer: ArtboardReferencer,
    file: Option<NonNull<File>>,
}

impl Default for ScriptInputArtboard {
    fn default() -> Self {
        Self {
            base: ScriptInputArtboardBase::default(),
            script_input: ScriptInput::default(),
            artboard_referencer: ArtboardReferencer::default(),
            file: None,
        }
    }
}

impl ScriptInputArtboard {
    fn custom_property_mut(&mut self) -> &mut CustomProperty {
        &mut self.base.base
    }

    fn name(&self) -> &str {
        self.base.base.base.base.base.name()
    }

    pub fn set_file(&mut self, value: Option<NonNull<File>>) {
        self.file = value;
    }

    pub fn init_scripted_value(&mut self) {
        self.sync_referenced_artboard();
    }

    pub fn validate_for_script_init(&self) -> bool {
        self.artboard_referencer.referenced_artboard().is_some()
    }

    pub fn validate_for_cold_script_init(&self) -> bool {
        true
    }

    pub fn validate_hydration_prerequisites(&self) -> bool {
        self.artboard_referencer.referenced_artboard().is_some()
    }

    pub fn hydrate_script_input(&mut self) -> bool {
        self.init_scripted_value();
        self.artboard_referencer.referenced_artboard().is_some()
    }

    fn sync_referenced_artboard(&mut self) {
        let Some(referenced_artboard) = self.artboard_referencer.referenced_artboard() else {
            return;
        };
        let name = self.name().to_owned();
        if let Some(mut object) = self.script_input.scripted_object() {
            unsafe { object.as_mut() }
                .set_artboard_input(name, referenced_artboard.as_ptr() as usize);
        }
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(backboard_importer) =
            import_stack.latest::<BackboardImporter>(BackboardBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let referencer: &mut dyn ArtboardReferencerBehavior = self;
        backboard_importer.add_artboard_referencer(NonNull::from(referencer));

        let Some(importer) =
            import_stack.latest::<ScriptedObjectImporter>(ScriptedDrawableBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        importer.add_input(
            NonNull::from(self.custom_property_mut()),
            ScriptInputArtboardBase::TYPE_KEY.into(),
        );

        if self
            .script_input
            .scripted_object()
            .is_some_and(|object| unsafe { object.as_ref() }.component().is_some())
        {
            return self.base.base.base.base.import(import_stack);
        }
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }

        let property = self.custom_property_mut() as *mut CustomProperty;
        if let Some(parent) = self
            .base
            .base
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_scripted_object_mut())
        {
            parent.add_property(property);
        }
        StatusCode::Ok
    }

    pub fn clone(&self) -> Self {
        let mut twin = Self::default();
        let twin_base = &mut twin.base as *mut ScriptInputArtboardBase;
        unsafe { twin_base.as_mut().unwrap() }.copy(&self.base, &mut twin);
        if let Some(referenced_artboard) = self.artboard_referencer.referenced_artboard() {
            twin.artboard_referencer
                .set_referenced_artboard(Some(referenced_artboard));
            twin.file = self.file;
        }
        twin
    }

    pub fn artboard_id_changed(&mut self) {
        let Some(mut file) = self.file else {
            return;
        };
        self.artboard_referencer.set_referenced_artboard(
            unsafe { file.as_mut() }.artboard(self.base.artboard_id() as usize),
        );
        self.sync_referenced_artboard();
    }

    pub fn update_artboard(
        &mut self,
        view_model_instance_artboard: Option<NonNull<ViewModelInstanceArtboard>>,
    ) {
        let parent_artboard = self.base.base.base.base.artboard_mut().map(NonNull::from);
        if let Some(referenced_artboard) = ArtboardReferencer::find_artboard(
            view_model_instance_artboard,
            parent_artboard,
            self.file,
        ) {
            self.artboard_referencer
                .set_referenced_artboard(Some(referenced_artboard));
            self.sync_referenced_artboard();
        }
    }

    pub fn referenced_artboard_id(&self) -> i32 {
        self.base.artboard_id() as i32
    }
}

impl ScriptInputBehavior for ScriptInputArtboard {
    fn script_input(&self) -> &ScriptInput {
        &self.script_input
    }

    fn script_input_mut(&mut self) -> &mut ScriptInput {
        &mut self.script_input
    }

    fn validate_for_script_init(&self) -> bool {
        ScriptInputArtboard::validate_for_script_init(self)
    }

    fn init_scripted_value(&mut self) {
        ScriptInputArtboard::init_scripted_value(self);
    }

    fn validate_for_cold_script_init(&self) -> bool {
        ScriptInputArtboard::validate_for_cold_script_init(self)
    }

    fn hydrate_script_input(&mut self) -> bool {
        ScriptInputArtboard::hydrate_script_input(self)
    }

    fn validate_hydration_prerequisites(&self) -> bool {
        ScriptInputArtboard::validate_hydration_prerequisites(self)
    }
}

impl ArtboardReferencerBehavior for ScriptInputArtboard {
    fn artboard_referencer(&self) -> &ArtboardReferencer {
        &self.artboard_referencer
    }

    fn artboard_referencer_mut(&mut self) -> &mut ArtboardReferencer {
        &mut self.artboard_referencer
    }

    fn update_artboard(
        &mut self,
        view_model_instance_artboard: Option<NonNull<ViewModelInstanceArtboard>>,
    ) {
        ScriptInputArtboard::update_artboard(self, view_model_instance_artboard);
    }

    fn referenced_artboard_id(&self) -> i32 {
        ScriptInputArtboard::referenced_artboard_id(self)
    }
}

impl CoreArtboardReferencer for ScriptInputArtboard {
    fn core(&mut self) -> &mut Core {
        &mut self.base.base.base.base.base.base
    }

    fn core_type(&self) -> u16 {
        ScriptInputArtboardBase::TYPE_KEY
    }
}

impl ScriptInputArtboardBaseCallbacks for ScriptInputArtboard {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn artboard_id_changed(&mut self) {
        ScriptInputArtboard::artboard_id_changed(self);
    }
}

impl Drop for ScriptInputArtboard {
    fn drop(&mut self) {
        let property = self.custom_property_mut() as *mut CustomProperty;
        if let Some(mut object) = self.script_input.scripted_object() {
            unsafe { object.as_mut() }.remove_property(property);
        }
        self.artboard_referencer.set_referenced_artboard(None);
    }
}
