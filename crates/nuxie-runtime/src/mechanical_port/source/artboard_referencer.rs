use std::ptr::NonNull;

use crate::mechanical_port::source::{
    artboard::Artboard,
    core::Core,
    file::File,
    generated::{
        nested_artboard_base::NestedArtboardBase,
        nested_artboard_layout_base::NestedArtboardLayoutBase,
        nested_artboard_leaf_base::NestedArtboardLeafBase,
        script_input_artboard_base::ScriptInputArtboardBase,
    },
    viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
};

#[derive(Default)]
pub struct ArtboardReferencer {
    referenced_artboard: Option<NonNull<Artboard>>,
}

impl ArtboardReferencer {
    pub fn find_artboard(
        view_model_instance_artboard: Option<NonNull<ViewModelInstanceArtboard>>,
        parent_artboard: Option<NonNull<Artboard>>,
        file: Option<NonNull<File>>,
    ) -> Option<NonNull<Artboard>> {
        let mut view_model_instance_artboard = view_model_instance_artboard?;
        let view_model_instance_artboard = unsafe { view_model_instance_artboard.as_mut() };

        if let Some(mut asset) = view_model_instance_artboard.asset() {
            let artboard = NonNull::from(asset.artboard());
            if parent_artboard
                .is_none_or(|parent| unsafe { !parent.as_ref().is_ancestor(artboard.as_ref()) })
            {
                return Some(artboard);
            }
            return None;
        }

        if let Some(mut file) = file {
            let property_value = view_model_instance_artboard.base.property_value();
            if let Some(artboard) = unsafe { file.as_mut() }.artboard(property_value as usize) {
                if parent_artboard
                    .is_none_or(|parent| unsafe { !parent.as_ref().is_ancestor(artboard.as_ref()) })
                {
                    return Some(artboard);
                }
            }
        }
        None
    }

    pub fn referenced_artboard(&self) -> Option<NonNull<Artboard>> {
        self.referenced_artboard
    }

    pub fn set_referenced_artboard(&mut self, artboard: Option<NonNull<Artboard>>) {
        self.referenced_artboard = artboard;
    }

    pub fn from(
        component: &mut dyn CoreArtboardReferencer,
    ) -> Option<&mut dyn ArtboardReferencerBehavior> {
        match component.core_type() {
            NestedArtboardBase::TYPE_KEY
            | NestedArtboardLeafBase::TYPE_KEY
            | NestedArtboardLayoutBase::TYPE_KEY
            | ScriptInputArtboardBase::TYPE_KEY => Some(component),
            _ => None,
        }
    }
}

pub trait ArtboardReferencerBehavior {
    fn artboard_referencer(&self) -> &ArtboardReferencer;
    fn artboard_referencer_mut(&mut self) -> &mut ArtboardReferencer;
    fn update_artboard(
        &mut self,
        view_model_instance_artboard: Option<NonNull<ViewModelInstanceArtboard>>,
    );
    fn referenced_artboard_id(&self) -> i32;

    fn set_referenced_artboard(&mut self, artboard: Option<NonNull<Artboard>>) {
        self.artboard_referencer_mut()
            .set_referenced_artboard(artboard);
    }
}

pub trait CoreArtboardReferencer: ArtboardReferencerBehavior {
    fn core(&mut self) -> &mut Core;
    fn core_type(&self) -> u16;
}
