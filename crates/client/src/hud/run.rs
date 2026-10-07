//! The run's goal on screen: the temple marker on the globe, the artifact above
//! its bearer, and the victory pill.
use super::minimap::Minimap;
use super::*;
use aoa_game::{WORLD_COLUMNS, WORLD_ROWS};

impl Hud {
    /// The temple island's marker: the one place revealed through the fog.
    pub(super) fn temple_marker(
        &mut self,
        atlas: &Atlas,
        snapshot: &WorldSnapshot,
        map: &Minimap,
        globe: [f32; 4],
    ) {
        let cell = crate::terrain::CELL;
        let size = Vec2::new(globe[2], globe[3]);
        let site = snapshot.temple_site;
        let center = Vec2::new(
            site.column as f32 + WORLD_COLUMNS as f32 / 2.0,
            site.row as f32 + WORLD_ROWS as f32 / 2.0,
        ) * cell;
        let p = Vec2::new(globe[0], globe[1]) + map.local_of(center) * size;
        // One island region spans about this much of the globe.
        let span = (map.local_of(Vec2::new(WORLD_COLUMNS as f32, 0.0) * cell)
            - map.local_of(Vec2::ZERO))
        .length()
            * size.x;
        let d = (span * 1.3).max(34.0 * size.x / 136.0);
        self.sprite(
            atlas,
            "goal_temple",
            [p.x - d / 2.0, p.y - d * 0.62, d, d],
            [1.0; 4],
        );
    }

    /// The run's end: the artifact is home. Reset game starts a new run.
    pub(super) fn victory(&mut self, atlas: &Atlas, width: f32, top: f32, s: f32) {
        let text = "Victory · the Artifact of the Gods is home";
        let icon = 30.0 * s;
        let w = Self::text_width(atlas, text, 15.0 * s) + icon + 44.0 * s;
        let pill = [(width - w) / 2.0, top, w, 42.0 * s];
        self.shape(pill, GLASS, 1.0, 21.0 * s);
        let left = pill[0] + 16.0 * s;
        self.sprite(
            atlas,
            "artifact",
            [left, top + 6.0 * s, icon, icon],
            [1.0; 4],
        );
        let center = left + icon + 6.0 * s + Self::text_width(atlas, text, 15.0 * s) / 2.0;
        self.text(atlas, text, (center, top + 27.0 * s), 15.0 * s, INK, true);
    }

    /// The Artifact of the Gods floating above its bearer's health bar.
    pub fn artifact_marker(&mut self, atlas: &Atlas, top: Vec2, scale: f32) {
        let size = 28.0 * scale;
        let rect = [top.x - size / 2.0, top.y - 9.0 * scale - size, size, size];
        self.sprite(atlas, "artifact", rect, [1.0; 4]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{GameWorld, ScenarioOutcome};

    #[test]
    fn victory_shows_the_artifact_only_once_the_run_is_won() {
        let assets = pollster::block_on(crate::assets::Assets::load());
        let atlas = build_atlas(&assets);
        let mut snapshot = GameWorld::default().snapshot();
        let artifact = atlas.sprites["artifact"];
        for (outcome, shown) in [
            (ScenarioOutcome::Running, false),
            (ScenarioOutcome::Won, true),
        ] {
            snapshot.scenario.outcome = outcome;
            for (width, height, scale) in [(1280.0, 800.0, 1.0), (390.0, 844.0, 2.0)] {
                let model = Model {
                    resource_island: 0,
                    snapshot: Some(&snapshot),
                    units: &[],
                    building: None,
                    ship: None,
                    build: BuildUi::Off,
                    show_grid: false,
                    toast: None,
                    camera: Vec2::ZERO,
                };
                let mut hud = Hud::new();
                hud.layout(&atlas, &model, width * scale, height * scale, scale);
                let pill = hud.quads.iter().find(|q| q.uv == artifact);
                assert_eq!(pill.is_some(), shown);
                if let Some(pill) = pill {
                    assert!(pill.rect[0] >= 0.0 && pill.rect[0] + pill.rect[2] <= width * scale);
                }
            }
        }
        let mut hud = Hud::new();
        hud.artifact_marker(&atlas, Vec2::new(100.0, 100.0), 2.0);
        assert_eq!(hud.quads[0].uv, artifact);
        assert!(hud.regions.is_empty(), "the marker never intercepts taps");
    }
}
