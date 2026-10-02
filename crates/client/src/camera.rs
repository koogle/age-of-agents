//! Three-quarter camera rig: pan, zoom and rotate around a ground target. Far
//! zoom tilts toward the horizon and bends the world into a small planet.
use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};

use crate::terrain::{COLUMNS, ROWS};

const MIN_DISTANCE: f32 = 5.0;
const MAX_DISTANCE: f32 = 70.0;
pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 200.0;
const FOV_Y: f32 = 36.0;

pub struct Rig {
    pub target: Vec3,
    pub distance: f32,
    yaw: f32,
    goal_yaw: f32,
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
        let yaw = std::f32::consts::FRAC_PI_4;
        Self {
            target: Vec3::new(15.0, 0.0, 10.5),
            distance: 15.0,
            yaw,
            goal_yaw: yaw,
            pitch: 0.92,
            width: 1.0,
            height: 1.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.yaw += (self.goal_yaw - self.yaw) * (dt * 8.0).min(1.0);
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
                self.yaw.sin() * horizontal,
                self.pitch.sin() * self.distance,
                self.yaw.cos() * horizontal,
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

    pub fn rotate(&mut self, radians: f32, smooth: bool) {
        self.goal_yaw += radians;
        if !smooth {
            self.yaw = self.goal_yaw;
        }
    }

    pub fn nudge(&mut self, dx: f32, dz: f32) {
        let right = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());
        let forward = Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos());
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
