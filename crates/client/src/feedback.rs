//! Short-lived action and resource labels from authoritative snapshots.
use aoa_game::{GatherPhase, Unit, UnitAction, WorldSnapshot};
use glam::Vec3;

use crate::{camera::Rig, hud, terrain};

const LIFETIME: f64 = 1.4;

fn drop_off_status(unit: &Unit) -> Option<String> {
    let cargo = unit.cargo.as_ref()?;
    let unloading = matches!(
        unit.action,
        UnitAction::Build { .. }
            | UnitAction::ExploreBuild { .. }
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

// Include the assignment target so replacing an order of the same kind flashes,
// while walking to a node and starting work remain one gathering assignment.
fn action_status(unit: &Unit, snapshot: &WorldSnapshot) -> Option<(String, String)> {
    let (text, target) = match &unit.action {
        UnitAction::AttackAnimal { animal_id, .. } => {
            ("Attacking wildlife".into(), animal_id.as_str())
        }
        UnitAction::Idle => ("Idle".into(), ""),
        UnitAction::Move { .. } => return None,
        UnitAction::Board { ship_id } => ("Boarding transport".into(), ship_id.as_str()),
        UnitAction::ExploreBuild { origin, kind } => {
            return Some((
                drop_off_status(unit).unwrap_or_else(|| "Exploring build site".into()),
                format!("{kind:?}:{},{}", origin.column, origin.row),
            ));
        }
        UnitAction::Build { building_id } => ("Building".into(), building_id.as_str()),
        UnitAction::Cultivate { resource_id } => ("Preparing field".into(), resource_id.as_str()),
        UnitAction::Deposit { building_id } => (drop_off_status(unit)?, building_id.as_str()),
        UnitAction::Gather { resource_id, .. } => {
            let text = snapshot
                .resources
                .iter()
                .find(|r| &r.id == resource_id)
                .map(|resource| {
                    format!(
                        "Gathering {}",
                        format!("{:?}", resource.kind).to_lowercase()
                    )
                })
                .unwrap_or_else(|| "Gathering".into());
            (text, resource_id.as_str())
        }
    };
    Some((drop_off_status(unit).unwrap_or(text), target.into()))
}

struct Label {
    unit_id: Option<String>,
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
        if next.tick < previous.tick || next.island_id != previous.island_id {
            self.labels.clear();
            return;
        }
        for animal in &next.animals {
            if let Some(before) = previous.animals.iter().find(|a| a.id == animal.id)
                && animal.health < before.health
            {
                let p = animal.position();
                let at = terrain::world_of(p.x, p.y);
                self.labels.push(Label {
                    unit_id: None,
                    text: format!("{}: {:.0} HP", animal.kind.name(), animal.health),
                    at: Vec3::new(at.x, 0.8, at.y),
                    born: now,
                });
            }
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
            for label in &mut self.labels {
                if label.unit_id.as_deref() == Some(&unit.unit.id) {
                    label.at.x = at.x;
                    label.at.z = at.y;
                }
            }
            let status = action_status(&unit.unit, next);
            let changed = status != action_status(&before.unit, previous);
            if let Some((text, _)) = status
                && changed
            {
                // Keep the latest status readable when orders change rapidly.
                self.labels
                    .retain(|label| label.unit_id.as_deref() != Some(&unit.unit.id));
                self.labels.push(Label {
                    unit_id: Some(unit.unit.id.clone()),
                    text,
                    at: Vec3::new(at.x, 0.8, at.y),
                    born: now,
                });
            }
            if unit.unit.health < before.unit.health {
                self.labels.push(Label {
                    unit_id: None,
                    text: format!("-{:.0} HP", before.unit.health - unit.unit.health),
                    at: Vec3::new(at.x, 1.1, at.y),
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
                unit_id: None,
                text: format!(
                    "+{:.0} {}",
                    cargo.amount,
                    format!("{:?}", cargo.kind).to_lowercase()
                ),
                at: Vec3::new(at.x, if changed { 1.15 } else { 0.8 }, at.y),
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
                hud.gain_label(atlas, &label.text, at, pixels_per_world, alpha, rig.width);
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
            resource_id: before
                .resources
                .iter()
                .find(|r| r.kind == ResourceKind::Food)
                .unwrap()
                .id
                .clone(),
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
            resource_id: match &next.units[0].unit.action {
                UnitAction::Gather { resource_id, .. } => resource_id.clone(),
                _ => unreachable!(),
            },
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
        assert_eq!(feedback.labels[0].text, "Gathering food");
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
            assert_eq!(feedback.labels.last().unwrap().text, "+20 wood");
        }
    }

    #[test]
    fn deposits_and_idle_show_once_and_reset_clears_labels() {
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
        assert_eq!(feedback.labels.len(), 2);
        assert_eq!(feedback.labels[0].text, "Idle");
        assert_eq!(feedback.labels[1].text, "+20 wood");
        feedback.observe(Some(&next), &next, 1.1);
        assert_eq!(feedback.labels.len(), 2);
        let mut stopped = before.clone();
        stopped.units[0].unit.action = UnitAction::Idle;
        feedback.observe(Some(&before), &stopped, 1.2);
        assert_eq!(feedback.labels.len(), 2);
        assert_eq!(feedback.labels[1].text, "Idle");
        feedback.observe(Some(&before), &world.snapshot(), 1.3);
        assert!(feedback.labels.is_empty());
    }

    #[test]
    fn assignments_flash_once_and_plain_walking_stays_quiet() {
        let before = GameWorld::default().snapshot();
        for (action, expected) in [
            (
                UnitAction::Build {
                    building_id: "base-1".into(),
                },
                "Building",
            ),
            (
                UnitAction::Cultivate {
                    resource_id: "field-1".into(),
                },
                "Preparing field",
            ),
            (
                UnitAction::Board {
                    ship_id: "ship-1".into(),
                },
                "Boarding transport",
            ),
            (
                UnitAction::Gather {
                    resource_id: before
                        .resources
                        .iter()
                        .find(|r| r.kind == ResourceKind::Wood)
                        .unwrap()
                        .id
                        .clone(),
                    phase: GatherPhase::ToResource,
                },
                "Gathering wood",
            ),
        ] {
            let mut next = before.clone();
            next.units[0].unit.action = action;
            let mut feedback = Feedback::default();
            feedback.observe(Some(&before), &next, 1.0);
            feedback.observe(Some(&next), &next, 1.1);
            assert_eq!(feedback.labels.len(), 1);
            assert_eq!(feedback.labels[0].text, expected);
        }
        let mut walking = before.clone();
        walking.units[0].unit.action = UnitAction::Move {
            to: before.units[0].unit.cell,
        };
        let mut feedback = Feedback::default();
        feedback.observe(Some(&before), &walking, 1.0);
        assert!(feedback.labels.is_empty());
        feedback.observe(Some(&walking), &before, 2.0);
        assert_eq!(feedback.labels[0].text, "Idle");
    }

    #[test]
    fn exploring_flashes_once_per_site_then_announces_construction() {
        let before = GameWorld::default().snapshot();
        let mut exploring = before.clone();
        exploring.units[0].unit.action = UnitAction::ExploreBuild {
            origin: aoa_game::CellCoordinate::new(40, 31),
            kind: aoa_game::BuildingKind::House,
        };
        let mut feedback = Feedback::default();
        feedback.observe(Some(&before), &exploring, 1.0);
        feedback.observe(Some(&exploring), &exploring, 1.1);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "Exploring build site");
        assert_eq!(feedback.labels[0].born, 1.0);
        let mut redirected = exploring.clone();
        redirected.units[0].unit.action = UnitAction::ExploreBuild {
            origin: aoa_game::CellCoordinate::new(50, 31),
            kind: aoa_game::BuildingKind::House,
        };
        feedback.observe(Some(&exploring), &redirected, 1.2);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].born, 1.2);
        let mut building = redirected.clone();
        building.units[0].unit.action = UnitAction::Build {
            building_id: "building-2".into(),
        };
        feedback.observe(Some(&redirected), &building, 1.3);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "Building");
    }

    #[test]
    fn gathering_phase_does_not_repeat_but_reassignment_does() {
        let mut before = GameWorld::default().snapshot();
        let trees: Vec<_> = before
            .resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Wood)
            .map(|r| r.id.clone())
            .collect();
        before.units[0].unit.action = UnitAction::Gather {
            resource_id: trees[0].clone(),
            phase: GatherPhase::ToResource,
        };
        let mut working = before.clone();
        working.units[0].unit.action = UnitAction::Gather {
            resource_id: trees[0].clone(),
            phase: GatherPhase::Gathering,
        };
        let mut feedback = Feedback::default();
        feedback.observe(Some(&before), &working, 1.0);
        assert!(feedback.labels.is_empty());
        let mut reassigned = working.clone();
        reassigned.units[0].unit.action = UnitAction::Gather {
            resource_id: trees[1].clone(),
            phase: GatherPhase::ToResource,
        };
        feedback.observe(Some(&working), &reassigned, 2.0);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "Gathering wood");
    }

    #[test]
    fn initial_snapshot_new_units_and_island_switch_do_not_flash_idle() {
        let before = GameWorld::default().snapshot();
        let mut next = before.clone();
        next.units[0].unit.id = "new-villager".into();
        let mut feedback = Feedback::default();
        feedback.observe(None, &before, 0.0);
        feedback.observe(Some(&before), &next, 1.0);
        assert!(feedback.labels.is_empty());
        next.units[1].unit.action = UnitAction::Build {
            building_id: "base-1".into(),
        };
        feedback.observe(Some(&before), &next, 2.0);
        assert_eq!(feedback.labels.len(), 1);
        let mut island = next.clone();
        island.island_id += 1;
        feedback.observe(Some(&next), &island, 3.0);
        assert!(feedback.labels.is_empty());
    }

    #[test]
    fn status_follows_the_unit_and_rapid_orders_replace_it() {
        let before = GameWorld::default().snapshot();
        let mut building = before.clone();
        building.units[0].unit.action = UnitAction::Build {
            building_id: "base-1".into(),
        };
        let mut feedback = Feedback::default();
        feedback.observe(Some(&before), &building, 1.0);
        let mut moving = building.clone();
        moving.units[0].position.x += 1.0;
        feedback.observe(Some(&building), &moving, 1.1);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(
            feedback.labels[0].at.x,
            moving.units[0].position.x as f32 * terrain::CELL
        );
        assert_eq!(feedback.labels[0].born, 1.0);
        let mut stopped = moving.clone();
        stopped.units[0].unit.action = UnitAction::Idle;
        feedback.observe(Some(&moving), &stopped, 1.2);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "Idle");
        assert_eq!(feedback.labels[0].born, 1.2);
    }
}
