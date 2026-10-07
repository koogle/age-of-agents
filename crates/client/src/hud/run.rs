//! The run's goal on screen: the temple marker on the globe and the artifact
//! above its bearer. Claim and victory speak through unit status feedback.
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

    #[test]
    fn the_artifact_marker_never_intercepts_taps() {
        let assets = pollster::block_on(crate::assets::Assets::load());
        let atlas = build_atlas(&assets);
        let mut hud = Hud::new();
        hud.artifact_marker(&atlas, Vec2::new(100.0, 100.0), 2.0);
        assert_eq!(hud.quads[0].uv, atlas.sprites["artifact"]);
        assert_eq!(hud.quads[0].rect[2], 56.0);
        assert!(hud.regions.is_empty());
    }
}
