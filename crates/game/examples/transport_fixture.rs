//! Reproducible coastal setup for manual/browser transport acceptance. Writes
//! an isolated save to stdout; never touches the configured game database.
use aoa_game::*;
fn main() {
    let mut world = GameWorld::default();
    world.stockpile.wood = 300.0;
    world.stockpile.timber = 100.0;
    world.stockpile.food = 100.0;
    for row in 2..WORLD_ROWS - 5 {
        for col in 2..WORLD_COLUMNS - 5 {
            let origin = CellCoordinate::new(col, row);
            let footprint = Footprint {
                origin,
                columns: 4,
                rows: 4,
            };
            if !footprint.cells().all(|c| {
                world.terrain[usize::from(c.row * WORLD_COLUMNS + c.column)]
                    .biome
                    .is_walkable()
            }) {
                continue;
            }
            let near: Vec<_> = world
                .terrain
                .iter()
                .filter(|t| t.column.abs_diff(col + 2) < 5 && t.row.abs_diff(row + 2) < 5)
                .collect();
            if !near.iter().any(|t| t.biome == TerrainBiome::Water) {
                continue;
            }
            let land: Vec<_> = near
                .iter()
                .filter(|t| {
                    t.biome.is_walkable()
                        && !footprint.contains(CellCoordinate::new(t.column, t.row))
                })
                .map(|t| CellCoordinate::new(t.column, t.row))
                .collect();
            for pair in land.windows(2) {
                let mut trial = world.clone();
                trial.units[0].cell = pair[0];
                trial.units[1].cell = pair[1];
                if trial.validate().is_err() {
                    continue;
                }
                if trial
                    .apply_command(Command::Build {
                        unit_id: trial.units[0].id.clone(),
                        origin,
                        kind: BuildingKind::Dock,
                    })
                    .is_err()
                {
                    continue;
                }
                for _ in 0..600 {
                    trial.tick(0.1);
                }
                if !trial.buildings.last().unwrap().is_complete() {
                    continue;
                }
                trial.explored_cells = trial
                    .terrain
                    .iter()
                    .map(|t| CellCoordinate::new(t.column, t.row))
                    .collect();
                trial.explored_cells.sort();
                trial.validate().unwrap();
                println!("{}", serde_json::to_string(&trial).unwrap());
                return;
            }
        }
    }
    panic!("no coastal fixture site");
}
