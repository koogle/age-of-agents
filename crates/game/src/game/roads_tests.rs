use super::*;
use crate::navigation::PathTree;

fn cell(x: u16, y: u16) -> CellCoordinate {
    CellCoordinate::new(x, y)
}
fn world() -> GameWorld {
    let mut w = fixture::fixture();
    w.resources.clear();
    w.units.truncate(1);
    w.units[0].cell = cell(10, 10);
    w.refresh_exploration();
    w
}
fn order(
    w: &mut GameWorld,
    kind: RoadKind,
    start: CellCoordinate,
    end: CellCoordinate,
) -> Result<(), CommandError> {
    w.apply_command(Command::BuildRoad {
        unit_id: w.units[0].id.clone(),
        kind,
        start,
        end,
    })
}
fn lay(
    w: &mut GameWorld,
    kind: RoadKind,
    start: CellCoordinate,
    end: CellCoordinate,
    work: Option<f64>,
) {
    w.roads
        .extend(
            road_line(start, end)
                .unwrap()
                .into_iter()
                .map(|cell| Road { cell, kind, work }),
        );
}

#[test]
fn roads_need_labour_and_charge_stone_once_across_stop_resume_and_reload() {
    for kind in [RoadKind::Dirt, RoadKind::Stone] {
        let mut w = world();
        w.inventories[0].stone = 10.0;
        order(&mut w, kind, cell(10, 11), cell(13, 11)).unwrap();
        assert_eq!(w.inventories[0].stone, 10.0 - 4.0 * kind.stone_per_cell());
        assert!(w.roads.iter().all(|r| r.work == Some(0.0)));
        w.tick(0.5);
        assert_eq!(w.roads[0].work, Some(0.5));
        w.apply_command(Command::Stop {
            unit_id: w.units[0].id.clone(),
        })
        .unwrap();
        let roads = w.roads.clone();
        w.tick(2.0);
        assert_eq!(roads, w.roads);
        let mut w: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
        order(&mut w, kind, cell(10, 11), cell(13, 11)).unwrap();
        for _ in 0..200 {
            w.tick(0.1);
        }
        assert_eq!(w.roads.len(), 4);
        assert!(w.roads.iter().all(|r| r.work.is_none()));
        assert_eq!(w.units[0].action, UnitAction::Idle);
        assert_eq!(w.inventories[0].stone, 10.0 - 4.0 * kind.stone_per_cell());
        w.validate().unwrap();
    }
}

#[test]
fn invalid_road_orders_are_atomic() {
    for (start, end, stone) in [
        (cell(10, 11), cell(12, 12), 10.0),
        (cell(10, 11), cell(13, 11), 1.0),
        (cell(119, 79), cell(119, 78), 10.0),
        (cell(120, 10), cell(121, 10), 10.0),
    ] {
        let mut w = world();
        w.inventories[0].stone = stone;
        let before = w.clone();
        assert!(order(&mut w, RoadKind::Stone, start, end).is_err());
        assert_eq!(w, before);
    }
    let mut w = world();
    let columns = usize::from(w.columns());
    w.terrain[11 * columns + 11].biome = TerrainBiome::Water;
    let before = w.clone();
    assert!(order(&mut w, RoadKind::Dirt, cell(10, 11), cell(13, 11)).is_err());
    assert_eq!(w, before);
}

#[test]
fn completed_roads_are_fifty_percent_faster_for_all_friendly_units_only() {
    for kind in [
        UnitKind::Villager,
        UnitKind::Guard,
        UnitKind::Archer,
        UnitKind::Healer,
        UnitKind::SiegeCart,
    ] {
        for road_kind in [RoadKind::Dirt, RoadKind::Stone] {
            let mut w = world();
            w.units[0].kind = kind;
            lay(&mut w, road_kind, cell(10, 10), cell(14, 10), None);
            w.travel(0, Goal::Cell(cell(14, 10)), 0.2);
            assert!((w.units[0].position().x - 11.4).abs() < 1e-9);
            w.validate().unwrap();
        }
    }
    let mut w = world();
    lay(
        &mut w,
        RoadKind::Dirt,
        cell(10, 10),
        cell(14, 10),
        Some(0.0),
    );
    w.travel(0, Goal::Cell(cell(14, 10)), 0.2);
    assert!((w.units[0].position().x - 11.1).abs() < 1e-9);
    for kind in [AnimalKind::Wolf, AnimalKind::Bear] {
        let mut plain = world();
        plain.animals.push(Animal {
            id: "beast".into(),
            kind,
            home: cell(14, 10),
            cell: cell(14, 10),
            step: Some(Step {
                to: cell(13, 10),
                progress: 0.0,
            }),
            health: kind.max_health(),
            attack_seconds: 0.0,
            heading: [-1, 0],
        });
        let mut paved = plain.clone();
        lay(
            &mut paved,
            RoadKind::Stone,
            cell(10, 10),
            cell(14, 10),
            None,
        );
        for _ in 0..10 {
            plain.tick(0.1);
            paved.tick(0.1);
            assert_eq!(plain.animals, paved.animals);
        }
    }
}

#[test]
fn weighted_routes_take_useful_detours_but_ignore_unfinished_roads_and_distant_roads() {
    let mut w = world();
    lay(&mut w, RoadKind::Dirt, cell(10, 12), cell(30, 12), None);
    let occupancy = w.occupancy();
    let path = w.static_paths(0, &occupancy).path_to(cell(30, 10));
    assert!(path.iter().any(|c| c.row == 12));
    assert!(
        !w.static_paths(0, &occupancy)
            .path_to(cell(12, 10))
            .iter()
            .any(|c| c.row == 12)
    );
    for r in &mut w.roads {
        r.work = Some(0.0);
    }
    assert!(
        w.static_paths(0, &occupancy)
            .path_to(cell(30, 10))
            .iter()
            .all(|c| c.row == 10)
    );
}

#[test]
fn bounded_weighted_routes_match_complete_search_and_never_cut_corners() {
    for walls in 0_u16..512 {
        let clear = |c: CellCoordinate| walls & (1 << (c.row * 3 + c.column)) == 0;
        let weight = |c: CellCoordinate| if c.row == 1 { 2 } else { 3 };
        let all = PathTree::search_weighted(3, 3, cell(0, 0), clear, weight);
        let goals = [cell(2, 2), cell(0, 2)];
        let expected = all
            .nearest(goals)
            .map(|g| (all.cost(g).unwrap(), all.path_to(g)));
        assert_eq!(
            PathTree::nearest_weighted(3, 3, cell(0, 0), &goals, clear, weight),
            expected
        );
        if let Some((_, path)) = expected {
            let mut prev = cell(0, 0);
            for next in path {
                assert!(clear(next));
                if prev.column != next.column && prev.row != next.row {
                    assert!(
                        clear(cell(prev.column, next.row)) && clear(cell(next.column, prev.row))
                    );
                }
                prev = next;
            }
        }
    }
}

#[test]
fn road_bonus_starts_and_ends_at_the_cell_boundary_and_diagonals_take_longer() {
    let mut w = world();
    lay(&mut w, RoadKind::Dirt, cell(11, 10), cell(11, 10), None);
    w.units[0].step = Some(Step {
        to: cell(11, 10),
        progress: 0.0,
    });
    w.travel(0, Goal::Cell(cell(11, 10)), 0.1);
    assert!((w.units[0].step.unwrap().progress - 0.3).abs() < 1e-9);
    w.travel(0, Goal::Cell(cell(11, 10)), 0.1);
    assert!((w.units[0].step.unwrap().progress - 0.65).abs() < 1e-9);
    w.travel(0, Goal::Cell(cell(11, 10)), 1.0);
    w.units[0].step = Some(Step {
        to: cell(12, 10),
        progress: 0.0,
    });
    w.travel(0, Goal::Cell(cell(12, 10)), 0.2);
    assert!((w.units[0].step.unwrap().progress - (0.5 + (0.2 - 0.5 / 4.5) * 3.0)).abs() < 1e-9);
    let mut w = world();
    lay(&mut w, RoadKind::Dirt, cell(10, 10), cell(11, 10), None);
    lay(&mut w, RoadKind::Dirt, cell(10, 11), cell(11, 11), None);
    w.travel(0, Goal::Cell(cell(11, 11)), 0.1);
    assert!((w.units[0].step.unwrap().progress - 0.45 / 2_f64.sqrt()).abs() < 1e-9);
}

#[test]
fn road_workers_unload_first_and_multiple_builders_do_not_duplicate_completion() {
    let mut w = world();
    w.units[0].cell = cell(29, 22);
    w.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Wood,
        amount: 3.0,
    });
    w.refresh_exploration();
    order(&mut w, RoadKind::Dirt, cell(29, 23), cell(30, 23)).unwrap();
    for _ in 0..300 {
        w.tick(0.1);
    }
    assert!(w.units[0].cargo.is_none());
    assert_eq!(w.inventories[0].wood, 3.0);
    assert!(w.roads.iter().all(|r| r.work.is_none()));
    let mut w = world();
    order(&mut w, RoadKind::Dirt, cell(10, 11), cell(12, 11)).unwrap();
    let mut helper = w.units[0].clone();
    helper.id = "helper".into();
    helper.cell = cell(11, 10);
    w.units.push(helper);
    for _ in 0..100 {
        w.tick(0.1);
    }
    assert_eq!(w.roads.len(), 3);
    assert!(w.roads.iter().all(|r| r.work.is_none()));
    assert!(w.units.iter().all(|u| u.action == UnitAction::Idle));
}

#[test]
fn delivery_selects_the_fastest_storage_route_with_roads() {
    let mut w = world();
    w.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Food,
        amount: 10.0,
    });
    w.buildings
        .push(building(BuildingKind::Granary, "near", cell(9, 17), None));
    w.buildings
        .push(building(BuildingKind::Granary, "far", cell(19, 9), None));
    assert_eq!(w.nearest_drop_site(0).unwrap().origin, cell(9, 17));
    lay(&mut w, RoadKind::Stone, cell(10, 10), cell(18, 10), None);
    assert_eq!(w.nearest_drop_site(0).unwrap().origin, cell(19, 9));
}

#[test]
fn corrupt_road_saves_fail_validation_without_panics() {
    let mut w = world();
    lay(&mut w, RoadKind::Dirt, cell(10, 10), cell(12, 10), None);
    let duplicate = w.roads[0].clone();
    w.roads.push(duplicate);
    assert!(w.validate().is_err());
    w.roads.pop();
    w.roads[0].work = Some(f64::NAN);
    assert!(w.validate().is_err());
    w.roads[0].work = None;
    w.terrain.remove(0);
    assert!(w.validate().is_err());
}

#[test]
fn crossing_roads_share_existing_surfaces_without_charging_twice() {
    let mut w = world();
    lay(&mut w, RoadKind::Dirt, cell(11, 9), cell(11, 13), None);
    w.inventories[0].stone = 3.0;
    order(&mut w, RoadKind::Stone, cell(10, 11), cell(13, 11)).unwrap();
    assert_eq!(w.inventories[0].stone, 0.0);
    assert_eq!(w.roads.len(), 8);
    for _ in 0..150 {
        w.tick(0.1);
    }
    assert!(w.roads.iter().all(|r| r.work.is_none()));
    assert_eq!(
        w.roads
            .iter()
            .find(|r| r.cell == cell(11, 11))
            .unwrap()
            .kind,
        RoadKind::Dirt
    );
}

#[test]
fn clicking_interrupted_road_resumes_connected_work_across_completed_bends() {
    for kind in [RoadKind::Dirt, RoadKind::Stone] {
        let mut w = world();
        w.inventories[0].stone = 10.0;
        order(&mut w, kind, cell(10, 11), cell(13, 11)).unwrap();
        w.tick(2.5);
        assert!(w.roads[0].work.is_none());
        assert!(w.roads[1].work.is_some_and(|work| work > 0.0));
        w.apply_command(Command::Stop {
            unit_id: w.units[0].id.clone(),
        })
        .unwrap();
        lay(
            &mut w,
            RoadKind::Dirt,
            cell(13, 12),
            cell(13, 13),
            Some(0.5),
        );
        w.roads
            .iter_mut()
            .find(|r| r.cell == cell(13, 12))
            .unwrap()
            .work = None;
        // Corner contact and a separate nearby road do not extend the assignment.
        lay(&mut w, kind, cell(14, 14), cell(15, 14), Some(0.25));
        lay(&mut w, kind, cell(10, 15), cell(12, 15), Some(0.0));
        let paid_stone = w.inventories[0].stone;
        let progress = w.roads.clone();
        order(&mut w, kind, cell(12, 11), cell(12, 11)).unwrap();
        assert_eq!(w.roads, progress);
        // The clicked cell is worked first; completion on either side is retained.
        let UnitAction::BuildRoad { cells } = &w.units[0].action else {
            panic!("expected road work")
        };
        assert_eq!(cells[0], cell(12, 11));
        assert_eq!(cells.len(), 4);
        w.validate().unwrap();
        let mut w: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
        w.validate().unwrap();
        for _ in 0..300 {
            w.tick(0.1);
        }
        assert!(w.roads[..6].iter().all(|r| r.work.is_none()));
        assert!(w.roads[6..].iter().all(|r| r.work.is_some()));
        assert_eq!(w.inventories[0].stone, paid_stone);
        assert_eq!(w.units[0].action, UnitAction::Idle);
        w.validate().unwrap();
    }
}

#[test]
fn blocked_connected_road_resume_rejects_without_changing_progress_or_order() {
    let mut w = world();
    lay(
        &mut w,
        RoadKind::Stone,
        cell(10, 11),
        cell(13, 11),
        Some(0.5),
    );
    w.buildings.push(building(
        BuildingKind::House,
        "over-road",
        cell(13, 11),
        None,
    ));
    w.validate().unwrap();
    let before = w.clone();
    assert_eq!(
        order(&mut w, RoadKind::Stone, cell(10, 11), cell(10, 11)),
        Err(CommandError::InvalidBuildSite)
    );
    assert_eq!(w, before);
}

#[test]
fn road_resume_keeps_existing_work_beyond_current_sight() {
    let mut w = world();
    w.buildings[0].origin = cell(70, 60);
    lay(
        &mut w,
        RoadKind::Dirt,
        cell(10, 11),
        cell(23, 11),
        Some(0.0),
    );
    assert!(!w.visible_cells().contains(&cell(23, 11)));
    order(&mut w, RoadKind::Dirt, cell(10, 11), cell(10, 11)).unwrap();
    for _ in 0..500 {
        w.tick(0.1);
    }
    assert!(w.roads.iter().all(|r| r.work.is_none()));
    assert_eq!(w.units[0].action, UnitAction::Idle);
    w.validate().unwrap();
}
