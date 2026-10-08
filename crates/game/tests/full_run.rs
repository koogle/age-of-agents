//! A whole run played only through player commands, as fast as the simulation
//! allows: gather, build a lumber mill and a dock, launch a ship, steer into the
//! fog to the temple island, land, claim the artifact, sail home and win.
//! No state is edited directly; every step is a typed command plus ticks.
use aoa_game::{
    BuildingKind, CellCoordinate, Command, GameWorld, ProductKind, ResourceKind, ScenarioOutcome,
    TEMPLE_ID, TerrainBiome, WORLD_COLUMNS, WORLD_ROWS, archipelago_plan,
};

const DT: f64 = 0.1;

struct Run {
    world: GameWorld,
    seconds: f64,
}

impl Run {
    fn tick_until(&mut self, what: &str, limit: f64, done: impl Fn(&GameWorld) -> bool) {
        let start = self.seconds;
        while !done(&self.world) {
            assert!(
                self.seconds - start < limit,
                "{what}: not done after {limit} s (units {:?})",
                self.world
                    .units
                    .iter()
                    .map(|u| (&u.id, u.cell, &u.action, u.health))
                    .collect::<Vec<_>>()
            );
            self.world.tick(DT);
            self.seconds += DT;
        }
        self.world.validate().unwrap();
        eprintln!("{:>7.1} s  {what}", self.seconds);
        // AOA_RUN_SNAPSHOTS=<dir> keeps each milestone's snapshot for browser review.
        if let Ok(dir) = std::env::var("AOA_RUN_SNAPSHOTS") {
            let name = what.replace([' ', '\'', '{', '}', ':', ','], "_");
            let path =
                std::path::Path::new(&dir).join(format!("{:04.0}-{name}.json", self.seconds));
            std::fs::write(path, serde_json::to_string(&self.world.snapshot()).unwrap()).unwrap();
        }
    }

    fn command(&mut self, command: Command) {
        self.world
            .apply_command(command.clone())
            .unwrap_or_else(|e| panic!("{command:?}: {e}"));
    }

    fn villagers(&self) -> Vec<String> {
        self.world.units.iter().map(|u| u.id.clone()).collect()
    }

    fn home(&self) -> CellCoordinate {
        self.world.buildings[0].origin
    }

    /// Both villagers gather the nearest node of `kind` until home holds `amount`.
    fn gather(&mut self, kind: ResourceKind, amount: f64) {
        let home = self.home();
        for unit_id in self.villagers() {
            let resource = self
                .world
                .resources
                .iter()
                .filter(|r| r.kind == kind && r.amount > 0.0 && r.field.is_none())
                .filter(|r| self.world.island_at(r.cell) == Some(0))
                .min_by_key(|r| r.cell.column.abs_diff(home.column) + r.cell.row.abs_diff(home.row))
                .expect("the start island has this resource")
                .id
                .clone();
            self.command(Command::Gather {
                unit_id,
                resource_id: resource,
            });
        }
        self.tick_until(&format!("gathered {amount} {kind:?}"), 900.0, |w| {
            w.inventories[0].amount(kind) >= amount
        });
    }

    /// The first villager places `kind` on the nearest accepted site; both build it.
    fn build(&mut self, kind: BuildingKind) -> String {
        let home = self.home();
        let units = self.villagers();
        let mut sites: Vec<_> = (0..WORLD_ROWS)
            .flat_map(|row| (0..WORLD_COLUMNS).map(move |column| CellCoordinate::new(column, row)))
            .collect();
        sites.sort_by_key(|c| c.column.abs_diff(home.column) + c.row.abs_diff(home.row));
        let before = self.world.buildings.len();
        // Like a player, pick visible ground: an order on fogged ground explores
        // first, so keep only sites where the foundation is placed at once.
        let origin = sites
            .into_iter()
            .find(|&origin| {
                let mut candidate = self.world.clone();
                candidate
                    .apply_command(Command::Build {
                        unit_id: units[0].clone(),
                        origin,
                        kind,
                    })
                    .is_ok()
                    && candidate.buildings.len() > before
            })
            .expect("a visible valid site");
        self.command(Command::Build {
            unit_id: units[0].clone(),
            origin,
            kind,
        });
        eprintln!("{:>7.1} s  {kind:?} foundation at {origin:?}", self.seconds);
        let id = self.world.buildings.last().unwrap().id.clone();
        self.command(Command::Construct {
            unit_id: units[1].clone(),
            building_id: id.clone(),
        });
        let check = id.clone();
        self.tick_until(&format!("{kind:?} built"), 600.0, move |w| {
            w.buildings.iter().any(|b| b.id == check && b.is_complete())
        });
        id
    }

    /// Walk the first villager toward the nearest open sea so its coast is visible.
    fn scout_coast(&mut self) {
        let home = self.home();
        let unit_id = self.villagers()[0].clone();
        // Open sea is the water connected to the map corner.
        let (columns, rows) = (WORLD_COLUMNS, WORLD_ROWS);
        let water = |c: CellCoordinate| {
            self.world.terrain
                [usize::from(c.row) * usize::from(self.world.columns()) + usize::from(c.column)]
            .biome
                == TerrainBiome::Water
        };
        let mut sea = std::collections::BTreeSet::from([CellCoordinate::new(0, 0)]);
        let mut frontier = vec![CellCoordinate::new(0, 0)];
        while let Some(c) = frontier.pop() {
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let (x, y) = (i32::from(c.column) + dx, i32::from(c.row) + dy);
                if x < 0 || y < 0 || x >= i32::from(columns) || y >= i32::from(rows) {
                    continue;
                }
                let n = CellCoordinate::new(x as u16, y as u16);
                if water(n) && sea.insert(n) {
                    frontier.push(n);
                }
            }
        }
        let mut shore: Vec<_> = (0..rows)
            .flat_map(|row| (0..columns).map(move |column| CellCoordinate::new(column, row)))
            .filter(|&c| !water(c))
            .filter(|c| {
                [(0i32, -1i32), (-1, 0), (1, 0), (0, 1)]
                    .iter()
                    .any(|(dx, dy)| {
                        let (x, y) = (i32::from(c.column) + dx, i32::from(c.row) + dy);
                        x >= 0 && y >= 0 && sea.contains(&CellCoordinate::new(x as u16, y as u16))
                    })
            })
            .collect();
        shore.sort_by_key(|c| c.column.abs_diff(home.column) + c.row.abs_diff(home.row));
        let to = shore
            .into_iter()
            .find(|&to| {
                self.world
                    .apply_command(Command::Move {
                        unit_id: unit_id.clone(),
                        to,
                    })
                    .is_ok()
            })
            .expect("a reachable beach");
        let id = unit_id.clone();
        self.tick_until(&format!("scouted the coast at {to:?}"), 300.0, move |w| {
            w.units.iter().any(|u| u.id == id && u.cell == to)
        });
    }

    fn sail_until_stopped(&mut self, what: &str) {
        self.tick_until(what, 1800.0, |w| w.ships[0].stopped());
    }
}

#[test]
fn a_run_is_won_through_player_commands_alone() {
    let seed = 7;
    let mut run = Run {
        world: GameWorld::generate(seed),
        seconds: 0.0,
    };
    let plan = archipelago_plan(seed);

    // Economy: wood and a little stone, a lumber mill for timber, then a dock.
    run.gather(ResourceKind::Stone, 10.0);
    run.gather(ResourceKind::Wood, 40.0);
    let mill = run.build(BuildingKind::LumberMill);
    run.gather(ResourceKind::Wood, 40.0);
    for _ in 0..4 {
        run.command(Command::Produce {
            building_id: mill.clone(),
            product: ProductKind::Timber,
        });
    }
    run.gather(ResourceKind::Wood, 30.0);
    run.scout_coast();
    let dock = run.build(BuildingKind::Dock);
    run.gather(ResourceKind::Wood, 60.0);
    run.tick_until("20 timber", 600.0, |w| {
        w.inventories[0].amount(ResourceKind::Timber) >= 20.0
    });
    run.command(Command::Produce {
        building_id: dock,
        product: ProductKind::TransportShip,
    });
    run.tick_until("ship launched", 300.0, |w| !w.ships.is_empty());
    let ship = run.world.ships[0].id.clone();

    // Voyage out: board, then steer into the fog toward the temple's site.
    for unit_id in run.villagers() {
        run.command(Command::Board {
            unit_id,
            ship_id: ship.clone(),
        });
    }
    run.tick_until("both aboard", 300.0, |w| w.ships[0].passengers.len() == 2);
    let temple_site = *plan.last().unwrap();
    let heart = CellCoordinate::new(
        temple_site.column + WORLD_COLUMNS / 2,
        temple_site.row + WORLD_ROWS / 2,
    );
    run.command(Command::Sail {
        ship_id: ship.clone(),
        to: heart,
    });
    run.sail_until_stopped("reached the temple island's waters");
    assert!(
        run.world.buildings.iter().any(|b| b.id == TEMPLE_ID),
        "temple discovered"
    );
    run.command(Command::Disembark {
        ship_id: ship.clone(),
    });
    run.tick_until("landed", 60.0, |w| w.units.len() == 2);

    // The goal: claim the artifact.
    run.command(Command::ClaimArtifact {
        unit_ids: run.villagers(),
        building_id: TEMPLE_ID.into(),
    });
    run.tick_until("artifact claimed", 600.0, |w| w.artifact_bearer.is_some());
    let bearer = run.world.artifact_bearer.clone().unwrap();

    // Home: board, take the one-tap return, land, walk beside the town center.
    for unit_id in run.villagers() {
        run.command(Command::Board {
            unit_id,
            ship_id: ship.clone(),
        });
    }
    run.tick_until("both aboard again", 600.0, |w| {
        w.ships[0].passengers.len() == 2
    });
    run.command(Command::Voyage {
        ship_id: ship.clone(),
        island_id: 0,
    });
    run.sail_until_stopped("home");
    run.command(Command::Disembark { ship_id: ship });
    run.tick_until("landed home", 60.0, |w| w.units.len() == 2);
    // Only the bearer walks home. The other villager idles where it landed,
    // possibly in the one-cell corridor beside the dock, and must make way.
    let home = run.home();
    let target = (0..8)
        .map(|d| CellCoordinate::new(home.column + 5 + d % 2, home.row + d / 2))
        .find(|&to| {
            run.world
                .apply_command(Command::Move {
                    unit_id: bearer.clone(),
                    to,
                })
                .is_ok()
        })
        .expect("ground beside the town center");
    eprintln!("bearer walks to {target:?}");
    run.tick_until("victory", 600.0, |w| {
        w.scenario.outcome == ScenarioOutcome::Won
    });
    assert_eq!(run.world.artifact_bearer.as_deref(), Some(bearer.as_str()));
    eprintln!("run won after {:.0} simulated seconds", run.seconds);
}
