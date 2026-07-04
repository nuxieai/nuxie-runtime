#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelNumberSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelNumberSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelBooleanSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelBooleanSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelStringSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelStringSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelColorSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelColorSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelEnumSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelEnumSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelSymbolListIndexSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelSymbolListIndexSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelAssetSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelAssetSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelArtboardSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelArtboardSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelTriggerSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelTriggerSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelListSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelListSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultViewModelViewModelSourceHandle {
    pub(crate) path: Vec<u32>,
}

impl RuntimeDefaultViewModelViewModelSourceHandle {
    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelNumberSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelNumberSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelBooleanSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelBooleanSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelStringSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelStringSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelColorSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelColorSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelEnumSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelEnumSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelSymbolListIndexSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelSymbolListIndexSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelAssetSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelAssetSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelArtboardSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelArtboardSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelTriggerSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelTriggerSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelListSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelListSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImportedViewModelViewModelSourceHandle {
    pub(crate) view_model_index: usize,
    pub(crate) instance_index: usize,
    pub(crate) path: Vec<u32>,
}

impl RuntimeImportedViewModelViewModelSourceHandle {
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }

    pub fn instance_index(&self) -> usize {
        self.instance_index
    }

    pub fn path(&self) -> &[u32] {
        &self.path
    }
}
