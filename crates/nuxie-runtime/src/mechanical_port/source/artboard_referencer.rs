use crate::mechanical_port::source::{
    artboard::Artboard,
    core::{Core, CoreHandle},
    file::RuntimeFileWeakHandle,
    generated::{
        nested_artboard_base::NestedArtboardBase,
        nested_artboard_layout_base::NestedArtboardLayoutBase,
        nested_artboard_leaf_base::NestedArtboardLeafBase,
        script_input_artboard_base::ScriptInputArtboardBase,
    },
};

#[derive(Default)]
pub struct ArtboardReferencer {
    referenced_artboard: Option<CoreHandle>,
}

impl ArtboardReferencer {
    pub fn find_artboard(
        view_model_instance_artboard: Option<CoreHandle>,
        parent_artboard: Option<CoreHandle>,
        file: Option<RuntimeFileWeakHandle>,
    ) -> Option<CoreHandle> {
        let value = view_model_instance_artboard?;
        let (asset, property_value) = value
            .with(|value| {
                let value = value.as_view_model_instance_artboard()?;
                Some((value.asset(), value.base.property_value()))
            })
            .flatten()?;
        let candidate = asset
            .and_then(|asset| asset.source_artboard_handle())
            .or_else(|| {
                file.and_then(|file| {
                    file.with_file(|file| file.artboard_handle(property_value as usize))
                        .flatten()
                })
            })?;
        let is_ancestor = parent_artboard.as_ref().is_some_and(|parent| {
            parent
                .with_downcast_mut::<Artboard, _>(|parent| {
                    parent.is_ancestor(Some(candidate.clone()))
                })
                .unwrap_or(false)
        });
        (!is_ancestor).then_some(candidate)
    }

    pub fn referenced_artboard(&self) -> Option<CoreHandle> {
        self.referenced_artboard.clone()
    }

    pub fn set_referenced_artboard(&mut self, artboard: Option<CoreHandle>) {
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
    fn update_artboard(&mut self, view_model_instance_artboard: Option<CoreHandle>);
    fn referenced_artboard_id(&self) -> i32;

    fn set_referenced_artboard(&mut self, artboard: Option<CoreHandle>) {
        self.artboard_referencer_mut()
            .set_referenced_artboard(artboard);
    }
}

pub trait CoreArtboardReferencer: ArtboardReferencerBehavior {
    fn core(&mut self) -> &mut Core;
    fn core_type(&self) -> u16;
}
