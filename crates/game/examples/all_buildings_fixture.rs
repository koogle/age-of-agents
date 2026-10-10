//! Flat meadow with every catalog building completed, for shadow previews.
use aoa_game::*;
fn main() {
    let mut world = GameWorld::default();
    world.animals.clear();
    world.resources.clear();
    world.units.truncate(2);
    world.units[0].cell = CellCoordinate::new(20, 50);
    world.units[1].cell = CellCoordinate::new(24, 50);
    for t in &mut world.terrain {
        t.biome = TerrainBiome::Meadow;
        t.elevation = 0.3;
    }
    world.explored_cells = world
        .terrain
        .iter()
        .map(|t| CellCoordinate::new(t.column, t.row))
        .collect();
    world.explored_cells.sort();
    let kinds = [
        BuildingKind::TownCenter,
        BuildingKind::MiningCamp,
        BuildingKind::Farm,
        BuildingKind::LumberMill,
        BuildingKind::Smelter,
        BuildingKind::Kiln,
        BuildingKind::Weaver,
        BuildingKind::Kitchen,
        BuildingKind::Barracks,
        BuildingKind::Range,
        BuildingKind::Workshop,
        BuildingKind::Infirmary,
        BuildingKind::Watchtower,
        BuildingKind::Monument,
        BuildingKind::House,
        BuildingKind::Granary,
        BuildingKind::Dock,
        BuildingKind::Temple,
    ];
    world.buildings.clear();
    let (mut col, mut row) = (30u16, 20u16);
    for (i, kind) in kinds.iter().enumerate() {
        let (c, r) = kind.size();
        world.buildings.push(Building {
            id: format!("b{i}"),
            kind: *kind,
            origin: CellCoordinate::new(col, row),
            construction: None,
            job: None,
            queue: Vec::new(),
            next_queue_id: 0,
        });
        col += c + 3;
        if (i + 1) % 6 == 0 {
            col = 14;
            row += 9;
        }
        let _ = r;
    }
    println!("{}", serde_json::to_string(&world.snapshot()).unwrap());
}
