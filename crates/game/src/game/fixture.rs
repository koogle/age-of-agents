//! The fixed all-land map the movement, gathering and soundness tests run on:
//! eight Voronoi biome regions, clustered resources, the town center at
//! (28, 17) and villagers beside the south edge of the town center. Mechanics tests use it so
//! they do not depend on how islands are generated.

use super::*;

const STARTING_TOWN_CENTER: CellCoordinate = CellCoordinate::new(28, 17);

pub(super) fn fixture() -> GameWorld {
    let terrain = generate_terrain();
    let villager = |number: u64, column, row| Unit {
        id: format!("villager-{number}"),
        kind: UnitKind::Villager,
        cell: CellCoordinate::new(column, row),
        step: None,
        action: UnitAction::Idle,
        cargo: None,
    };
    let mut world = GameWorld::generate(DEFAULT_SEED);
    world.terrain = terrain.clone();
    let south = STARTING_TOWN_CENTER.row + BuildingKind::TownCenter.size().1;
    world.units = vec![villager(1, 29, south), villager(2, 31, south)];
    world.resources = generate_resources(&terrain);
    world.buildings = vec![town_center("base-1", STARTING_TOWN_CENTER, None)];
    world.explored_cells.clear();
    world.refresh_exploration();
    world
}

fn generate_terrain() -> Vec<TerrainCell> {
    const SITES: [(u16, u16, TerrainBiome); 8] = [
        (6, 6, TerrainBiome::Meadow),
        (22, 4, TerrainBiome::Forest),
        (42, 6, TerrainBiome::Prairie),
        (54, 14, TerrainBiome::Highland),
        (8, 28, TerrainBiome::Wetland),
        (24, 34, TerrainBiome::Scrubland),
        (40, 26, TerrainBiome::Heath),
        (54, 34, TerrainBiome::Clayland),
    ];

    let mut terrain = Vec::with_capacity(usize::from(WORLD_COLUMNS * WORLD_ROWS));
    for row in 0..WORLD_ROWS {
        for column in 0..WORLD_COLUMNS {
            let (_, _, biome) = SITES
                .iter()
                .min_by_key(|(site_column, site_row, _)| {
                    let dx = i32::from(column) - i32::from(*site_column);
                    let dy = i32::from(row) - i32::from(*site_row);
                    dx * dx + dy * dy
                })
                .expect("the fixed Voronoi map has sites");
            terrain.push(TerrainCell {
                column,
                row,
                biome: *biome,
                elevation: 0.3,
            });
        }
    }
    terrain
}

pub(super) fn compatible_biomes(kind: ResourceKind) -> &'static [TerrainBiome] {
    match kind {
        ResourceKind::Wood => &[TerrainBiome::Forest, TerrainBiome::Heath],
        ResourceKind::Food => &[TerrainBiome::Meadow, TerrainBiome::Prairie],
        ResourceKind::Stone => &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        ResourceKind::Gold => &[TerrainBiome::Highland],
        ResourceKind::Iron => &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        ResourceKind::Clay => &[TerrainBiome::Clayland, TerrainBiome::Wetland],
        ResourceKind::Fiber => &[TerrainBiome::Wetland, TerrainBiome::Prairie],
        ResourceKind::Coal
        | ResourceKind::Timber
        | ResourceKind::Steel
        | ResourceKind::Bricks
        | ResourceKind::Cloth
        | ResourceKind::Rations => &[],
    }
}

/// Resource kinds and how they cluster: (kind, id prefix, clusters, nodes per
/// cluster, amount per node, preferred distance from the base in cells).
/// Woodlines stretch east-west; the rest are clumps.
const RESOURCE_CLUSTERS: [(ResourceKind, &str, usize, usize, f64, f64); 7] = [
    (ResourceKind::Wood, "tree", 3, 10, 30.0, 11.0),
    (ResourceKind::Food, "berries", 2, 5, 30.0, 8.0),
    (ResourceKind::Stone, "stone", 2, 4, 40.0, 14.0),
    (ResourceKind::Gold, "gold", 1, 4, 40.0, 15.0),
    (ResourceKind::Iron, "iron", 1, 4, 40.0, 16.0),
    (ResourceKind::Clay, "clay", 2, 3, 40.0, 17.0),
    (ResourceKind::Fiber, "fiber", 2, 3, 30.0, 17.0),
];

fn generate_resources(terrain: &[TerrainCell]) -> Vec<ResourceNode> {
    // Clusters ring the town center's neighbourhood, not the map's middle.
    let base = Position { x: 30.0, y: 20.0 };
    let squared = |a: CellCoordinate, b: CellCoordinate, stretch: i32| {
        let dx = i32::from(a.column) - i32::from(b.column);
        let dy = i32::from(a.row) - i32::from(b.row);
        dx * dx + stretch * dy * dy
    };

    let mut resources: Vec<ResourceNode> = Vec::new();
    for (kind, prefix, clusters, size, amount, preferred) in RESOURCE_CLUSTERS {
        let stretch = if kind == ResourceKind::Wood { 4 } else { 1 };
        let mut number = 0;
        for _ in 0..clusters {
            // A cell is free for this cluster when it suits the kind, keeps the
            // base clear, and stays well away from every earlier cluster.
            let taken = resources.len();
            let free = |cell: &&TerrainCell, resources: &[ResourceNode]| {
                let center = cell.coordinate().center();
                compatible_biomes(kind).contains(&cell.biome)
                    && center.distance(base) >= STARTING_BASE_RESOURCE_CLEARANCE
                    && resources[..taken].iter().all(|resource| {
                        resource.cell.center().distance(center) + f64::EPSILON
                            >= RESOURCE_CLUSTER_SEPARATION
                    })
                    && resources[taken..]
                        .iter()
                        .all(|resource| resource.cell != cell.coordinate())
            };
            let site = terrain
                .iter()
                .filter(|cell| free(cell, &resources))
                .min_by_key(|cell| {
                    let off = (cell.coordinate().center().distance(base) - preferred).abs();
                    ((off * 16.0).round() as i64, cell.row, cell.column)
                })
                .expect("fixed terrain has room for every resource cluster")
                .coordinate();
            for _ in 0..size {
                let cell = terrain
                    .iter()
                    .filter(|cell| free(cell, &resources))
                    .min_by_key(|cell| {
                        (
                            squared(cell.coordinate(), site, stretch),
                            cell.row,
                            cell.column,
                        )
                    })
                    .expect("fixed terrain has room for every resource node")
                    .coordinate();
                number += 1;
                resources.push(ResourceNode {
                    field: None,
                    id: format!("{prefix}-{number}"),
                    kind,
                    cell,
                    amount,
                    capacity: amount,
                });
            }
        }
    }
    resources
}
