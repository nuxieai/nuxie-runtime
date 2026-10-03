//! Artboard::drawDrawableRange receiver boundaries (upstream 53419065).
use nuxie_render_api::NullRenderer;
use nuxie_runtime::source::{
    artboard::Artboard,
    core::{
        Core, CoreArena, CoreObject, PropertySetterCompletion, binary_reader::BinaryReader,
        field_types::core_callback_type::CallbackData,
    },
    drawable::{Drawable, RuntimeDrawableOccurrence},
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        drawable_base::DrawableBase,
    },
    renderer::Renderer,
    shapes::shape::Shape,
};
use std::{any::Any, cell::RefCell, rc::Rc};

// One virtual Drawable observes the actual range walk. Inherited registry
// operations use a real Shape; only the draw predicates and draw are probes.
struct ProbeDrawable {
    shape: Shape,
    events: Rc<RefCell<Vec<&'static str>>>,
    empty_delta: i32,
    visible: bool,
    clip_start: bool,
    draws: usize,
}

impl ProbeDrawable {
    const TYPE_KEY: u16 = 65534;

    fn subtype(key: u16) -> bool {
        key == Self::TYPE_KEY || DrawableBase::is_type_of(key)
    }

    fn read_self_again(&self) {
        let owner = self.core().handle().unwrap();
        assert_eq!(owner.with(|owner| owner.as_any().is::<Self>()), Some(true));
    }
}

impl CoreCapabilities for ProbeDrawable {
    fn as_drawable(&self) -> Option<&Drawable> {
        self.events.borrow_mut().push("prev");
        self.shape.as_drawable()
    }

    fn drawable_empty_clip_count(&mut self) -> i32 {
        self.events.borrow_mut().push("empty");
        self.empty_delta
    }

    fn drawable_will_draw(&self) -> bool {
        self.events.borrow_mut().push("will_draw");
        // The shared virtual getter can re-enter the same receiver for a
        // shared read. Grouping it under a mutable loan would panic here.
        self.read_self_again();
        self.visible
    }

    fn drawable_is_clip_start(&self) -> bool {
        self.events.borrow_mut().push("clip_start");
        self.read_self_again();
        self.clip_start
    }

    fn drawable_is_clip_end(&self) -> bool {
        panic!("isClipEnd must not run without pending clips")
    }

    fn drawable_draw(&mut self, _: &mut Renderer) -> bool {
        // Reaching this mutable dispatch proves the shared prelude loan ended.
        self.draws += 1;
        self.events.borrow_mut().push("draw");
        true
    }
}

impl CoreObject for ProbeDrawable {
    fn core(&self) -> &Core {
        self.shape.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.shape.core_mut()
    }
    fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.shape.deserialize(key, reader)
    }
}

impl CoreRegistryObject for ProbeDrawable {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(&mut self, f: CoreField, v: u32, c: &mut PropertySetterCompletion) {
        self.shape.set_uint_with_completion(f, v, c);
    }
    fn set_string_with_completion(
        &mut self,
        f: CoreField,
        v: String,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_string_with_completion(f, v, c);
    }
    fn set_color_with_completion(
        &mut self,
        f: CoreField,
        v: i32,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_color_with_completion(f, v, c);
    }
    fn set_bool_with_completion(
        &mut self,
        f: CoreField,
        v: bool,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_bool_with_completion(f, v, c);
    }
    fn set_double_with_completion(
        &mut self,
        f: CoreField,
        v: f32,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_double_with_completion(f, v, c);
    }
    fn set_callback_with_completion(
        &mut self,
        f: CoreField,
        v: CallbackData<'_>,
        c: &mut PropertySetterCompletion,
    ) {
        self.shape.set_callback_with_completion(f, v, c);
    }
    fn set_int_with_completion(&mut self, f: CoreField, v: i32, c: &mut PropertySetterCompletion) {
        self.shape.set_int_with_completion(f, v, c);
    }
    fn get_uint(&mut self, f: CoreField) -> u32 {
        self.shape.get_uint(f)
    }
    fn get_string(&mut self, f: CoreField) -> String {
        self.shape.get_string(f)
    }
    fn get_color(&mut self, f: CoreField) -> i32 {
        self.shape.get_color(f)
    }
    fn get_bool(&mut self, f: CoreField) -> bool {
        self.shape.get_bool(f)
    }
    fn get_double(&mut self, f: CoreField) -> f32 {
        self.shape.get_double(f)
    }
    fn get_int(&mut self, f: CoreField) -> i32 {
        self.shape.get_int(f)
    }
}

#[test]
fn draw_range_preserves_shared_getters_order_short_circuits_and_draw_release() {
    let cases: &[(i32, bool, bool, &[&str])] = &[
        (
            0,
            true,
            false,
            &["empty", "will_draw", "clip_start", "draw", "prev"],
        ),
        (0, false, false, &["empty", "will_draw", "prev"]),
        // willDraw is still called before the empty-clip suppression test.
        (1, true, false, &["empty", "will_draw", "prev"]),
        (0, true, true, &["empty", "will_draw", "clip_start", "prev"]),
    ];
    for &(empty_delta, visible, clip_start, expected) in cases {
        let arena = CoreArena::default();
        let root = arena.insert(Artboard::default());
        let events = Rc::new(RefCell::new(Vec::new()));
        let drawable = arena.insert(ProbeDrawable {
            shape: Shape::default(),
            events: events.clone(),
            empty_delta,
            visible,
            clip_start,
            draws: 0,
        });
        Artboard::draw_drawable_range_handle(
            &root,
            &mut NullRenderer::new(),
            Some(RuntimeDrawableOccurrence::Authored(drawable.clone())),
            None,
        );
        assert_eq!(events.borrow().as_slice(), expected);
        assert_eq!(
            drawable.with_downcast::<ProbeDrawable, _>(|owner| owner.draws),
            Some(usize::from(expected.contains(&"draw"))),
        );
    }
}
