//! One budget per script VM for the host memory its bindings hold.
//!
//! The VM's memory ceiling covers the VM heap only. A binding that grows a
//! host-side container on a script's behalf (path commands, vertex data,
//! text runs, decoded images, copied strings) charges the bytes it is about
//! to reserve against this budget first, and the payload that owns the
//! container refunds them when it drops. The budget counts capacity a binding
//! reserves (or a stated upper bound of it, where the container is not ours),
//! never length, so a container that keeps its capacity after a clear keeps
//! its charge until it is dropped.
//!
//! The budget lives in the VM's app data, so every binding reaches it from
//! the `Lua` it is called with; each charge keeps its own handle, so a
//! payload refunds without a `Lua`.

use std::cell::Cell;
use std::rc::Rc;

use crate::luau::{Error, Lua, Result};

/// The budget a VM gets when its host does not choose one: the same number
/// as the default ceiling of a File's VM (`DEFAULT_SCRIPT_VM_MEMORY_BYTES`).
pub const DEFAULT_SCRIPT_HOST_MEMORY_BYTES: usize = super::DEFAULT_SCRIPT_VM_MEMORY_BYTES;

struct BudgetState {
    limit: Cell<usize>,
    used: Cell<usize>,
}


/// The host memory one script VM's bindings may hold, in bytes.
#[derive(Clone)]
pub(crate) struct HostMemoryBudget(Rc<BudgetState>);

impl HostMemoryBudget {
    fn new(limit: usize) -> Self {
        Self(Rc::new(BudgetState {
            limit: Cell::new(limit),
            used: Cell::new(0),
        }))
    }

    /// Installs the VM's budget with `limit`, or sets the limit of the one
    /// already installed (charges already made are kept).
    pub(crate) fn install(lua: &Lua, limit: usize) -> Result<()> {
        if let Some(budget) = lua.try_app_data_ref::<Self>()? {
            budget.0.limit.set(limit);
            return Ok(());
        }
        lua.try_set_app_data(Self::new(limit))?;
        Ok(())
    }

    /// The VM's budget. A `Lua` that no `ScriptVm` set up (a test's bare
    /// state) gets the default budget on first use.
    pub(crate) fn for_lua(lua: &Lua) -> Result<Self> {
        if let Some(budget) = lua.try_app_data_ref::<Self>()? {
            return Ok(budget.clone());
        }
        let budget = Self::new(DEFAULT_SCRIPT_HOST_MEMORY_BYTES);
        lua.try_set_app_data(budget.clone())?;
        Ok(budget)
    }

    pub(crate) fn limit(&self) -> usize {
        self.0.limit.get()
    }

    pub(crate) fn used(&self) -> usize {
        self.0.used.get()
    }

    fn charge(&self, bytes: usize) -> Result<()> {
        let used = self.0.used.get();
        let remaining = self.limit().saturating_sub(used);
        if bytes > remaining {
            return Err(budget_exceeded(self.limit()));
        }
        // `bytes <= limit - used`, so the sum is at most `limit`.
        self.0.used.set(used.saturating_add(bytes));
        Ok(())
    }

    fn refund(&self, bytes: usize) {
        self.0.used.set(self.0.used.get().saturating_sub(bytes));
    }
}

/// The one error every binding raises when its charge does not fit.
pub(crate) fn budget_exceeded(limit: usize) -> Error {
    Error::runtime(format!(
        "script VM exceeded its {limit}-byte host memory budget"
    ))
}

/// Bytes one payload holds against its VM's budget, refunded on drop.
///
/// A charge made without a `Lua` at hand (a payload built by host code) is
/// detached: it counts nothing until [`Self::attach`] gives it the VM's
/// budget, which every growing binding does before it charges.
#[derive(Default)]
pub(crate) struct HostMemoryCharge {
    budget: Option<HostMemoryBudget>,
    bytes: usize,
}

impl HostMemoryCharge {
    /// A charge of `bytes` against the VM's budget.
    pub(crate) fn for_lua(lua: &Lua, bytes: usize) -> Result<Self> {
        let mut charge = Self::default();
        charge.attach(lua)?;
        charge.reserve_to(bytes)?;
        Ok(charge)
    }

    /// Gives a detached charge the VM's budget, charging what it records.
    pub(crate) fn attach(&mut self, lua: &Lua) -> Result<()> {
        if self.budget.is_some() {
            return Ok(());
        }
        let budget = HostMemoryBudget::for_lua(lua)?;
        budget.charge(self.bytes)?;
        self.budget = Some(budget);
        Ok(())
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// Whether raising the charge to `bytes` would fit the budget.
    pub(crate) fn fits(&self, bytes: usize) -> bool {
        let growth = bytes.saturating_sub(self.bytes);
        self.budget
            .as_ref()
            .is_none_or(|budget| growth <= budget.limit().saturating_sub(budget.used()))
    }

    /// Refuses, with the budget's error, a charge of `bytes` that would not
    /// fit; checked before copying something that is charged once copied.
    pub(crate) fn check_fits(&self, bytes: usize) -> Result<()> {
        match &self.budget {
            Some(budget) if !self.fits(bytes) => Err(budget_exceeded(budget.limit())),
            _ => Ok(()),
        }
    }

    /// Raises the charge to `bytes` if it is lower; never lowers it, since a
    /// container keeps the capacity it grew to.
    pub(crate) fn reserve_to(&mut self, bytes: usize) -> Result<()> {
        let Some(growth) = bytes.checked_sub(self.bytes) else {
            return Ok(());
        };
        if let Some(budget) = &self.budget {
            budget.charge(growth)?;
        }
        self.bytes = bytes;
        Ok(())
    }

    /// Raises the charge by `bytes`.
    pub(crate) fn grow(&mut self, bytes: usize) -> Result<()> {
        let total = self.bytes.checked_add(bytes).ok_or_else(|| {
            budget_exceeded(self.budget.as_ref().map_or(usize::MAX, HostMemoryBudget::limit))
        })?;
        self.reserve_to(total)
    }

    /// Sets the charge to exactly `bytes`, refunding a decrease, for a
    /// container that released its capacity.
    pub(crate) fn set(&mut self, bytes: usize) -> Result<()> {
        match self.bytes.checked_sub(bytes) {
            Some(decrease) => {
                if let Some(budget) = &self.budget {
                    budget.refund(decrease);
                }
                self.bytes = bytes;
                Ok(())
            }
            None => self.reserve_to(bytes),
        }
    }
}

impl std::fmt::Debug for HostMemoryCharge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostMemoryCharge")
            .field("bytes", &self.bytes)
            .field("attached", &self.budget.is_some())
            .finish()
    }
}

impl Drop for HostMemoryCharge {
    fn drop(&mut self) {
        if let Some(budget) = &self.budget {
            budget.refund(self.bytes);
        }
    }
}

/// Bytes `count` elements of `T` occupy, saturating: a saturated size is
/// refused by any budget.
pub(crate) const fn bytes_of<T>(count: usize) -> usize {
    count.saturating_mul(std::mem::size_of::<T>())
}

/// The capacity `Vec` growth reaches when `len + additional` elements must
/// fit a vector of capacity `capacity`: at least double, as the standard
/// library grows, so that charging it keeps pushes amortized.
pub(crate) fn grown_capacity(capacity: usize, len: usize, additional: usize) -> usize {
    let required = len.saturating_add(additional);
    if required <= capacity {
        capacity
    } else {
        required.max(capacity.saturating_mul(2)).max(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charges_refund_on_drop_and_refuse_past_the_limit() {
        let lua = Lua::new();
        HostMemoryBudget::install(&lua, 100).unwrap();
        let budget = HostMemoryBudget::for_lua(&lua).unwrap();
        let mut first = HostMemoryCharge::for_lua(&lua, 60).unwrap();
        assert_eq!(budget.used(), 60);
        let error = HostMemoryCharge::for_lua(&lua, 41).err().unwrap();
        assert!(
            error
                .to_string()
                .ends_with("script VM exceeded its 100-byte host memory budget"),
            "{error}"
        );
        // A refused charge leaves the budget as it was.
        assert_eq!(budget.used(), 60);
        first.reserve_to(40).unwrap();
        assert_eq!(first.bytes(), 60, "reserve_to never lowers a charge");
        first.set(10).unwrap();
        assert_eq!(budget.used(), 10);
        let second = HostMemoryCharge::for_lua(&lua, 90).unwrap();
        assert_eq!(budget.used(), 100);
        drop(second);
        drop(first);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn a_detached_charge_counts_once_attached() {
        let lua = Lua::new();
        HostMemoryBudget::install(&lua, 100).unwrap();
        let budget = HostMemoryBudget::for_lua(&lua).unwrap();
        let mut charge = HostMemoryCharge::default();
        charge.grow(70).unwrap();
        assert_eq!(budget.used(), 0);
        charge.attach(&lua).unwrap();
        assert_eq!(budget.used(), 70);
        assert!(charge.grow(31).is_err());
        assert_eq!(charge.bytes(), 70);
    }

    #[test]
    fn growth_at_least_doubles() {
        assert_eq!(grown_capacity(0, 0, 1), 4);
        assert_eq!(grown_capacity(4, 4, 1), 8);
        assert_eq!(grown_capacity(8, 3, 2), 8);
        assert_eq!(grown_capacity(8, 8, 100), 108);
        assert_eq!(grown_capacity(usize::MAX, usize::MAX, 1), usize::MAX);
    }
}

/// Each binding that grows host memory for a script, against a small budget:
/// every case grew without limit before the budget.
#[cfg(all(test, feature = "compiler"))]
mod binding_tests {
    use super::HostMemoryBudget;
    use crate::vm::{ScriptExecutionLimits, ScriptVm};
    use nuxie_render_api::{PersistentFactory, RecordingFactory};

    const EXCEEDED: &str = "host memory budget";

    struct Vm {
        vm: ScriptVm,
        _factory: PersistentFactory<RecordingFactory>,
    }

    impl Vm {
        fn with_budget(bytes: usize) -> Self {
            let vm = ScriptVm::new_with_execution_limits(
                ScriptExecutionLimits::new()
                    .with_max_host_memory_bytes(bytes)
                    .with_max_interrupts_per_callback(1_000_000),
            )
            .unwrap();
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            vm.install_render_factory(&mut factory).unwrap();
            vm.install_rive_globals().unwrap();
            Self {
                vm,
                _factory: factory,
            }
        }

        fn used(&self) -> usize {
            HostMemoryBudget::for_lua(self.vm.lua()).unwrap().used()
        }

        /// Runs `script`, which returns whether its guarded part failed and
        /// the error text.
        fn refused(&self, script: &str) -> String {
            let (failed, error): (bool, String) = self.vm.eval(script).unwrap();
            assert!(failed, "expected a refusal, got success");
            assert!(error.contains(EXCEEDED), "{error}");
            error
        }

        fn set_global(&self, name: &str, value: impl crate::luau::IntoLua) {
            let globals = self.vm.lua().globals();
            let readonly = globals.is_readonly();
            globals.set_readonly(false);
            globals.set(name, value).unwrap();
            globals.set_readonly(readonly);
        }
    }

    #[test]
    fn the_limit_comes_from_the_execution_limits() {
        let vm = Vm::with_budget(12_345);
        let budget = HostMemoryBudget::for_lua(vm.vm.lua()).unwrap();
        assert_eq!(budget.limit(), 12_345);
        let default = ScriptVm::new();
        assert_eq!(
            HostMemoryBudget::for_lua(default.lua()).unwrap().limit(),
            super::DEFAULT_SCRIPT_HOST_MEMORY_BYTES
        );
        assert_eq!(
            ScriptExecutionLimits::new().max_host_memory_bytes(),
            64 * 1024 * 1024
        );
    }

    // Inventory row 1: each `p:add(p)` doubled the path with no bound.
    #[test]
    fn a_path_added_to_itself_stops_at_the_budget() {
        let vm = Vm::with_budget(1024 * 1024);
        let error = vm.refused(
            "p = Path.new() \
             p:moveTo(Vector.xy(0, 0)) p:lineTo(Vector.xy(1, 1)) \
             local ok, err = pcall(function() for i = 1, 30 do p:add(p) end end) \
             return not ok, tostring(err)",
        );
        assert!(error.contains("1048576-byte"), "{error}");
        let verbs: usize = vm.vm.eval("return #p").unwrap();
        // 27 bytes are charged per verb and point pair; the last doubling
        // that fit left at most half the budget's worth.
        assert!(verbs <= 1024 * 1024 / 27, "{verbs}");
        assert!(vm.used() <= 1024 * 1024);
        vm.vm.eval::<()>("p:reset()").unwrap();
        assert_eq!(vm.used(), 0);
    }

    // Inventory row 12: drawing commands in a loop.
    #[test]
    fn path_commands_and_extracts_stop_at_the_budget() {
        let vm = Vm::with_budget(64 * 1024);
        vm.refused(
            "local p = Path.new() \
             local ok, err = pcall(function() \
               for i = 1, 100000 do p:cubicTo(Vector.xy(i, 0), Vector.xy(i, 1), Vector.xy(i, 2)) end \
             end) \
             return not ok, tostring(err)",
        );
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), 0, "a collected path refunds");
        vm.refused(
            "local source = Path.new() \
             source:moveTo(Vector.xy(0, 0)) \
             for i = 1, 200 do source:lineTo(Vector.xy(i, i % 2)) end \
             local measure = source:measure() \
             local destination = Path.new() \
             local ok, err = pcall(function() \
               for i = 1, 1000 do measure:extract(0, 1000, destination) end \
             end) \
             return not ok, tostring(err)",
        );
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), 0, "every path refunds when collected");
    }

    // Inventory rows 10 and 11.
    #[test]
    fn vertex_and_triangle_buffers_stop_at_the_budget() {
        let vm = Vm::with_budget(64 * 1024);
        vm.refused(
            "local v = VertexBuffer() \
             local ok, err = pcall(function() for i = 1, 100000 do v:add(Vector.xy(i, i)) end end) \
             return not ok, tostring(err)",
        );
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), 0);
        vm.refused(
            "local t = TriangleBuffer() \
             local ok, err = pcall(function() for i = 1, 100000 do t:add(0, 1, 2) end end) \
             return not ok, tostring(err)",
        );
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), 0);
        // A reset keeps the capacity, and so the charge.
        vm.vm
            .eval::<()>("local t = TriangleBuffer() for i = 1, 100 do t:add(0, 1, 2) end t:reset() keep = t")
            .unwrap();
        assert_eq!(vm.used(), 512 * 2 * 2, "300 indices grow to 512 slots");
    }

    // The mesh instances' bytes are charged to the same budget; their own
    // count limit and its text are unchanged (lua_renderer's test).
    #[test]
    fn mesh_instances_charge_the_shared_budget() {
        let vm = Vm::with_budget(64 * 1024 * 1024);
        vm.vm.eval::<()>("m = MeshInstances(1000)").unwrap();
        assert_eq!(vm.used(), 1000 * 3 * 48);
        let small = Vm::with_budget(100 * 144);
        small.refused(
            "local ok, err = pcall(function() return MeshInstances(101) end) \
             return not ok, tostring(err)",
        );
    }

    // Codex review: an empty interval appends nothing, so it needs no room.
    #[test]
    fn an_empty_extract_needs_no_room() {
        let vm = Vm::with_budget(1000);
        let (empty, contour): (usize, usize) = vm
            .vm
            .eval(
                "local source = Path.new() \
                 source:moveTo(Vector.xy(0, 0)) \
                 for i = 1, 10 do source:lineTo(Vector.xy(i, 0)) end \
                 local dest = Path.new() \
                 source:measure():extract(5, 5, dest) \
                 local contour = Path.new() \
                 source:contours():extract(3, 1, contour) \
                 return #dest, #contour",
            )
            .unwrap();
        assert_eq!((empty, contour), (0, 0));
    }

    /// A Text's first font-list growth: four 24-byte entries.
    const FONT_LIST_BYTES: usize = 4 * 24;

    fn font_vm(budget: usize) -> Vm {
        let vm = Vm::with_budget(budget);
        let bytes = include_bytes!("../../../../fixtures/fonts/ibm-plex-sans-arabic.ttf");
        vm.set_global("fontBytes", vm.vm.lua().create_buffer(bytes).unwrap());
        vm
    }

    // Inventory row 20: a decoded font keeps its bytes.
    #[test]
    fn font_decode_is_charged() {
        let vm = font_vm(100_000);
        vm.refused(
            "local ok, err = pcall(function() return Font.decode(fontBytes) end) \
             return not ok, tostring(err)",
        );
        let vm = font_vm(1024 * 1024);
        vm.vm.eval::<()>("font = Font.decode(fontBytes)").unwrap();
        assert_eq!(vm.used(), 226_004);
    }

    // Codex review: the charge went when the decoded font was collected, while
    // a derived font or a Text shaped with it still kept the face alive.
    #[test]
    fn a_font_stays_charged_while_anything_keeps_its_face() {
        let vm = font_vm(1024 * 1024);
        vm.vm
            .eval::<()>(
                "local font = Font.decode(fontBytes)                  derived = font:withOptions({}, {})                  text = Text.new()                  text:append('x', {font = font})",
            )
            .unwrap();
        vm.vm.lua().gc_collect().unwrap();
        let text_bytes = 24 + 64 + 256 + FONT_LIST_BYTES;
        assert_eq!(vm.used(), 226_004 + text_bytes);
        vm.vm.eval::<()>("derived = nil").unwrap();
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), 226_004 + text_bytes, "the text keeps the face");
        // Shaped runs keep the face past a clear, so the charge stays too.
        vm.vm.eval::<()>("text:clear()").unwrap();
        assert_eq!(vm.used(), 226_004 + text_bytes, "a clear keeps the face");
        vm.vm.eval::<()>("text = nil").unwrap();
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), 0);
    }

    // Codex review: `glyph.font` returned an uncharged font that kept the
    // decoded face alive after the font and the Text were collected.
    #[test]
    fn a_glyph_font_shares_the_decoded_charge() {
        let vm = font_vm(1024 * 1024);
        vm.vm
            .eval::<()>(
                "local font = Font.decode(fontBytes) \
                 local text = Text.new() \
                 text:append('x', {font = font}) \
                 for _, line in text:lines() do \
                   for _, glyph in line:glyphs() do kept = glyph.font end \
                 end",
            )
            .unwrap();
        assert!(vm.vm.eval::<bool>("return kept ~= nil").unwrap());
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), 226_004, "the glyph's font keeps the face");
    }

    // Inventory row 13: appended characters, runs and styles.
    #[test]
    fn text_append_stops_at_the_budget_and_clear_keeps_the_charge() {
        let vm = font_vm(1024 * 1024);
        vm.refused(
            "local font = Font.decode(fontBytes) \
             local text = Text.new() \
             local s = string.rep('x', 10000) \
             local ok, err = pcall(function() for i = 1, 100 do text:append(s, {font = font}) end end) \
             return not ok, tostring(err)",
        );
        vm.vm.lua().gc_collect().unwrap();
        let fonts = vm.used();
        vm.vm
            .eval::<()>(
                "local font = Font.decode(fontBytes) \
                 text = Text.new() \
                 text:append(string.rep('x', 1000), {font = font}) \
                 text:clear() \
                 text:append(string.rep('x', 1000), {font = font})",
            )
            .unwrap();
        let charged = vm.used() - fonts - 226_004;
        assert_eq!(
            charged,
            24 * 1000 + 64 + 256 + FONT_LIST_BYTES,
            "a clear keeps the high-water charge"
        );
    }

    // Inventory row 13: a style id is a `u16`; the 65,537th distinct style
    // used to be stored with a truncated id, drawing with another's paint.
    #[test]
    fn a_text_past_65536_styles_is_an_error() {
        let vm = font_vm(64 * 1024 * 1024);
        vm.vm
            .eval::<()>("font = Font.decode(fontBytes) text = Text.new() n = 0")
            .unwrap();
        for _ in 0..4 {
            vm.vm
                .eval::<()>(
                    "for i = 1, 16384 do \
                       text:append('x', {font = font, foregroundColor = n}) n += 1 \
                     end",
                )
                .unwrap();
        }
        // An existing style is still accepted at the cap.
        vm.vm
            .eval::<()>("text:append('y', {font = font, foregroundColor = 7})")
            .unwrap();
        let (failed, error): (bool, String) = vm
            .vm
            .eval(
                "local ok, err = pcall(function() text:append('z', {font = font, foregroundColor = n}) end) \
                 return not ok, tostring(err)",
            )
            .unwrap();
        assert!(failed);
        assert!(error.contains("Text may hold at most 65536 styles"), "{error}");
    }

    // Inventory row 24: a string data value's copy.
    #[test]
    fn a_data_value_string_is_charged() {
        let vm = Vm::with_budget(64 * 1024);
        vm.refused(
            "local d = DataValue.string() \
             local ok, err = pcall(function() d.value = string.rep('x', 100000) end) \
             return not ok, tostring(err)",
        );
        vm.vm
            .eval::<()>("d = DataValue.string() d.value = string.rep('x', 1000)")
            .unwrap();
        assert_eq!(vm.used(), 1000);
        vm.vm.eval::<()>("d.value = 'ab'").unwrap();
        assert_eq!(vm.used(), 2, "a shorter string refunds");
    }

    // Codex review: a string cut at its first NUL kept the whole copy's
    // allocation (charged by capacity, or uncharged as a CoreString).
    #[test]
    fn a_data_value_string_cut_at_nul_keeps_only_its_bytes() {
        let vm = Vm::with_budget(256 * 1024);
        vm.vm
            .eval::<()>(
                "d = DataValue.string() \
                 d.value = '\\0' .. string.rep('x', 100000) \
                 assert(d.value == '') \
                 d.value = string.char(255, 0) .. string.rep('x', 100000)",
            )
            .unwrap();
        assert_eq!(vm.used(), 1);
    }

    // Lane E2b review round 4: the whole string was copied out of the VM
    // before the charge was checked, so a string over the budget made the
    // host hold a copy of it first. It is now refused before the copy, even
    // when what is kept (the bytes before a NUL) would fit.
    #[test]
    fn a_data_value_string_is_checked_before_it_is_copied() {
        let vm = Vm::with_budget(64 * 1024);
        vm.refused(
            "local d = DataValue.string() \
             local ok, err = pcall(function() d.value = '\\0' .. string.rep('x', 100000) end) \
             return not ok, tostring(err)",
        );
        assert_eq!(vm.used(), 0);
    }

    // Codex review: alternating fonts across clears grew the Text's font
    // list without a charge.
    #[test]
    fn a_texts_font_list_is_charged_across_clears() {
        let vm = font_vm(1024 * 1024);
        vm.refused(
            "local a = Font.decode(fontBytes) \
             local b = a:withOptions({}, {}) \
             local text = Text.new() \
             local ok, err = pcall(function() \
               for i = 1, 200000 do \
                 text:clear() \
                 text:append('x', {font = if i % 2 == 0 then a else b}) \
               end \
             end) \
             return not ok, tostring(err)",
        );
    }
}
