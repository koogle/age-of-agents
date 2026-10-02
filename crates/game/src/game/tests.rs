use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::movement::interaction_cells;
use super::*;

pub(super) fn cell(column: u16, row: u16) -> CellCoordinate {
    CellCoordinate::new(column, row)
}

/// Moves `unit` onto a free cell beside `footprint`, as if it had walked there.
pub(super) fn stand_beside(world: &mut GameWorld, unit: usize, footprint: Footprint) {
    let occupancy = world.occupancy();
    let spot = interaction_cells(footprint)
        .find(|candidate| occupancy.is_free_for(*candidate, Some(unit)))
        .expect("a free interaction cell");
    world.units[unit].cell = spot;
    world.units[unit].step = None;
}

fn start_gather_at_resource(world: &mut GameWorld, resource_index: usize, amount: f64) {
    world.resources[resource_index].amount = amount;
    world.resources[resource_index].capacity = amount.max(1.0);
    let footprint = world.resources[resource_index].footprint();
    stand_beside(world, 0, footprint);
    world
        .apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: world.resources[resource_index].id.clone(),
        })
        .unwrap();
}

fn assert_gather_phase(unit: &Unit, expected: GatherPhase) {
    assert!(
        matches!(unit.action, UnitAction::Gather { phase, .. } if phase == expected),
        "expected {expected:?}, got {:?}",
        unit.action
    );
}

fn run(world: &mut GameWorld, seconds: f64) {
    for _ in 0..(seconds * 10.0).round() as u32 {
        world.tick(0.1);
    }
}

#[test]
fn voronoi_terrain_is_fixed_and_deterministic() {
    assert_eq!(GameWorld::default().terrain, GameWorld::default().terrain);
    let world = GameWorld::default();
    let coordinates: BTreeSet<_> = world.terrain.iter().map(|cell| cell.coordinate()).collect();
    assert_eq!(world.terrain.len(), 2400);
    assert_eq!(coordinates.len(), 2400);
    let biomes: BTreeSet<_> = world.terrain.iter().map(|cell| cell.biome).collect();
    assert_eq!(biomes.len(), 8);
}

#[test]
fn every_voronoi_biome_is_one_coherent_region() {
    let world = GameWorld::default();
    let mut regions: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for terrain in &world.terrain {
        regions
            .entry(terrain.biome)
            .or_default()
            .insert(terrain.coordinate());
    }
    for (biome, region) in regions {
        let mut reached = BTreeSet::new();
        let mut queue = VecDeque::from([*region.first().unwrap()]);
        while let Some(current) = queue.pop_front() {
            if !reached.insert(current) {
                continue;
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(next) =
                    crate::navigation::offset(current, dx, dy, WORLD_COLUMNS, WORLD_ROWS)
                    && region.contains(&next)
                {
                    queue.push_back(next);
                }
            }
        }
        assert_eq!(reached, region, "{biome:?} is not coherent");
    }
}

#[test]
fn resources_are_deterministic_clustered_and_biome_compatible() {
    let world = GameWorld::default();
    assert_eq!(world.resources, GameWorld::default().resources);
    let mut counts = BTreeMap::new();
    for resource in &world.resources {
        *counts.entry(resource.kind).or_insert(0) += 1;
    }
    assert_eq!(
        counts,
        BTreeMap::from([
            (ResourceKind::Wood, 30),
            (ResourceKind::Food, 10),
            (ResourceKind::Stone, 8),
            (ResourceKind::Gold, 4),
            (ResourceKind::Iron, 4),
            (ResourceKind::Clay, 6),
            (ResourceKind::Fiber, 6),
        ])
    );
    let base = Position {
        x: f64::from(WORLD_COLUMNS) / 2.0,
        y: f64::from(WORLD_ROWS) / 2.0,
    };
    for (index, resource) in world.resources.iter().enumerate() {
        let biome = world.terrain[usize::from(resource.cell.row) * usize::from(WORLD_COLUMNS)
            + usize::from(resource.cell.column)]
        .biome;
        assert!(compatible_biomes(resource.kind).contains(&biome));
        assert!(resource.cell.center().distance(base) >= STARTING_BASE_RESOURCE_CLEARANCE);
        assert_eq!(resource.amount, resource.capacity);
        for other in &world.resources[index + 1..] {
            assert_ne!(resource.cell, other.cell);
            // Nodes of one kind may touch inside a cluster; different kinds
            // never share a cluster.
            if resource.kind != other.kind {
                assert!(
                    resource.cell.center().distance(other.cell.center()) + f64::EPSILON
                        >= RESOURCE_CLUSTER_SEPARATION
                );
            }
        }
        // Every node in a cluster touches another node of its kind.
        assert!(world.resources.iter().any(|other| other.id != resource.id
            && other.kind == resource.kind
            && other.cell.touches(resource.cell)));
    }
}

#[test]
fn default_world_is_valid_and_has_a_productive_base() {
    let world = GameWorld::default();
    world.validate().unwrap();
    assert_eq!(world.buildings.len(), 1);
    let base = &world.buildings[0];
    assert_eq!(base.kind, BuildingKind::TownCenter);
    assert!(base.is_complete());
    assert_eq!(base.footprint().cells().count(), 16);
    assert_eq!(base.researches, TechnologyKind::ALL);
    for unit in &world.units {
        assert!(base.footprint().is_interaction_cell(unit.cell));
    }
}

#[test]
fn idle_world_is_invariant_except_tick() {
    let mut world = GameWorld::default();
    let initial = world.clone();
    world.tick(10.0);
    world.tick = 0;
    world.scenario.elapsed_ticks = 0;
    assert_eq!(world, initial);
}

#[test]
fn move_walks_cell_by_cell_to_the_exact_destination() {
    let mut world = GameWorld::default();
    let destination = cell(14, 15);
    world
        .apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to: destination,
        })
        .unwrap();
    let mut previous = world.units[0].cell;
    for _ in 0..200 {
        world.tick(0.1);
        let unit = &world.units[0];
        assert!(unit.cell == previous || unit.cell.touches(previous));
        previous = unit.cell;
        if unit.action == UnitAction::Idle {
            break;
        }
    }
    assert_eq!(world.units[0].cell, destination);
    assert_eq!(world.units[0].step, None);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    assert!(world.explored_cells.contains(&cell(14, 18)));
}

#[test]
fn move_rejects_invalid_destinations_without_mutation() {
    let mut world = GameWorld::default();
    world
        .apply_command(Command::Move {
            unit_id: "villager-2".into(),
            to: cell(40, 28),
        })
        .unwrap();
    let resource = world.resources[0].cell;
    let before = world.clone();
    for (to, expected) in [
        (cell(60, 0), CommandError::InvalidDestination),
        (cell(0, 40), CommandError::InvalidDestination),
        (cell(28, 17), CommandError::DestinationOccupied),
        (resource, CommandError::DestinationOccupied),
        (cell(31, 21), CommandError::DestinationOccupied),
        (cell(40, 28), CommandError::DestinationOccupied),
    ] {
        assert_eq!(
            world.apply_command(Command::Move {
                unit_id: "villager-1".into(),
                to,
            }),
            Err(expected),
            "{to:?}"
        );
        assert_eq!(world, before);
    }
}

#[test]
fn walled_off_destination_is_unreachable() {
    let mut world = GameWorld::default();
    world.resources.clear();
    for (index, origin) in [cell(0, 4), cell(4, 4), cell(4, 0)].into_iter().enumerate() {
        world
            .buildings
            .push(town_center(&format!("wall-{index}"), origin, None));
    }
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to: cell(0, 0),
        }),
        Err(CommandError::TargetUnreachable)
    );
    assert_eq!(world, before);
}

#[test]
fn walkers_route_around_buildings_and_never_enter_them() {
    let mut world = GameWorld::default();
    world
        .apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to: cell(14, 7),
        })
        .unwrap();
    let base = world.buildings[0].footprint();
    for _ in 0..100 {
        world.tick(0.1);
        let unit = &world.units[0];
        assert!(!base.contains(unit.cell));
        assert!(unit.step.is_none_or(|step| !base.contains(step.to)));
    }
    assert_eq!(world.units[0].cell, cell(14, 7));
}

#[test]
fn partial_last_load_is_carried_then_deposited_before_becoming_idle() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 15.0);
    world.tick(15.0 / GATHER_RATE);
    assert_eq!(
        world.units[0].cargo,
        Some(CarriedResource {
            kind: ResourceKind::Wood,
            amount: 15.0,
        })
    );
    assert_eq!(world.resources[0].amount, 0.0);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);
    world.tick(100.0);
    assert_gather_phase(&world.units[0], GatherPhase::Depositing);
    assert_eq!(world.stockpile.wood, 0.0);
    world.tick(0.1);
    assert_eq!(world.stockpile.wood, 15.0);
    assert_eq!(world.units[0].cargo, None);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn full_loads_deposit_once_resume_and_finish_after_depletion() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 45.0);
    for expected in [20.0, 40.0, 45.0] {
        world.tick(100.0);
        assert_gather_phase(&world.units[0], GatherPhase::Returning);
        world.tick(100.0);
        assert_gather_phase(&world.units[0], GatherPhase::Depositing);
        world.tick(0.1);
        assert_eq!(world.stockpile.wood, expected);
    }
    assert_eq!(world.resources[0].amount, 0.0);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.tick(100.0);
    assert_eq!(world.stockpile.wood, 45.0);
}

#[test]
fn gatherer_stays_beside_the_node_until_the_load_is_full() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 100.0);
    let spot = world.units[0].cell;
    world.tick(4.0);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 8.0);
    assert_gather_phase(&world.units[0], GatherPhase::Gathering);
    assert_eq!(world.units[0].cell, spot);
    world.tick(6.0);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);
}

#[test]
fn deleted_resource_returns_existing_cargo_and_then_finishes() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 20.0);
    world.tick(0.5);
    world.resources.remove(0);
    world.units[0].action = UnitAction::Gather {
        resource_id: "gone".into(),
        phase: GatherPhase::Gathering,
    };
    world.resources.push(ResourceNode {
        id: "gone".into(),
        kind: ResourceKind::Wood,
        cell: cell(0, 0),
        amount: 0.0,
        capacity: 1.0,
    });
    world.tick(0.1);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);
    world.tick(100.0);
    world.tick(0.1);
    assert_eq!(world.stockpile.wood, GATHER_RATE * 0.5);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn missing_town_center_retains_cargo_until_a_deposit_is_possible() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 10.0);
    world.tick(10.0 / GATHER_RATE);
    let spot = world.units[0].cell;
    let town_center = world.buildings.remove(0);
    world.tick(100.0);
    assert_eq!(world.units[0].cell, spot);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 10.0);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);
    world.buildings.push(town_center);
    world.tick(100.0);
    world.tick(0.1);
    assert_eq!(world.stockpile.wood, 10.0);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn deposits_route_each_resource_to_its_typed_stockpile() {
    for kind in [
        ResourceKind::Wood,
        ResourceKind::Food,
        ResourceKind::Stone,
        ResourceKind::Gold,
        ResourceKind::Iron,
        ResourceKind::Clay,
        ResourceKind::Fiber,
    ] {
        let mut world = GameWorld::default();
        let index = world
            .resources
            .iter()
            .position(|resource| resource.kind == kind)
            .unwrap();
        start_gather_at_resource(&mut world, index, 10.0);
        world.tick(10.0 / GATHER_RATE);
        world.tick(100.0);
        world.tick(0.1);
        let totals: BTreeMap<_, _> = world.stockpile.entries().into_iter().collect();
        let name = serde_json::to_value(kind).unwrap();
        for (resource, amount) in totals {
            let expected = if name == resource { 10.0 } else { 0.0 };
            assert_eq!(amount, expected, "{kind:?} deposited into {resource}");
        }
    }
}

#[test]
fn town_center_trains_one_villager_at_a_time_and_reserves_food_once() {
    let mut world = GameWorld::default();
    world.stockpile.food = 100.0;
    let produce = Command::Produce {
        building_id: "base-1".into(),
        product: ProductKind::Villager,
    };
    world.apply_command(produce.clone()).unwrap();
    assert_eq!(world.stockpile.food, 50.0);
    let before = world.clone();
    assert_eq!(
        world.apply_command(produce),
        Err(CommandError::BuildingBusy)
    );
    assert_eq!(world, before);
    world.tick(VILLAGER_PRODUCTION_SECONDS - 0.1);
    assert_eq!(world.units.len(), 2);
    world.tick(0.1);
    assert_eq!(world.units.len(), 3);
    assert_eq!(world.units[2].id, "villager-3");
    assert!(
        world.buildings[0]
            .footprint()
            .is_interaction_cell(world.units[2].cell)
    );
    assert_eq!(world.buildings[0].job, None);
    assert_eq!(world.stockpile.food, 50.0);
}

#[test]
fn trained_villager_waits_for_a_free_cell_beside_the_building() {
    let mut world = GameWorld::default();
    world.resources.clear();
    let ring: Vec<_> = interaction_cells(world.buildings[0].footprint()).collect();
    world.units = ring
        .iter()
        .enumerate()
        .map(|(index, &spot)| Unit {
            id: format!("blocker-{index}"),
            kind: UnitKind::Villager,
            cell: spot,
            step: None,
            action: UnitAction::Idle,
            cargo: None,
        })
        .collect();
    world.stockpile.food = VILLAGER_FOOD_COST;
    world
        .apply_command(Command::Produce {
            building_id: "base-1".into(),
            product: ProductKind::Villager,
        })
        .unwrap();
    world.tick(VILLAGER_PRODUCTION_SECONDS);
    world.tick(1.0);
    assert_eq!(world.units.len(), ring.len());
    assert!(world.buildings[0].job.is_some());
    world.units.pop();
    world.tick(0.1);
    assert_eq!(world.units.len(), ring.len());
    assert_eq!(world.buildings[0].job, None);
}

#[test]
fn research_uses_the_slot_reserves_once_and_enforces_prerequisites() {
    let mut world = GameWorld::default();
    world.stockpile.food = 100.0;
    world.stockpile.wood = 100.0;
    let research = |technology| Command::Research {
        building_id: "base-1".into(),
        technology,
    };
    let before = world.clone();
    assert_eq!(
        world.apply_command(research(TechnologyKind::Mining)),
        Err(CommandError::MissingTechnologyPrerequisite)
    );
    assert_eq!(world, before);
    world
        .apply_command(research(TechnologyKind::Masonry))
        .unwrap();
    assert_eq!((world.stockpile.food, world.stockpile.wood), (60.0, 80.0));
    assert_eq!(
        world.apply_command(Command::Produce {
            building_id: "base-1".into(),
            product: ProductKind::Villager,
        }),
        Err(CommandError::BuildingBusy)
    );
    world.tick(RESEARCH_SECONDS - 0.1);
    assert!(world.researched_technologies.is_empty());
    world.tick(0.1);
    assert_eq!(world.researched_technologies, vec![TechnologyKind::Masonry]);
    assert_eq!(
        world.apply_command(research(TechnologyKind::Masonry)),
        Err(CommandError::TechnologyAlreadyResearched)
    );
}

#[test]
fn researched_technologies_make_their_resources_twenty_percent_faster() {
    for (technology, kind) in [
        (TechnologyKind::Forestry, ResourceKind::Wood),
        (TechnologyKind::Agriculture, ResourceKind::Food),
        (TechnologyKind::Masonry, ResourceKind::Clay),
        (TechnologyKind::Mining, ResourceKind::Iron),
        (TechnologyKind::Textiles, ResourceKind::Fiber),
    ] {
        let mut world = GameWorld {
            researched_technologies: vec![technology],
            ..GameWorld::default()
        };
        let index = world
            .resources
            .iter()
            .position(|resource| resource.kind == kind)
            .unwrap();
        start_gather_at_resource(&mut world, index, 20.0);
        world.tick(1.0);
        assert_eq!(
            world.resources[index].amount,
            20.0 - GATHER_RATE * GATHERING_TECH_MULTIPLIER
        );
    }
}

#[test]
fn simulation_speed_is_authoritative_validated_and_can_pause() {
    let mut world = GameWorld::default();
    world
        .apply_command(Command::SetSimulationSpeed { multiplier: 0.0 })
        .unwrap();
    let paused = world.clone();
    world.tick(100.0);
    assert_eq!(world, paused);
    world
        .apply_command(Command::SetSimulationSpeed { multiplier: 2.0 })
        .unwrap();
    start_gather_at_resource(&mut world, 0, 100.0);
    world.tick(1.0);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 4.0);
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::SetSimulationSpeed { multiplier: 3.0 }),
        Err(CommandError::InvalidSimulationSpeed)
    );
    assert_eq!(world, before);
}

#[test]
fn busy_unit_rejects_commands_without_mutation() {
    let mut world = GameWorld::default();
    world
        .apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: "tree-1".into(),
        })
        .unwrap();
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to: cell(5, 5),
        }),
        Err(CommandError::UnitBusy)
    );
    assert_eq!(world, before);
}

#[test]
fn build_places_one_foundation_immediately_and_charges_once() {
    let mut world = GameWorld::default();
    world.stockpile.wood = TOWN_CENTER_WOOD_COST;
    world
        .apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(20, 24),
        })
        .unwrap();
    assert_eq!(world.stockpile.wood, 0.0);
    assert_eq!(world.buildings.len(), 2);
    let foundation = world.buildings[1].clone();
    assert_eq!(foundation.id, "building-2");
    assert_eq!(foundation.construction, Some(0.0));
    let claimed = world.claimed_cells();
    assert!(foundation.footprint().cells().all(|c| claimed.contains(&c)));

    // A foundation is not a working town center.
    world.stockpile.food = 100.0;
    assert_eq!(
        world.apply_command(Command::Produce {
            building_id: "building-2".into(),
            product: ProductKind::Villager,
        }),
        Err(CommandError::BuildingUnderConstruction)
    );
    // Nobody can be ordered onto it.
    assert_eq!(
        world.apply_command(Command::Move {
            unit_id: "villager-2".into(),
            to: cell(21, 25),
        }),
        Err(CommandError::DestinationOccupied)
    );

    run(&mut world, 30.0);
    assert!(world.buildings[1].is_complete());
    assert_eq!(world.buildings.len(), 2);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    assert!(
        world.buildings[1]
            .footprint()
            .is_interaction_cell(world.units[0].cell)
    );
    run(&mut world, 10.0);
    assert_eq!(world.buildings.len(), 2);
    assert_eq!(world.stockpile.wood, 0.0);
}

#[test]
fn build_rejects_blocked_or_unaffordable_sites_without_mutation() {
    let mut world = GameWorld::default();
    let build = |origin| Command::Build {
        unit_id: "villager-1".into(),
        origin,
    };
    let before = world.clone();
    assert_eq!(
        world.apply_command(build(cell(44, 30))),
        Err(CommandError::InsufficientWood)
    );
    assert_eq!(world, before);
    world.stockpile.wood = 100.0;
    world
        .apply_command(Command::Move {
            unit_id: "villager-2".into(),
            to: cell(40, 30),
        })
        .unwrap();
    let resource = world.resources[0].cell;
    let before = world.clone();
    for origin in [
        cell(58, 5),  // straddles the east edge
        cell(5, 38),  // straddles the south edge
        cell(26, 16), // overlaps the town center
        resource,     // overlaps a resource
        cell(27, 18), // overlaps villager-1 and the base
        cell(30, 18), // overlaps villager-2 and the base
        cell(38, 28), // covers villager-2's reservation
    ] {
        assert_eq!(
            world.apply_command(build(origin)),
            Err(CommandError::InvalidBuildSite),
            "{origin:?}"
        );
        assert_eq!(world, before);
    }
}

#[test]
fn unreachable_build_site_is_rejected_and_leaves_no_foundation() {
    let mut world = GameWorld::default();
    world.resources.clear();
    world.stockpile.wood = 100.0;
    world.units[0].cell = cell(0, 0);
    // Seal villager-1 into the north-west pocket.
    for (index, origin) in [cell(4, 0), cell(0, 6), cell(4, 4)].into_iter().enumerate() {
        world
            .buildings
            .push(town_center(&format!("wall-{index}"), origin, None));
    }
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(20, 15),
        }),
        Err(CommandError::TargetUnreachable)
    );
    assert_eq!(world, before);
}

#[test]
fn a_second_villager_can_help_and_construction_completes_exactly_once() {
    let mut solo = GameWorld::default();
    solo.stockpile.wood = TOWN_CENTER_WOOD_COST;
    solo.apply_command(Command::Build {
        unit_id: "villager-1".into(),
        origin: cell(34, 19),
    })
    .unwrap();
    let mut pair = solo.clone();
    pair.apply_command(Command::Construct {
        unit_id: "villager-2".into(),
        building_id: "building-2".into(),
    })
    .unwrap();

    let mut solo_ticks = 0;
    while !solo.buildings[1].is_complete() {
        solo.tick(0.1);
        solo_ticks += 1;
    }
    let mut pair_ticks = 0;
    while !pair.buildings[1].is_complete() {
        pair.tick(0.1);
        pair_ticks += 1;
    }
    assert!(pair_ticks < solo_ticks, "{pair_ticks} >= {solo_ticks}");
    pair.tick(0.1);
    assert!(
        pair.units
            .iter()
            .all(|unit| unit.action == UnitAction::Idle)
    );
    assert_eq!(pair.buildings.len(), 2);
    assert_eq!(
        pair.apply_command(Command::Construct {
            unit_id: "villager-2".into(),
            building_id: "building-2".into(),
        }),
        Err(CommandError::BuildingAlreadyComplete)
    );
}

#[test]
fn a_cell_someone_is_walking_through_can_be_reserved_and_is_reached() {
    let mut world = GameWorld::default();
    world
        .apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to: cell(14, 16),
        })
        .unwrap();
    while world.units[0].step.is_none_or(|step| step.to.row < 13) {
        world.tick(0.1);
    }
    let crossing = world.units[0].step.unwrap().to;
    world
        .apply_command(Command::Move {
            unit_id: "villager-2".into(),
            to: crossing,
        })
        .unwrap();
    run(&mut world, 20.0);
    assert_eq!(world.units[0].cell, cell(14, 16));
    assert_eq!(world.units[1].cell, crossing);
    assert!(
        world
            .units
            .iter()
            .all(|unit| unit.action == UnitAction::Idle)
    );
}

#[test]
fn a_unit_whose_order_ends_on_a_reserved_cell_makes_way() {
    let mut world = GameWorld::default();
    world
        .apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to: cell(14, 16),
        })
        .unwrap();
    while world.units[0].step.is_none() {
        world.tick(0.1);
    }
    let crossing = world.units[0].step.unwrap().to;
    world
        .apply_command(Command::Move {
            unit_id: "villager-2".into(),
            to: crossing,
        })
        .unwrap();
    // The walker's order ends mid-stride, onto the reserved cell.
    world.units[0].action = UnitAction::Idle;
    run(&mut world, 20.0);
    assert_eq!(world.units[1].cell, crossing);
    assert_ne!(world.units[0].cell, crossing);
    assert!(
        world
            .units
            .iter()
            .all(|unit| unit.action == UnitAction::Idle)
    );
}

#[test]
fn head_on_walkers_in_a_corridor_pass_each_other() {
    let mut world = GameWorld::default();
    world.resources.clear();
    world.buildings.clear();
    // A one-cell-wide corridor along row 5 with a pocket at column 10.
    for column in 0..WORLD_COLUMNS {
        for row in [4, 6] {
            if !(column == 10 && row == 6) {
                world.resources.push(ResourceNode {
                    id: format!("wall-{column}-{row}"),
                    kind: ResourceKind::Stone,
                    cell: cell(column, row),
                    amount: 1.0,
                    capacity: 1.0,
                });
            }
        }
    }
    world.units[0].cell = cell(5, 5);
    world.units[1].cell = cell(15, 5);
    for (unit_id, to) in [("villager-1", cell(18, 5)), ("villager-2", cell(2, 5))] {
        world
            .apply_command(Command::Move {
                unit_id: unit_id.into(),
                to,
            })
            .unwrap();
    }
    run(&mut world, 30.0);
    assert_eq!(world.units[0].cell, cell(18, 5));
    assert_eq!(world.units[1].cell, cell(2, 5));
}

#[test]
fn snapshots_round_trip_through_json_for_remote_clients() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 40.0);
    run(&mut world, 3.0);
    let snapshot = world.snapshot();
    let json = serde_json::to_string(&snapshot).unwrap();
    let decoded: WorldSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, snapshot);
}

#[test]
fn stop_ends_any_task_keeping_cargo_and_foundation_progress() {
    let mut world = GameWorld::default();
    world.stockpile.wood = TOWN_CENTER_WOOD_COST;
    world
        .apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: "berries-1".into(),
        })
        .unwrap();
    let mut ticks = 0;
    while !world.units[0]
        .cargo
        .as_ref()
        .is_some_and(|cargo| cargo.amount >= 2.0)
    {
        world.tick(0.1);
        ticks += 1;
        assert!(ticks < 600, "villager-1 never started picking");
    }
    world
        .apply_command(Command::Build {
            unit_id: "villager-2".into(),
            origin: cell(34, 20),
        })
        .unwrap();
    while world.buildings[1].construction.unwrap_or(0.0) < 0.5 {
        world.tick(0.1);
        ticks += 1;
        assert!(ticks < 600, "villager-2 never started building");
    }
    for unit_id in ["villager-1", "villager-2"] {
        world
            .apply_command(Command::Stop {
                unit_id: unit_id.into(),
            })
            .unwrap();
    }
    let cargo = world.units[0].cargo.clone();
    let progress = world.buildings[1].construction;
    run(&mut world, 3.0);
    world.validate().unwrap();
    assert!(
        world
            .units
            .iter()
            .all(|unit| unit.action == UnitAction::Idle && unit.step.is_none())
    );
    assert_eq!(world.units[0].cargo, cargo);
    assert_eq!(world.buildings[1].construction, progress);
    assert_eq!(world.stockpile.wood, 0.0);
    // Stopping an idle unit is harmless; an unknown one is rejected untouched.
    let before = world.clone();
    world
        .apply_command(Command::Stop {
            unit_id: "villager-1".into(),
        })
        .unwrap();
    assert_eq!(world, before);
    assert_eq!(
        world.apply_command(Command::Stop {
            unit_id: "villager-9".into(),
        }),
        Err(CommandError::UnitNotFound)
    );
    // A stopped villager takes a fresh order straight away.
    world
        .apply_command(Command::Construct {
            unit_id: "villager-2".into(),
            building_id: "building-2".into(),
        })
        .unwrap();
}
