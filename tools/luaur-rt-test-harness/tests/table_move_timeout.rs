use luaur_rt::Lua;
use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};

const CHILD_ENV: &str = "NUXIE_LUAUR_TABLE_MOVE_TIMEOUT_CHILD";

fn run_large_sparse_destination_cases() {
    luaur_common::DFFlag::LuauTableMoveTimeoutFix.push_test_override(true);

    let lua = Lua::new();
    let completed: bool = lua
        .load(
            r#"
            -- Destination array growth must not hide a sparse move.
            local a = {1, 2, 3}
            table.move({[1] = "first", [10000000] = "last"}, 1, 10000000, 2, a)
            assert(a[1] == 1)
            assert(a[2] == "first")
            assert(a[3] == nil)
            assert(a[10000001] == "last")

            -- Starting at destination index 1 must not allocate the full range.
            a = {}
            table.move({[1] = "first", [10000000] = "last"}, 1, 10000000, 1, a)
            assert(a[1] == "first")
            assert(a[10000000] == "last")
            return true
            "#,
        )
        .eval()
        .expect("large sparse table.move cases should complete");

    luaur_common::DFFlag::LuauTableMoveTimeoutFix.pop_test_override();
    assert!(completed);
}

#[test]
fn sparse_table_move_uses_bounded_iteration_when_enabled() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_large_sparse_destination_cases();
        return;
    }

    let mut child = Command::new(std::env::current_exe().expect("current test executable"))
        .args([
            "--exact",
            "sparse_table_move_uses_bounded_iteration_when_enabled",
            "--nocapture",
        ])
        .env(CHILD_ENV, "1")
        .spawn()
        .expect("spawn bounded table.move regression child");
    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        if let Some(status) = child.try_wait().expect("poll table.move child") {
            assert!(
                status.success(),
                "table.move regression child failed: {status}"
            );
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill timed-out table.move child");
            let _ = child.wait();
            panic!("large sparse table.move exceeded the five-second bound");
        }
        thread::sleep(Duration::from_millis(10));
    }
}
