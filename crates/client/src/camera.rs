//! Fixed-angle isometric camera: pan and zoom around a ground target.
use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};

use crate::terrain::{COLUMNS, ROWS};

const MIN_DISTANCE: f32 = 5.0;
const MAX_DISTANCE: f32 = 70.0;
pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 200.0;
const FOV_Y: f32 = 36.0;
const YAW: f32 = std::f32::consts::FRAC_PI_4;
// atan(1 / sqrt(2)): square-cell edges project at the art's 30-degree angle.
pub(crate) const PITCH: f32 = 0.6154797;

pub struct Rig {
    pub target: Vec3,
    pub distance: f32,
    pub width: f32,
    pub height: f32,
}

impl Rig {
    pub fn new() -> Self {
        Self {
            target: Vec3::new(15.0, 0.0, 10.5),
            distance: 15.0,
            width: 1.0,
            height: 1.0,
        }
    }

    pub fn eye(&self) -> Vec3 {
        let horizontal = PITCH.cos() * self.distance;
        self.target
            + Vec3::new(
                YAW.sin() * horizontal,
                PITCH.sin() * self.distance,
                YAW.cos() * horizontal,
            )
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }

    pub fn projection(&self) -> Mat4 {
        let half_height = self.distance * (FOV_Y.to_radians() * 0.5).tan();
        let half_width = half_height * self.width / self.height.max(1.0);
        Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            NEAR,
            FAR,
        )
    }

    pub fn view_proj(&self) -> Mat4 {
        self.projection() * self.view()
    }

    /// Screen pixel (top-left origin) of a world point.
    pub fn screen_of(&self, world: Vec3) -> Option<Vec2> {
        let clip = self.view_proj() * world.extend(1.0);
        if clip.w <= 0.0 {
            return None;
        }
        let ndc = clip.xyz() / clip.w;
        Some(Vec2::new(
            (ndc.x + 1.0) * 0.5 * self.width,
            (1.0 - ndc.y) * 0.5 * self.height,
        ))
    }

    /// The view ray through a screen pixel: origin on the near plane and the
    /// (unnormalized) direction to the far plane.
    fn ray(&self, pixel: Vec2) -> (Vec3, Vec3) {
        let ndc = Vec2::new(
            pixel.x / self.width * 2.0 - 1.0,
            1.0 - pixel.y / self.height * 2.0,
        );
        let inverse = self.view_proj().inverse();
        let near = inverse.project_point3(ndc.extend(0.0));
        let far = inverse.project_point3(ndc.extend(1.0));
        (near, far - near)
    }

    /// The point on the horizontal plane at `height` under a screen pixel.
    pub fn plane_at(&self, pixel: Vec2, height: f32) -> Option<Vec3> {
        let (near, direction) = self.ray(pixel);
        if direction.y.abs() < 1e-6 {
            return None;
        }
        let t = (height - near.y) / direction.y;
        (t > 0.0).then(|| near + direction * t)
    }

    /// The first point where the view ray through a pixel meets the terrain
    /// surface `height(x, z)`: marched from above the highest hill, then refined.
    pub fn ground_at(&self, pixel: Vec2, height: impl Fn(f32, f32) -> f32) -> Option<Vec3> {
        const TOP: f32 = 2.0;
        const BOTTOM: f32 = -0.5;
        let (near, _) = self.ray(pixel);
        let mut from = self.plane_at(pixel, TOP).unwrap_or(near);
        let to = self.plane_at(pixel, BOTTOM)?;
        let steps = ((to - from).length() / 0.05).ceil().max(1.0) as usize;
        let step = (to - from) / steps as f32;
        let below = |p: Vec3| p.y <= height(p.x, p.z);
        if below(from) {
            return Some(from);
        }
        for _ in 0..steps {
            let next = from + step;
            if below(next) {
                let (mut above, mut under) = (from, next);
                for _ in 0..8 {
                    let middle = (above + under) * 0.5;
                    if below(middle) {
                        under = middle;
                    } else {
                        above = middle;
                    }
                }
                return Some(under);
            }
            from = next;
        }
        Some(to)
    }

    fn clamp(&mut self) {
        self.distance = self.distance.clamp(MIN_DISTANCE, MAX_DISTANCE);
        self.target.x = self.target.x.clamp(-1.0, COLUMNS + 1.0);
        self.target.z = self.target.z.clamp(-1.0, ROWS + 1.0);
    }

    /// Keep the ground point grabbed at `from` under the pointer now at `to`.
    pub fn drag(&mut self, from: Vec3, to: Vec3) {
        self.target += Vec3::new(from.x - to.x, 0.0, from.z - to.z);
        self.clamp();
    }

    pub fn zoom(&mut self, factor: f32) {
        self.distance *= factor;
        self.clamp();
    }

    pub fn nudge(&mut self, dx: f32, dz: f32) {
        let right = Vec3::new(YAW.cos(), 0.0, -YAW.sin());
        let forward = Vec3::new(-YAW.sin(), 0.0, -YAW.cos());
        self.target += (right * dx + forward * dz) * self.distance * 0.05;
        self.clamp();
    }

    pub fn look_at(&mut self, x: f32, z: f32) {
        self.target = Vec3::new(x, 0.0, z);
        self.clamp();
    }

    /// Camera right and up vectors in world space, for billboards.
    pub fn basis(&self) -> (Vec3, Vec3) {
        let view = self.view();
        (view.row(0).xyz(), view.row(1).xyz())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_and_zoom_keep_ground_geometry_aligned() {
        let mut rig = Rig::new();
        rig.width = 900.0;
        rig.height = 650.0;
        let point = Vec3::new(10.0, 0.0, 10.0);
        let edge =
            |rig: &Rig| rig.screen_of(point + Vec3::X).unwrap() - rig.screen_of(point).unwrap();
        let before = edge(&rig);
        rig.look_at(20.0, 15.0);
        assert!(edge(&rig).abs_diff_eq(before, 1e-4));
        let target = rig.target;
        rig.zoom(4.0);
        assert_eq!(rig.target, target);
        assert!(edge(&rig).abs_diff_eq(before / 4.0, 1e-4));
    }

    #[test]
    fn navigation_preserves_heading() {
        let mut rig = Rig::new();
        let heading = |rig: &Rig| {
            let offset = rig.eye() - rig.target;
            Vec2::new(offset.x, offset.z).normalize()
        };
        let initial = heading(&rig);
        for factor in [0.4, 2.0, 10.0] {
            rig.zoom(factor);
            rig.nudge(1.0, -1.0);
            rig.drag(Vec3::new(10.0, 0.0, 8.0), Vec3::new(9.0, 0.0, 7.0));
            rig.look_at(12.0, 9.0);
            assert!(heading(&rig).abs_diff_eq(initial, 1e-6));
        }
    }
}
