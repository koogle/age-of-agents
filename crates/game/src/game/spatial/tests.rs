use crate::game::{fixture, *};

fn plot(column: u16, row: u16, columns: u16, rows: u16) -> Footprint {
    Footprint {
        origin: CellCoordinate::new(column, row),
        columns,
        rows,
    }
}

#[test]
fn snapshot_and_world_agree_on_known_placement_geometry() {
    let mut world = fixture::fixture();
    world.buildings[0].origin = CellCoordinate::new(10, 10);
    world.units[0].cell = CellCoordinate::new(4, 4);
    world.units[0].step = Some(Step {
        to: CellCoordinate::new(5, 4),
        progress: 0.5,
    });
    world.units[0].action = UnitAction::Move {
        to: CellCoordinate::new(8, 4),
    };
    world.units[1].cell = CellCoordinate::new(7, 7);
    let node = |id: &str, column, amount, field| ResourceNode {
        id: id.into(),
        kind: ResourceKind::Food,
        cell: CellCoordinate::new(column, 20),
        amount,
        capacity: 120.0,
        field,
    };
    world.resources = vec![
        node("live", 20, 10.0, None),
        node("depleted", 25, 0.0, None),
        node("field", 30, 0.0, Some(FieldState { work: None })),
    ];
    world.terrain[1].biome = TerrainBiome::Water;
    world.explored_cells = world.terrain.iter().map(|t| t.coordinate()).collect();
    world.explored_cells.sort_unstable();
    world.validate().unwrap();
    let snapshot = world.snapshot();

    // Body, next step, destination reservation, building and live resource all block.
    // A depleted natural node releases space, but a depleted field retains all 3x3 cells.
    for (column, row, expected) in [
        (4, 4, false),
        (5, 4, false),
        (8, 4, false),
        (7, 7, false),
        (10, 10, false),
        (14, 14, false),
        (15, 14, true),
        (14, 15, true),
        (20, 20, false),
        (25, 20, true),
        (30, 20, false),
        (32, 22, false),
        (33, 22, true),
        (1, 0, false),
    ] {
        let footprint = plot(column, row, 1, 1);
        assert_eq!(
            world.footprint_is_free(footprint),
            expected,
            "world: {footprint:?}"
        );
        assert_eq!(
            snapshot.footprint_is_free(footprint),
            expected,
            "snapshot: {footprint:?}"
        );
    }
    for column in (0..=WORLD_COLUMNS).step_by(7) {
        for row in (0..=WORLD_ROWS).step_by(7) {
            for size in [1, 3, 5] {
                let footprint = plot(column, row, size, size);
                assert_eq!(
                    snapshot.footprint_is_free(footprint),
                    world.footprint_is_free(footprint),
                    "{footprint:?}"
                );
            }
        }
    }
    // Coast adjacency uses an edge, never just a diagonal corner.
    for (footprint, expected) in [(plot(1, 1, 3, 3), true), (plot(2, 1, 3, 3), false)] {
        assert_eq!(world.touches_sea(footprint), expected);
        assert_eq!(snapshot.touches_sea(footprint), expected);
    }
}

#[test]
fn previews_respect_hidden_terrain_and_do_not_decide_affordability() {
    let mut world = fixture::fixture();
    world.resources.clear();
    let footprint = plot(0, 0, 3, 3);
    world.stockpile.wood = 0.0;
    assert!(world.footprint_is_free(footprint));
    assert!(!world.snapshot().footprint_is_free(footprint));
    world.explored_cells = world.terrain.iter().map(|t| t.coordinate()).collect();
    world.explored_cells.sort_unstable();
    assert!(world.snapshot().footprint_is_free(footprint));
    assert!(!world.stockpile.affords(BuildingKind::House.cost()));
}

#[test]
fn placement_rejects_out_of_bounds_rectangles_before_iteration() {
    let world = fixture::fixture();
    let snapshot = world.snapshot();
    assert!(plot(WORLD_COLUMNS - 3, WORLD_ROWS - 3, 3, 3).fits_in(WORLD_COLUMNS, WORLD_ROWS));
    for footprint in [
        plot(WORLD_COLUMNS - 2, 0, 3, 3),
        plot(0, WORLD_ROWS - 2, 3, 3),
        plot(u16::MAX, u16::MAX, 3, 3),
    ] {
        assert!(!world.footprint_is_free(footprint));
        assert!(!snapshot.footprint_is_free(footprint));
    }
    let mut next = world.clone();
    assert_eq!(
        next.apply_command(Command::Build {
            unit_id: world.units[0].id.clone(),
            origin: CellCoordinate::new(u16::MAX, u16::MAX),
            kind: BuildingKind::House,
        }),
        Err(CommandError::InvalidBuildSite)
    );
    assert_eq!(next, world);
}
