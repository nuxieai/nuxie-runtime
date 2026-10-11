//! Artboard::drawContent's drawables in draw order, as a flat list built from
//! the drawables' links. The links change only where the Artboard sorts or
//! splices its draw order, which bumps the version here; the list is rebuilt
//! from the links on the next draw. Each entry keeps the cells of a plain
//! Shape so its draw needs no borrow of the shape.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    drawable::{Drawable, DrawableCells, RuntimeDrawableOccurrence, RuntimeDrawableWeakOccurrence},
    generated::shapes::shape_base::ShapeBase,
    shapes::shape_cells::ShapeCells,
};

#[derive(Default)]
pub(crate) struct DrawLinks {
    version: Cell<u64>,
    list: RefCell<Option<Rc<DrawList>>>,
    // The version the last draw without a list saw. A list is built only
    // once the links have held across a draw, so a draw order rewritten
    // every frame keeps the plain link walk.
    unlisted: Cell<Option<u64>>,
}

impl DrawLinks {
    /// The drawables' links or the first drawable changed.
    pub(crate) fn relinked(&self) {
        self.version.set(self.version.get().wrapping_add(1));
        self.list.borrow_mut().take();
    }

    /// Mark the links changed now and again when the returned guard drops,
    /// around code that rewrites them.
    pub(crate) fn relinking(self: &Rc<Self>) -> Relinking {
        self.relinked();
        Relinking(Rc::clone(self))
    }

    #[inline]
    pub(crate) fn version(&self) -> u64 {
        self.version.get()
    }

    /// The list from `first` under the current links, building it when the
    /// links changed since the last build and have held across a draw. None
    /// for the first draw under new links, or when a callback relinked the
    /// drawables while the list was being built.
    pub(crate) fn list_from(&self, first: &RuntimeDrawableOccurrence) -> Option<Rc<DrawList>> {
        if let Some(list) = self.list.borrow().as_ref()
            && list.version == self.version.get()
            && list.starts_at(first)
        {
            return Some(Rc::clone(list));
        }
        let version = self.version.get();
        if self.unlisted.replace(Some(version)) != Some(version) {
            return None;
        }
        let list = Rc::new(DrawList::build(first, version));
        if self.version.get() != version {
            return None;
        }
        *self.list.borrow_mut() = Some(Rc::clone(&list));
        Some(list)
    }
}

pub(crate) struct Relinking(Rc<DrawLinks>);

impl Drop for Relinking {
    fn drop(&mut self) {
        self.0.relinked();
    }
}

pub(crate) struct DrawList {
    pub(crate) version: u64,
    pub(crate) entries: Vec<DrawEntry>,
}

pub(crate) struct DrawEntry {
    occurrence: RuntimeDrawableWeakOccurrence,
    /// A Shape's cells, for the draw that reads them instead of the shape.
    pub(crate) shape: Option<ShapeDraw>,
}

pub(crate) struct ShapeDraw {
    pub(crate) cells: Rc<ShapeCells>,
    pub(crate) drawable: DrawableCells,
}

impl DrawEntry {
    /// The occurrence the link walk reaches at this entry; None where that
    /// walk's link upgrade fails.
    #[inline]
    pub(crate) fn occurrence(&self) -> Option<RuntimeDrawableOccurrence> {
        self.occurrence.upgrade()
    }

    fn of(drawable: &RuntimeDrawableOccurrence) -> Self {
        let shape = match drawable {
            RuntimeDrawableOccurrence::Authored(handle)
                if handle.occupied_core_type() == Some(ShapeBase::TYPE_KEY) =>
            {
                handle
                    .with(|object| {
                        object.as_shape().map(|shape| ShapeDraw {
                            cells: Rc::clone(shape.cells()),
                            drawable: shape.base.draw_cells(),
                        })
                    })
                    .flatten()
            }
            _ => None,
        };
        Self {
            occurrence: drawable.downgrade(),
            shape,
        }
    }
}

impl DrawList {
    /// Follow the links from `first` as drawDrawableRange does.
    fn build(first: &RuntimeDrawableOccurrence, version: u64) -> Self {
        let mut entries = Vec::new();
        let mut drawable = Some(first.clone());
        while let Some(current) = drawable {
            entries.push(DrawEntry::of(&current));
            drawable = current.with(Drawable::prev_drawable_for_draw_range).flatten();
        }
        Self { version, entries }
    }

    fn starts_at(&self, first: &RuntimeDrawableOccurrence) -> bool {
        self.entries
            .first()
            .and_then(DrawEntry::occurrence)
            .is_some_and(|start| &start == first)
    }
}
