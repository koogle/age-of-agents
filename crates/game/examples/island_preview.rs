//! Export a fully explored, paused island for terrain/browser inspection.
//! cargo run -p aoa-game --example island_preview -- 7 > /tmp/island.json
use aoa_game::{CellCoordinate, GameWorld};

fn main() {
    let seed = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "7".into())
        .parse()
        .expect("integer seed");
    let mut world = GameWorld::generate(seed);
    world.simulation_speed = 0.0;
    world.explored_cells = world
        .terrain
        .iter()
        .map(|cell| CellCoordinate::new(cell.column, cell.row))
        .collect();
    println!("{}", serde_json::to_string(&world.snapshot()).unwrap());
}
