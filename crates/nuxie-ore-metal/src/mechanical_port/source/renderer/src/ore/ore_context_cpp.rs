//! renderer/src/ore/ore_context.cpp.
#![allow(non_snake_case)]
use crate::context::{ContextState, GpuPassTiming, kGpuProfileReportFrames};
use crate::types::RenderPassDesc;

pub fn gpuPassLabel(desc: &RenderPassDesc<'_>) -> String {
    if let Some(label) = desc.label {
        let label = label.split('\0').next().unwrap();
        if !label.is_empty() {
            return label.to_owned();
        }
    }
    if desc.colorCount > 0 {
        if let Some(view) = desc.colorAttachments[0].view {
            let view = view
                .textureViewBase()
                .expect("render attachment texture view");
            return format!(
                "{}x{} fmt{} x{} ms{}{}",
                view.width(),
                view.height(),
                view.texture().format().expect("texture format") as u32,
                desc.colorCount,
                view.texture().sampleCount().expect("texture sample count"),
                if desc.depthStencil.view.is_some() {
                    " +depth"
                } else {
                    ""
                }
            );
        }
    }
    if let Some(view) = desc.depthStencil.view {
        let view = view
            .textureViewBase()
            .expect("depth attachment texture view");
        return format!("depth {}x{}", view.width(), view.height());
    }
    "empty".to_owned()
}

impl ContextState {
    pub fn publishGpuPassTimings(&self, rows: &[GpuPassTiming]) {
        let mut profile = self
            .gpuProfile
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for row in rows {
            *profile.totals.entry(row.label.clone()).or_default() += row.milliseconds;
        }
        profile.frames += 1;
        if profile.frames < kGpuProfileReportFrames {
            return;
        }
        let mut report: Vec<_> = profile
            .totals
            .iter()
            .map(|(label, ms)| GpuPassTiming {
                label: label.clone(),
                milliseconds: ms / f64::from(profile.frames),
            })
            .collect();
        report.sort_unstable_by(|a, b| {
            b.milliseconds
                .partial_cmp(&a.milliseconds)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for row in report {
            println!(
                "[ore gpu] {:8.3} ms  {}",
                row.milliseconds,
                row.label.split('\0').next().unwrap()
            );
        }
        profile.totals.clear();
        profile.frames = 0;
    }
}
