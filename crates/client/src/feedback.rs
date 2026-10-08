//! Fading unit feedback from authoritative snapshots and command outcomes.
use aoa_game::{GatherPhase, ScenarioOutcome, Unit, UnitAction, WorldSnapshot};
use glam::Vec3;

use crate::{camera::Rig, hud, terrain};

const LIFETIME: f64 = 1.4;
/// The run's milestones linger above the bearer; ordinary statuses do not.
const MILESTONE_LIFETIME: f64 = 8.0;
/// A reason a unit gave up an order is a sentence: long enough to read.
const NOTICE_LIFETIME: f64 = 3.0;

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
        UnitAction::BuildRoad { cells } => {
            return Some((
                drop_off_status(unit).unwrap_or_else(|| "Building road".into()),
                format!("road:{cells:?}"),
            ));
        }
        UnitAction::AttackAnimal { animal_id, .. } => {
            ("Attacking wildlife".into(), animal_id.as_str())
        }
        UnitAction::Idle => ("Idle".into(), ""),
        UnitAction::Move { to } => {
            return Some(("Moving".into(), format!("{},{}", to.column, to.row)));
        }
        UnitAction::Board { ship_id } => ("Boarding transport".into(), ship_id.as_str()),
        UnitAction::ClaimArtifact { building_id } => {
            ("Seeking the artifact".into(), building_id.as_str())
        }
        UnitAction::ExploreBuild { origin, kind } => {
            return Some((
                drop_off_status(unit).unwrap_or_else(|| "Exploring build site".into()),
                format!("{kind:?}:{},{}", origin.column, origin.row),
            ));
        }
        UnitAction::Build { building_id } => ("Building".into(), building_id.as_str()),
        UnitAction::Cultivate { resource_id } => ("Preparing field".into(), resource_id.as_str()),
        UnitAction::Deposit { storage_id } => (drop_off_status(unit)?, storage_id.as_str()),
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
    // Identifies feedback to replace; status labels keep their original anchor.
    entity_id: Option<String>,
    text: String,
    at: Vec3,
    born: f64,
    lifetime: f64,
}

#[derive(Default)]
pub struct Feedback {
    labels: Vec<Label>,
}

impl Feedback {
    /// Replace the unit's transient status, using the same spawn anchor and fade as activities.
    pub fn message(&mut self, snapshot: &WorldSnapshot, units: &[String], text: &str, now: f64) {
        for unit in snapshot.units.iter().filter(|u| units.contains(&u.unit.id)) {
            let at = terrain::world_of(unit.position.x, unit.position.y);
            self.labels
                .retain(|label| label.entity_id.as_deref() != Some(&unit.unit.id));
            self.labels.push(Label {
                entity_id: Some(unit.unit.id.clone()),
                text: text.into(),
                at: Vec3::new(at.x, 0.8, at.y),
                born: now,
                lifetime: LIFETIME,
            });
        }
        if self.labels.len() > 64 {
            self.labels.drain(..self.labels.len() - 64);
        }
    }

    pub fn observe(&mut self, previous: Option<&WorldSnapshot>, next: &WorldSnapshot, now: f64) {
        let Some(previous) = previous else {
            return;
        };
        if next.tick < previous.tick || next.island_id != previous.island_id {
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
            let status = action_status(&unit.unit, next);
            let changed = status != action_status(&before.unit, previous);
            if let Some((text, _)) = status
                && changed
            {
                // Keep the latest status readable when orders change rapidly.
                self.labels
                    .retain(|label| label.entity_id.as_deref() != Some(&unit.unit.id));
                self.labels.push(Label {
                    entity_id: Some(unit.unit.id.clone()),
                    text,
                    at: Vec3::new(at.x, 0.8, at.y),
                    born: now,
                    lifetime: LIFETIME,
                });
            }
            // Why it gave up an order replaces the plain "Idle" status.
            if unit.unit.notice != before.unit.notice
                && let Some(notice) = &unit.unit.notice
            {
                self.labels
                    .retain(|label| label.entity_id.as_deref() != Some(&unit.unit.id));
                self.labels.push(Label {
                    entity_id: Some(unit.unit.id.clone()),
                    text: notice.message.clone(),
                    at: Vec3::new(at.x, 0.8, at.y),
                    born: now,
                    lifetime: NOTICE_LIFETIME,
                });
            }
            let Some(cargo) = &before.unit.cargo else {
                continue;
            };
            if unit.unit.cargo.is_some() || cargo.amount <= 0.0 {
                continue;
            }
            self.labels.push(Label {
                entity_id: None,
                text: format!(
                    "+{:.0} {}",
                    cargo.amount,
                    format!("{:?}", cargo.kind).to_lowercase()
                ),
                at: Vec3::new(at.x, if changed { 1.15 } else { 0.8 }, at.y),
                born: now,
                lifetime: LIFETIME,
            });
        }
        // The artifact's claim and homecoming speak from above its bearer.
        let claimed = previous.artifact_bearer.is_none() && next.artifact_bearer.is_some();
        let won = previous.scenario.outcome != ScenarioOutcome::Won
            && next.scenario.outcome == ScenarioOutcome::Won;
        if let Some(bearer) = next
            .units
            .iter()
            .find(|u| Some(&u.unit.id) == next.artifact_bearer.as_ref())
            && (claimed || won)
        {
            let at = terrain::world_of(bearer.position.x, bearer.position.y);
            self.labels
                .retain(|label| label.entity_id.as_deref() != Some(&bearer.unit.id));
            self.labels.push(Label {
                entity_id: Some(bearer.unit.id.clone()),
                text: if won {
                    "Victory · the Artifact of the Gods is home".into()
                } else {
                    "Claimed the Artifact of the Gods".into()
                },
                at: Vec3::new(at.x, 0.8, at.y),
                born: now,
                lifetime: MILESTONE_LIFETIME,
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
        self.labels
            .retain(|label| now - label.born < label.lifetime);
        for label in &self.labels {
            let age = (now - label.born) as f32;
            // Ordinary statuses end before the rise cap; milestones then hold still.
            let rise = age.min(LIFETIME as f32) * 0.45;
            let position = label.at + Vec3::Y * (heights.at(label.at.x, label.at.z) + rise);
            if let Some(at) = rig.screen_of(position) {
                let (_, up) = rig.basis();
                let Some(top) = rig.screen_offset(position, up) else {
                    continue;
                };
                let pixels_per_world = at.distance(top);
                let fade_from = label.lifetime as f32 - 0.5;
                let alpha = (age * 6.0).min(1.0) * (1.0 - ((age - fade_from) / 0.5).max(0.0));
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
    fn a_rejected_build_site_says_why_instead_of_idle() {
        let mut before = GameWorld::default().snapshot();
        before.units[0].unit.action = UnitAction::ExploreBuild {
            origin: aoa_game::CellCoordinate::new(10, 10),
            kind: aoa_game::BuildingKind::House,
        };
        let mut next = before.clone();
        next.tick += 1;
        next.units[0].unit.action = UnitAction::Idle;
        next.units[0].unit.notice = Some(aoa_game::UnitNotice {
            message: "Cannot build here. That spot is not clear for building.".into(),
            tick: next.tick,
        });
        let mut feedback = Feedback::default();
        feedback.observe(Some(&before), &next, 0.0);
        let texts: Vec<_> = feedback.labels.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(
            texts,
            ["Cannot build here. That spot is not clear for building."]
        );
        assert_eq!(feedback.labels[0].lifetime, NOTICE_LIFETIME);
        // The same notice in a later snapshot is not repeated.
        let mut later = next.clone();
        later.tick += 1;
        feedback.observe(Some(&next), &later, 1.0);
        assert_eq!(feedback.labels.len(), 1);
    }

    #[test]
    fn claiming_and_bringing_home_the_artifact_speak_above_the_bearer() {
        let before = GameWorld::default().snapshot();
        let mut claimed = before.clone();
        claimed.tick += 1;
        claimed.artifact_bearer = Some(before.units[0].unit.id.clone());
        let mut feedback = Feedback::default();
        feedback.observe(Some(&before), &claimed, 0.0);
        let label = feedback.labels.last().unwrap();
        assert_eq!(label.text, "Claimed the Artifact of the Gods");
        assert_eq!(label.entity_id, claimed.artifact_bearer);
        let mut won = claimed.clone();
        won.tick += 1;
        won.scenario.outcome = aoa_game::ScenarioOutcome::Won;
        feedback.observe(Some(&claimed), &won, 1.0);
        let texts: Vec<_> = feedback.labels.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, ["Victory · the Artifact of the Gods is home"]);
        // Unlike an ordinary status, the victory line is still there seconds later.
        feedback.labels.retain(|l| 6.0 - l.born < l.lifetime);
        assert_eq!(feedback.labels.len(), 1);
        // A later snapshot without a change says nothing new.
        let mut later = won.clone();
        later.tick += 1;
        feedback.observe(Some(&won), &later, 2.0);
        assert_eq!(feedback.labels.len(), 1);
    }

    #[test]
    fn complaints_replace_status_and_stay_at_the_original_anchor() {
        let before = GameWorld::default().snapshot();
        let id = before.units[0].unit.id.clone();
        let mut feedback = Feedback::default();
        feedback.message(&before, std::slice::from_ref(&id), "Blocked", 1.0);
        feedback.message(&before, std::slice::from_ref(&id), "Not enough stone", 1.2);
        let mut next = before.clone();
        next.units[0].position.x += 2.0;
        feedback.observe(Some(&before), &next, 1.3);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].entity_id.as_deref(), Some(id.as_str()));
        assert_eq!(feedback.labels[0].text, "Not enough stone");
        assert_eq!(feedback.labels[0].born, 1.2);
        assert_eq!(
            feedback.labels[0].at.x,
            before.units[0].position.x as f32 * terrain::CELL
        );
    }

    #[test]
    fn damage_does_not_replace_status_with_numbers() {
        let before = GameWorld::default().snapshot();
        let mut next = before.clone();
        next.units[0].unit.health -= 35.0;
        let mut feedback = Feedback::default();
        feedback.message(&before, &[before.units[0].unit.id.clone()], "Moving", 0.0);
        feedback.observe(Some(&before), &next, 0.1);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "Moving");
    }

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
                storage_id: "base-1".into(),
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
                storage_id: "base-1".into(),
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
    fn assignments_and_movement_flash_once() {
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
        feedback.observe(Some(&walking), &walking, 1.1);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "Moving");
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
    fn status_stays_at_its_origin_and_rapid_orders_replace_it() {
        let before = GameWorld::default().snapshot();
        let mut building = before.clone();
        building.units[0].unit.action = UnitAction::Build {
            building_id: "base-1".into(),
        };
        let mut feedback = Feedback::default();
        feedback.observe(Some(&before), &building, 1.0);
        let origin = feedback.labels[0].at;
        let mut moving = building.clone();
        moving.units[0].position.x += 1.0;
        moving.units[0].position.y += 1.0;
        feedback.observe(Some(&building), &moving, 1.1);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].at, origin);
        assert_eq!(feedback.labels[0].born, 1.0);
        let mut stopped = moving.clone();
        stopped.units[0].unit.action = UnitAction::Idle;
        feedback.observe(Some(&moving), &stopped, 1.2);
        assert_eq!(feedback.labels.len(), 1);
        assert_eq!(feedback.labels[0].text, "Idle");
        assert_eq!(feedback.labels[0].born, 1.2);
        let at = terrain::world_of(stopped.units[0].position.x, stopped.units[0].position.y);
        assert_eq!(feedback.labels[0].at, Vec3::new(at.x, 0.8, at.y));
    }
}
