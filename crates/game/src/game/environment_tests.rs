use super::*;

#[test]
fn drought_warns_escalates_and_recovers_at_exact_boundaries() {
    for (time, phase, cycle, remaining, duration) in [
        (0.0, EnvironmentPhase::Calm, 1, 300.0, 60.0),
        (299.0, EnvironmentPhase::Calm, 1, 1.0, 60.0),
        (300.0, EnvironmentPhase::DroughtWarning, 1, 60.0, 60.0),
        (359.0, EnvironmentPhase::DroughtWarning, 1, 1.0, 60.0),
        (360.0, EnvironmentPhase::Drought, 1, 60.0, 60.0),
        (419.0, EnvironmentPhase::Drought, 1, 1.0, 60.0),
        (420.0, EnvironmentPhase::Calm, 2, 300.0, 90.0),
        (780.0, EnvironmentPhase::Drought, 2, 90.0, 90.0),
        (870.0, EnvironmentPhase::Calm, 3, 300.0, 120.0),
        (1230.0, EnvironmentPhase::Drought, 3, 120.0, 120.0),
        (1350.0, EnvironmentPhase::Calm, 4, 300.0, 120.0),
        (1710.0, EnvironmentPhase::Drought, 4, 120.0, 120.0),
    ] {
        let view = EnvironmentView::at(time);
        assert_eq!(
            (
                view.phase,
                view.cycle,
                view.remaining_seconds,
                view.drought_seconds
            ),
            (phase, cycle, remaining, duration)
        );
    }
}

#[test]
fn clock_obeys_pause_speed_reset_and_rejects_invalid_time() {
    let mut world = fixture::fixture();
    world.environment_seconds = 350.0;
    world.simulation_speed = 0.0;
    let paused = world.clone();
    world.tick(1.0);
    assert_eq!(world, paused);
    world.simulation_speed = 2.0;
    world.tick(5.0);
    assert_eq!(world.environment_seconds, 360.0);
    assert_eq!(
        world.snapshot().environment.phase,
        EnvironmentPhase::Drought
    );
    let before = world.clone();
    for dt in [f64::NAN, f64::INFINITY, -1.0, 0.0, f64::MAX] {
        world.tick(dt);
        assert_eq!(world, before);
    }
    assert_eq!(GameWorld::generate(123).environment_seconds, 0.0);
    for time in [-1.0, f64::NAN, f64::INFINITY] {
        world.environment_seconds = time;
        assert!(world.validate().is_err());
    }
}

#[test]
fn drought_only_slows_food_preserving_cargo_orders_and_recovery() {
    for kind in [ResourceKind::Food, ResourceKind::Wood, ResourceKind::Stone] {
        let mut world = fixture::fixture();
        world.resources.clear();
        world.units[0].cell = CellCoordinate::new(10, 10);
        world.resources.push(ResourceNode {
            id: "node".into(),
            kind,
            cell: CellCoordinate::new(11, 10),
            amount: 100.0,
            capacity: 100.0,
            field: None,
        });
        world.units[0].action = UnitAction::Gather {
            resource_id: "node".into(),
            phase: GatherPhase::Gathering,
        };
        let order = world.units[0].action.clone();
        let stock = world.stockpile.clone();
        world.environment_seconds = 359.0;
        world.tick(1.0);
        assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 2.0);
        world.tick(1.0);
        let expected = if kind == ResourceKind::Food { 3.0 } else { 4.0 };
        assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, expected);
        assert_eq!(world.units[0].action, order);
        assert_eq!(world.stockpile, stock);
        world.environment_seconds = 420.0;
        world.tick(1.0);
        assert_eq!(
            world.units[0].cargo.as_ref().unwrap().amount,
            expected + 2.0
        );
        assert_eq!(world.resources[0].amount, 100.0 - expected - 2.0);
        world.validate().unwrap();
    }
}

#[test]
fn reload_keeps_warning_active_drought_and_future_schedule() {
    for time in [330.0, 390.0, 810.0, 1300.0] {
        let mut world = fixture::fixture();
        world.environment_seconds = time;
        let json = serde_json::to_string(&world).unwrap();
        let mut restored: GameWorld = serde_json::from_str(&json).unwrap();
        restored.validate().unwrap();
        assert_eq!(
            restored.snapshot().environment,
            world.snapshot().environment
        );
        for _ in 0..5 {
            world.tick(0.1);
            restored.tick(0.1);
        }
        assert_eq!(restored, world);
    }
}

#[test]
fn drought_scales_field_harvest_with_farm_and_research_bonuses() {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.units[0].cell = CellCoordinate::new(10, 10);
    world.buildings.push(building(
        BuildingKind::Farm,
        "farm",
        CellCoordinate::new(15, 10),
        None,
    ));
    world
        .researched_technologies
        .push(TechnologyKind::Agriculture);
    world.resources.push(ResourceNode {
        id: "field".into(),
        kind: ResourceKind::Food,
        cell: CellCoordinate::new(11, 10),
        amount: FIELD_FOOD,
        capacity: FIELD_FOOD,
        field: Some(FieldState { work: None }),
    });
    world.units[0].action = UnitAction::Gather {
        resource_id: "field".into(),
        phase: GatherPhase::Gathering,
    };
    world.environment_seconds = 360.0;
    world.tick(1.0);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 1.5);
    assert_eq!(world.resources[0].amount, FIELD_FOOD - 1.5);
    assert_eq!(world.resources[0].field.as_ref().unwrap().work, None);
    assert_eq!(world.units[1].action, UnitAction::Idle);
    world.validate().unwrap();
}
