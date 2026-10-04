//! Fixed-angle isometric camera: pan and zoom around a ground target.
use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};

use crate::terrain::{COLUMNS, ROWS};

const MIN_DISTANCE: f32 = 5.0;
const MAX_DISTANCE: f32 = 140.0;
pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 400.0;
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

    /// Keep settlement views flat; ease into the distant planet view.
    pub fn curve(&self) -> f32 {
        let t = ((self.distance - 60.0) / (MAX_DISTANCE - 60.0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t) * 0.007
    }

    /// Match common.wgsl so picking and overlays follow the rendered ground.
    fn bend(&self, mut world: Vec3) -> Vec3 {
        let away = Vec2::new(world.x - self.target.x, world.z - self.target.z);
        world.y -= self.curve() * away.length_squared();
        world
    }

    fn unbend(&self, mut world: Vec3) -> Vec3 {
        let away = Vec2::new(world.x - self.target.x, world.z - self.target.z);
        world.y += self.curve() * away.length_squared();
        world
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
        self.screen_offset(world, Vec3::ZERO)
    }

    /// Billboards bend at their anchor, then extend in the camera plane.
    pub fn screen_offset(&self, anchor: Vec3, offset: Vec3) -> Option<Vec2> {
        let clip = self.view_proj() * (self.bend(anchor) + offset).extend(1.0);
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

    /// The point at unbent ground `height` under a pixel; sky has no hit.
    pub fn plane_at(&self, pixel: Vec2, height: f32) -> Option<Vec3> {
        let (near, direction) = self.ray(pixel);
        // Intersect the ray with y = height - curve * distance_from_target².
        let away = Vec2::new(near.x - self.target.x, near.z - self.target.z);
        let horizontal = Vec2::new(direction.x, direction.z);
        let a = self.curve() * horizontal.length_squared();
        let b = direction.y + 2.0 * self.curve() * away.dot(horizontal);
        let c = near.y - height + self.curve() * away.length_squared();
        let t = if a < 1e-6 {
            if b.abs() < 1e-6 {
                return None;
            }
            -c / b
        } else {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 {
                return None; // Sky above the curved horizon.
            }
            let root = discriminant.sqrt();
            // Avoid cancellation as the curve first eases away from zero.
            let q = -0.5 * (b + root.copysign(b));
            let first = (q / a).min(c / q);
            let second = (q / a).max(c / q);
            if first > 0.0 { first } else { second }
        };
        (t > 0.0 && t <= 1.0).then(|| self.unbend(near + direction * t))
    }

    /// The first point where the view ray through a pixel meets the terrain
    /// surface `height(x, z)`: marched from above the highest hill, then refined.
    pub fn ground_at(&self, pixel: Vec2, height: impl Fn(f32, f32) -> f32) -> Option<Vec3> {
        const TOP: f32 = 2.0;
        const BOTTOM: f32 = -0.5;
        let (near, _) = self.ray(pixel);
        let mut from = self
            .plane_at(pixel, TOP)
            .map(|p| self.bend(p))
            .unwrap_or(near);
        let to = self.bend(self.plane_at(pixel, BOTTOM)?);
        let steps = ((to - from).length() / 0.05).ceil().max(1.0) as usize;
        let step = (to - from) / steps as f32;
        let below = |p: Vec3| self.unbend(p).y <= height(p.x, p.z);
        if below(from) {
            return Some(self.unbend(from));
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
                return Some(self.unbend(under));
            }
            from = next;
        }
        Some(self.unbend(to))
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

    /// Zoom while keeping the terrain under `pixel` at the same screen position.
    /// Over the sky, or beyond the pan bounds, retain the normal camera limits.
    pub fn zoom_at(&mut self, pixel: Vec2, factor: f32, height: impl Fn(f32, f32) -> f32) {
        let grabbed = self.ground_at(pixel, height);
        self.zoom(factor);
        if let Some(grabbed) = grabbed
            && let Some(now) = self.plane_at(pixel, grabbed.y)
        {
            self.drag(grabbed, now);
        }
    }

    /// Move toward screen right/up by physical pixels, compensating for zoom
    /// and the ground's isometric foreshortening.
    pub fn pan_screen(&mut self, pixels: Vec2) {
        let units_per_pixel =
            2.0 * self.distance * (FOV_Y.to_radians() * 0.5).tan() / self.height.max(1.0);
        let right = Vec3::new(YAW.cos(), 0.0, -YAW.sin());
        let forward = Vec3::new(-YAW.sin(), 0.0, -YAW.cos());
        self.target += (right * pixels.x + forward * (pixels.y / PITCH.sin())) * units_per_pixel;
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
        (
            Vec3::new(YAW.cos(), 0.0, -YAW.sin()),
            Vec3::new(
                -PITCH.sin() * YAW.sin(),
                PITCH.cos(),
                -PITCH.sin() * YAW.cos(),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_pan_tracks_zoom_dpi_and_direction() {
        for scale in [1.0, 2.0] {
            for distance in [MIN_DISTANCE, 15.0, 60.0, MAX_DISTANCE] {
                for direction in [Vec2::X, Vec2::Y, -Vec2::ONE.normalize()] {
                    let mut rig = Rig::new();
                    rig.width = 1280.0 * scale;
                    rig.height = 800.0 * scale;
                    rig.distance = distance;
                    rig.look_at(COLUMNS / 2.0, ROWS / 2.0);
                    let point = rig.target;
                    let before = rig.screen_of(point).unwrap();
                    let pixels = direction * 5.0 * scale;
                    rig.pan_screen(pixels);
                    let movement = (rig.screen_of(point).unwrap() - before) / scale;
                    let expected = Vec2::new(-direction.x, direction.y) * 5.0;
                    // Curved overview terrain adds a small second-order vertical shift.
                    assert!(
                        movement.abs_diff_eq(expected, 0.1),
                        "{movement:?} != {expected:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn screen_pan_is_frame_rate_independent_and_bounded() {
        for distance in [MIN_DISTANCE, 15.0, MAX_DISTANCE] {
            let mut targets = Vec::new();
            for frames in [30, 60, 144] {
                let mut rig = Rig::new();
                rig.height = 800.0;
                rig.distance = distance;
                rig.look_at(COLUMNS / 2.0, ROWS / 2.0);
                for _ in 0..frames {
                    rig.pan_screen(Vec2::new(60.0, 60.0) / frames as f32);
                }
                targets.push(rig.target);
                rig.pan_screen(Vec2::splat(100_000.0));
                assert!((-1.0..=COLUMNS + 1.0).contains(&rig.target.x));
                assert!((-1.0..=ROWS + 1.0).contains(&rig.target.z));
            }
            assert!(
                targets
                    .iter()
                    .all(|target| target.abs_diff_eq(targets[0], 0.001))
            );
        }
    }

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
    fn distant_view_preserves_ground_picking_on_desktop_and_phone() {
        for (width, height) in [(1280.0, 800.0), (390.0, 844.0)] {
            let mut rig = Rig::new();
            rig.width = width;
            rig.height = height;
            rig.look_at(COLUMNS / 2.0, ROWS / 2.0);
            assert_eq!(rig.curve(), 0.0);
            rig.zoom(100.0);
            assert_eq!(rig.distance, MAX_DISTANCE);
            assert!(rig.curve() > 0.0);
            for offset in [
                Vec3::ZERO,
                Vec3::new(8.0, 0.0, -5.0),
                Vec3::new(-12.0, 0.0, 4.0),
            ] {
                let point = rig.target + offset;
                let pixel = rig.screen_of(point).unwrap();
                let picked = rig.ground_at(pixel, |_, _| 0.0).unwrap();
                assert!(picked.abs_diff_eq(point, 0.002), "{picked:?} != {point:?}");
                let clip = rig.view_proj() * rig.bend(point).extend(1.0);
                assert!((0.0..1.0).contains(&clip.z));
            }
            rig.zoom(0.001);
            assert_eq!(rig.curve(), 0.0);
            assert_eq!(rig.distance, MIN_DISTANCE);
        }
    }

    #[test]
    fn curve_transition_and_horizon_have_stable_picking() {
        let mut rig = Rig::new();
        rig.width = 1280.0;
        rig.height = 800.0;
        for distance in [60.0, 60.001, 60.1, 80.0, 100.0, MAX_DISTANCE] {
            rig.distance = distance;
            let point = rig.target + Vec3::new(5.0, 0.7, -3.0);
            let pixel = rig.screen_of(point).unwrap();
            assert!(
                rig.plane_at(pixel, point.y)
                    .unwrap()
                    .abs_diff_eq(point, 0.002)
            );
        }
        assert!(rig.ground_at(Vec2::new(640.0, 0.0), |_, _| 0.0).is_none());
    }

    #[test]
    fn pointer_zoom_preserves_raised_ground_at_all_distances_and_pixel_scales() {
        for (width, height) in [(1280.0, 800.0), (780.0, 1688.0)] {
            for distance in [5.0, 15.0, 60.0, 80.0, MAX_DISTANCE] {
                for factor in [0.5, 0.9, 1.1, 1.5] {
                    for offset in [Vec3::new(2.0, 0.7, -1.0), Vec3::new(-2.0, 0.7, 1.0)] {
                        let mut rig = Rig::new();
                        rig.width = width;
                        rig.height = height;
                        rig.look_at(COLUMNS / 2.0, ROWS / 2.0);
                        rig.distance = distance;
                        let point = rig.target + offset;
                        let pixel = rig.screen_of(point).unwrap();
                        rig.zoom_at(pixel, factor, |_, _| point.y);
                        let after = rig.screen_of(point).unwrap();
                        assert!(
                            after.abs_diff_eq(pixel, 0.05),
                            "distance={distance}, factor={factor}: {after:?} != {pixel:?}"
                        );
                        assert_eq!(
                            rig.distance,
                            (distance * factor).clamp(MIN_DISTANCE, MAX_DISTANCE)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn pointer_zoom_over_sky_and_at_pan_bounds_stays_finite_and_bounded() {
        let mut rig = Rig::new();
        rig.width = 1280.0;
        rig.height = 800.0;
        rig.distance = MAX_DISTANCE;
        let sky = Vec2::new(640.0, 0.0);
        assert!(rig.ground_at(sky, |_, _| 0.0).is_none());
        let target = rig.target;
        rig.zoom_at(sky, 0.9, |_, _| 0.0);
        assert_eq!(rig.target, target);
        for (x, z) in [(-1.0, -1.0), (COLUMNS + 1.0, ROWS + 1.0)] {
            rig.look_at(x, z);
            for pixel in [Vec2::ZERO, Vec2::new(1280.0, 800.0)] {
                rig.zoom_at(pixel, 0.5, |_, _| 0.0);
                assert!(rig.target.is_finite());
                assert!((-1.0..=COLUMNS + 1.0).contains(&rig.target.x));
                assert!((-1.0..=ROWS + 1.0).contains(&rig.target.z));
            }
        }
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
