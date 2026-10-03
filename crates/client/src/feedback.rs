//! Short-lived deposit labels inferred from authoritative cargo transitions.
use aoa_game::WorldSnapshot;
use glam::Vec3;

use crate::{camera::Rig, hud, terrain};

const LIFETIME: f64 = 1.4;

struct Gain {
    text: String,
    at: Vec3,
    born: f64,
}

#[derive(Default)]
pub struct Feedback {
    gains: Vec<Gain>,
}

impl Feedback {
    pub fn observe(&mut self, previous: Option<&WorldSnapshot>, next: &WorldSnapshot, now: f64) {
        let Some(previous) = previous else {
            return;
        };
        if next.tick < previous.tick {
            self.gains.clear();
            return;
        }
        for unit in &next.units {
            let Some(before) = previous
                .units
                .iter()
                .find(|old| old.unit.id == unit.unit.id)
            else {
                continue;
            };
            let Some(cargo) = &before.unit.cargo else {
                continue;
            };
            if unit.unit.cargo.is_some() || cargo.amount <= 0.0 {
                continue;
            }
            let at = terrain::world_of(unit.position.x, unit.position.y);
            self.gains.push(Gain {
                text: format!(
                    "+{:.0} {}",
                    cargo.amount,
                    format!("{:?}", cargo.kind).to_lowercase()
                ),
                at: Vec3::new(at.x, 0.8, at.y),
                born: now,
            });
        }
        // Presentation stays bounded even after a burst of network snapshots.
        if self.gains.len() > 64 {
            self.gains.drain(..self.gains.len() - 64);
        }
    }

    pub fn draw(
        &mut self,
        hud: &mut hud::Hud,
        atlas: &hud::Atlas,
        rig: &Rig,
        heights: &terrain::Heights,
        now: f64,
    ) {
        self.gains.retain(|gain| now - gain.born < LIFETIME);
        for gain in &self.gains {
            let age = (now - gain.born) as f32;
            let position = gain.at + Vec3::Y * (heights.at(gain.at.x, gain.at.z) + age * 0.45);
            if let Some(at) = rig.screen_of(position) {
                let (_, up) = rig.basis();
                let Some(top) = rig.screen_offset(position, up) else {
                    continue;
                };
                let pixels_per_world = at.distance(top);
                let alpha = (age * 6.0).min(1.0) * (1.0 - ((age - 0.9) / 0.5).max(0.0));
                hud.gain_label(atlas, &gain.text, at, pixels_per_world, alpha);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{CarriedResource, GameWorld, GatherPhase, ResourceKind, UnitAction};

    #[test]
    fn every_unloading_task_emits_a_gain() {
        for action in [
            UnitAction::Deposit {
                building_id: "base-1".into(),
            },
            UnitAction::Build {
                building_id: "base-1".into(),
            },
            UnitAction::Gather {
                resource_id: "tree-1".into(),
                phase: GatherPhase::Depositing,
            },
        ] {
            let mut before = GameWorld::default().snapshot();
            before.units[0].unit.action = action;
            before.units[0].unit.cargo = Some(CarriedResource {
                kind: ResourceKind::Wood,
                amount: 20.0,
            });
            let mut next = before.clone();
            next.tick += 1;
            next.units[0].unit.cargo = None;
            let mut feedback = Feedback::default();
            feedback.observe(Some(&before), &next, 1.0);
            assert_eq!(feedback.gains.len(), 1);
        }
    }

    #[test]
    fn deposits_show_once_but_loading_stopping_and_resetting_do_not() {
        let world = GameWorld::default();
        let mut before = world.snapshot();
        before.tick = 10;
        before.units[0].unit.cargo = Some(CarriedResource {
            kind: ResourceKind::Wood,
            amount: 20.0,
        });
        before.units[0].unit.action = UnitAction::Gather {
            resource_id: "tree-1".into(),
            phase: GatherPhase::Depositing,
        };
        let mut next = before.clone();
        next.tick += 1;
        next.units[0].unit.cargo = None;
        next.units[0].unit.action = UnitAction::Idle;
        let mut feedback = Feedback::default();
        feedback.observe(None, &before, 0.0);
        assert!(feedback.gains.is_empty());
        feedback.observe(Some(&before), &next, 1.0);
        assert_eq!(feedback.gains.len(), 1);
        assert_eq!(feedback.gains[0].text, "+20 wood");
        feedback.observe(Some(&next), &next, 1.1);
        assert_eq!(feedback.gains.len(), 1);
        let mut stopped = before.clone();
        stopped.units[0].unit.action = UnitAction::Idle;
        feedback.observe(Some(&before), &stopped, 1.2);
        assert_eq!(feedback.gains.len(), 1);
        feedback.observe(Some(&before), &world.snapshot(), 1.3);
        assert!(feedback.gains.is_empty());
    }
}
