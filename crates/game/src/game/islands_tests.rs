use super::*;

fn expedition() -> GameWorld {
    let mut w = GameWorld::default();
    w.ships.push(TransportShip {
        id: "transport-test".into(),
        cell: CellCoordinate::new(0, 0),
        step: None,
        destination: None,
        heading: [1, 0],
        passengers: vec![w.units.remove(0)],
        home_dock_id: None,
    });
    w.discover_island();
    w.validate().unwrap();
    w
}
fn voyage(w: &mut GameWorld, id: u64) -> Result<(), CommandError> {
    w.apply_command(Command::Voyage {
        ship_id: "transport-test".into(),
        island_id: id,
    })
}

#[test]
fn discovery_is_deterministic_complementary_and_saved_without_replacing_home() {
    for seed in 0..8 {
        let mut a = GameWorld::generate(seed);
        let home = a.terrain.clone();
        a.discover_island();
        a.discover_island();
        a.discover_island();
        let mut b = GameWorld::generate(seed);
        for _ in 0..3 {
            b.discover_island();
        }
        assert_eq!(a, b);
        assert_eq!(a.terrain, home);
        for (island, expected) in a.islands.iter().zip([
            vec![ResourceKind::Iron, ResourceKind::Coal],
            vec![ResourceKind::Clay],
            vec![ResourceKind::Fiber],
        ]) {
            assert_ne!(island.terrain, home);
            assert!(island.explored_cells.is_empty());
            for node in &island.resources {
                assert!(
                    worldgen::grows_in(node.kind).contains(
                        &island.terrain
                            [usize::from(node.cell.row * WORLD_COLUMNS + node.cell.column)]
                        .biome
                    )
                );
            }
            for kind in &expected {
                assert!(island.resources.iter().any(|r| r.kind == *kind));
            }
            assert!(
                island
                    .resources
                    .iter()
                    .all(|r| STARTER_RESOURCES.contains(&r.kind) || expected.contains(&r.kind))
            );
        }
        let saved: GameWorld = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert_eq!(saved, a);
        saved.validate().unwrap();
    }
}

#[test]
fn voyage_lands_founders_with_shared_resources_and_preserves_settlements() {
    let mut w = expedition();
    w.stockpile.wood = 100.0;
    let home = w.clone();
    voyage(&mut w, 1).unwrap();
    assert_eq!(w.island_id, 1);
    assert_eq!(w.stockpile.wood, 100.0);
    assert!(w.buildings.is_empty());
    assert_eq!(w.ships[0].passengers[0], home.ships[0].passengers[0]);
    w.apply_command(Command::Disembark {
        ship_id: "transport-test".into(),
    })
    .unwrap();
    w.stockpile.wood -= 40.0; // Shared spending remains visible after a return voyage.
    assert_eq!(w.units.len(), 1);
    let destination_terrain = w.terrain.clone();
    let destination_fog = w.explored_cells.clone();
    voyage(&mut w, 0).unwrap();
    assert_eq!(w.terrain, home.terrain);
    assert_eq!(w.buildings, home.buildings);
    assert_eq!(w.units, home.units);
    assert_eq!(w.stockpile.wood, 60.0);
    assert!(
        home.explored_cells
            .iter()
            .all(|c| w.explored_cells.contains(c))
    );
    let mut restored: GameWorld =
        serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    voyage(&mut restored, 1).unwrap();
    assert_eq!(restored.terrain, destination_terrain);
    assert_eq!(restored.stockpile.wood, 60.0);
    assert_eq!(restored.units.len(), 1);
    assert!(
        destination_fog
            .iter()
            .all(|c| restored.explored_cells.contains(c))
    );
    restored.validate().unwrap();
}

#[test]
fn invalid_or_blocked_voyages_are_atomic_and_do_not_generate_maps() {
    let mut w = expedition();
    for id in [0, 12, u64::MAX] {
        let before = w.clone();
        assert!(voyage(&mut w, id).is_err());
        assert_eq!(w, before);
    }
    w.ships[0].destination = Some(CellCoordinate::new(1, 0));
    let before = w.clone();
    assert_eq!(voyage(&mut w, 2), Err(CommandError::ShipMustBeStopped));
    assert_eq!(w, before);
    w.ships[0].destination = None;
    for cell in &mut w.islands[0].terrain {
        cell.biome = TerrainBiome::Water;
    }
    w.islands[0].resources.clear();
    let before = w.clone();
    assert_eq!(voyage(&mut w, 1), Err(CommandError::ShoreBlocked));
    assert_eq!(w, before);
}

#[test]
fn old_saves_can_discover_and_corrupt_archives_are_rejected() {
    let w = expedition();
    let mut json = serde_json::to_value(&w).unwrap();
    json.as_object_mut().unwrap().remove("island_id");
    json.as_object_mut().unwrap().remove("islands");
    let mut old: GameWorld = serde_json::from_value(json).unwrap();
    voyage(&mut old, 1).unwrap();
    old.validate().unwrap();
    old.islands[0].terrain.pop();
    assert!(old.validate().is_err());
    let mut duplicate = w;
    duplicate.islands[0].id = 0;
    assert!(duplicate.validate().is_err());
}

#[test]
fn frontier_keeps_expanding_and_revisits_do_not_regenerate() {
    let mut w = expedition();
    for id in 1..=5 {
        voyage(&mut w, id).unwrap();
        assert_eq!(w.island_id, id);
        assert_eq!(w.islands.len() as u64, id);
        w.validate().unwrap();
    }
    let terrain = w.terrain.clone();
    voyage(&mut w, 0).unwrap();
    voyage(&mut w, 5).unwrap();
    assert_eq!(w.terrain, terrain);
    assert_eq!(w.islands.len(), 5);
}

#[test]
fn four_founders_can_land_and_build_from_shared_wood() {
    let mut w = expedition();
    w.stockpile.wood = 100.0;
    w.stockpile.food = 100.0;
    w.apply_command(Command::Research {
        building_id: "base-1".into(),
        technology: TechnologyKind::Forestry,
    })
    .unwrap();
    let template = w.ships[0].passengers[0].clone();
    for n in 3..=5 {
        let mut passenger = template.clone();
        passenger.id = format!("villager-{n}");
        w.ships[0].passengers.push(passenger);
    }
    w.next_unit_id = 6;
    voyage(&mut w, 1).unwrap();
    w.apply_command(Command::Disembark {
        ship_id: "transport-test".into(),
    })
    .unwrap();
    assert_eq!(w.units.len(), 4);
    let unit_id = w.units[0].id.clone();
    let mut sites = w
        .terrain
        .iter()
        .filter(|c| c.biome.is_walkable())
        .map(|c| c.coordinate())
        .collect::<Vec<_>>();
    sites.sort_by_key(|c| {
        c.column.abs_diff(w.units[0].cell.column) + c.row.abs_diff(w.units[0].cell.row)
    });
    assert!(sites.into_iter().any(|origin| {
        w.apply_command(Command::Build {
            unit_id: unit_id.clone(),
            origin,
            kind: BuildingKind::TownCenter,
        })
        .is_ok()
    }));
    for _ in 0..600 {
        w.tick(0.1);
    }
    assert!(w.buildings[0].is_complete());
    assert_eq!(w.stockpile.wood, 60.0);
    w.stockpile.food = 100.0;
    let before = w.clone();
    assert_eq!(
        w.apply_command(Command::Research {
            building_id: w.buildings[0].id.clone(),
            technology: TechnologyKind::Forestry
        }),
        Err(CommandError::TechnologyInProgress)
    );
    assert_eq!(w, before);
    // A deposit on the new island joins the same pool used back home.
    w.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Iron,
        amount: 7.0,
    });
    w.apply_command(Command::Deposit {
        unit_id: w.units[0].id.clone(),
        building_id: w.buildings[0].id.clone(),
    })
    .unwrap();
    for _ in 0..600 {
        w.tick(0.1);
    }
    assert_eq!(w.stockpile.iron, 7.0);
    voyage(&mut w, 0).unwrap();
    assert_eq!(w.stockpile.wood, 60.0);
    assert_eq!(w.stockpile.iron, 7.0);
    w.apply_command(Command::Produce {
        product: ProductKind::Villager,
        building_id: "base-1".into(),
    })
    .unwrap();
    assert_eq!(w.stockpile.food, 50.0);
    voyage(&mut w, 1).unwrap();
    assert_eq!(w.stockpile.food, 50.0);
    w.validate().unwrap();
}

#[test]
fn ocean_departure_does_not_require_a_specific_corner_to_be_unoccupied() {
    let mut w = expedition();
    let mut other = w.ships[0].clone();
    other.id = "another-ship".into();
    other.passengers.clear();
    w.ships[0].cell = CellCoordinate::new(1, 0);
    w.ships.push(other);
    voyage(&mut w, 1).unwrap();
    assert_eq!(w.islands[0].ships.len(), 1);
    w.validate().unwrap();
}
