//! Export a paused run snapshot for archipelago review:
//! cargo run -p aoa-game --example archipelago_preview -- <seed> <charted islands>
//! With more than one charted island, a ship explores from the start island.
use aoa_game::{CellCoordinate, Command, GameWorld, TerrainBiome, TransportShip};

fn main() {
    let mut args = std::env::args().skip(1);
    let seed = args.next().map_or(7, |s| s.parse().expect("integer seed"));
    let charted: usize = args.next().map_or(1, |s| s.parse().expect("island count"));
    let mut world = GameWorld::generate(seed);
    world.tick(0.1);
    if charted > 1 {
        let base = world.buildings[0].origin;
        let water = world
            .terrain
            .iter()
            .filter(|c| c.biome == TerrainBiome::Water)
            .min_by_key(|c| c.column.abs_diff(base.column) + c.row.abs_diff(base.row))
            .map(|c| CellCoordinate::new(c.column, c.row))
            .expect("the start island has a coast");
        world.ships.push(TransportShip {
            id: "transport-preview".into(),
            cell: water,
            step: None,
            destination: None,
            heading: [1, 0],
            passengers: Vec::new(),
            cargo: Default::default(),
            home_dock_id: None,
        });
        while world.island_origins.len() < charted {
            let island_id = world.island_origins.len() as u64;
            world
                .apply_command(Command::Voyage {
                    ship_id: "transport-preview".into(),
                    island_id,
                })
                .expect("an uncharted island remains");
            while !world.ships[0].stopped() {
                world.tick(0.1);
            }
        }
    }
    world.simulation_speed = 0.0;
    println!("{}", serde_json::to_string(&world.snapshot()).unwrap());
}
