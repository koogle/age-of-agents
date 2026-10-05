//! Shared mouse/touch gestures and platform-aware unit selection.
use crate::{App, hud};
use glam::{Vec2, Vec3};
use winit::event::{MouseButton, TouchPhase};
use winit::keyboard::ModifiersState;

const DRAG_THRESHOLD: f32 = 8.0;
const EDGE_PAN_HEIGHTS_PER_SECOND: f32 = 0.75;

fn additive_modifier(modifiers: ModifiersState, mac: bool) -> bool {
    if mac {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    }
}

pub(super) fn additive_selection(modifiers: ModifiersState) -> bool {
    #[cfg(target_arch = "wasm32")]
    let mac = web_sys::window()
        .and_then(|window| window.navigator().platform().ok())
        .is_some_and(|platform| platform.starts_with("Mac"));
    #[cfg(not(target_arch = "wasm32"))]
    let mac = cfg!(target_os = "macos");
    additive_modifier(modifiers, mac)
}

/// Gentle speed ramp through the outer 32 logical pixels; corners pan diagonally.
fn edge_direction(pixel: Vec2, size: Vec2, scale: f32) -> Vec2 {
    if pixel.x < 0.0 || pixel.y < 0.0 || pixel.x > size.x || pixel.y > size.y {
        return Vec2::ZERO;
    }
    let band = (32.0 * scale).min(size.min_element() * 0.25).max(1.0);
    let near = (Vec2::ONE - pixel / band).clamp(Vec2::ZERO, Vec2::ONE);
    let far = (Vec2::ONE - (size - pixel) / band).clamp(Vec2::ZERO, Vec2::ONE);
    Vec2::new(far.x - near.x, near.y - far.y).clamp_length_max(1.0)
}

pub(super) struct Pointer {
    pub(super) on_hud: bool,
    pub(super) down_at: Vec2,
    pub(super) button: MouseButton,
    pub(super) grabbed: Option<Vec3>,
    pub(super) dragging: bool,
    pub(super) box_select: bool,
    additive: bool,
}

impl App {
    pub(super) fn edge_pan(&mut self, dt: f32) {
        if !self.focused
            || !self.mouse_inside
            || self.pointer.is_some()
            || !self.touches.is_empty()
            || self.gesture
            || self.hud.covers(self.cursor)
        {
            return;
        }
        let Some(game) = &self.game else {
            return;
        };
        let direction = edge_direction(
            self.cursor,
            Vec2::new(self.rig.width, self.rig.height),
            game.window.scale_factor() as f32,
        );
        // Screen-space speed follows the visible extent as the camera zooms.
        self.rig
            .pan_screen(direction * self.rig.height * EDGE_PAN_HEIGHTS_PER_SECOND * dt);
    }

    pub(super) fn press(&mut self, pixel: Vec2, button: MouseButton, shift: bool, additive: bool) {
        let on_hud = button == MouseButton::Left && self.hud.press(pixel);
        let box_select = (shift || additive)
            && !on_hud
            && button == MouseButton::Left
            && self.build == hud::BuildUi::Off
            && matches!(self.target_at(pixel), Some(crate::Target::Ground(_)));
        self.pointer = Some(Pointer {
            on_hud,
            down_at: pixel,
            button,
            grabbed: self.ground_at(pixel),
            dragging: false,
            box_select,
            additive,
        });
    }

    /// One finger pans, taps and presses the HUD like the mouse; two fingers
    /// pinch to zoom and move together to pan.
    pub(super) fn touch(&mut self, id: u64, phase: TouchPhase, pixel: Vec2) {
        self.mouse_inside = false;
        let previous = self.touches.clone();
        match phase {
            TouchPhase::Started => self.touches.push((id, pixel)),
            TouchPhase::Moved => {
                if let Some(entry) = self.touches.iter_mut().find(|(t, _)| *t == id) {
                    entry.1 = pixel;
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => self.touches.retain(|(t, _)| *t != id),
        }
        if self.touches.len() >= 2 || (self.gesture && !self.touches.is_empty()) {
            if !self.gesture {
                // A second finger turns the press into a gesture: nothing is tapped.
                self.gesture = true;
                self.pointer = None;
                self.hud.release();
            }
            if let ([(a, a0), (b, b0), ..], [(c, a1), (d, b1), ..]) =
                (previous.as_slice(), self.touches.as_slice())
                && a == c
                && b == d
            {
                self.pinch((*a0, *b0), (*a1, *b1));
            }
            return;
        }
        if self.gesture {
            // The last finger of a gesture lifted.
            self.gesture = false;
            return;
        }
        match phase {
            TouchPhase::Started => {
                self.cursor = pixel;
                self.press(pixel, MouseButton::Left, false, false);
            }
            TouchPhase::Moved => self.moved(pixel),
            TouchPhase::Ended => self.release(pixel),
            TouchPhase::Cancelled => {
                self.pointer = None;
                self.hud.release();
            }
        }
    }

    /// Two fingers moved from `before` to `after`.
    fn pinch(&mut self, before: (Vec2, Vec2), after: (Vec2, Vec2)) {
        let (span0, span1) = (before.1 - before.0, after.1 - after.0);
        let (mid0, mid1) = ((before.0 + before.1) * 0.5, (after.0 + after.1) * 0.5);
        if span0.length() > 8.0 && span1.length() > 8.0 {
            let heights = &self.view.heights;
            self.rig.zoom_at(
                mid0,
                (span0.length() / span1.length()).clamp(0.8, 1.25),
                |x, z| heights.at(x, z),
            );
        }
        if let Some(grabbed) = self.ground_at(mid0)
            && let Some(now) = self.rig.plane_at(mid1, grabbed.y)
        {
            self.rig.drag(grabbed, now);
        }
    }

    pub(super) fn moved(&mut self, pixel: Vec2) {
        self.cursor = pixel;
        self.hud.hover = Some(pixel);
        let Some(pointer) = self.pointer.as_mut() else {
            return;
        };
        if pointer.on_hud {
            self.hud.drag_cargo(pointer.down_at, pixel, false);
            return;
        }
        if matches!(
            self.build,
            hud::BuildUi::Placing(_) | hud::BuildUi::PlacingField
        ) && pointer.button == MouseButton::Left
        {
            return;
        }
        if pointer.down_at.distance(pixel) > DRAG_THRESHOLD {
            pointer.dragging = true;
        }
        if pointer.dragging
            && !pointer.box_select
            && pointer.button != MouseButton::Right
            && let Some(grabbed) = pointer.grabbed
            && let Some(now) = self.rig.plane_at(pixel, grabbed.y)
        {
            self.rig.drag(grabbed, now);
        }
    }

    pub(super) fn release(&mut self, pixel: Vec2) {
        let Some(pointer) = self.pointer.take() else {
            return;
        };
        if pointer.on_hud {
            self.hud.drag_cargo(pointer.down_at, pixel, true);
            if let Some(action) = self.hud.release() {
                self.act(action);
            }
        } else if pointer.box_select && pointer.dragging {
            let ids = self.view.units_in_box(&self.rig, pointer.down_at, pixel);
            self.selection.select_units(ids, pointer.additive);
        } else if !pointer.dragging && pointer.button == MouseButton::Left {
            self.tap(pixel, pointer.additive);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_adds_only_with_the_platform_primary_modifier() {
        for (modifiers, mac, other) in [
            (ModifiersState::empty(), false, false),
            (ModifiersState::SHIFT, false, false),
            (ModifiersState::CONTROL, false, true),
            (ModifiersState::SUPER, true, false),
            (ModifiersState::SHIFT | ModifiersState::CONTROL, false, true),
            (ModifiersState::SHIFT | ModifiersState::SUPER, true, false),
        ] {
            assert_eq!(additive_modifier(modifiers, true), mac);
            assert_eq!(additive_modifier(modifiers, false), other);
        }
    }

    #[test]
    fn edges_and_corners_pan_toward_the_pointer_without_diagonal_speedup() {
        let size = Vec2::new(1280.0, 800.0);
        for (pixel, expected) in [
            (Vec2::new(0.0, 400.0), -Vec2::X),
            (Vec2::new(1280.0, 400.0), Vec2::X),
            (Vec2::new(640.0, 0.0), Vec2::Y),
            (Vec2::new(640.0, 800.0), -Vec2::Y),
            (Vec2::ZERO, Vec2::new(-1.0, 1.0).normalize()),
            (size, Vec2::new(1.0, -1.0).normalize()),
            (Vec2::new(1280.0, 0.0), Vec2::ONE.normalize()),
            (Vec2::new(0.0, 800.0), -Vec2::ONE.normalize()),
        ] {
            assert!(edge_direction(pixel, size, 1.0).abs_diff_eq(expected, 1e-6));
        }
    }

    #[test]
    fn edge_speed_ramps_in_logical_pixels_and_stops_in_the_interior_or_outside() {
        let size = Vec2::new(1280.0, 800.0);
        for scale in [1.0, 2.0] {
            for (x, speed) in [(0.0, -1.0), (16.0, -0.5), (32.0, 0.0), (640.0, 0.0)] {
                assert_eq!(
                    edge_direction(Vec2::new(x, 400.0) * scale, size * scale, scale),
                    Vec2::new(speed, 0.0)
                );
            }
        }
        for pixel in [
            Vec2::new(-1.0, 400.0),
            Vec2::new(1281.0, 400.0),
            Vec2::new(640.0, -1.0),
            Vec2::new(640.0, 801.0),
        ] {
            assert_eq!(edge_direction(pixel, size, 1.0), Vec2::ZERO);
        }
    }
}
