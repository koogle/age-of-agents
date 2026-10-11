//! The barbarian raid on the second island: armed when the first unit sets foot
//! there, landing 5–10 minutes later. Raiders hunt the island's units and tear
//! down its buildings until the player defeats them.
use super::*;
use crate::navigation::{PathTree, offset};

/// The second island, the first one the player reaches by ship.
const RAID_ISLAND: usize = 1;
/// Seconds from the first landing until the raiders arrive.
const RAID_DELAY_SECONDS: (u64, u64) = (300, 600);
/// Raiders per raid, the chieftain included.
const RAID_SIZE: (u64, u64) = (10, 20);
/// The landing beach lies at least this far from the player's units and buildings.
const LANDING_CLEARANCE: f64 = 20.0;
const RAID_SALT: u64 = 0x7261_6964;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Raid {
    /// No unit has set foot on the second island yet.
    Waiting,
    /// The raiders land when the countdown runs out.
    Incoming {
        seconds_left: f64,
    },
    Landed,
}

fn seeded(seed: u64, salt: u64, (low, high): (u64, u64)) -> u64 {
    low + worldgen::mix(seed ^ RAID_SALT, salt) % (high - low + 1)
}

impl GameWorld {
    pub(super) fn tick_raid(&mut self, dt: f64) {
        match self.raid {
            Raid::Waiting => {
                if self
                    .units
                    .iter()
                    .any(|u| self.island_at(u.cell) == Some(RAID_ISLAND))
                {
                    let delay = seeded(self.seed, 1, RAID_DELAY_SECONDS);
                    self.raid = Raid::Incoming {
                        seconds_left: delay as f64,
                    };
                }
            }
            Raid::Incoming { seconds_left } if seconds_left > dt => {
                self.raid = Raid::Incoming {
                    seconds_left: seconds_left - dt,
                };
            }
            Raid::Incoming { .. } => {
                self.land_raiders();
                self.raid = Raid::Landed;
            }
            Raid::Landed => {}
        }
    }

    /// Put the war band ashore on the free beach nearest the player's presence
    /// on the island that still keeps its distance, spreading inland from there.
    fn land_raiders(&mut self) {
        let origin = self.island_origins[RAID_ISLAND];
        let island = Footprint {
            origin,
            columns: WORLD_COLUMNS,
            rows: WORLD_ROWS,
        };
        let presence: Vec<Position> = self
            .units
            .iter()
            .filter(|u| self.island_at(u.cell) == Some(RAID_ISLAND))
            .map(|u| u.cell.center())
            .chain(
                self.buildings
                    .iter()
                    .filter(|b| b.id != TEMPLE_ID && self.island_at(b.origin) == Some(RAID_ISLAND))
                    .map(|b| b.footprint().center()),
            )
            .collect();
        let aim = if presence.is_empty() {
            island.center()
        } else {
            let n = presence.len() as f64;
            Position {
                x: presence.iter().map(|p| p.x).sum::<f64>() / n,
                y: presence.iter().map(|p| p.y).sum::<f64>() / n,
            }
        };
        let occupancy = self.occupancy();
        let biome = |c: CellCoordinate| {
            self.terrain[usize::from(c.row) * usize::from(self.columns()) + usize::from(c.column)]
                .biome
        };
        let free = |c: CellCoordinate| {
            island.contains(c) && biome(c).is_walkable() && occupancy.is_free_for(c, None)
        };
        let (columns, rows) = (self.columns(), self.rows());
        let neighbors = move |c| {
            [(0, -1), (-1, 0), (1, 0), (0, 1)]
                .into_iter()
                .filter_map(move |(dx, dy)| offset(c, dx, dy, columns, rows))
        };
        let beach = island
            .cells()
            .filter(|&c| free(c) && neighbors(c).any(|n| biome(n) == TerrainBiome::Water))
            .map(|c| {
                let distance = c.center().distance(aim);
                let clear = presence
                    .iter()
                    .all(|p| p.distance(c.center()) >= LANDING_CLEARANCE);
                // Prefer the nearest beach at a distance; failing that, the farthest.
                let rank = if clear { distance } else { 1e6 - distance };
                (rank, c)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
            .map(|(_, c)| c);
        let Some(beach) = beach else {
            return;
        };
        let count = seeded(self.seed, 2, RAID_SIZE) as usize;
        let mut cells = vec![beach];
        let mut cursor = 0;
        while cells.len() < count && cursor < cells.len() {
            for next in neighbors(cells[cursor]) {
                if free(next) && !cells.contains(&next) {
                    cells.push(next);
                }
            }
            cursor += 1;
        }
        cells.truncate(count);
        for (number, cell) in cells.into_iter().enumerate() {
            // The chieftain leads; every third raider after him carries a torch.
            let kind = match number {
                0 => AnimalKind::Chieftain,
                n if n % 3 == 0 => AnimalKind::Torchbearer,
                _ => AnimalKind::Barbarian,
            };
            self.animals.push(Animal {
                id: format!("raider-{number}"),
                kind,
                home: cell,
                cell,
                step: None,
                health: kind.max_health(),
                attack_seconds: 0.0,
                heading: [1, 0],
            });
        }
    }

    /// Units within sight come first; otherwise the nearest reachable building,
    /// and once the island is razed, any unit still on it.
    pub(super) fn tick_raider(&mut self, index: usize, dt: f64) {
        let Animal { cell, kind, .. } = self.animals[index];
        let island = self.island_at(cell);
        let near: Vec<usize> = (0..self.units.len())
            .filter(|&u| {
                self.units[u].health > 0.0
                    && self.island_at(self.units[u].cell) == island
                    && self.units[u].cell.center().distance(cell.center()) <= kind.aggro()
            })
            .collect();
        let struck = near
            .iter()
            .copied()
            .filter(|&u| {
                let target = self.units[u].cell;
                self.units[u].step.is_none()
                    && cell.touches(target)
                    && self.melee_clear(cell, target)
            })
            .min_by(|&a, &b| self.units[a].id.cmp(&self.units[b].id));
        if let Some(unit) = struck {
            if self.swing(index, self.units[unit].cell, dt) {
                self.units[unit].health -= kind.damage();
            }
            return;
        }
        let buildings: Vec<usize> = (0..self.buildings.len())
            .filter(|&b| {
                self.buildings[b].id != TEMPLE_ID
                    && self.island_at(self.buildings[b].origin) == island
            })
            .collect();
        let beside = buildings.iter().copied().find_map(|b| {
            self.buildings[b]
                .footprint()
                .cells()
                .find(|c| c.touches(cell))
                .map(|c| (b, c))
        });
        if near.is_empty()
            && let Some((building, wall)) = beside
        {
            if self.swing(index, wall, dt) {
                self.buildings[building].damage += kind.building_damage();
                if self.buildings[building].damage >= self.buildings[building].kind.max_health() {
                    self.destroy_building(building);
                }
            }
            return;
        }
        self.animals[index].attack_seconds = 0.0;
        let occupancy = self.occupancy();
        let clear = |c: CellCoordinate| occupancy.is_free_for(c, None);
        let unit_at = |u: &Unit| Footprint {
            origin: u.cell,
            columns: 1,
            rows: 1,
        };
        let tiers = [
            near.iter().map(|&u| unit_at(&self.units[u])).collect(),
            buildings
                .iter()
                .map(|&b| self.buildings[b].footprint())
                .collect(),
            self.units
                .iter()
                .filter(|u| self.island_at(u.cell) == island)
                .map(unit_at)
                .collect::<Vec<_>>(),
        ];
        // The first tier with a free cell beside it sets the goals.
        let goals = tiers
            .into_iter()
            .map(|tier| {
                tier.into_iter()
                    .flat_map(movement::interaction_cells)
                    .filter(|&c| clear(c))
                    .collect::<Vec<_>>()
            })
            .find(|goals| !goals.is_empty())
            .unwrap_or_default();
        if let Some((_, path)) =
            PathTree::route_to_nearest(self.columns(), self.rows(), cell, &goals, clear)
            && let Some(&to) = path.first()
        {
            self.animals[index].heading = heading(cell, to);
            self.animals[index].step = Some(Step { to, progress: 0.0 });
        }
    }

    /// Face `toward` and wind up; true when the blow lands this tick.
    fn swing(&mut self, index: usize, toward: CellCoordinate, dt: f64) -> bool {
        let raider = &mut self.animals[index];
        raider.heading = heading(raider.cell, toward);
        raider.attack_seconds += dt;
        if raider.attack_seconds >= 1.0 {
            raider.attack_seconds = 0.0;
            return true;
        }
        false
    }

    /// A razed building takes its queue with it; orders aimed at it end.
    fn destroy_building(&mut self, index: usize) {
        let id = self.buildings.remove(index).id;
        for unit in &mut self.units {
            if let UnitAction::Build {
                building_id: target,
            }
            | UnitAction::Deposit { storage_id: target } = &unit.action
                && *target == id
            {
                unit.action = UnitAction::Idle;
            }
        }
    }

    pub(super) fn validate_raid(&self) -> Result<(), String> {
        if let Raid::Incoming { seconds_left } = self.raid
            && !(seconds_left.is_finite() && seconds_left > 0.0)
        {
            return Err("invalid raid countdown".into());
        }
        if self
            .buildings
            .iter()
            .any(|b| !b.damage.is_finite() || !(0.0..b.kind.max_health()).contains(&b.damage))
        {
            return Err("invalid building damage".into());
        }
        Ok(())
    }
}

fn heading(from: CellCoordinate, to: CellCoordinate) -> [i8; 2] {
    [
        (i32::from(to.column) - i32::from(from.column)).signum() as i8,
        (i32::from(to.row) - i32::from(from.row)).signum() as i8,
    ]
}
