use super::*;

/// A run with every island charted, so the temple exists.
fn charted() -> GameWorld {
    let mut world = GameWorld::default();
    while world.island_origins.len() < archipelago_plan(world.seed).len() {
        world.discover_island();
    }
    world
}

fn temple(world: &GameWorld) -> Building {
    world
        .buildings
        .iter()
        .find(|b| b.kind == BuildingKind::Temple)
        .cloned()
        .expect("the final island holds the temple")
}

fn claim(world: &mut GameWorld, ids: &[&str]) -> Result<(), CommandError> {
    world.apply_command(Command::ClaimArtifact {
        unit_ids: ids.iter().map(|id| id.to_string()).collect(),
        building_id: temple::TEMPLE_ID.into(),
    })
}

/// Put a unit on the cell just west of the temple, inside its open ring.
fn beside_temple(world: &mut GameWorld, unit: usize) {
    let origin = temple(world).origin;
    world.units[unit].cell = CellCoordinate::new(origin.column - 1, origin.row);
}

#[test]
fn only_the_final_island_gets_one_complete_temple_that_lights_no_fog() {
    let mut world = GameWorld::default();
    let plan = archipelago_plan(world.seed);
    while world.island_origins.len() < plan.len() - 1 {
        world.discover_island();
        assert!(
            world
                .buildings
                .iter()
                .all(|b| b.kind != BuildingKind::Temple)
        );
    }
    let seen = world.visible_cells();
    world.discover_island();
    let temple = temple(&world);
    assert!(temple.is_complete());
    assert_eq!(world.island_at(temple.origin), Some(plan.len() - 1));
    assert_eq!(
        world.visible_cells(),
        seen,
        "the sanctuary is not the player's"
    );
    assert!(!world.available_buildings().contains(&BuildingKind::Temple));
    // Unexplored, it stays out of snapshots.
    assert!(
        world
            .snapshot()
            .buildings
            .iter()
            .all(|b| b.building.kind != BuildingKind::Temple)
    );
    world.validate().unwrap();
}

#[test]
fn the_first_unit_to_arrive_takes_the_artifact_and_later_claims_are_rejected() {
    let mut world = charted();
    beside_temple(&mut world, 0);
    claim(&mut world, &["villager-1"]).unwrap();
    world.tick(0.1);
    assert_eq!(world.artifact_bearer.as_deref(), Some("villager-1"));
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.validate().unwrap();
    let before = world.clone();
    assert_eq!(
        claim(&mut world, &["villager-2"]),
        Err(CommandError::ArtifactUnavailable)
    );
    assert_eq!(world, before);
    // Only the temple holds the artifact.
    assert_eq!(
        world.apply_command(Command::ClaimArtifact {
            unit_ids: vec!["villager-2".into()],
            building_id: "base-1".into(),
        }),
        Err(CommandError::BuildingNotFound)
    );
}

#[test]
fn a_fallen_bearer_returns_the_artifact_and_a_passenger_keeps_it() {
    let mut world = charted();
    beside_temple(&mut world, 0);
    claim(&mut world, &["villager-1"]).unwrap();
    world.tick(0.1);
    // Aboard a ship the bearer still carries it.
    let unit = world.units.remove(0);
    world.ships.push(TransportShip {
        id: "transport-test".into(),
        cell: CellCoordinate::new(0, 0),
        step: None,
        destination: None,
        heading: [1, 0],
        passengers: vec![unit],
        cargo: Default::default(),
        home_dock_id: None,
    });
    world.tick(0.1);
    assert_eq!(world.artifact_bearer.as_deref(), Some("villager-1"));
    world.validate().unwrap();
    // Lost with its bearer, it returns to the temple.
    world.ships[0].passengers.clear();
    world.tick(0.1);
    assert_eq!(world.artifact_bearer, None);
    world.validate().unwrap();
}

#[test]
fn bringing_the_artifact_beside_the_home_town_center_wins_the_run() {
    let mut world = charted();
    beside_temple(&mut world, 0);
    claim(&mut world, &["villager-1"]).unwrap();
    world.tick(0.1);
    assert_eq!(world.scenario.outcome, ScenarioOutcome::Running);
    // Four cells west of the home town center is not yet home; three is.
    let home = world.buildings[0].origin;
    world.units[0].cell = CellCoordinate::new(home.column - 4, home.row);
    world.tick(0.1);
    assert_eq!(world.scenario.outcome, ScenarioOutcome::Running);
    world.units[0].cell = CellCoordinate::new(home.column - 3, home.row);
    world.tick(0.1);
    assert_eq!(world.scenario.outcome, ScenarioOutcome::Won);
    assert_eq!(world.scenario.objective_progress.completed, 1);
    let saved: GameWorld = serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
    assert_eq!(saved, world);
    world.validate().unwrap();
}

#[test]
fn every_run_places_a_temple_reachable_on_foot_from_its_coast() {
    for seed in 0..24 {
        let mut world = GameWorld::generate(seed);
        while world.island_origins.len() < archipelago_plan(seed).len() {
            world.discover_island();
        }
        let temple = temple(&world).footprint();
        let stride = usize::from(world.columns());
        let biome = |c: CellCoordinate| {
            world.terrain[usize::from(c.row) * stride + usize::from(c.column)].biome
        };
        // Flood the walkable land from the temple's open ring until it meets the sea.
        let start = CellCoordinate::new(temple.origin.column - 1, temple.origin.row);
        let mut seen = BTreeSet::from([start]);
        let mut frontier = vec![start];
        let mut coast = false;
        while let Some(cell) = frontier.pop() {
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let Some(next) =
                    crate::navigation::offset(cell, dx, dy, world.columns(), world.rows())
                else {
                    continue;
                };
                coast |= biome(next) == TerrainBiome::Water;
                if biome(next).is_walkable() && !temple.contains(next) && seen.insert(next) {
                    frontier.push(next);
                }
            }
        }
        assert!(
            coast,
            "seed {seed}: the temple cannot be reached from the sea"
        );
        world.validate().unwrap();
    }
}
