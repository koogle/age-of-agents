//! Isolated, deterministic browser acceptance fixture; print a full current save.
use aoa_game::*;
fn fixture() -> GameWorld {
    let mut world = GameWorld::default();
    world.animals.clear();
    world.resources.clear();
    world.ships.clear();
    for tile in &mut world.terrain {
        tile.biome = if tile.column == 24 && tile.row != 30 {
            TerrainBiome::River
        } else {
            TerrainBiome::Meadow
        };
        tile.elevation = 0.3;
    }
    world.buildings[0].origin = CellCoordinate::new(28, 17);
    let mut farm = world.buildings[0].clone();
    farm.id = "water-test-farm".into();
    farm.kind = BuildingKind::Farm;
    farm.origin = CellCoordinate::new(34, 22);
    world.buildings.push(farm);
    world.units[0].cell = CellCoordinate::new(26, 22);
    world.units[1].cell = CellCoordinate::new(33, 24);
    world.resources.push(ResourceNode {
        id: "water-test".into(),
        kind: ResourceKind::Water,
        cell: CellCoordinate::new(25, 22),
        amount: 120.0,
        capacity: 120.0,
        field: None,
    });
    world.inventories[0].wood = 100.0;
    world.inventories[0].stone = 100.0;
    world.simulation_speed = 2.0;
    world.explored_cells = world
        .terrain
        .iter()
        .map(|t| CellCoordinate::new(t.column, t.row))
        .collect();
    world.explored_cells.sort();
    world.validate().unwrap();
    world
}

fn main() {
    use std::io::{BufRead, Write};
    let mut world = fixture();
    if !std::env::args().any(|arg| arg == "--serve") {
        println!("{}", serde_json::to_string(&world).unwrap());
        return;
    }
    for line in std::io::stdin().lock().lines() {
        let value: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let error = value.get("command").and_then(|command| {
            world
                .apply_command(serde_json::from_value(command.clone()).unwrap())
                .err()
                .map(|e| format!("{e:?}"))
        });
        for _ in 0..value["ticks"].as_u64().unwrap_or(1) {
            world.tick(0.1);
            world.validate().unwrap();
        }
        println!(
            "{}",
            serde_json::json!({"error": error, "world": world.snapshot()})
        );
        std::io::stdout().flush().unwrap();
    }
}
