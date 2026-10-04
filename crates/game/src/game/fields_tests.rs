use super::tests::{cell, run};
use super::*;

fn world() -> GameWorld {
    let mut w = fixture::fixture();
    w.resources.clear();
    w.units[0].cell = cell(9, 10);
    w.units[1].cell = cell(9, 12);
    w.buildings
        .push(building(BuildingKind::Farm, "farm", cell(14, 10), None));
    w.stockpile.wood = 100.0;
    w.stockpile.stone = 100.0;
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
    for case in 0..4 {
        let mut w = world();
        w.units[0].action = UnitAction::Move { to: cell(8, 10) };
        match case {
            0 => {
                w.buildings.pop();
            }
            1 => w.buildings.last_mut().unwrap().construction = Some(0.0),
            2 => w.stockpile.stone = 4.0,
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
    assert_eq!((w.stockpile.wood, w.stockpile.stone), (90.0, 95.0));
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
    assert_eq!((w.stockpile.wood, w.stockpile.stone), (90.0, 95.0));
    assert_eq!(cultivate(&mut w, 1), Err(CommandError::FieldNotDepleted));
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
    assert_eq!(w.stockpile.food, FIELD_FOOD);
    assert_eq!(w.resources[0].amount, 0.0);
    assert_eq!(w.units[0].action, UnitAction::Idle);
    for at in w.resources[0].footprint().cells() {
        assert!(!w.occupancy().is_free_for(at, None));
    }
    w.stockpile.wood = 0.0;
    let before = serde_json::to_string(&w).unwrap();
    assert_eq!(
        cultivate(&mut w, 1),
        Err(CommandError::InsufficientResources(ResourceKind::Wood))
    );
    assert_eq!(before, serde_json::to_string(&w).unwrap());
    w.stockpile.wood = 20.0;
    cultivate(&mut w, 1).unwrap();
    assert_eq!((w.stockpile.wood, w.stockpile.stone), (10.0, 90.0));
    assert_eq!(w.resources[0].amount, 0.0);
    run(&mut w, 20.0);
    assert!(w.resources[0].amount < FIELD_FOOD);
    assert_eq!(w.stockpile.food, FIELD_FOOD);
    assert!(matches!(w.units[0].action, UnitAction::Gather { .. }));
    run(&mut w, 200.0);
    assert_eq!(w.stockpile.food, FIELD_FOOD * 2.0);
    assert_eq!(w.resources[0].amount, 0.0);
    assert_eq!(w.resources[0].field.as_ref().unwrap().work, None);
    assert_eq!(w.units[0].action, UnitAction::Idle);
    assert_eq!((w.stockpile.wood, w.stockpile.stone), (10.0, 90.0));
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
    assert_eq!(w.stockpile.food, 20.0);
    assert!(w.units[0].cargo.is_none());
    run(&mut w, 200.0);
    assert_eq!(w.stockpile.food, FIELD_FOOD + 20.0);
    assert_eq!(w.resources[0].amount, 0.0);
    assert_eq!(w.resources[0].field.as_ref().unwrap().work, None);
    w.validate().unwrap();
}

#[test]
fn old_resource_saves_load_and_invalid_field_states_are_rejected() {
    let old = r#"{"id":"berries","kind":"food","cell":{"column":1,"row":1},"amount":30.0,"capacity":30.0}"#;
    let node: ResourceNode = serde_json::from_str(old).unwrap();
    assert!(node.field.is_none());
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
    assert_eq!(w.stockpile.food, FIELD_FOOD + 3.0);
    assert_eq!(w.resources[0].field.as_ref().unwrap().work, None);
    assert!(w.units.iter().all(|u| u.action == UnitAction::Idle));
}
