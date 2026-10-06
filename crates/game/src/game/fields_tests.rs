use super::tests::{cell, run};
use super::*;

fn world() -> GameWorld {
    let mut w = fixture::fixture();
    w.resources.clear();
    w.units[0].cell = cell(9, 10);
    w.units[1].cell = cell(9, 12);
    w.buildings
        .push(building(BuildingKind::Farm, "farm", cell(14, 10), None));
    w.inventories[0].wood = 100.0;
    w.inventories[0].stone = 100.0;
    w.inventories[0].water = 100.0;
    w
}

fn plant(w: &mut GameWorld) -> Result<(), CommandError> {
    w.apply_command(Command::PlantField {
        unit_id: "villager-1".into(),
        origin: cell(10, 10),
    })
}

fn cultivate(w: &mut GameWorld, number: u8) -> Result<(), CommandError> {
    w.apply_command(Command::Cultivate {
        unit_id: format!("villager-{number}"),
        resource_id: "field-10-10".into(),
    })
}

#[test]
fn fields_require_a_completed_farm_materials_and_free_land_atomically() {
    for case in 0..5 {
        let mut w = world();
        w.units[0].action = UnitAction::Move { to: cell(8, 10) };
        match case {
            0 => {
                w.buildings.pop();
            }
            1 => w.buildings.last_mut().unwrap().construction = Some(0.0),
            2 => w.inventories[0].stone = 4.0,
            3 => w.inventories[0].water = 9.0,
            _ => w.units[1].cell = cell(12, 12),
        }
        let before = serde_json::to_string(&w).unwrap();
        assert!(plant(&mut w).is_err());
        assert_eq!(before, serde_json::to_string(&w).unwrap());
    }
}

#[test]
fn preparation_pauses_persists_and_shared_work_never_charges_twice() {
    let mut w = world();
    plant(&mut w).unwrap();
    assert_eq!(w.inventories[0].water, 90.0);
    assert_eq!(
        (w.inventories[0].wood, w.inventories[0].stone),
        (90.0, 95.0)
    );
    run(&mut w, 3.0);
    let progress = w.resources[0].field.as_ref().unwrap().work.unwrap();
    assert!((2.9..3.1).contains(&progress));
    w.apply_command(Command::Stop {
        unit_id: "villager-1".into(),
    })
    .unwrap();
    run(&mut w, 20.0);
    assert_eq!(w.resources[0].field.as_ref().unwrap().work, Some(progress));
    assert_eq!(w.resources[0].amount, 0.0);
    w = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    w.validate().unwrap();
    cultivate(&mut w, 1).unwrap();
    cultivate(&mut w, 2).unwrap();
    run(&mut w, 4.0);
    assert_eq!(w.resources[0].amount, 0.0);
    run(&mut w, 1.0);
    assert!(w.resources[0].field.as_ref().unwrap().work.is_none());
    assert!(w.resources[0].amount > 0.0);
    assert!(w.units.iter().all(|u| matches!(&u.action,
        UnitAction::Gather { resource_id, .. } if resource_id == "field-10-10")));
    assert_eq!(
        (w.inventories[0].wood, w.inventories[0].stone),
        (90.0, 95.0)
    );
    assert_eq!(cultivate(&mut w, 1), Err(CommandError::FieldNotDepleted));
    assert_eq!(w.inventories[0].water, 90.0);
    w.validate().unwrap();
}

#[test]
fn harvesting_exhausts_fields_and_replenishment_requires_materials_and_labor() {
    let mut w = world();
    plant(&mut w).unwrap();
    run(&mut w, 13.0);
    for _ in 0..2000 {
        w.tick(0.1);
        w.validate().unwrap();
    }
    assert_eq!(w.inventories[0].food, FIELD_FOOD);
    assert_eq!(w.resources[0].amount, 0.0);
    assert_eq!(w.units[0].action, UnitAction::Idle);
    for at in w.resources[0].footprint().cells() {
        assert!(!w.occupancy().is_free_for(at, None));
    }
    w.inventories[0].wood = 0.0;
    let before = serde_json::to_string(&w).unwrap();
    assert_eq!(
        cultivate(&mut w, 1),
        Err(CommandError::InsufficientResources(ResourceKind::Wood))
    );
    assert_eq!(before, serde_json::to_string(&w).unwrap());
    w.inventories[0].wood = 20.0;
    w.inventories[0].water = 9.0;
    let before = w.clone();
    assert_eq!(
        cultivate(&mut w, 1),
        Err(CommandError::InsufficientResources(ResourceKind::Water))
    );
    assert_eq!(w, before);
    w.inventories[0].water = 10.0;
    cultivate(&mut w, 1).unwrap();
    assert_eq!(w.inventories[0].water, 0.0);
    assert_eq!(
        (w.inventories[0].wood, w.inventories[0].stone),
        (10.0, 90.0)
    );
    assert_eq!(w.resources[0].amount, 0.0);
    run(&mut w, 20.0);
    assert!(w.resources[0].amount < FIELD_FOOD);
    assert_eq!(w.inventories[0].food, FIELD_FOOD);
    assert!(matches!(w.units[0].action, UnitAction::Gather { .. }));
    run(&mut w, 200.0);
    assert_eq!(w.inventories[0].food, FIELD_FOOD * 2.0);
    assert_eq!(w.resources[0].amount, 0.0);
    assert_eq!(w.resources[0].field.as_ref().unwrap().work, None);
    assert_eq!(w.units[0].action, UnitAction::Idle);
    assert_eq!(
        (w.inventories[0].wood, w.inventories[0].stone),
        (10.0, 90.0)
    );
    w.validate().unwrap();
}

#[test]
fn preparing_a_field_delivers_existing_cargo_first() {
    let mut w = world();
    w.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Food,
        amount: 20.0,
    });
    plant(&mut w).unwrap();
    w.tick(0.1);
    assert_eq!(w.resources[0].field.as_ref().unwrap().work, Some(0.0));
    // The initial load must arrive before any paid preparation progresses.
    for _ in 0..200 {
        if w.resources[0].field.as_ref().unwrap().work != Some(0.0) {
            break;
        }
        w.tick(0.1);
        w.validate().unwrap();
    }
    assert!(w.resources[0].field.as_ref().unwrap().work.unwrap() > 0.0);
    assert_eq!(w.inventories[0].food, 20.0);
    assert!(w.units[0].cargo.is_none());
    run(&mut w, 200.0);
    assert_eq!(w.inventories[0].food, FIELD_FOOD + 20.0);
    assert_eq!(w.resources[0].amount, 0.0);
    assert_eq!(w.resources[0].field.as_ref().unwrap().work, None);
    w.validate().unwrap();
}

#[test]
fn non_field_nodes_round_trip_and_invalid_field_states_are_rejected() {
    let node = GameWorld::default().resources[0].clone();
    let loaded: ResourceNode =
        serde_json::from_str(&serde_json::to_string(&node).unwrap()).unwrap();
    assert_eq!(loaded, node);
    assert!(loaded.field.is_none());
    let mut w = world();
    plant(&mut w).unwrap();
    w.resources[0].amount = 1.0;
    assert!(w.validate().is_err());
    w.resources[0].amount = 0.0;
    w.resources[0].field.as_mut().unwrap().work = Some(f64::NAN);
    assert!(w.validate().is_err());
}

#[test]
fn fields_reject_out_of_bounds_and_unreachable_sites_without_spending() {
    let mut w = world();
    for origin in [cell(u16::MAX, u16::MAX), cell(WORLD_COLUMNS - 2, 10)] {
        let before = serde_json::to_string(&w).unwrap();
        assert_eq!(
            w.apply_command(Command::PlantField {
                unit_id: "villager-1".into(),
                origin
            }),
            Err(CommandError::InvalidBuildSite)
        );
        assert_eq!(before, serde_json::to_string(&w).unwrap());
    }
    let plot = Footprint {
        origin: cell(10, 10),
        columns: 3,
        rows: 3,
    };
    w.units[0].cell = cell(5, 5);
    w.units[1].cell = cell(6, 5);
    for at in super::movement::interaction_cells(plot) {
        w.terrain[usize::from(at.row) * usize::from(WORLD_COLUMNS) + usize::from(at.column)]
            .biome = TerrainBiome::Water;
    }
    let before = serde_json::to_string(&w).unwrap();
    assert_eq!(plant(&mut w), Err(CommandError::TargetUnreachable));
    assert_eq!(before, serde_json::to_string(&w).unwrap());
}

#[test]
fn finishing_preparation_preserves_stopped_workers_and_helpers_delivery() {
    let mut w = world();
    plant(&mut w).unwrap();
    cultivate(&mut w, 2).unwrap();
    w.apply_command(Command::Stop {
        unit_id: "villager-2".into(),
    })
    .unwrap();
    run(&mut w, FIELD_WORK_SECONDS + 0.1);
    assert_eq!(w.units[1].action, UnitAction::Idle);
    assert!(matches!(w.units[0].action, UnitAction::Gather { .. }));
    w.validate().unwrap();

    let mut w = world();
    plant(&mut w).unwrap();
    w.resources[0].field.as_mut().unwrap().work = Some(FIELD_WORK_SECONDS - 0.1);
    w.units[1].cell = cell(40, 40);
    w.units[1].cargo = Some(CarriedResource {
        kind: ResourceKind::Food,
        amount: 3.0,
    });
    cultivate(&mut w, 2).unwrap();
    w.tick(0.1);
    assert!(matches!(
        w.units[1].action,
        UnitAction::Gather {
            phase: GatherPhase::Returning,
            ..
        }
    ));
    assert_eq!(w.units[1].cargo.as_ref().unwrap().amount, 3.0);
    for _ in 0..2000 {
        w.tick(0.1);
        w.validate().unwrap();
    }
    assert_eq!(w.inventories[0].food, FIELD_FOOD + 3.0);
    assert_eq!(w.resources[0].field.as_ref().unwrap().work, None);
    assert!(w.units.iter().all(|u| u.action == UnitAction::Idle));
}

// A field can be reached from either side of this strip, but must not cut
// its western workers off from the farm where they deposit their harvest.
fn bottleneck_world(exit: bool) -> GameWorld {
    let mut w = world();
    w.buildings.remove(0);
    for tile in &mut w.terrain {
        tile.biome = if (8..25).contains(&tile.column)
            && (10..if exit { 14 } else { 13 }).contains(&tile.row)
        {
            TerrainBiome::Meadow
        } else {
            TerrainBiome::Water
        };
    }
    w.validate().unwrap();
    w
}

#[test]
fn field_placement_cannot_cut_off_delivery_routes_for_any_worker() {
    for planter in ["villager-1", "villager-2"] {
        let mut w = bottleneck_world(false);
        w.units[1].cell = cell(13, 12);
        w.units[0].cargo = Some(CarriedResource {
            kind: ResourceKind::Food,
            amount: 3.0,
        });
        let before = w.clone();
        assert_eq!(
            w.apply_command(Command::PlantField {
                unit_id: planter.into(),
                origin: cell(10, 10),
            }),
            Err(CommandError::TargetUnreachable)
        );
        assert_eq!(w, before);
        w.validate().unwrap();
    }
}

#[test]
fn field_with_an_exit_completes_repeated_deliveries_after_reload() {
    let mut w = bottleneck_world(true);
    w.units.truncate(1);
    plant(&mut w).unwrap();
    run(&mut w, 20.0);
    w = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    for _ in 0..3000 {
        w.tick(0.1);
        w.validate().unwrap();
    }
    assert!(
        (w.inventories[0].food - FIELD_FOOD).abs() < 1e-8,
        "food={}, units={:?}, resources={:?}",
        w.inventories[0].food,
        w.units,
        w.resources
    );
    assert_eq!(w.resources[0].amount, 0.0);
    assert!(
        w.units
            .iter()
            .all(|u| u.action == UnitAction::Idle && u.cargo.is_none())
    );
    assert_eq!(
        (w.inventories[0].wood, w.inventories[0].stone),
        (90.0, 95.0)
    );
}

#[test]
fn gatherers_continue_between_fields_and_wild_food_in_both_directions() {
    for start_at_field in [false, true] {
        let mut w = world();
        plant(&mut w).unwrap();
        w.resources[0].field.as_mut().unwrap().work = None;
        w.resources[0].amount = 23.0;
        w.resources.push(ResourceNode {
            id: "berries".into(),
            kind: ResourceKind::Food,
            cell: cell(8, 8),
            amount: 24.0,
            capacity: 24.0,
            field: None,
        });
        w.apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: if start_at_field {
                "field-10-10"
            } else {
                "berries"
            }
            .into(),
        })
        .unwrap();
        for _ in 0..2000 {
            w.tick(0.1);
            w.validate().unwrap();
        }
        assert!((w.inventories[0].food - 47.0).abs() < 1e-8);
        assert!(w.resources.iter().all(|r| r.amount == 0.0));
        assert_eq!(w.units[0].action, UnitAction::Idle);
        assert!(w.units[0].cargo.is_none());
        assert_eq!(
            (w.inventories[0].wood, w.inventories[0].stone),
            (90.0, 95.0)
        );
    }
}

#[test]
fn returning_harvester_waits_for_replenishment_and_resumes_without_another_order() {
    let mut w = world();
    plant(&mut w).unwrap();
    run(&mut w, FIELD_WORK_SECONDS + 0.1);
    // The last harvester is delivering the exhausted field's final load while
    // another villager starts replenishing it.
    w.resources[0].amount = 0.0;
    w.units[1].action = UnitAction::Gather {
        resource_id: "field-10-10".into(),
        phase: GatherPhase::Returning,
    };
    w.units[1].cargo = Some(CarriedResource {
        kind: ResourceKind::Food,
        amount: 20.0,
    });
    let town = w.buildings[0].footprint();
    super::tests::stand_beside(&mut w, 1, town);
    cultivate(&mut w, 1).unwrap();
    run(&mut w, 1.0);
    assert_eq!(w.inventories[0].food, 20.0);
    assert!(matches!(&w.units[1].action,
        UnitAction::Gather { resource_id, .. } if resource_id == "field-10-10"));
    // Waiting orders must survive persistence as well as normal ticks.
    w = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    run(&mut w, FIELD_WORK_SECONDS + 5.0);
    assert!(
        w.units[1]
            .cargo
            .as_ref()
            .is_some_and(|c| c.kind == ResourceKind::Food)
    );
    w.validate().unwrap();
}

#[test]
fn harvesters_keep_preparing_fields_in_every_gather_phase_but_stop_still_cancels() {
    for phase in [
        GatherPhase::ToResource,
        GatherPhase::Gathering,
        GatherPhase::Returning,
        GatherPhase::Depositing,
    ] {
        let mut w = world();
        plant(&mut w).unwrap();
        w.units[1].action = UnitAction::Gather {
            resource_id: "field-10-10".into(),
            phase,
        };
        // A nearby food node must not steal the assignment during preparation.
        w.resources.push(ResourceNode {
            id: "berries".into(),
            kind: ResourceKind::Food,
            cell: cell(8, 12),
            amount: 120.0,
            capacity: 120.0,
            field: None,
        });
        run(&mut w, 1.0);
        assert!(
            matches!(&w.units[1].action,
            UnitAction::Gather { resource_id, .. } if resource_id == "field-10-10"),
            "lost assignment from {phase:?}"
        );
        w.apply_command(Command::Stop {
            unit_id: "villager-2".into(),
        })
        .unwrap();
        run(&mut w, FIELD_WORK_SECONDS + 1.0);
        assert_eq!(w.units[1].action, UnitAction::Idle);
        assert!(w.units[1].cargo.is_none());
        assert_eq!(w.resources[1].amount, 120.0);
        w.validate().unwrap();

        // A new gather order also replaces the waiting assignment.
        w.resources[0].amount = 0.0;
        cultivate(&mut w, 1).unwrap();
        w.units[1].action = UnitAction::Gather {
            resource_id: "field-10-10".into(),
            phase: GatherPhase::ToResource,
        };
        w.apply_command(Command::Gather {
            unit_id: "villager-2".into(),
            resource_id: "berries".into(),
        })
        .unwrap();
        run(&mut w, FIELD_WORK_SECONDS + 0.1);
        assert!(w.resources[1].amount < 120.0);
        assert!(matches!(&w.units[1].action,
            UnitAction::Gather { resource_id, .. } if resource_id == "berries"));
        w.validate().unwrap();
    }
}

#[test]
fn replenishment_handoff_handles_worker_order_partial_cargo_and_reload() {
    for reverse in [false, true] {
        for cargo_kind in [ResourceKind::Food, ResourceKind::Wood] {
            let mut w = world();
            plant(&mut w).unwrap();
            // Use an exhausted, previously prepared field.
            w.resources[0].field.as_mut().unwrap().work = None;
            w.units[0].action = UnitAction::Idle;
            cultivate(&mut w, 1).unwrap();
            w.resources[0].field.as_mut().unwrap().work = Some(FIELD_WORK_SECONDS - 0.05);
            w.units[1].cell = cell(20, 20);
            w.units[1].cargo = Some(CarriedResource {
                kind: cargo_kind,
                amount: 3.0,
            });
            cultivate(&mut w, 2).unwrap();
            if reverse {
                w.units.reverse();
            }
            w = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
            w.tick(0.1);
            assert!(w.resources[0].field.as_ref().unwrap().work.is_none());
            assert!(
                w.units
                    .iter()
                    .all(|u| matches!(u.action, UnitAction::Gather { .. }))
            );
            let food_before = w.inventories[0].food;
            let wood_before = w.inventories[0].wood;
            run(&mut w, 200.0);
            assert!(
                (w.inventories[0].food
                    - food_before
                    - FIELD_FOOD
                    - if cargo_kind == ResourceKind::Food {
                        3.0
                    } else {
                        0.0
                    })
                .abs()
                    < 1e-8
            );
            assert_eq!(
                w.inventories[0].wood - wood_before,
                if cargo_kind == ResourceKind::Wood {
                    3.0
                } else {
                    0.0
                }
            );
            assert!(w.units.iter().all(|u| u.action == UnitAction::Idle));
            w.validate().unwrap();
        }
    }
}

#[test]
fn granary_yield_uses_completed_non_stacking_edge_to_edge_range() {
    for (origins, construction, expected) in [
        (vec![], None, 120.0),
        (vec![cell(18, 18)], None, 180.0), // Diagonal six-cell boundary.
        (vec![cell(19, 18)], None, 120.0), // Seven cells from field's edge.
        (vec![cell(2, 2)], None, 180.0),   // Opposite footprint edges, not origins.
        (vec![cell(18, 18)], Some(0.0), 120.0),
        (vec![cell(18, 18), cell(2, 2)], None, 180.0),
    ] {
        let mut w = world();
        for (i, origin) in origins.into_iter().enumerate() {
            w.buildings.push(building(
                BuildingKind::Granary,
                &format!("granary-{i}"),
                origin,
                construction,
            ));
        }
        plant(&mut w).unwrap();
        w.tick_cultivate(0, "field-10-10", FIELD_WORK_SECONDS);
        assert_eq!(w.resources[0].capacity, expected);
        assert_eq!(w.resources[0].amount, expected);
        w.validate().unwrap();
        w = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
        run(&mut w, 300.0);
        assert!((w.inventories[0].food - expected).abs() < 1e-8);
        assert_eq!(w.resources[0].amount, 0.0);
        assert_eq!(w.units[0].action, UnitAction::Idle);
        w.validate().unwrap();
    }
}

#[test]
fn granary_yield_is_recalculated_at_each_preparation_completion() {
    let mut w = world();
    plant(&mut w).unwrap();
    // A granary completed during preparation benefits this harvest.
    w.buildings.push(building(
        BuildingKind::Granary,
        "granary",
        cell(18, 18),
        None,
    ));
    w.tick_cultivate(0, "field-10-10", FIELD_WORK_SECONDS);
    assert_eq!(w.resources[0].capacity, 180.0);
    w.buildings.pop();
    run(&mut w, 300.0);
    assert!((w.inventories[0].food - 180.0).abs() < 1e-8);
    cultivate(&mut w, 1).unwrap();
    run(&mut w, 300.0);
    assert_eq!(w.resources[0].capacity, 120.0);
    assert!((w.inventories[0].food - 300.0).abs() < 1e-8);
    assert_eq!(
        (w.inventories[0].wood, w.inventories[0].stone),
        (80.0, 90.0)
    );
    w.validate().unwrap();
}
