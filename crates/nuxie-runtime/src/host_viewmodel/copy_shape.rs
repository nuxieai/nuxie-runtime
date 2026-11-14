//! The measure of a view-model copy, taken before the copy is made.

/// What a copy of an authored instance would make.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ViewModelCopyShape {
    /// Instances, the copied one and every nested one it reaches.
    pub instances: usize,
    /// Property values, over all of them.
    pub properties: usize,
    /// The bytes of their property names.
    pub name_bytes: usize,
    /// The bytes of their string values.
    pub string_bytes: usize,
    /// List items, over all of their lists.
    pub list_items: usize,
}

/// [`crate::ScriptViewModel::named_copy_shape`]'s answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyShape {
    /// The model has no authored instance of that name.
    NoTemplate,
    /// The copy reaches more instances than the walk was allowed.
    Unbounded,
    /// What the copy would make.
    Shape(ViewModelCopyShape),
}
