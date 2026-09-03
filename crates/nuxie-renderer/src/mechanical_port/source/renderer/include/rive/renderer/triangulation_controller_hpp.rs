//! Mechanical translation of `renderer/include/rive/renderer/triangulation_controller.hpp`.

#![allow(dead_code)]
#![allow(non_snake_case)]

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriangulationThresholds {
    pub minArea: f32,
    pub maxVerbs: usize,
    pub frameBudgetMs: f32,
}

impl Default for TriangulationThresholds {
    fn default() -> Self {
        Self {
            minArea: 512.0 * 512.0,
            maxVerbs: 256,
            frameBudgetMs: 2.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TriangulationController {
    pub(crate) m_thresholds: TriangulationThresholds,
    pub(crate) m_scoreThreshold: f32,
    pub(crate) m_timeEwmaMs: f64,
    pub(crate) m_frameCacheHits: usize,
    pub(crate) m_frameSeconds: f64,
    pub(crate) m_frameBuilt: usize,
    pub(crate) m_frameMinAdmittedScore: f32,
    pub(crate) m_frameMaxRejectedScore: f32,
}

impl Default for TriangulationController {
    fn default() -> Self {
        Self {
            m_thresholds: TriangulationThresholds::default(),
            m_scoreThreshold: 1.0,
            m_timeEwmaMs: 0.0,
            m_frameCacheHits: 0,
            m_frameSeconds: 0.0,
            m_frameBuilt: 0,
            m_frameMinAdmittedScore: f32::INFINITY,
            m_frameMaxRejectedScore: 0.0,
        }
    }
}

impl TriangulationController {
    pub fn beginFrame(&mut self, thresholds: &TriangulationThresholds) {
        self.m_thresholds = *thresholds;
        self.m_frameCacheHits = 0;
        self.m_frameSeconds = 0.0;
        self.m_frameBuilt = 0;
        self.m_frameMinAdmittedScore = f32::INFINITY;
        self.m_frameMaxRejectedScore = 0.0;
    }

    pub fn endFrame(&mut self) {
        crate::mechanical_port::source::renderer::src::triangulation_controller_cpp::endFrame(self);
    }

    pub fn isEligible(&self, area: f32, verb_count: usize) -> bool {
        self.m_thresholds.frameBudgetMs > 0.0
            && area >= self.m_thresholds.minArea
            && verb_count <= self.m_thresholds.maxVerbs
    }

    pub fn admits(&mut self, area: f32, verb_count: usize) -> bool {
        crate::mechanical_port::source::renderer::src::triangulation_controller_cpp::admits(
            self, area, verb_count,
        )
    }

    pub fn recordCacheHit(&mut self) {
        self.m_frameCacheHits += 1;
    }

    pub fn recordBuilt(&mut self, seconds: f64) {
        self.m_frameBuilt += 1;
        self.m_frameSeconds += seconds;
    }

    #[cfg(feature = "with-rive-tools")]
    pub fn testingOnly_scoreThreshold(&self) -> f32 {
        self.m_scoreThreshold
    }

    #[cfg(feature = "with-rive-tools")]
    pub fn testingOnly_secondsThisFrame(&self) -> f64 {
        self.m_frameSeconds
    }

    #[cfg(feature = "with-rive-tools")]
    pub fn testingOnly_builtThisFrame(&self) -> usize {
        self.m_frameBuilt
    }

    #[cfg(feature = "with-rive-tools")]
    pub fn testingOnly_cacheHitsThisFrame(&self) -> usize {
        self.m_frameCacheHits
    }

    #[cfg(feature = "with-rive-tools")]
    pub fn testingOnly_budgetExhausted(&self) -> bool {
        self.budgetExhausted()
    }

    pub(crate) fn budgetExhausted(&self) -> bool {
        self.m_frameSeconds * 1e3 >= self.m_thresholds.frameBudgetMs as f64
    }
}
