//! Isolated, flat road acceptance save on stdout. Never opens a database.
use aoa_game::*;
fn main() {
    let mut world = GameWorld::default();
    world.animals.clear();
    world.resources.clear();
    world.units.truncate(1);
    world.units[0].cell = CellCoordinate::new(29, 23);
    world.buildings[0].origin = CellCoordinate::new(28, 17);
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
    world.inventories[0].stone = 30.0;
    world.tick(0.1);
    world.validate().unwrap();
    println!("{}", serde_json::to_string(&world).unwrap());
}
