//! Text field looks and texts through the C edge, on pages written by hand
//! here, each field shaped as the compiler shapes an `input`: a box holding
//! a placeholder holder (a text, its style, its fill and its runs) and a
//! `words` box holding the field (its focus, its style with a `wght` axis,
//! its text drawable with a solid fill, its selection and its caret), with
//! the semantic record on the box.
//!
//! Oracle: each fixture's records. Every expected value below is one the
//! fixture writes: the field's text, font size, letter spacing, weight,
//! font asset index, colors, alignment, limit and flags, and the
//! placeholder's runs and color.
use super::*;
use nuxie_schema::definition_by_name;
use std::cell::{Cell, RefCell};
use std::ffi::c_void;

/// One property value as the file writes it.
#[derive(Clone, Copy)]
enum V {
    U(u64),
    F(f32),
    S(&'static str),
    C(u32),
    B(&'static [u8]),
}

fn var(b: &mut Vec<u8>, mut n: u64) {
    while n >= 128 {
        b.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    b.push(n as u8);
}

/// The key of `kind`'s property `name`, its own or an ancestor's.
fn key(kind: &str, name: &str) -> u16 {
    let definition = definition_by_name(kind).unwrap_or_else(|| panic!("type {kind}"));
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(definition_by_name)
        .flat_map(|definition| definition.properties)
        .find(|property| property.name == name)
        .unwrap_or_else(|| panic!("{kind}.{name}"))
        .key
        .int
}

/// One object of `kind` with `props`, then its terminator.
fn obj(b: &mut Vec<u8>, kind: &str, props: &[(&str, V)]) {
    let kind_key = definition_by_name(kind)
        .unwrap_or_else(|| panic!("type {kind}"))
        .type_key
        .int;
    var(b, kind_key.into());
    for (name, value) in props {
        var(b, key(kind, name).into());
        match value {
            V::U(n) => var(b, *n),
            V::F(v) => b.extend(v.to_le_bytes()),
            V::S(s) => {
                var(b, s.len() as u64);
                b.extend(s.as_bytes());
            }
            V::C(c) => b.extend(c.to_le_bytes()),
            V::B(bytes) => {
                var(b, bytes.len() as u64);
                b.extend(*bytes);
            }
        }
    }
    var(b, 0);
}

const FONT: &[u8] = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");

/// The page's file-global artboard id: the Backboard is record 0 and the
/// two fonts, each an asset and its contents, are records 1 to 4, in a
/// file with no view model.
const PAGE_GLOBAL_ID: u32 = 5;

/// The field's text paint, the placeholder's (the same gray at a third of
/// its strength) and the selection's.
const INK: u32 = 0xff1d_1d1f;
const HINT: u32 = 0x551d_1d1f;
const SELECTION: u32 = 0x331d_1d1f;

/// A field as a fixture writes it.
#[derive(Clone, Copy)]
struct Spec {
    text: &'static str,
    /// Whether it has a placeholder: "Your " and "name", in two runs.
    placeholder: bool,
    /// Its style's `wght`, when it sets one.
    weight: Option<f32>,
    /// The index of its font in the file's assets, when it names one.
    font: Option<u64>,
    obscured: bool,
    /// Whether its semantic record is disabled, as the compiler marks a
    /// disabled `input`.
    disabled: bool,
    /// Its FocusData's flags: 7 can focus, be touched and be reached in
    /// order; 6 cannot focus.
    focus_flags: u64,
    /// Whether its style's font is bound to the view model's `font`, which
    /// names the file's first font.
    font_bound: bool,
}

/// "Ada" in the second font at 17 points, spaced 0.5, weight 600,
/// centered, at most 20 characters, with its placeholder.
const AT_REST: Spec = Spec {
    text: "Ada",
    placeholder: true,
    weight: Some(600.0),
    font: Some(1),
    obscured: false,
    disabled: false,
    focus_flags: 7,
    font_bound: false,
};

fn size(width: f32, height: f32) -> Vec<(&'static str, V)> {
    use V::*;
    vec![
        ("widthKind", U(1)),
        ("width", F(width)),
        ("heightKind", U(1)),
        ("height", F(height)),
    ]
}

/// Absolute, at (`left`, `top`) in its containing block.
fn at(left: f32, top: f32, width: f32, height: f32) -> Vec<(&'static str, V)> {
    use V::*;
    let mut css = vec![
        ("position", U(2)),
        ("insetLeftKind", U(1)),
        ("insetLeft", F(left)),
        ("insetTopKind", U(1)),
        ("insetTop", F(top)),
    ];
    css.extend(size(width, height));
    css
}

/// One artboard's objects, its local ids counted as they are written.
struct Board {
    b: Vec<u8>,
    count: usize,
    style_base: usize,
    styled: Vec<usize>,
    ids: Vec<(&'static str, usize)>,
}

impl Board {
    fn add(&mut self, kind: &str, props: &[(&str, V)]) -> usize {
        obj(&mut self.b, kind, props);
        self.count += 1;
        self.count - 1
    }

    fn named(&mut self, name: &'static str, kind: &str, props: &[(&str, V)]) -> usize {
        let id = self.add(kind, props);
        self.ids.push((name, id));
        id
    }

    fn next_style(&mut self, owner: usize) -> u64 {
        self.styled.push(owner);
        (self.style_base + self.styled.len() - 1) as u64
    }

    /// A CSS box under `parent`: a LayoutComponent and its record.
    fn boxed(&mut self, name: &'static str, parent: usize, css: Vec<(&'static str, V)>) -> usize {
        use V::*;
        let id = self.count;
        let style = self.next_style(id);
        self.named(
            name,
            "LayoutComponent",
            &[
                ("name", S(name)),
                ("parentId", U(parent as u64)),
                ("styleId", U(style)),
            ],
        );
        let mut record = vec![("parentId", U(id as u64))];
        record.extend(css);
        self.add("LayoutCssStyle", &record);
        id
    }

    /// A solid fill of `color` under `parent`; the color is named `name`.
    fn fill(&mut self, name: &'static str, parent: usize, color: u32) {
        use V::*;
        let fill = self.add("Fill", &[("parentId", U(parent as u64))]);
        self.named(
            name,
            "SolidColor",
            &[("parentId", U(fill as u64)), ("colorValue", C(color))],
        );
    }

    /// A field as `spec` writes it, under `parent`, its box at (16, 100).
    fn field(&mut self, parent: usize, spec: Spec) {
        use V::*;
        let font = spec.font.map(|index| ("fontAssetId", U(index)));
        let field = self.boxed("Field", parent, at(16.0, 100.0, 200.0, 44.0));
        let mut record = vec![
            ("parentId", U(field as u64)),
            // A text field, which can be disabled.
            ("role", U(6)),
            ("traitFlags", U(1 << 5)),
        ];
        if spec.disabled {
            record.push(("stateFlags", U(1 << 6)));
        }
        self.add("SemanticData", &record);
        if spec.placeholder {
            let holder = self.boxed("Placeholder", field, at(0.0, 0.0, 200.0, 44.0));
            let hint = self.named("Hint", "Text", &[("parentId", U(holder as u64))]);
            let mut paint = vec![("parentId", U(hint as u64)), ("fontSize", F(17.0))];
            paint.extend(font);
            let style = self.add("TextStylePaint", &paint);
            self.fill("Hint color", style, HINT);
            for words in ["Your ", "name"] {
                self.add(
                    "TextValueRun",
                    &[
                        ("parentId", U(hint as u64)),
                        ("styleId", U(style as u64)),
                        ("text", S(words)),
                    ],
                );
            }
        }
        let words = self.boxed("Words", field, size(200.0, 44.0));
        let input = self.named(
            "Input",
            "TextInput",
            &[
                ("name", S("Input")),
                ("parentId", U(words as u64)),
                ("text", S(spec.text)),
                ("multiline", U(0)),
                ("maxCharacters", U(20)),
                ("alignValue", U(2)),
                ("obscured", U(u64::from(spec.obscured))),
            ],
        );
        self.add(
            "FocusData",
            &[
                ("parentId", U(input as u64)),
                ("focusFlags", U(spec.focus_flags)),
            ],
        );
        let mut paint = vec![
            ("parentId", U(input as u64)),
            ("fontSize", F(17.0)),
            ("letterSpacing", F(0.5)),
        ];
        paint.extend(font);
        let style = self.named("Input style", "TextStylePaint", &paint);
        if spec.font_bound {
            // The style's font, bound to property 0 of view model 0. A
            // binding is not one of the artboard's objects and takes no id.
            obj(
                &mut self.b,
                "DataBindContext",
                &[
                    (
                        "propertyKey",
                        U(u64::from(key("TextStylePaint", "fontAssetId"))),
                    ),
                    ("sourcePathIds", B(&[0, 0])),
                ],
            );
        }
        if let Some(weight) = spec.weight {
            self.named(
                "Weight",
                "TextStyleAxis",
                &[
                    ("parentId", U(style as u64)),
                    ("tag", U(u64::from(u32::from_be_bytes(*b"wght")))),
                    ("axisValue", F(weight)),
                ],
            );
        }
        let text = self.add("TextInputText", &[("parentId", U(input as u64))]);
        self.fill("Input color", text, INK);
        let selection = self.add("TextInputSelection", &[("parentId", U(input as u64))]);
        self.fill("Selection color", selection, SELECTION);
        let caret = self.add("TextInputCursor", &[("parentId", U(input as u64))]);
        self.fill("Caret color", caret, INK);
    }
}

/// What an artboard holds.
enum Holds {
    /// A field.
    Field(Spec),
    /// A keyed list, "Cards", whose rows are the artboard at this index.
    Rows(u64),
}

/// One artboard: its name, its size and what it holds.
struct Artboard {
    name: &'static str,
    size: (f32, f32),
    holds: Holds,
}

/// Whether `artboard` holds a field whose font is bound.
fn binds_font(artboard: &Artboard) -> bool {
    matches!(artboard.holds, Holds::Field(spec) if spec.font_bound)
}

fn board(artboard: &Artboard, style_base: usize) -> Board {
    use V::*;
    let mut board = Board {
        b: Vec::new(),
        count: 0,
        style_base,
        styled: Vec::new(),
        ids: Vec::new(),
    };
    let style = board.next_style(0);
    let mut record = vec![
        ("name", S(artboard.name)),
        ("width", F(artboard.size.0)),
        ("height", F(artboard.size.1)),
        ("styleId", U(style)),
        ("clip", U(0)),
    ];
    if binds_font(artboard) {
        record.push(("viewModelId", U(0)));
    }
    board.add("Artboard", &record);
    board.add("LayoutCssStyle", &[("parentId", U(0))]);
    match artboard.holds {
        Holds::Field(spec) => board.field(0, spec),
        Holds::Rows(rows) => {
            board.named(
                "Cards",
                "ArtboardComponentList",
                &[
                    ("name", S("Cards")),
                    ("parentId", U(0)),
                    ("rowArtboardId", U(rows)),
                ],
            );
        }
    }
    board
}

/// A file of two fonts and `artboards`, the first with a state machine,
/// and the local ids of each artboard's named objects.
fn file(artboards: &[Artboard]) -> (Vec<u8>, Vec<Vec<(&'static str, usize)>>) {
    let mut b = b"RIVE".to_vec();
    for n in [7, 3, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    for asset in 0..2 {
        obj(&mut b, "FontAsset", &[("assetId", V::U(asset))]);
        obj(&mut b, "FileAssetContents", &[("bytes", V::B(FONT))]);
    }
    if artboards.iter().any(binds_font) {
        // A view model whose `font` names the file's first font.
        obj(&mut b, "ViewModel", &[("name", V::S("Field"))]);
        obj(
            &mut b,
            "ViewModelPropertyAssetFont",
            &[("name", V::S("font"))],
        );
        obj(&mut b, "ViewModelInstance", &[("viewModelId", V::U(0))]);
        obj(
            &mut b,
            "ViewModelInstanceAssetFont",
            &[("viewModelPropertyId", V::U(0)), ("propertyValue", V::U(0))],
        );
    }
    let mut ids = Vec::new();
    for (index, artboard) in artboards.iter().enumerate() {
        // The styles follow every other object; count those first.
        let objects = board(artboard, 0).count;
        let mut written = board(artboard, objects);
        for owner in written.styled.clone() {
            written.add("LayoutComponentStyle", &[("parentId", V::U(owner as u64))]);
        }
        b.extend(written.b);
        if index == 0 {
            // A state machine with one empty layer, so the player takes
            // focus, and a text listener on the field, so typing reaches
            // it: a field takes text only when a text listener opts it in.
            obj(&mut b, "StateMachine", &[("name", V::S("machine"))]);
            if let Some(&(_, input)) = written.ids.iter().find(|(name, _)| *name == "Input") {
                obj(
                    &mut b,
                    "StateMachineListenerSingle",
                    &[
                        ("targetId", V::U(input as u64)),
                        ("listenerTypeValue", V::U(8)),
                    ],
                );
            }
            obj(&mut b, "StateMachineLayer", &[("name", V::S("layer"))]);
            for state in ["EntryState", "AnyState", "ExitState"] {
                obj(&mut b, state, &[]);
            }
        }
        ids.push(written.ids);
    }
    (b, ids)
}

fn page(spec: Spec) -> Artboard {
    Artboard {
        name: "Page",
        size: (400.0, 600.0),
        holds: Holds::Field(spec),
    }
}

fn sized<T>() -> u32 {
    std::mem::size_of::<T>() as u32
}

fn bytes(key: &'static str) -> NuxByteView {
    NuxByteView {
        data: key.as_ptr(),
        len: key.len(),
    }
}

/// An object to name, owning the path and row keys that name it.
struct Object {
    local_id: usize,
    path: Vec<NuxScenePathSegment>,
    keys: Vec<NuxSceneRowKey>,
}

impl Object {
    /// An object of the player's own artboard.
    fn root(local_id: usize) -> Self {
        Self {
            local_id,
            path: Vec::new(),
            keys: Vec::new(),
        }
    }

    /// An object of the row `key` of the page's list `list`.
    fn in_row(list: usize, key: &'static str, local_id: usize) -> Self {
        Self {
            local_id,
            path: vec![NuxScenePathSegment {
                artboard_global_id: PAGE_GLOBAL_ID,
                local_id: list,
            }],
            keys: vec![NuxSceneRowKey {
                artboard_global_id: PAGE_GLOBAL_ID,
                host_local_id: list,
                key: bytes(key),
            }],
        }
    }

    fn raw(&self) -> NuxSceneObject {
        NuxSceneObject {
            struct_size: sized::<NuxSceneObject>(),
            local_id: self.local_id,
            path: if self.path.is_empty() {
                ptr::null()
            } else {
                self.path.as_ptr()
            },
            path_len: self.path.len(),
            row_keys: if self.keys.is_empty() {
                ptr::null()
            } else {
                self.keys.as_ptr()
            },
            row_key_count: self.keys.len(),
        }
    }
}

/// A record naming `field` and, when given, `placeholder`.
fn record(field: &Object, placeholder: Option<&Object>) -> NuxTextFieldObjects {
    NuxTextFieldObjects {
        struct_size: sized::<NuxTextFieldObjects>(),
        field: field.raw(),
        has_placeholder: u32::from(placeholder.is_some()),
        placeholder: placeholder.map_or_else(|| Object::root(0).raw(), Object::raw),
    }
}

/// One write a driver makes in the next step's first round.
type Write = Box<dyn FnOnce(*mut NuxSceneWriter) -> NuxStatus>;

/// What a driver does in the next step's first round: rows it mounts in
/// the list, then writes.
#[derive(Default)]
struct Drive {
    list: Cell<usize>,
    mount: RefCell<Vec<&'static str>>,
    writes: RefCell<Vec<Write>>,
}

unsafe extern "C" fn drive(
    context: *mut c_void,
    round: *const NuxPlayerDriverRound,
    writer: *mut NuxSceneWriter,
) -> NuxStatus {
    let drive = unsafe { &*context.cast::<Drive>() };
    if unsafe { (*round).index } != 0 {
        return NuxStatus::Ok;
    }
    let list = Object::root(drive.list.get()).raw();
    for key in drive.mount.take() {
        let status =
            unsafe { nux_scene_writer_mount_row(writer, &list, bytes(key), ptr::null_mut()) };
        if status != NuxStatus::Ok {
            return status;
        }
    }
    for write in drive.writes.take() {
        let status = write(writer);
        if status != NuxStatus::Ok {
            return status;
        }
    }
    NuxStatus::Ok
}

/// `object`'s text written as `text`.
fn write_text(object: Object, text: &'static str) -> Write {
    Box::new(move |writer: *mut NuxSceneWriter| unsafe {
        nux_scene_writer_set_text(
            writer,
            &object.raw(),
            key("TextInput", "text"),
            NuxStringView {
                data: text.as_ptr().cast(),
                len: text.len(),
            },
            ptr::null_mut(),
        )
    })
}

/// `object`'s number property `key` written as `value`.
fn write_number(object: Object, key: u16, value: f32) -> Write {
    Box::new(move |writer: *mut NuxSceneWriter| unsafe {
        nux_scene_writer_set_number(writer, &object.raw(), key, value, ptr::null_mut())
    })
}

/// `object`'s color written as `value`.
fn write_color(object: Object, value: u32) -> Write {
    Box::new(move |writer: *mut NuxSceneWriter| unsafe {
        nux_scene_writer_set_color(
            writer,
            &object.raw(),
            key("SolidColor", "colorValue"),
            value,
            ptr::null_mut(),
        )
    })
}

/// The default player on artboard 0 of a file, with a driver.
struct Played {
    file: *mut NuxFile,
    instance: *mut NuxArtboardInstance,
    /// The view model bound to the page, when a field's font is bound.
    model: *mut NuxViewModelInstance,
    player: *mut NuxPlayer,
    drive: Box<Drive>,
    ids: Vec<Vec<(&'static str, usize)>>,
}

impl Played {
    fn new(artboards: &[Artboard]) -> Self {
        let (bytes, ids) = file(artboards);
        let mut file = ptr::null_mut();
        let mut instance = ptr::null_mut();
        let mut model = ptr::null_mut();
        let mut player = ptr::null_mut();
        let driving = Box::new(Drive::default());
        unsafe {
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            if artboards.iter().any(binds_font) {
                assert_eq!(
                    nux_view_model_instance_new_default(instance, &mut model),
                    NuxStatus::Ok
                );
                assert_eq!(
                    nux_artboard_instance_bind_view_model(instance, model),
                    NuxStatus::Ok
                );
            }
            assert_eq!(nux_player_new_default(instance, &mut player), NuxStatus::Ok);
            let driver = NuxPlayerDriver {
                struct_size: sized::<NuxPlayerDriver>(),
                context: (&raw const *driving).cast_mut().cast(),
                drive: Some(drive),
                release: None,
            };
            assert_eq!(nux_player_driver_set(player, &driver), NuxStatus::Ok);
        }
        Self {
            file,
            instance,
            model,
            player,
            drive: driving,
            ids,
        }
    }

    /// The local id of the object `name` in artboard `index`.
    fn id(&self, index: usize, name: &str) -> usize {
        self.ids[index]
            .iter()
            .find(|(item, _)| *item == name)
            .unwrap_or_else(|| panic!("no object {name}"))
            .1
    }

    /// The object `name` of the page.
    fn object(&self, name: &str) -> Object {
        Object::root(self.id(0, name))
    }

    /// Writes the next step's driver makes.
    fn write(&self, write: Write) {
        self.drive.writes.borrow_mut().push(write);
    }

    /// Steps until the scene settles, presenting each, and returns the last.
    fn settle(&self) -> Step {
        for _ in 0..4 {
            self.step(&[]).present(self.player);
        }
        self.step(&[])
    }

    fn step(&self, focus: &[NuxPlayerFocusInput]) -> Step {
        let step = NuxPlayerStep {
            elapsed_seconds: 1.0 / 60.0,
            focus_inputs: if focus.is_empty() {
                ptr::null()
            } else {
                focus.as_ptr()
            },
            focus_input_count: focus.len(),
            ..NuxPlayerStep::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(self.player, &step, &mut result) },
            NuxStatus::Ok
        );
        Step(result)
    }

    fn read(
        &self,
        step: &Step,
        fields: &[NuxTextFieldObjects],
    ) -> Result<Vec<NuxTextFieldLook>, NuxStatus> {
        let mut out = vec![
            NuxTextFieldLook {
                struct_size: sized::<NuxTextFieldLook>(),
                ..Default::default()
            };
            fields.len()
        ];
        let status = unsafe {
            nux_player_text_fields(
                self.player,
                step.0,
                fields.as_ptr(),
                fields.len(),
                out.as_mut_ptr(),
            )
        };
        if status == NuxStatus::Ok {
            Ok(out)
        } else {
            Err(status)
        }
    }

    /// The page's field's look, with its placeholder when it has one.
    fn look(&self, step: &Step) -> NuxTextFieldLook {
        let field = self.object("Input");
        let hint = self.ids[0]
            .iter()
            .any(|(name, _)| *name == "Hint")
            .then(|| self.object("Hint"));
        self.read(step, &[record(&field, hint.as_ref())])
            .unwrap_or_else(|status| panic!("the field: {status:?}"))[0]
    }

    /// `object`'s text, copied through a buffer of exactly its length.
    fn copy(&self, step: &Step, object: &Object) -> Result<String, NuxStatus> {
        let raw = object.raw();
        let mut length = 0;
        let status = unsafe {
            nux_player_scene_text_copy(self.player, step.0, &raw, ptr::null_mut(), 0, &mut length)
        };
        if status != NuxStatus::Ok {
            return Err(status);
        }
        let mut buffer = vec![0u8; length];
        let status = unsafe {
            nux_player_scene_text_copy(
                self.player,
                step.0,
                &raw,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut length,
            )
        };
        if status != NuxStatus::Ok {
            return Err(status);
        }
        assert_eq!(length, buffer.len());
        Ok(String::from_utf8(buffer).expect("UTF-8"))
    }
}

impl Drop for Played {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(nux_player_free(self.player), NuxStatus::Ok);
            if !self.model.is_null() {
                assert_eq!(nux_view_model_instance_free(self.model), NuxStatus::Ok);
            }
            assert_eq!(nux_artboard_instance_free(self.instance), NuxStatus::Ok);
            assert_eq!(nux_file_free(self.file), NuxStatus::Ok);
        }
    }
}

struct Step(*mut NuxPlayerStepResult);

impl Step {
    fn present(&self, player: *mut NuxPlayer) {
        unsafe {
            assert_eq!(
                nux_player_acknowledge_presented(player, self.revision()),
                NuxStatus::Ok
            );
        }
    }

    fn revision(&self) -> u64 {
        let mut scheduling = NuxPlayerSchedulingInfo {
            struct_size: sized::<NuxPlayerSchedulingInfo>(),
            ..Default::default()
        };
        assert_eq!(
            unsafe { nux_player_step_result_scheduling(self.0, &mut scheduling) },
            NuxStatus::Ok
        );
        scheduling.render_revision
    }
}

impl Drop for Step {
    fn drop(&mut self) {
        assert_eq!(
            unsafe { nux_player_step_result_free(self.0) },
            NuxStatus::Ok
        );
    }
}

/// The field at rest reads what the fixture writes: "Ada", 3 bytes, in
/// font 1 of the two at 17 points, spaced 0.5, weight 600, in INK,
/// centered (2), at most 20 characters, shown in the clear and enabled;
/// its placeholder "Your " + "name", 9 bytes, in HINT.
#[test]
fn a_field_at_rest_reads_its_look() {
    let scene = Played::new(&[page(AT_REST)]);
    let step = scene.settle();
    let look = scene.look(&step);
    assert_eq!(
        look,
        NuxTextFieldLook {
            struct_size: sized::<NuxTextFieldLook>(),
            render_revision: step.revision(),
            font_asset: 1,
            font_from_host: 0,
            font_size: 17.0,
            has_weight: 1,
            weight: 600.0,
            letter_spacing: 0.5,
            color: INK,
            has_placeholder: 1,
            placeholder_color: HINT,
            align: 2,
            obscured: 0,
            max_characters: 20,
            disabled: 0,
            value_length: 3,
            placeholder_length: 9,
        }
    );
    assert_eq!(
        scene.copy(&step, &scene.object("Input")).as_deref(),
        Ok("Ada")
    );
    assert_eq!(
        scene.copy(&step, &scene.object("Hint")).as_deref(),
        Ok("Your name")
    );
}

/// A field with no placeholder, no `wght` and no font reads none of them.
#[test]
fn a_field_without_a_placeholder_weight_or_font_reads_none() {
    let scene = Played::new(&[page(Spec {
        placeholder: false,
        weight: None,
        font: None,
        ..AT_REST
    })]);
    let step = scene.settle();
    let look = scene.look(&step);
    assert_eq!(look.has_placeholder, 0);
    assert_eq!(look.placeholder_color, 0);
    assert_eq!(look.placeholder_length, 0);
    assert_eq!(look.has_weight, 0);
    assert_eq!(look.weight, 0.0);
    assert_eq!(look.font_asset, u32::MAX);
    assert_eq!(look.font_from_host, 0);
    assert_eq!(look.font_size, 17.0);
    assert_eq!(look.value_length, 3);
}

/// After a step that writes the field's text, its text color, its font
/// size and its weight, the read gives each new value, and the step
/// before the writes is stale.
#[test]
fn the_read_follows_writes_to_the_scene() {
    let scene = Played::new(&[page(AT_REST)]);
    let before = scene.settle();
    assert_eq!(scene.look(&before).font_size, 17.0);
    scene.write(write_text(scene.object("Input"), "Grace"));
    scene.write(write_color(scene.object("Input color"), 0xff0a_84ff));
    scene.write(write_number(
        scene.object("Input style"),
        key("TextStyle", "fontSize"),
        21.0,
    ));
    scene.write(write_number(
        scene.object("Weight"),
        key("TextStyleAxis", "axisValue"),
        700.0,
    ));
    let step = scene.settle();
    let field = scene.object("Input");
    assert_eq!(
        scene.read(&before, &[record(&field, None)]),
        Err(NuxStatus::HandleMismatch)
    );
    let look = scene.look(&step);
    assert_eq!(look.render_revision, step.revision());
    assert_eq!(look.value_length, 5);
    assert_eq!(look.color, 0xff0a_84ff);
    assert_eq!(look.font_size, 21.0);
    assert_eq!((look.has_weight, look.weight), (1, 700.0));
    assert_eq!(scene.copy(&step, &field).as_deref(), Ok("Grace"));
}

/// Text typed through the engine's own input into an empty field is its
/// value after the step.
#[test]
fn typed_text_reads_as_the_value() {
    let scene = Played::new(&[page(Spec {
        text: "",
        ..AT_REST
    })]);
    scene.settle().present(scene.player);
    let focus = |kind| NuxPlayerFocusInput {
        kind,
        ..NuxPlayerFocusInput::default()
    };
    scene
        .step(&[focus(NUX_PLAYER_FOCUS_KIND_NEXT)])
        .present(scene.player);
    let mut state = NuxPlayerFocusState::default();
    assert_eq!(
        unsafe { nux_player_focus_state(scene.player, &mut state) },
        NuxStatus::Ok
    );
    assert_eq!(state.has_focus, 1, "the field takes focus");
    let typed = "Bob";
    scene
        .step(&[NuxPlayerFocusInput {
            text: NuxStringView {
                data: typed.as_ptr().cast(),
                len: typed.len(),
            },
            ..focus(NUX_PLAYER_FOCUS_KIND_TEXT)
        }])
        .present(scene.player);
    let step = scene.settle();
    assert_eq!(scene.look(&step).value_length, 3);
    assert_eq!(
        scene.copy(&step, &scene.object("Input")).as_deref(),
        Ok("Bob")
    );
}

/// A font binding's choice is the font that draws, and the read names it:
/// the file's first font, though the style names the second. A font the
/// host gives through the binding has no index in the file's assets, and
/// the read says so.
#[test]
fn a_bound_font_reads_as_the_font_that_draws() {
    use nuxie::runtime::{
        text::font_hb::HbFont,
        viewmodel::{
            viewmodel_instance::ViewModelInstance,
            viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
        },
    };
    let scene = Played::new(&[page(Spec {
        font_bound: true,
        ..AT_REST
    })]);
    let step = scene.settle();
    let look = scene.look(&step);
    assert_eq!((look.font_asset, look.font_from_host), (0, 0));
    drop(step);

    // The C API sets no font in a view model; a host on the Rust API does,
    // as this does.
    let font = HbFont::decode(FONT).expect("the fixture's font");
    unsafe { &*scene.model }
        .instance
        .native_handle()
        .with_downcast::<ViewModelInstance, _>(|model| model.property_value_named("font"))
        .flatten()
        .expect("the view model's font")
        .with_downcast_mut::<ViewModelInstanceAssetFont, _>(|font_value| {
            font_value.set_value(Some(font))
        })
        .expect("a font value");
    let step = scene.settle();
    let look = scene.look(&step);
    assert_eq!((look.font_asset, look.font_from_host), (u32::MAX, 1));
}

/// A field that hides its text reads obscured, and the copy gives the
/// text in the clear.
#[test]
fn a_password_reads_obscured_and_copies_its_text() {
    let scene = Played::new(&[page(Spec {
        text: "s3cret",
        obscured: true,
        ..AT_REST
    })]);
    let step = scene.settle();
    let look = scene.look(&step);
    assert_eq!(look.obscured, 1);
    assert_eq!(look.value_length, 6);
    assert_eq!(
        scene.copy(&step, &scene.object("Input")).as_deref(),
        Ok("s3cret")
    );
}

/// A field marked disabled as the compiler marks a disabled `input` (its
/// semantic record disabled, its FocusData unable to focus) reads
/// disabled. A FocusData unable to focus with no disabled mark reads
/// enabled.
#[test]
fn disabled_comes_from_the_semantic_record_not_the_focus() {
    let marked = Played::new(&[page(Spec {
        disabled: true,
        focus_flags: 6,
        ..AT_REST
    })]);
    let step = marked.settle();
    assert_eq!(marked.look(&step).disabled, 1);

    let unfocusable = Played::new(&[page(Spec {
        focus_flags: 6,
        ..AT_REST
    })]);
    let step = unfocusable.settle();
    assert_eq!(unfocusable.look(&step).disabled, 0);
}

/// The copy asks for its length with a null buffer, refuses a short
/// buffer without copying and copies into an exact one; a text copies
/// its runs joined; a box is refused; a step before a change is stale.
#[test]
fn the_copy_keeps_its_contract() {
    let scene = Played::new(&[page(AT_REST)]);
    let step = scene.settle();
    let field = scene.object("Input").raw();
    let copy = |step: &Step, object: &NuxSceneObject, buffer: &mut [u8], length: &mut usize| unsafe {
        nux_player_scene_text_copy(
            scene.player,
            step.0,
            object,
            if buffer.is_empty() {
                ptr::null_mut()
            } else {
                buffer.as_mut_ptr()
            },
            buffer.len(),
            length,
        )
    };

    let mut length = 7;
    assert_eq!(copy(&step, &field, &mut [], &mut length), NuxStatus::Ok);
    assert_eq!(length, 3);

    let mut short = [b'x'; 2];
    let mut length = 0;
    assert_eq!(
        copy(&step, &field, &mut short, &mut length),
        NuxStatus::LimitExceeded
    );
    assert_eq!(short, [b'x'; 2], "nothing is copied");
    assert_eq!(length, 3);

    let mut exact = [0u8; 3];
    assert_eq!(copy(&step, &field, &mut exact, &mut length), NuxStatus::Ok);
    assert_eq!(&exact, b"Ada");
    assert_eq!(length, 3);

    assert_eq!(
        scene.copy(&step, &scene.object("Hint")).as_deref(),
        Ok("Your name")
    );
    assert_eq!(
        scene.copy(&step, &scene.object("Field")),
        Err(NuxStatus::InvalidArgument)
    );
    assert_eq!(
        scene.copy(&step, &Object::root(9_999)),
        Err(NuxStatus::NotFound)
    );

    scene.write(write_text(scene.object("Input"), "Ann"));
    let after = scene.settle();
    assert_eq!(
        copy(&step, &field, &mut exact, &mut length),
        NuxStatus::HandleMismatch
    );
    assert_eq!(length, 0);
    assert_eq!(
        scene.copy(&after, &scene.object("Input")).as_deref(),
        Ok("Ann")
    );
}

/// The same field in two keyed rows reads each row's own text: row b's
/// is written "Bea", row a's stays "Ada".
#[test]
fn each_row_reads_its_own_field() {
    let scene = Played::new(&[
        Artboard {
            name: "Page",
            size: (400.0, 600.0),
            holds: Holds::Rows(1),
        },
        Artboard {
            name: "Card",
            size: (400.0, 200.0),
            holds: Holds::Field(AT_REST),
        },
    ]);
    let list = scene.id(0, "Cards");
    scene.drive.list.set(list);
    scene.drive.mount.replace(vec!["a", "b"]);
    scene.settle().present(scene.player);
    let (input, hint) = (scene.id(1, "Input"), scene.id(1, "Hint"));
    scene.write(write_text(Object::in_row(list, "b", input), "Bea"));
    let step = scene.settle();
    let row = |key| {
        (
            Object::in_row(list, key, input),
            Object::in_row(list, key, hint),
        )
    };
    let ((a, a_hint), (b, b_hint)) = (row("a"), row("b"));
    let looks = scene
        .read(
            &step,
            &[record(&a, Some(&a_hint)), record(&b, Some(&b_hint))],
        )
        .expect("both rows");
    assert_eq!(looks[0].value_length, 3);
    assert_eq!(looks[1].value_length, 3);
    assert_eq!(looks[0].placeholder_length, 9);
    assert_eq!(scene.copy(&step, &a).as_deref(), Ok("Ada"));
    assert_eq!(scene.copy(&step, &b).as_deref(), Ok("Bea"));
}

/// The read's contract: a stale step, a record of the wrong size, a box
/// as a field or as a placeholder, a placeholder flag that is not 0 or 1,
/// a missing object, no fields, and a batch with one bad field, which
/// writes nothing.
#[test]
fn the_read_keeps_its_contract() {
    let scene = Played::new(&[page(AT_REST)]);
    let step = scene.settle();
    let (field, hint, field_box) = (
        scene.object("Input"),
        scene.object("Hint"),
        scene.object("Field"),
    );
    let missing = Object::root(9_999);
    let good = record(&field, Some(&hint));
    let sentinel = NuxTextFieldLook {
        struct_size: sized::<NuxTextFieldLook>(),
        font_size: 7.0,
        ..Default::default()
    };
    let read = |step: &Step, fields: &[NuxTextFieldObjects], out: &mut [NuxTextFieldLook]| unsafe {
        nux_player_text_fields(
            scene.player,
            step.0,
            fields.as_ptr(),
            fields.len(),
            out.as_mut_ptr(),
        )
    };

    assert_eq!(read(&step, &[], &mut []), NuxStatus::Ok, "count 0");
    let mut out = [sentinel];
    assert_eq!(
        read(&step, &[record(&field_box, None)], &mut out),
        NuxStatus::InvalidArgument,
        "a box as the field"
    );
    assert_eq!(
        read(&step, &[record(&field, Some(&field_box))], &mut out),
        NuxStatus::InvalidArgument,
        "a box as the placeholder"
    );
    assert_eq!(
        read(
            &step,
            &[NuxTextFieldObjects {
                has_placeholder: 2,
                ..good
            }],
            &mut out
        ),
        NuxStatus::InvalidArgument,
        "a placeholder flag of 2"
    );
    assert_eq!(
        read(&step, &[record(&missing, None)], &mut out),
        NuxStatus::NotFound
    );
    assert_eq!(
        read(&step, &[record(&field, Some(&missing))], &mut out),
        NuxStatus::NotFound
    );
    assert_eq!(out, [sentinel]);

    // Both arrays step by their struct's size, so a record of another size
    // is refused.
    let short = NuxTextFieldObjects {
        struct_size: sized::<NuxTextFieldObjects>() - 8,
        ..good
    };
    let long = NuxTextFieldObjects {
        struct_size: sized::<NuxTextFieldObjects>() + 8,
        ..good
    };
    for fields in [short, long] {
        assert_eq!(
            read(&step, &[fields], &mut out),
            NuxStatus::InvalidStructSize
        );
    }
    for size in [
        sized::<NuxTextFieldLook>() - 8,
        sized::<NuxTextFieldLook>() + 8,
    ] {
        let mut odd = [NuxTextFieldLook {
            struct_size: size,
            ..sentinel
        }];
        assert_eq!(read(&step, &[good], &mut odd), NuxStatus::InvalidStructSize);
        assert_eq!(odd[0].font_size, 7.0);
    }
    assert_eq!(out, [sentinel]);

    let mut out = [sentinel; 3];
    assert_eq!(
        read(&step, &[good, record(&field_box, None), good], &mut out),
        NuxStatus::InvalidArgument
    );
    assert_eq!(out, [sentinel; 3], "one bad field writes nothing");
    assert_eq!(read(&step, &[good, good], &mut out[..2]), NuxStatus::Ok);
    assert_eq!(out[0], out[1]);
    assert_eq!(out[0].value_length, 3);

    scene.write(write_text(scene.object("Input"), "Ann"));
    let _after = scene.settle();
    let mut out = [sentinel];
    assert_eq!(read(&step, &[good], &mut out), NuxStatus::HandleMismatch);
    assert_eq!(out, [sentinel]);
}
