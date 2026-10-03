//! Drop-off status and short-lived labels from authoritative cargo state.
use aoa_game::{GatherPhase, Unit, UnitAction, WorldSnapshot};
use glam::Vec3;

use crate::{camera::Rig, hud, terrain};

const LIFETIME: f64 = 1.4;

fn drop_off_status(unit: &Unit) -> Option<String> {
    let cargo = unit.cargo.as_ref()?;
    let unloading = matches!(
        unit.action,
        UnitAction::Build { .. }
            | UnitAction::Cultivate { .. }
            | UnitAction::Deposit { .. }
            | UnitAction::Gather {
                phase: GatherPhase::Returning | GatherPhase::Depositing,
                ..
            }
    );
    (unloading && cargo.amount > 0.0).then(|| {
        format!(
            "Dropping off {}",
            format!("{:?}", cargo.kind).to_lowercase()
        )
    })
}

struct Label {
    text: String,
    at: Vec3,
    born: f64,
}

#[derive(Default)]
pub struct Feedback {
    labels: Vec<Label>,
}

impl Feedback {
    pub fn observe(&mut self, previous: Option<&WorldSnapshot>, next: &WorldSnapshot, now: f64) {
        let Some(previous) = previous else {
            return;
        };
        if next.tick < previous.tick {
            self.labels.clear();
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
            let at = terrain::world_of(unit.position.x, unit.position.y);
            if let Some(text) = drop_off_status(&unit.unit)
                && drop_off_status(&before.unit).as_ref() != Some(&text)
            {
                self.labels.push(Label {
                    text,
                    at: Vec3::new(at.x, 0.8, at.y),
                    born: now,
                });
            }
            let Some(cargo) = &before.unit.cargo else {
                continue;
            };
            if unit.unit.cargo.is_some() || cargo.amount <= 0.0 {
                continue;
            }
            self.labels.push(Label {
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
        if self.labels.len() > 64 {
            self.labels.drain(..self.labels.len() - 64);
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
        self.labels.retain(|label| now - label.born < LIFETIME);
        for label in &self.labels {
            let age = (now - label.born) as f32;
            let position = label.at + Vec3::Y * (heights.at(label.at.x, label.at.z) + age * 0.45);
            if let Some(at) = rig.screen_of(position) {
                let (_, up) = rig.basis();
                let Some(top) = rig.screen_offset(position, up) else {
                    continue;
                };
                let pixels_per_world = at.distance(top);
                let alpha = (age * 6.0).min(1.0) * (1.0 - ((age - 0.9) / 0.5).max(0.0));
                hud.gain_label(atlas, &label.text, at, pixels_per_world, alpha);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{CarriedResource, GameWorld, GatherPhase, ResourceKind, UnitAction};

    #[test]
    fn unloading_announces_once_using_the_gain_animation() {
        let mut before = GameWorld::default().snapshot();
        before.units[0].unit.cargo = Some(CarriedResource {
            kind: ResourceKind::Wood,
            amount: 7.0,
        });
        let mut next = before.clone();
        next.units[0].unit.action = UnitAction::Gather {
            resource_id: "food-1".into(),
            phase: GatherPhase::Returning,
        };
        let mut feedback = Feedback::default();
        feedback.observe(None, &next, 0.0);
        assert!(feedback.labels.is_empty());
        feedback.observe(Some(&before), &next, 1.0);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "Dropping off wood");
        assert_eq!(feedback.labels[0].born, 1.0);
        // Repeated snapshots and arrival at the drop site do not replay it.
        feedback.observe(Some(&next), &next, 2.0);
        let mut arrived = next.clone();
        arrived.units[0].unit.action = UnitAction::Gather {
            resource_id: "food-1".into(),
            phase: GatherPhase::Depositing,
        };
        feedback.observe(Some(&next), &arrived, 3.0);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].born, 1.0);
        // Actual delivery still gets its separate resource-gain message.
        let mut delivered = arrived.clone();
        delivered.units[0].unit.cargo = None;
        feedback.observe(Some(&arrived), &delivered, 4.0);
        assert_eq!(feedback.labels.len(), 2);
        assert_eq!(feedback.labels[1].text, "+7 wood");
    }

    #[test]
    fn drop_off_status_tracks_cargo_and_current_orders() {
        let mut unit = GameWorld::default().snapshot().units[0].unit.clone();
        let cargo = CarriedResource {
            kind: ResourceKind::Wood,
            amount: 7.0,
        };
        for action in [
            UnitAction::Build {
                building_id: "base-1".into(),
            },
            UnitAction::Cultivate {
                resource_id: "field-1".into(),
            },
            UnitAction::Deposit {
                building_id: "base-1".into(),
            },
            UnitAction::Gather {
                resource_id: "food-1".into(),
                phase: GatherPhase::Returning,
            },
            UnitAction::Gather {
                resource_id: "food-1".into(),
                phase: GatherPhase::Depositing,
            },
        ] {
            unit.action = action;
            unit.cargo = Some(cargo.clone());
            assert_eq!(drop_off_status(&unit).as_deref(), Some("Dropping off wood"));
            unit.cargo = None;
            assert_eq!(drop_off_status(&unit), None);
        }
        unit.cargo = Some(cargo);
        for action in [
            UnitAction::Idle,
            UnitAction::Move { to: unit.cell },
            UnitAction::Gather {
                resource_id: "tree-1".into(),
                phase: GatherPhase::ToResource,
            },
            UnitAction::Gather {
                resource_id: "tree-1".into(),
                phase: GatherPhase::Gathering,
            },
        ] {
            unit.action = action;
            assert_eq!(drop_off_status(&unit), None);
        }
    }

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
            assert_eq!(feedback.labels.len(), 1);
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
        assert!(feedback.labels.is_empty());
        feedback.observe(Some(&before), &next, 1.0);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "+20 wood");
        feedback.observe(Some(&next), &next, 1.1);
        assert_eq!(feedback.labels.len(), 1);
        let mut stopped = before.clone();
        stopped.units[0].unit.action = UnitAction::Idle;
        feedback.observe(Some(&before), &stopped, 1.2);
        assert_eq!(feedback.labels.len(), 1);
        feedback.observe(Some(&before), &world.snapshot(), 1.3);
        assert!(feedback.labels.is_empty());
    }
}
