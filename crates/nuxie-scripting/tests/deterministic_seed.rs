//! Local integration regression for the native Luau deterministic-seed boundary.
//! This dedicated test binary isolates process-global File settings.
#![cfg(feature = "luau")]

use luaur_rt::{Function, Table};
use nuxie_runtime::File;
use nuxie_scripting::vm::ScriptVm;

#[test]
fn deterministic_seed_repeats_and_selects_the_native_luau_sequence() {
    struct Restore(bool, i32);
    impl Drop for Restore {
        fn drop(&mut self) {
            File::set_deterministic_seed(self.1);
            File::set_deterministic_mode(self.0);
        }
    }
    let _restore = Restore(File::deterministic_mode(), File::deterministic_seed());
    fn sequence(seed: i32) -> [f64; 8] {
        File::set_deterministic_seed(seed);
        let vm = ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let math: Table = vm.lua().globals().get("math").unwrap();
        let random: Function = math.get("random").unwrap();
        std::array::from_fn(|_| random.call::<f64>(()).unwrap())
    }
    File::set_deterministic_mode(true);
    let first = sequence(1);
    assert_eq!(first, sequence(1));
    let second = sequence(42);
    assert_ne!(first, second);
    assert_eq!(second, sequence(42));
    // Negative signed seeds are accepted by the same native math.randomseed API.
    assert_eq!(sequence(-7), sequence(-7));
}
