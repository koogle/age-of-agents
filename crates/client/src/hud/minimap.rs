//! One isometric mapping for terrain sampling, camera markers and navigation.
use super::Quad;
use crate::camera::Rig;
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

    pub(super) fn quad(&self, rect: [f32; 4]) -> Quad {
        let size = self.center * 2.0;
        let inverse = self.projection.inverse();
        let horizontal = inverse.x_axis / size;
        let vertical = inverse.y_axis / size;
        Quad {
            rect,
            uv: [0.5, 0.5, horizontal.x, horizontal.y],
            color: [1.0; 4],
            params: [3.0, vertical.x, vertical.y, 0.0],
        }
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
            let q = map.quad([0.0, 0.0, 80.0, 80.0]);
            let uv = Vec2::new(q.uv[0], q.uv[1])
                + (local.x - 0.5) * Vec2::new(q.uv[2], q.uv[3])
                + (local.y - 0.5) * Vec2::new(q.params[1], q.params[2]);
            assert!((uv * map.center * 2.0).abs_diff_eq(point, 1e-4));
        }
    }
}
