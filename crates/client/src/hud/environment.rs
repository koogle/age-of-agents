//! Always-visible forecasts use the same authoritative countdown in all clients.
use super::*;
use aoa_game::{EnvironmentPhase, EnvironmentView};

fn forecast(environment: EnvironmentView) -> (String, &'static str) {
    let seconds = environment.remaining_seconds.ceil() as u64;
    let time = format!("{}:{:02}", seconds / 60, seconds % 60);
    match environment.phase {
        EnvironmentPhase::Calm => (
            format!("Calm · warning in {time}"),
            "Store food before the next drought.",
        ),
        EnvironmentPhase::DroughtWarning => (
            format!("Drought {} in {time}", environment.cycle),
            "Prepare: food gathering will be halved.",
        ),
        EnvironmentPhase::Drought => (
            format!("Drought {} · ends in {time}", environment.cycle),
            "Food gathering at 50% on all islands.",
        ),
    }
}

impl Hud {
    pub(super) fn environment_banner(
        &mut self,
        atlas: &Atlas,
        environment: EnvironmentView,
        width: f32,
        s: f32,
        top: f32,
    ) -> f32 {
        let (title, detail) = forecast(environment);
        let room = (width - 48.0 * s).min(360.0 * s);
        let lines = Self::wrapped_lines(atlas, detail, 12.0 * s, room);
        let height = (30.0 + lines.len() as f32 * 15.0) * s;
        let left = (width - room) / 2.0;
        self.shape(
            [left - 12.0 * s, top, room + 24.0 * s, height],
            GLASS,
            1.0,
            12.0 * s,
        );
        self.text(
            atlas,
            &title,
            (width / 2.0, top + 18.0 * s),
            13.0 * s,
            ACCENT,
            true,
        );
        for (index, line) in lines.iter().enumerate() {
            self.text(
                atlas,
                line,
                (width / 2.0, top + (34.0 + index as f32 * 15.0) * s),
                12.0 * s,
                INK,
                true,
            );
        }
        top + height + 6.0 * s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forecast_explains_each_phase_and_uses_simulation_countdown() {
        assert_eq!(
            forecast(EnvironmentView::at(0.0)).0,
            "Calm · warning in 5:00"
        );
        assert_eq!(forecast(EnvironmentView::at(300.0)).0, "Drought 1 in 1:00");
        let (title, detail) = forecast(EnvironmentView::at(360.0));
        assert_eq!(title, "Drought 1 · ends in 1:00");
        assert!(detail.contains("50% on all islands"));
        assert_eq!(
            forecast(EnvironmentView::at(420.0)).0,
            "Calm · warning in 5:00"
        );
    }
}
