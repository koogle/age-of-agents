//! One isometric mapping for terrain sampling, camera markers and navigation.
use super::{Atlas, Hud, Quad};
use crate::camera::Rig;
use aoa_game::{WORLD_COLUMNS, WORLD_ROWS, WorldSnapshot};
use glam::{Mat2, Vec2};

pub(super) struct Minimap {
    center: Vec2,
    projection: Mat2,
}

impl Minimap {
    pub(super) fn new(size: Vec2) -> Self {
        let center = size * 0.5;
        let (right, up) = Rig::new().basis();
        let projection = Mat2::from_cols(Vec2::new(right.x, -up.x), Vec2::new(right.z, -up.z));
        // Fit all four projected map corners inside the circular frame.
        let radius = (projection * center)
            .length()
            .max((projection * Vec2::new(center.x, -center.y)).length());
        Self {
            center,
            projection: projection / (radius * 2.0),
        }
    }

    pub(super) fn local_of(&self, world: Vec2) -> Vec2 {
        Vec2::splat(0.5) + self.projection * (world - self.center)
    }

    pub(super) fn world_at(&self, local: Vec2) -> Vec2 {
        self.center + self.projection.inverse() * (local - Vec2::splat(0.5))
    }

    /// The globe quad; `texture` is the world size the cell texture covers,
    /// which may be smaller than the fitted map. The fitted map's extent in
    /// texture UV rides in `color.xy` so the shader can fog it like unexplored land.
    pub(super) fn quad(&self, rect: [f32; 4], texture: Vec2) -> Quad {
        let inverse = self.projection.inverse();
        let horizontal = inverse.x_axis / texture;
        let vertical = inverse.y_axis / texture;
        let center = self.center / texture;
        Quad {
            rect,
            uv: [center.x, center.y, horizontal.x, horizontal.y],
            color: [
                self.center.x * 2.0 / texture.x,
                self.center.y * 2.0 / texture.y,
                1.0,
                1.0,
            ],
            params: [3.0, vertical.x, vertical.y, 0.0],
        }
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3Swizzles;

    #[test]
    fn axes_match_the_world_view_and_corners_fit_the_globe() {
        let map = Minimap::new(Vec2::new(152.0, 112.0));
        let mut rig = Rig::new();
        rig.width = 800.0;
        rig.height = 600.0;
        let screen_center = rig.screen_of(rig.target).unwrap();
        for axis in [glam::Vec3::X, glam::Vec3::Z] {
            let screen = rig.screen_of(rig.target + axis).unwrap() - screen_center;
            let mini = map.local_of(map.center + axis.xz()) - Vec2::splat(0.5);
            assert!(screen.normalize().abs_diff_eq(mini.normalize(), 1e-5));
        }
        for point in [
            Vec2::ZERO,
            map.center * 2.0,
            Vec2::new(map.center.x * 2.0, 0.0),
            Vec2::new(0.0, map.center.y * 2.0),
            map.center,
        ] {
            let local = map.local_of(point);
            assert!(local.distance(Vec2::splat(0.5)) <= 0.50001);
            assert!(map.world_at(local).abs_diff_eq(point, 1e-4));
            // Reproduce the vertex shader's cross-axis texture coordinates.
            let q = map.quad([0.0, 0.0, 80.0, 80.0], map.center * 2.0);
            let uv = Vec2::new(q.uv[0], q.uv[1])
                + (local.x - 0.5) * Vec2::new(q.uv[2], q.uv[3])
                + (local.y - 0.5) * Vec2::new(q.params[1], q.params[2]);
            assert!((uv * map.center * 2.0).abs_diff_eq(point, 1e-4));
        }
    }
}
