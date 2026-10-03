use super::movement::interaction_cells;
use super::tests::{cell, run};
use super::*;

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
    let mut world = world();
    world.buildings.push(building(
        BuildingKind::Granary,
        "granary",
        cell(12, 10),
        None,
    ));
    let near = world.buildings[1].footprint();
    // Only one approach cell remains, and an idle villager occupies it.
    let entrance = cell(11, 11);
    for at in interaction_cells(near).filter(|at| *at != entrance) {
        let index = usize::from(at.row) * usize::from(WORLD_COLUMNS) + usize::from(at.column);
        world.terrain[index].biome = TerrainBiome::Water;
    }
    world.units[1].cell = entrance;
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
    assert_eq!(
        world.nearest_drop_site(0),
        Some(world.buildings[0].footprint())
    );
    run(&mut world, 30.0);
    assert!(world.stockpile.food >= 20.0);
    assert!(
        world.resources[0].amount < 40.0,
        "gathering resumes after unloading"
    );
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
        let before = world.stockpile.food;
        world.tick(0.1);
        if world.stockpile.food > before {
            deliveries += 1;
            assert!(world.is_beside(0, world.buildings[1].footprint()));
            assert!((world.stockpile.food - f64::from(deliveries) * 20.0).abs() < 1e-9);
            assert!(world.units[0].cargo.is_none());
        }
    }
    assert_eq!(deliveries, 3);
    assert_eq!(world.resources[0].amount, 0.0);
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
    assert_eq!(world.stockpile.food, 0.0);
    world.units[1].action = UnitAction::Idle;
    run(&mut world, 30.0);
    assert!(world.stockpile.food >= 20.0);
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
            world.stockpile.food >= 20.0,
            "the preserved loop gathers and unloads"
        );
    }
}
