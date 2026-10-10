//! Export a paused run snapshot for archipelago review:
//! cargo run -p aoa-game --example archipelago_preview -- <seed> <charted islands>
//! With more than one charted island, a ship steers into the fog from the start island.
//! An optional third argument (`pride`, `snakes`, `vipers`, `found`, `carried`,
//! `home` or `won`) stages a review snapshot. `pride` and `snakes` chart two islands
//! and land villager-1 beside the second island's worldgen lion pride or sea
//! serpent; `vipers` charts the whole run and lands him beside a temple viper, so
//! those snapshots show real spawns. The others stage the artifact: as a review fixture it places villager-1 beside
//! the temple, claims the artifact, and for `home`/`won` places the bearer four or
//! three cells from the home town center (only three wins).
use aoa_game::{CellCoordinate, Command, GameWorld, TerrainBiome, TransportShip};

fn main() {
    let mut args = std::env::args().skip(1);
    let seed = args.next().map_or(7, |s| s.parse().expect("integer seed"));
    let mut charted: usize = args.next().map_or(1, |s| s.parse().expect("island count"));
    let stage = args.next();
    if matches!(stage.as_deref(), Some("pride" | "snakes")) {
        charted = 2; // the ship stops at the second island, so the camera opens there
    } else if stage.is_some() {
        charted = aoa_game::archipelago_plan(seed).len();
    }
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
        world.tick(0.1); // the ocean opens to the whole run once a ship exists
        // Steer into the fog toward each planned site in run order, as a player would.
        for site in aoa_game::archipelago_plan(seed).into_iter().skip(1) {
            if world.island_origins.len() >= charted {
                break;
            }
            if world.island_origins.contains(&site) {
                continue; // discovered on the way to an earlier site
            }
            let to = CellCoordinate::new(
                site.column + aoa_game::WORLD_COLUMNS / 2,
                site.row + aoa_game::WORLD_ROWS / 2,
            );
            world
                .apply_command(Command::Sail {
                    ship_id: "transport-preview".into(),
                    to,
                })
                .expect("open sea before discovery");
            while !world.ships[0].stopped() {
                world.tick(0.1);
            }
        }
    }
    if let Some(kind) = match stage.as_deref() {
        Some("pride") => Some(aoa_game::AnimalKind::Lion),
        Some("snakes") => Some(aoa_game::AnimalKind::SeaSerpent),
        Some("vipers") => Some(aoa_game::AnimalKind::Viper),
        _ => None,
    } {
        let island = world.island_origins.len() - 1;
        let lair = world
            .animals
            .iter()
            .find(|a| a.id.starts_with(&format!("animal-{island}-")) && a.kind == kind)
            .unwrap_or_else(|| panic!("island {island} spawns a {}", kind.name()))
            .home;
        // Sail to the water nearest the animal so the opening camera frames it,
        // then land the observer: placing him first would let the animal hunt him
        // during the crossing. Two cells is inside an ambusher's reveal radius.
        let shore = world
            .terrain
            .iter()
            .filter(|c| c.biome == TerrainBiome::Water)
            .min_by_key(|c| c.column.abs_diff(lair.column) + c.row.abs_diff(lair.row))
            .map(|c| CellCoordinate::new(c.column, c.row))
            .expect("the island has a coast");
        world
            .apply_command(Command::Sail {
                ship_id: "transport-preview".into(),
                to: shore,
            })
            .expect("open sea to the lair's coast");
        while !world.ships[0].stopped() {
            world.tick(0.1);
        }
        let (dx, dy) = if kind == aoa_game::AnimalKind::Lion {
            (-4, 3)
        } else {
            (-2, 0)
        };
        world.units[0].cell = CellCoordinate::new(
            (i32::from(lair.column) + dx) as u16,
            (i32::from(lair.row) + dy) as u16,
        );
        world.tick(0.1);
        world.validate().expect("a valid wildlife fixture");
    } else if let Some(stage) = stage.as_deref() {
        let temple = world
            .buildings
            .iter()
            .find(|b| b.id == aoa_game::TEMPLE_ID)
            .expect("the charted run has a temple")
            .origin;
        world.units[0].cell = CellCoordinate::new(temple.column - 1, temple.row + 2);
        world.tick(0.1);
        if stage != "found" {
            world
                .apply_command(Command::ClaimArtifact {
                    unit_ids: vec!["villager-1".into()],
                    building_id: aoa_game::TEMPLE_ID.into(),
                })
                .expect("the artifact waits in the temple");
            world.tick(0.1);
        }
        if stage == "home" || stage == "won" {
            // Four cells from the home town center is not yet home; three wins.
            let reach = if stage == "won" { 3 } else { 4 };
            let home = world.buildings[0].origin;
            world.units[0].cell = CellCoordinate::new(home.column - reach, home.row + 2);
            world.tick(0.1);
        }
        world.validate().expect("a valid staged run");
    }
    world.simulation_speed = 0.0;
    println!("{}", serde_json::to_string(&world.snapshot()).unwrap());
}
