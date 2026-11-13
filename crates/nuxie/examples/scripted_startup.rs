//! Scripting startup, repeated so a whole-process counter can measure it.
//!
//! `scripted_startup <stage> <iterations> <fixture.riv>` runs one stage
//! `iterations` times after reading the fixture once:
//!
//! - `read`: nothing more (the baseline to subtract);
//! - `vm`: `ScriptVm::new` plus `install_rive_globals`, which ends in the
//!   sandbox;
//! - `import`: the whole scripted import of the fixture (a VM, its globals and
//!   sandbox, every script asset's bytecode loaded and initialised), dropped
//!   at the end of each iteration.
//!
//! Measure with `/usr/bin/time -l`, which prints the process's retired
//! instructions, at two iteration counts; the difference divided by the
//! difference in iterations is the per-start cost. Built with
//! `scripting`
//! (docs/evidence/luau-safe-vm-phase3-cutover-prep-2026-11-13.md; the feature
//! names there predate the Phase 3 flip).
use anyhow::{Context, Result, bail};
use nuxie::{
    FileImportLimits, PersistentFactory, ScriptExecutionLimits, ScriptVm,
    import_unsigned_scripted,
};
use nuxie_render_api::SerializingFactory;
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let [_, stage, iterations, fixture] = args.as_slice() else {
        bail!("usage: scripted_startup <read|vm|import> <iterations> <fixture.riv>");
    };
    let iterations: u32 = iterations.parse().context("iterations")?;
    let bytes = std::fs::read(fixture).with_context(|| format!("read {fixture}"))?;
    let started = Instant::now();
    for _ in 0..iterations {
        match stage.as_str() {
            "read" => {}
            "vm" => {
                let vm = ScriptVm::new();
                vm.install_rive_globals()
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
            }
            "import" => {
                let mut factory = PersistentFactory::new(SerializingFactory::new());
                let scripted = import_unsigned_scripted(
                    &bytes,
                    &mut factory,
                    None,
                    FileImportLimits::new(),
                    ScriptExecutionLimits::new(),
                )?;
                drop(scripted);
            }
            other => bail!("unknown stage {other}"),
        }
    }
    let elapsed = started.elapsed();
    println!(
        "stage={stage} iterations={iterations} wall_us_total={} wall_us_per={:.1}",
        elapsed.as_micros(),
        elapsed.as_secs_f64() * 1e6 / f64::from(iterations.max(1))
    );
    Ok(())
}
