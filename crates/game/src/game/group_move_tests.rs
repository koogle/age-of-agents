use std::collections::BTreeSet;

use super::tests::{cell, house};
use super::*;

fn train_until(world: &mut GameWorld, count: usize) {
    house(world, count);
    world.inventories[0].food = 1_000.0;
    while world.units.len() < count {
        if world.buildings[0].job.is_none() {
            world
                .apply_command(Command::Produce {
                    building_id: "base-1".into(),
                    product: ProductKind::Villager,
                })
                .unwrap();
        }
        world.tick(0.5);
    }
}

#[test]
fn group_move_assigns_distinct_deterministic_reachable_destinations() {
    let mut world = fixture::fixture();
    train_until(&mut world, 6);
    let ids: Vec<_> = world
        .units
        .iter()
        .rev()
        .map(|unit| unit.id.clone())
        .collect();
    let target = cell(10, 15);
    let mut repeat = world.clone();
    let command = Command::GroupMove {
        unit_ids: ids.clone(),
        to: target,
    };
    world.apply_command(command.clone()).unwrap();
    repeat.apply_command(command).unwrap();
    assert_eq!(world, repeat);

    let destinations: BTreeSet<_> = world
        .units
        .iter()
        .map(|unit| match unit.action {
            UnitAction::Move { to } => to,
            ref other => panic!("{} received {other:?}", unit.id),
        })
        .collect();
    assert_eq!(destinations.len(), 6);
    assert!(destinations.contains(&target));
    assert!(
        destinations
            .iter()
            .all(|to| to.center().distance(target.center()) < 2.0)
    );

    for _ in 0..400 {
        world.tick(0.1);
    }
    let arrived: BTreeSet<_> = world.units.iter().map(|unit| unit.cell).collect();
    assert_eq!(arrived, destinations);
    assert!(
        world
            .units
            .iter()
            .all(|unit| unit.action == UnitAction::Idle)
    );
}

#[test]
fn group_move_is_atomic_when_any_member_is_invalid() {
    let mut world = fixture::fixture();
    world
        .apply_command(Command::Gather {
            unit_id: "villager-2".into(),
            resource_id: "tree-1".into(),
        })
        .unwrap();
    let before = world.clone();
    for (unit_ids, expected) in [
        (vec![], CommandError::EmptyUnitGroup),
        (
            vec!["villager-1".into(), "villager-1".into()],
            CommandError::DuplicateUnit,
        ),
        (
            vec!["villager-1".into(), "ghost".into()],
            CommandError::UnitNotFound,
        ),
    ] {
        assert_eq!(
            world.apply_command(Command::GroupMove {
                unit_ids,
                to: cell(5, 5),
            }),
            Err(expected)
        );
        assert_eq!(world, before);
    }
}

#[test]
fn crossing_groups_never_share_a_cell() {
    let mut world = fixture::fixture();
    world.resources.clear();
    train_until(&mut world, 8);
    let (west, east): (Vec<_>, Vec<_>) = world
        .units
        .iter()
        .map(|unit| unit.id.clone())
        .enumerate()
        .partition(|(index, _)| index % 2 == 0);
    let ids = |group: Vec<(usize, String)>| group.into_iter().map(|(_, id)| id).collect();
    world
        .apply_command(Command::GroupMove {
            unit_ids: ids(west),
            to: cell(3, 10),
        })
        .unwrap();
    world
        .apply_command(Command::GroupMove {
            unit_ids: ids(east),
            to: cell(26, 10),
        })
        .unwrap();
    // `tick` validates exclusive claims every step in debug builds; this also
    // checks it explicitly so release-mode test runs keep the guarantee.
    for _ in 0..600 {
        world.tick(0.1);
        world.validate().unwrap();
    }
    assert!(
        world
            .units
            .iter()
            .all(|unit| unit.action == UnitAction::Idle)
    );
}

/// A mountain wall across the map with a single one-cell gap at `(gap, row)`.
fn walled_with_gap(gap: u16, row: u16) -> GameWorld {
    let mut world = fixture::fixture();
    let stride = usize::from(world.columns());
    for column in 0..world.columns() {
        if column != gap {
            world.terrain[usize::from(row) * stride + usize::from(column)].biome =
                TerrainBiome::Mountain;
        }
    }
    world
}

#[test]
fn an_idle_unit_in_a_one_cell_gap_steps_aside_for_a_stalled_mover() {
    let (gap, wall) = (90, 45);
    let mut world = walled_with_gap(gap, wall);
    // villager-2 idles in the gap; villager-1 must pass through it.
    world.units[1].cell = cell(gap, wall);
    world.units[0].cell = cell(gap, wall + 4);
    world.validate().unwrap();
    let to = cell(gap, wall - 6);
    world
        .apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to,
        })
        .unwrap();
    for _ in 0..200 {
        world.tick(0.1);
        world.validate().unwrap();
    }
    assert_eq!(world.units[0].cell, to, "the mover got through the gap");
    // The bystander only made way: it is idle again, near where it stood, and off
    // the gap so it no longer blocks the passage.
    assert_eq!(world.units[1].action, UnitAction::Idle);
    assert_ne!(world.units[1].cell, cell(gap, wall));
    assert!(world.units[1].cell.column.abs_diff(gap) <= 2);
    assert!(world.units[1].cell.row.abs_diff(wall) <= 2);
}

#[test]
fn an_idle_unit_beside_a_working_builder_stays_put() {
    let mut world = fixture::fixture();
    world.inventories[0].wood = 100.0;
    let origin = super::tests::free_site(&mut world, BuildingKind::House);
    world
        .apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin,
            kind: BuildingKind::House,
        })
        .unwrap();
    // Let the builder arrive and start working, then park villager-2 beside it.
    for _ in 0..100 {
        world.tick(0.1);
        if world.units[0].step.is_none()
            && world.buildings.last().unwrap().construction.unwrap_or(0.0) > 0.0
        {
            break;
        }
    }
    let builder = world.units[0].cell;
    let occupancy = world.occupancy();
    let beside = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
        .into_iter()
        .filter_map(|(dx, dy)| {
            crate::navigation::offset(builder, dx, dy, world.columns(), world.rows())
        })
        .find(|&c| occupancy.is_free_for(c, None))
        .expect("room beside the builder");
    world.units[1].cell = beside;
    world.units[1].action = UnitAction::Idle;
    world.validate().unwrap();
    for _ in 0..30 {
        world.tick(0.1);
        assert_eq!(world.units[1].cell, beside);
        assert_eq!(world.units[1].action, UnitAction::Idle);
    }
}
