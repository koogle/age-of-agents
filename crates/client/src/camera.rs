//! Three-quarter camera rig: pan and zoom around a ground target with a fixed heading. Far
//! zoom tilts toward the horizon and bends the world into a small planet.
use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};

use crate::terrain::{COLUMNS, ROWS};

const MIN_DISTANCE: f32 = 5.0;
const MAX_DISTANCE: f32 = 70.0;
pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 200.0;
const FOV_Y: f32 = 36.0;
const YAW: f32 = std::f32::consts::FRAC_PI_4;

pub struct Rig {
    pub target: Vec3,
    pub distance: f32,
    pitch: f32,
    pub width: f32,
    pub height: f32,
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Rig {
    pub fn new() -> Self {
        Self {
            target: Vec3::new(15.0, 0.0, 10.5),
            distance: 15.0,
            pitch: 0.92,
            width: 1.0,
            height: 1.0,
        }
    }

    pub fn update(&mut self) {
        // Closer views tilt toward the horizon; distant views look down like a
        // map, then ease back so the planet's limb shows.
        let play = ((self.distance - MIN_DISTANCE) / (34.0 - MIN_DISTANCE)).clamp(0.0, 1.0);
        let orbit = smoothstep(34.0, MAX_DISTANCE, self.distance);
        let near_pitch = 0.72 + (1.12 - 0.72) * play;
        self.pitch = near_pitch + (0.78 - near_pitch) * orbit;
    }

    pub fn eye(&self) -> Vec3 {
        let horizontal = self.pitch.cos() * self.distance;
        self.target
            + Vec3::new(
                YAW.sin() * horizontal,
                self.pitch.sin() * self.distance,
                YAW.cos() * horizontal,
            )
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }

    pub fn projection(&self) -> Mat4 {
        Mat4::perspective_rh(
            FOV_Y.to_radians(),
            self.width / self.height.max(1.0),
            NEAR,
            FAR,
        )
    }

    pub fn view_proj(&self) -> Mat4 {
        self.projection() * self.view()
    }

    /// Planet bend strength for the current zoom.
    pub fn curve(&self) -> f32 {
        smoothstep(24.0, 70.0, self.distance) * 0.014
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
        if self.distance > 30.0 {
            self.target = self
                .target
                .lerp(Vec3::new(COLUMNS / 2.0, 0.0, ROWS / 2.0), 0.08);
        }
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
    fn navigation_preserves_heading() {
        let mut rig = Rig::new();
        let heading = |rig: &Rig| {
            let offset = rig.eye() - rig.target;
            Vec2::new(offset.x, offset.z).normalize()
        };
        let initial = heading(&rig);
        for factor in [0.4, 2.0, 10.0] {
            rig.zoom(factor);
            rig.update();
            rig.nudge(1.0, -1.0);
            rig.drag(Vec3::new(10.0, 0.0, 8.0), Vec3::new(9.0, 0.0, 7.0));
            rig.look_at(12.0, 9.0);
            assert!(heading(&rig).abs_diff_eq(initial, 1e-6));
        }
    }
}
