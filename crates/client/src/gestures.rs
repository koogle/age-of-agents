//! Shared mouse/touch gestures; Shift-drag from ground selects instead of panning.
use crate::{App, hud};
use glam::{Vec2, Vec3};
use winit::event::{MouseButton, TouchPhase};

const DRAG_THRESHOLD: f32 = 8.0;

pub(super) struct Pointer {
    pub(super) on_hud: bool,
    pub(super) down_at: Vec2,
    pub(super) button: MouseButton,
    pub(super) grabbed: Option<Vec3>,
    pub(super) dragging: bool,
    pub(super) box_select: bool,
}

impl App {
    pub(super) fn press(&mut self, pixel: Vec2, button: MouseButton, shift: bool) {
        let on_hud = button == MouseButton::Left && self.hud.press(pixel);
        let box_select = shift
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
        });
    }

    /// One finger pans, taps and presses the HUD like the mouse; two fingers
    /// pinch to zoom and move together to pan.
    pub(super) fn touch(&mut self, id: u64, phase: TouchPhase, pixel: Vec2) {
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
                self.press(pixel, MouseButton::Left, false);
            }
            TouchPhase::Moved => self.moved(pixel),
            TouchPhase::Ended => self.release(pixel, false),
            TouchPhase::Cancelled => self.pointer = None,
        }
    }

    /// Two fingers moved from `before` to `after`.
    fn pinch(&mut self, before: (Vec2, Vec2), after: (Vec2, Vec2)) {
        let (span0, span1) = (before.1 - before.0, after.1 - after.0);
        if span0.length() > 8.0 && span1.length() > 8.0 {
            self.rig
                .zoom((span0.length() / span1.length()).clamp(0.8, 1.25));
        }
        let (mid0, mid1) = ((before.0 + before.1) * 0.5, (after.0 + after.1) * 0.5);
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
            return;
        }
        if matches!(self.build, hud::BuildUi::Placing(_)) && pointer.button == MouseButton::Left {
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

    pub(super) fn release(&mut self, pixel: Vec2, additive: bool) {
        let Some(pointer) = self.pointer.take() else {
            return;
        };
        if pointer.on_hud {
            if let Some(action) = self.hud.release() {
                self.act(action);
            }
        } else if pointer.box_select && pointer.dragging {
            let ids = self.view.units_in_box(&self.rig, pointer.down_at, pixel);
            self.selection.add_units(ids);
        } else if !pointer.dragging && pointer.button == MouseButton::Left {
            self.tap(pixel, additive);
        }
    }
}
