//! Mechanical translation of `renderer/src/triangulation_controller.cpp`.

#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::mechanical_port::source::renderer::include::rive::renderer::triangulation_controller_hpp::TriangulationController;

pub fn admits(controller: &mut TriangulationController, area: f32, verb_count: usize) -> bool {
    assert!(controller.isEligible(area, verb_count));
    if controller.m_thresholds.frameBudgetMs.is_infinite() {
        return true;
    }
    if controller.budgetExhausted() {
        return false;
    }

    assert!(verb_count > 1);
    let n = verb_count as f32;
    let score = area / (n * n.log2());
    if score > controller.m_scoreThreshold {
        controller.m_frameMinAdmittedScore = controller.m_frameMinAdmittedScore.min(score);
        true
    } else {
        controller.m_frameMaxRejectedScore = controller.m_frameMaxRejectedScore.max(score);
        false
    }
}

pub fn endFrame(controller: &mut TriangulationController) {
    let frame_budget_ms = controller.m_thresholds.frameBudgetMs as f64;
    if frame_budget_ms <= 0.0 || frame_budget_ms.is_infinite() {
        return;
    }

    let frame_ms = controller.m_frameSeconds * 1e3;
    const ALPHA: f64 = 0.1;
    controller.m_timeEwmaMs += ALPHA * (frame_ms - controller.m_timeEwmaMs);

    let ratio = controller.m_timeEwmaMs / frame_budget_ms;
    if ratio > 1.05 || ratio < 0.95 {
        const BRACKET_BACKOFF: f32 = 0.99;
        let factor = ratio.sqrt().clamp(0.5, 2.0);
        if factor > 1.0 {
            if controller.m_frameMinAdmittedScore != f32::INFINITY {
                controller.m_scoreThreshold = (controller.m_scoreThreshold * factor as f32)
                    .max(controller.m_frameMinAdmittedScore / BRACKET_BACKOFF);
            }
        } else if controller.m_frameMaxRejectedScore != 0.0 {
            controller.m_scoreThreshold = (controller.m_scoreThreshold * factor as f32)
                .min(controller.m_frameMaxRejectedScore * BRACKET_BACKOFF);
        }
    }
    controller.m_scoreThreshold = controller.m_scoreThreshold.clamp(1.0, 1e9);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::renderer::include::rive::renderer::triangulation_controller_hpp::TriangulationThresholds;

    const BIG_AREA: f32 = 1024.0 * 1024.0;
    const RECT_VERBS: usize = 5;

    fn run_frames(
        controller: &mut TriangulationController,
        frames: usize,
        draws_per_frame: usize,
        frame_budget_ms: f32,
        seconds_each: f64,
    ) -> usize {
        let mut admitted = 0;
        for _ in 0..frames {
            admitted = 0;
            controller.beginFrame(&TriangulationThresholds {
                frameBudgetMs: frame_budget_ms,
                ..TriangulationThresholds::default()
            });
            assert!(controller.isEligible(BIG_AREA, RECT_VERBS));
            for _ in 0..draws_per_frame {
                if controller.admits(BIG_AREA, RECT_VERBS) {
                    admitted += 1;
                    controller.recordBuilt(seconds_each);
                }
            }
            controller.endFrame();
        }
        admitted
    }

    #[test]
    fn interior_triangulation_budget_controller_adapts() {
        let mut controller = TriangulationController::default();
        let initial = controller.m_scoreThreshold;
        run_frames(&mut controller, 40, 8, 2.0, 0.01);
        let tightened = controller.m_scoreThreshold;
        assert!(tightened > initial);
        assert_eq!(run_frames(&mut controller, 40, 8, 2.0, 0.0), 8);
        assert!(controller.m_scoreThreshold < tightened);
    }

    #[test]
    fn interior_triangulation_stops_once_frame_budget_is_spent() {
        let mut controller = TriangulationController::default();
        controller.beginFrame(&TriangulationThresholds {
            frameBudgetMs: 2.0,
            ..TriangulationThresholds::default()
        });
        let mut admitted = 0;
        for _ in 0..8 {
            if controller.admits(BIG_AREA, RECT_VERBS) {
                admitted += 1;
                controller.recordBuilt(0.0015);
            }
        }
        assert_eq!(admitted, 2);
        assert_eq!(controller.m_frameBuilt, 2);
        assert!(controller.budgetExhausted());
        assert!((controller.m_frameSeconds - 0.003).abs() < f64::EPSILON);
        let threshold = controller.m_scoreThreshold;
        controller.endFrame();
        assert_eq!(controller.m_scoreThreshold, threshold);
    }

    #[test]
    fn infinite_and_zero_budgets_are_guard_only_modes() {
        let mut controller = TriangulationController::default();
        assert_eq!(run_frames(&mut controller, 20, 8, f32::INFINITY, 1.0), 8);
        assert_eq!(controller.m_scoreThreshold, 1.0);

        controller.beginFrame(&TriangulationThresholds {
            frameBudgetMs: 0.0,
            ..TriangulationThresholds::default()
        });
        assert!(!controller.isEligible(BIG_AREA, RECT_VERBS));
        controller.endFrame();
        assert_eq!(controller.m_scoreThreshold, 1.0);
    }

    #[test]
    fn infinity_uses_guards_without_spending_or_tuning() {
        let mut controller = TriangulationController::default();
        controller.beginFrame(&TriangulationThresholds {
            minArea: 1.0,
            maxVerbs: 256,
            frameBudgetMs: f32::INFINITY,
        });
        assert!(controller.admits(1024.0, 4));
        controller.recordBuilt(10.0);
        controller.endFrame();
        assert_eq!(controller.m_scoreThreshold, 1.0);
    }

    #[test]
    fn exhausted_budget_rejects_without_recording_score() {
        let mut controller = TriangulationController::default();
        controller.beginFrame(&TriangulationThresholds {
            minArea: 1.0,
            maxVerbs: 256,
            frameBudgetMs: 2.0,
        });
        controller.recordBuilt(0.002);
        assert!(!controller.admits(1024.0, 4));
        assert_eq!(controller.m_frameMaxRejectedScore, 0.0);
    }
}
