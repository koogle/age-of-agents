use std::collections::BTreeSet;

use super::tests::cell;
use super::*;

fn train_until(world: &mut GameWorld, count: usize) {
    world.stockpile.food = 1_000.0;
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
    let mut world = GameWorld::default();
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
    let mut world = GameWorld::default();
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
        (
            vec!["villager-1".into(), "villager-2".into()],
            CommandError::UnitBusy,
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
    let mut world = GameWorld::default();
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
