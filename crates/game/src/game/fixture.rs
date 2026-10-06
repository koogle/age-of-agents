//! Flat mechanics fixture with fixed resource patches; real terrain generation
//! is covered by worldgen tests, not by a second generator inside the test suite.
use super::*;

pub(super) fn fixture() -> GameWorld {
    let mut world = GameWorld::generate(DEFAULT_SEED);
    world.animals.clear();
    world.economy_rules = EconomyRules::Unrestricted;
    for cell in &mut world.terrain {
        cell.biome = TerrainBiome::Meadow;
        cell.elevation = 0.3;
    }
    world.units = [29, 31]
        .into_iter()
        .enumerate()
        .map(|(index, column)| Unit {
            id: format!("villager-{}", index + 1),
            health: UNIT_HEALTH,
            kind: UnitKind::Villager,
            cell: CellCoordinate::new(column, 22),
            step: None,
            action: UnitAction::Idle,
            cargo: None,
        })
        .collect();
    world.buildings = vec![town_center("base-1", CellCoordinate::new(28, 17), None)];
    world.resources = PATCHES
        .iter()
        .flat_map(|&(prefix, kind, amount, cells)| {
            cells
                .iter()
                .enumerate()
                .map(move |(index, &(column, row))| ResourceNode {
                    id: format!("{prefix}-{}", index + 1),
                    kind,
                    cell: CellCoordinate::new(column, row),
                    amount,
                    capacity: amount,
                    field: None,
                })
        })
        .collect();
    world.explored_cells.clear();
    world.refresh_exploration();
    world
}

// Preserve patch locations/order so mechanics cases keep their original geometry.
type Patch = (&'static str, ResourceKind, f64, &'static [(u16, u16)]);
const PATCHES: &[Patch] = &[
    (
        "tree",
        ResourceKind::Wood,
        30.0,
        &[
            (24, 10),
            (23, 10),
            (25, 10),
            (24, 9),
            (22, 10),
            (26, 10),
            (24, 11),
            (23, 9),
            (25, 9),
            (23, 11),
            (39, 25),
            (38, 25),
            (40, 25),
            (39, 24),
            (37, 25),
            (41, 25),
            (39, 26),
            (38, 24),
            (40, 24),
            (38, 26),
            (19, 16),
            (18, 16),
            (20, 16),
            (19, 15),
            (17, 16),
            (21, 16),
            (19, 17),
            (18, 15),
            (20, 15),
            (18, 17),
        ],
    ),
    (
        "berries",
        ResourceKind::Food,
        30.0,
        &[
            (33, 12),
            (33, 11),
            (34, 12),
            (32, 11),
            (34, 11),
            (38, 15),
            (39, 15),
            (39, 14),
            (40, 15),
            (39, 13),
        ],
    ),
    (
        "stone",
        ResourceKind::Stone,
        40.0,
        &[
            (23, 32),
            (23, 31),
            (22, 32),
            (24, 32),
            (33, 33),
            (33, 32),
            (32, 33),
            (33, 34),
        ],
    ),
    (
        "gold",
        ResourceKind::Gold,
        40.0,
        &[(45, 17), (45, 16), (46, 17), (46, 16)],
    ),
    (
        "iron",
        ResourceKind::Iron,
        40.0,
        &[(17, 29), (18, 29), (17, 30), (18, 28)],
    ),
    (
        "clay",
        ResourceKind::Clay,
        40.0,
        &[(13, 23), (13, 22), (12, 23), (12, 17), (11, 17), (10, 17)],
    ),
    (
        "fiber",
        ResourceKind::Fiber,
        30.0,
        &[(41, 7), (41, 6), (40, 7), (33, 3), (33, 2), (34, 3)],
    ),
];
