//! A deterministic, world-wide drought cycle. Only elapsed simulation time is
//! stored; phase and severity are derived so saves cannot contain conflicting timers.
use super::*;

pub const DROUGHT_CALM_SECONDS: f64 = 300.0;
pub const DROUGHT_WARNING_SECONDS: f64 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentPhase {
    Calm,
    DroughtWarning,
    Drought,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentView {
    pub phase: EnvironmentPhase,
    /// One-based drought number (the upcoming drought during calm/warning).
    pub cycle: u64,
    pub remaining_seconds: f64,
    pub drought_seconds: f64,
    pub food_gather_multiplier: f64,
}

impl EnvironmentView {
    pub fn at(elapsed: f64) -> Self {
        // First two cycles last 420/450 seconds. From cycle three onward,
        // droughts are capped at two minutes, with five minutes of recovery.
        let onset = DROUGHT_CALM_SECONDS + DROUGHT_WARNING_SECONDS;
        let first_end = onset + 60.0;
        let second_end = first_end + onset + 90.0;
        let capped_cycle = onset + 120.0;
        let (cycle, within, drought_seconds) = if elapsed < first_end {
            (1, elapsed, 60.0)
        } else if elapsed < second_end {
            (2, elapsed - first_end, 90.0)
        } else {
            let later = elapsed - second_end;
            (
                3u64.saturating_add((later / capped_cycle).floor() as u64),
                later % capped_cycle,
                120.0,
            )
        };
        let (phase, remaining_seconds) = if within < DROUGHT_CALM_SECONDS {
            (EnvironmentPhase::Calm, DROUGHT_CALM_SECONDS - within)
        } else if within < onset {
            (EnvironmentPhase::DroughtWarning, onset - within)
        } else {
            (EnvironmentPhase::Drought, onset + drought_seconds - within)
        };
        Self {
            phase,
            cycle,
            remaining_seconds,
            drought_seconds,
            food_gather_multiplier: if phase == EnvironmentPhase::Drought {
                0.5
            } else {
                1.0
            },
        }
    }
}

impl GameWorld {
    pub(super) fn environment_gather_multiplier(&self, kind: ResourceKind) -> f64 {
        if kind == ResourceKind::Food {
            EnvironmentView::at(self.environment_seconds).food_gather_multiplier
        } else {
            1.0
        }
    }
}
