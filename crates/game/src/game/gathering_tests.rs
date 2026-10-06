use super::movement::interaction_cells;
use super::tests::{cell, run};
use super::*;

#[test]
fn water_collection_renews_delivers_resumes_and_stops() {
    for speed in [1.0, 2.0] {
        let mut w = fixture::fixture();
        w.resources.clear();
        w.units[0].cell = CellCoordinate::new(26, 22);
        w.resources.push(ResourceNode {
            id: "water-test".into(),
            kind: ResourceKind::Water,
            cell: CellCoordinate::new(25, 22),
            amount: 120.0,
            capacity: 120.0,
            field: None,
        });
        w.refresh_exploration();
        w.simulation_speed = speed;
        w.apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: "water-test".into(),
        })
        .unwrap();
        for _ in 0..2500 {
            w.tick(0.1);
            w.validate().unwrap();
            assert_eq!(w.resources[0].amount, 120.0);
            assert!(w.units[0].cargo.as_ref().is_none_or(
                |c| c.kind == ResourceKind::Water && c.amount <= VILLAGER_CARRY_CAPACITY
            ));
            if w.inventories[0].water >= 60.0 {
                break;
            }
        }
        assert!(
            w.inventories[0].water >= 60.0,
            "three deliveries at {speed}x"
        );
        let loaded: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
        assert_eq!(loaded, w);
        w.apply_command(Command::Stop {
            unit_id: "villager-1".into(),
        })
        .unwrap();
        let stored = w.inventories[0].water;
        for _ in 0..100 {
            w.tick(0.1);
        }
        assert_eq!(w.units[0].action, UnitAction::Idle);
        assert_eq!(w.inventories[0].water, stored);
    }
}

fn world() -> GameWorld {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.units[0].cell = cell(10, 10);
    world.units[1].cell = cell(20, 10);
    world.buildings = vec![town_center("base-1", cell(25, 10), None)];
    world
}

fn node(id: &str, at: CellCoordinate, amount: f64) -> ResourceNode {
    ResourceNode {
        field: None,
        id: id.into(),
        kind: ResourceKind::Food,
        cell: at,
        amount,
        capacity: amount.max(1.0),
    }
}

#[test]
fn automatic_drop_off_uses_the_nearest_compatible_complete_building() {
    let mut world = world();
    world
        .buildings
        .push(building(BuildingKind::House, "house", cell(11, 6), None));
    world.buildings.push(building(
        BuildingKind::Granary,
        "unfinished",
        cell(11, 10),
        Some(0.0),
    ));
    world.buildings.push(building(
        BuildingKind::Granary,
        "granary",
        cell(15, 10),
        None,
    ));
    for kind in [
        ResourceKind::Food,
        ResourceKind::Fiber,
        ResourceKind::Wood,
        ResourceKind::Stone,
    ] {
        world.units[0].cargo = Some(CarriedResource { kind, amount: 20.0 });
        let expected = if matches!(kind, ResourceKind::Food | ResourceKind::Fiber) {
            3
        } else {
            0
        };
        assert_eq!(
            world.nearest_drop_site(0),
            Some(world.buildings[expected].footprint())
        );
    }
}

#[test]
fn a_blocked_nearby_drop_off_does_not_hide_an_accessible_one() {
    for (building_kind, resource_kind) in [
        (BuildingKind::Granary, ResourceKind::Food),
        (BuildingKind::LumberMill, ResourceKind::Wood),
    ] {
        let mut world = world();
        world
            .buildings
            .push(building(building_kind, "drop-site", cell(12, 10), None));
        let near = world.buildings[1].footprint();
        // Only one approach cell remains, and an idle villager occupies it.
        let entrance = cell(11, 11);
        for at in interaction_cells(near).filter(|at| *at != entrance) {
            let index = usize::from(at.row) * usize::from(WORLD_COLUMNS) + usize::from(at.column);
            world.terrain[index].biome = TerrainBiome::Water;
        }
        world.units[1].cell = entrance;
        world.units[0].cargo = Some(CarriedResource {
            kind: resource_kind,
            amount: 20.0,
        });
        let mut resource = node("berries", cell(8, 10), 40.0);
        resource.kind = resource_kind;
        world.resources.push(resource);
        world.units[0].action = UnitAction::Gather {
            resource_id: "berries".into(),
            phase: GatherPhase::Returning,
        };
        world.validate().unwrap();
        assert_eq!(
            world.nearest_drop_site(0),
            Some(world.buildings[0].footprint())
        );
        run(&mut world, 30.0);
        assert!(world.inventories[0].amount(resource_kind) >= 20.0);
        assert!(
            world.resources[0].amount < 40.0,
            "gathering resumes after unloading"
        );
    }
}

#[test]
fn repeated_granary_deliveries_resume_until_the_resource_is_exhausted() {
    let mut world = world();
    world.buildings.push(building(
        BuildingKind::Granary,
        "granary",
        cell(12, 10),
        None,
    ));
    world.resources.push(node("berries", cell(8, 10), 60.0));
    world
        .apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: "berries".into(),
        })
        .unwrap();
    let mut deliveries = 0;
    for _ in 0..600 {
        let before = world.inventories[0].food;
        world.tick(0.1);
        if world.inventories[0].food > before {
            deliveries += 1;
            assert!(world.is_beside(0, world.buildings[1].footprint()));
            assert!((world.inventories[0].food - f64::from(deliveries) * 20.0).abs() < 1e-9);
            assert!(world.units[0].cargo.is_none());
        }
    }
    assert_eq!(deliveries, 3);
    assert_eq!(world.resources[0].amount, 0.0);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn lumber_mill_deliveries_resume_and_credit_full_and_partial_wood_loads_once() {
    let mut world = world();
    world.buildings.push(building(
        BuildingKind::LumberMill,
        "mill",
        cell(12, 10),
        None,
    ));
    let mut tree = node("tree", cell(8, 10), 47.0);
    tree.kind = ResourceKind::Wood;
    world.resources.push(tree);
    world
        .apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: "tree".into(),
        })
        .unwrap();
    let mut delivered = Vec::new();
    for _ in 0..1000 {
        let before = world.inventories[0].wood;
        world.tick(0.1);
        if world.inventories[0].wood > before {
            assert!(world.is_beside(0, world.buildings[1].footprint()));
            assert!(world.units[0].cargo.is_none());
            delivered.push(world.inventories[0].wood - before);
        }
    }
    assert_eq!(delivered.len(), 3);
    for (actual, expected) in delivered.into_iter().zip([20.0, 20.0, 7.0]) {
        assert!((actual - expected).abs() < 1e-9);
    }
    assert!((world.inventories[0].wood - 47.0).abs() < 1e-9);
    assert_eq!(world.inventories[0].timber, 0.0);
    assert_eq!(world.resources[0].amount, 0.0);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.validate().unwrap();
}

#[test]
fn lumber_mill_requires_completion_and_wood_for_automatic_and_explicit_unloading() {
    let mut world = world();
    world.buildings.push(building(
        BuildingKind::LumberMill,
        "mill",
        cell(12, 10),
        Some(0.0),
    ));
    world.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Wood,
        amount: 7.0,
    });
    let deposit = Command::Deposit {
        unit_id: "villager-1".into(),
        storage_id: "mill".into(),
    };
    let before = world.clone();
    assert_eq!(
        world.apply_command(deposit.clone()),
        Err(CommandError::BuildingUnderConstruction)
    );
    assert_eq!(world, before);
    assert_eq!(
        world.nearest_drop_site(0),
        Some(world.buildings[0].footprint())
    );
    world.buildings[1].construction = None;
    for kind in ROADMAP_RESOURCES
        .into_iter()
        .filter(|kind| *kind != ResourceKind::Wood)
    {
        world.units[0].cargo.as_mut().unwrap().kind = kind;
        let before = world.clone();
        assert_eq!(
            world.apply_command(deposit.clone()),
            Err(CommandError::BuildingRefusesCargo)
        );
        assert_eq!(world, before);
        assert_eq!(
            world.nearest_drop_site(0),
            Some(world.buildings[0].footprint())
        );
    }
    world.units[0].cargo.as_mut().unwrap().kind = ResourceKind::Wood;
    assert_eq!(
        world.nearest_drop_site(0),
        Some(world.buildings[1].footprint())
    );
    world.apply_command(deposit).unwrap();
    run(&mut world, 10.0);
    assert_eq!(world.inventories[0].wood, 7.0);
    assert!(world.units[0].cargo.is_none());
    assert!(world.is_beside(0, world.buildings[1].footprint()));
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn a_busy_only_drop_site_keeps_cargo_and_retries() {
    let mut world = world();
    world.buildings = vec![building(
        BuildingKind::Granary,
        "granary",
        cell(12, 10),
        None,
    )];
    let site = world.buildings[0].footprint();
    let entrance = cell(11, 11);
    for at in interaction_cells(site).filter(|at| *at != entrance) {
        let index = usize::from(at.row) * usize::from(WORLD_COLUMNS) + usize::from(at.column);
        world.terrain[index].biome = TerrainBiome::Water;
    }
    world.units[1].action = UnitAction::Move { to: entrance };
    world.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Food,
        amount: 20.0,
    });
    world.resources.push(node("berries", cell(8, 10), 40.0));
    world.units[0].action = UnitAction::Gather {
        resource_id: "berries".into(),
        phase: GatherPhase::Returning,
    };
    world.validate().unwrap();
    assert_eq!(world.nearest_drop_site(0), Some(site));
    world.tick_gather(0, "berries".into(), GatherPhase::Returning, 0.1);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 20.0);
    assert_eq!(world.inventories[0].food, 0.0);
    world.units[1].action = UnitAction::Idle;
    run(&mut world, 30.0);
    assert!(world.inventories[0].food >= 20.0);
    assert!(world.resources[0].amount < 40.0);
}

#[test]
fn reserved_resource_approach_waits_then_resumes_instead_of_idling() {
    for exhausted in [false, true] {
        let mut world = world();
        world.resources.push(node(
            "berries",
            cell(12, 10),
            if exhausted { 0.0 } else { 40.0 },
        ));
        world.resources.push(node("next", cell(14, 10), 40.0));
        let target = if exhausted { 1 } else { 0 };
        let entrance = cell(world.resources[target].cell.column - 1, 10);
        for at in
            interaction_cells(world.resources[target].footprint()).filter(|at| *at != entrance)
        {
            let index = usize::from(at.row) * usize::from(WORLD_COLUMNS) + usize::from(at.column);
            world.terrain[index].biome = TerrainBiome::Water;
        }
        world.units[1].action = UnitAction::Move { to: entrance };
        world.units[0].action = UnitAction::Gather {
            resource_id: "berries".into(),
            phase: GatherPhase::ToResource,
        };
        world.validate().unwrap();
        // Tick only this villager while the other still reserves the approach.
        world.tick_gather(0, "berries".into(), GatherPhase::ToResource, 0.1);
        assert!(
            matches!(world.units[0].action, UnitAction::Gather { .. }),
            "temporary reservation must preserve the order"
        );
        world.units[1].action = UnitAction::Idle;
        run(&mut world, 30.0);
        assert!(
            world.inventories[0].food >= 20.0,
            "the preserved loop gathers and unloads"
        );
    }
}

#[test]
fn reassignment_unloads_the_old_kind_and_repeats_the_new_gathering_loop() {
    for speed in [1.0, 2.0] {
        for amount in [7.0, 20.0] {
            let mut world = world();
            world.simulation_speed = speed;
            world.resources.push(node("berries", cell(8, 10), 100.0));
            world.units[0].cargo = Some(CarriedResource {
                kind: ResourceKind::Wood,
                amount,
            });
            world.units[0].step = Some(Step {
                to: cell(9, 10),
                progress: 0.5,
            });
            world
                .apply_command(Command::Gather {
                    unit_id: "villager-1".into(),
                    resource_id: "berries".into(),
                })
                .unwrap();
            let mut new_deliveries = 0;
            for _ in 0..600 {
                let before = world.inventories[0].food;
                world.tick(0.1);
                if world.inventories[0].food - before > 1e-9 {
                    new_deliveries += 1;
                }
                if world.inventories[0].wood > 0.0 && world.resources[0].amount > 1e-9 {
                    assert!(
                        matches!(world.units[0].action, UnitAction::Gather { .. }),
                        "reassignment stops after unloading: {:?}",
                        world.units[0]
                    );
                }
                if new_deliveries >= 2 {
                    break;
                }
            }
            assert_eq!(world.inventories[0].wood, amount);
            assert!(new_deliveries >= 2, "the reassigned loop must repeat");
        }
    }
}

#[test]
fn mixed_cargo_group_reassignment_on_the_generated_island_keeps_gathering() {
    let mut world = GameWorld::generate(DEFAULT_SEED);
    let target = world
        .resources
        .iter()
        .find(|r| r.kind == ResourceKind::Food)
        .unwrap()
        .id
        .clone();
    for (i, kind) in [ResourceKind::Wood, ResourceKind::Stone]
        .into_iter()
        .enumerate()
    {
        world.units[i].cargo = Some(CarriedResource { kind, amount: 7.0 });
        world
            .apply_command(Command::Gather {
                unit_id: world.units[i].id.clone(),
                resource_id: target.clone(),
            })
            .unwrap();
    }
    let mut deliveries = [0; 2];
    let mut stalled = [0; 2];
    for _ in 0..1200 {
        let before = world.units.clone();
        world.tick(0.1);
        for (i, unit) in world.units.iter().enumerate() {
            if before[i]
                .cargo
                .as_ref()
                .is_some_and(|c| c.kind == ResourceKind::Food)
                && unit.cargo.is_none()
            {
                deliveries[i] += 1;
            }
            if matches!(
                unit.action,
                UnitAction::Gather {
                    phase: GatherPhase::ToResource | GatherPhase::Returning,
                    ..
                }
            ) && unit.cell == before[i].cell
                && unit.step.is_none()
            {
                stalled[i] += 1;
            } else {
                stalled[i] = 0;
            }
            assert!(
                stalled[i] < 200,
                "stalled reassigned villager: {:?}; others: {:?}",
                unit,
                world.units
            );
        }
        if deliveries.iter().all(|n| *n >= 2) {
            break;
        }
    }
    assert!(
        deliveries.iter().all(|n| *n >= 2),
        "deliveries={deliveries:?}; units={:?}",
        world.units
    );
}

#[test]
fn even_a_partial_same_kind_load_is_unloaded_before_the_new_assignment() {
    let mut world = world();
    world.resources.push(node("berries", cell(8, 10), 100.0));
    world.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Food,
        amount: 7.0,
    });
    world
        .apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: "berries".into(),
        })
        .unwrap();
    assert!(matches!(
        world.units[0].action,
        UnitAction::Gather {
            phase: GatherPhase::Returning,
            ..
        }
    ));
    for _ in 0..300 {
        world.tick(0.1);
        assert_eq!(
            world.resources[0].amount, 100.0,
            "the new task cannot start while the old load is being delivered"
        );
        if world.units[0].cargo.is_none() {
            break;
        }
    }
    assert_eq!(world.inventories[0].food, 7.0);
    assert!(
        matches!(&world.units[0].action, UnitAction::Gather { resource_id, phase: GatherPhase::ToResource } if resource_id == "berries")
    );
    run(&mut world, 30.0);
    assert!(
        world.inventories[0].food >= 27.0,
        "the new assignment continues automatically"
    );
}

#[test]
fn a_builder_waits_for_unloading_instead_of_starting_with_cargo() {
    let mut world = world();
    world.buildings = vec![building(
        BuildingKind::House,
        "foundation",
        cell(12, 10),
        Some(0.0),
    )];
    world.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Wood,
        amount: 7.0,
    });
    world
        .apply_command(Command::Construct {
            unit_id: "villager-1".into(),
            building_id: "foundation".into(),
        })
        .unwrap();
    run(&mut world, 10.0);
    assert_eq!(world.buildings[0].construction, Some(0.0));
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 7.0);
    assert!(matches!(world.units[0].action, UnitAction::Build { .. }));
    world
        .buildings
        .push(town_center("base-1", cell(25, 10), None));
    run(&mut world, 30.0);
    assert_eq!(world.inventories[0].wood, 7.0);
    assert!(world.units[0].cargo.is_none());
    assert!(world.buildings[0].is_complete());
}

#[test]
fn generated_bush_gatherers_resume_after_delivery_at_double_speed() {
    for seed in [DEFAULT_SEED, 1, 2, 3, 42, 123, 999] {
        let mut world = GameWorld::generate(seed);
        world.simulation_speed = 2.0;
        let target = world
            .resources
            .iter()
            .find(|r| r.kind == ResourceKind::Food && world.can_reach_beside(0, r.footprint()))
            .unwrap()
            .id
            .clone();
        for i in 0..world.units.len() {
            world
                .apply_command(Command::Gather {
                    unit_id: world.units[i].id.clone(),
                    resource_id: target.clone(),
                })
                .unwrap();
        }
        let mut deliveries = vec![0; world.units.len()];
        for _ in 0..2000 {
            let before = world.units.clone();
            world.tick(0.1);
            for (i, old) in before.iter().enumerate() {
                if old.cargo.is_some() && world.units[i].cargo.is_none() {
                    deliveries[i] += 1;
                }
                if let UnitAction::Gather { resource_id, .. } = &old.action
                    && world.units[i].action == UnitAction::Idle
                {
                    let node = world
                        .resources
                        .iter()
                        .find(|r| &r.id == resource_id)
                        .unwrap();
                    assert!(
                        node.amount <= f64::EPSILON,
                        "seed={seed} abandoned live node {node:?}; before={old:?}; after={:?}",
                        world.units[i]
                    );
                }
            }
            if world.units.iter().all(|u| u.action == UnitAction::Idle) {
                break;
            }
        }
        assert!(
            deliveries.iter().all(|n| *n >= 2),
            "seed={seed}: {deliveries:?}; {:?}",
            world.units
        );
    }
}

#[test]
fn exhausted_patch_continuation_survives_distant_deliveries() {
    for kind in [ResourceKind::Food, ResourceKind::Wood, ResourceKind::Stone] {
        for drop_site in [cell(25, 10), cell(80, 40)] {
            for speed in [1.0, 2.0] {
                let mut w = world();
                w.simulation_speed = speed;
                w.units.truncate(1);
                w.buildings[0].origin = drop_site;
                w.units[0].cell = cell(9, 10);
                w.resources = vec![
                    node("patch-left", cell(10, 10), 10.0),
                    node("patch-right", cell(11, 10), 10.0),
                    node("next-patch", cell(21, 10), 40.0),
                    node("distant-patch", cell(45, 10), 30.0),
                    node("unrelated-old-patch", cell(40, 10), 0.0),
                ];
                for resource in &mut w.resources {
                    resource.kind = kind;
                }
                w.apply_command(Command::Gather {
                    unit_id: "villager-1".into(),
                    resource_id: "patch-right".into(),
                })
                .unwrap();
                // Right then left fills the first load. The last bush is eleven cells
                // from the next patch, but its connected neighbor is within ten.
                for _ in 0..6000 {
                    w.tick(0.1);
                    if w.inventories[0].amount(kind) >= 20.0 - 1e-8 && w.resources[2].amount > 0.0 {
                        assert!(
                            matches!(w.units[0].action, UnitAction::Gather { .. }),
                            "{kind:?} {drop_site:?} {speed}x lost the patch after delivery: {:?}",
                            w.units[0]
                        );
                    }
                    if w.units[0].action == UnitAction::Idle {
                        break;
                    }
                }
                assert!((w.inventories[0].amount(kind) - 60.0).abs() < 1e-8);
                assert_eq!(w.resources[2].amount, 0.0);
                assert_eq!(
                    w.resources[3].amount, 30.0,
                    "do not search the whole island"
                );
                assert_eq!(w.units[0].action, UnitAction::Idle);
                assert!(w.units[0].cargo.is_none());
                w.validate().unwrap();
            }
        }
    }
}

#[test]
fn live_resource_assignment_survives_repeated_distant_deliveries() {
    for kind in [ResourceKind::Food, ResourceKind::Wood, ResourceKind::Stone] {
        for speed in [1.0, 2.0] {
            let mut w = world();
            w.simulation_speed = speed;
            w.units.truncate(1);
            w.buildings[0].origin = cell(80, 40);
            let mut resource = node("source", cell(11, 10), 100.0);
            resource.kind = kind;
            w.resources = vec![resource];
            w.apply_command(Command::Gather {
                unit_id: "villager-1".into(),
                resource_id: "source".into(),
            })
            .unwrap();
            for _ in 0..6000 {
                w.tick(0.1);
                if w.resources[0].amount > 0.0 {
                    assert!(
                        matches!(&w.units[0].action, UnitAction::Gather { resource_id, .. } if resource_id == "source"),
                        "{kind:?} {speed}x abandoned a live distant resource"
                    );
                }
                if w.inventories[0].amount(kind) >= 60.0 - 1e-8 {
                    break;
                }
            }
            assert!((w.inventories[0].amount(kind) - 60.0).abs() < 1e-8);
            assert!((w.resources[0].amount - 40.0).abs() < 1e-8);
            assert!(
                matches!(&w.units[0].action, UnitAction::Gather { resource_id, .. } if resource_id == "source")
            );
            w.validate().unwrap();
        }
    }
}
