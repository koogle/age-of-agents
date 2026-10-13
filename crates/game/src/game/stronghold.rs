//! The barbarian stronghold: the war bands' fortified headquarters on a later
//! island, guarded by a garrison. Razing it ends the raids on the second island.
use super::movement::{Goal, Travel};
use super::raid::war_band_member;
use super::*;

pub const STRONGHOLD_ID: &str = "stronghold";
/// Garrison raiders, the chieftain included.
const GARRISON_SIZE: (u64, u64) = (10, 14);
/// Garrison homes lie this many cells around the walls (Chebyshev distance).
const GARRISON_RING: (u16, u16) = (2, 5);
/// How far a garrison raider leaves its post to fight intruders.
pub(super) const GARRISON_TERRITORY: f64 = 14.0;
const STRONGHOLD_SALT: u64 = 0x0068_666f_7274;

impl GameWorld {
    /// The stronghold stands on the k-th island the player discovers after home,
    /// never the temple's, with k seeded from 2 up to the last such island, so it
    /// is always a later island than the one the raids strike. `None` until then.
    pub(super) fn stronghold_island(&self) -> Option<usize> {
        let plan = archipelago_plan(self.seed);
        let temple = *plan.last().expect("planned sites");
        let ordinary = plan.len() as u64 - 2;
        let k = 2 + worldgen::mix(self.seed ^ STRONGHOLD_SALT, 0) % (ordinary - 1);
        self.island_origins
            .iter()
            .enumerate()
            .skip(1)
            .filter(|&(_, &o)| o != temple)
            .nth(k as usize - 1)
            .map(|(island, _)| island)
    }

    /// Raise the stronghold near the island's centre and post its garrison around it.
    pub(super) fn place_stronghold(&mut self, site: CellCoordinate, island: usize) {
        let Some(origin) = self.central_plot(site, BuildingKind::Stronghold) else {
            return;
        };
        self.buildings.push(building(
            BuildingKind::Stronghold,
            STRONGHOLD_ID,
            origin,
            None,
        ));
        let walls = self.buildings.last().expect("just placed").footprint();
        let occupancy = self.occupancy();
        let mut posts: Vec<CellCoordinate> = self
            .terrain
            .iter()
            .map(|t| t.coordinate())
            .filter(|&c| {
                let gap = |v: u16, start: u16, length: u16| {
                    start
                        .saturating_sub(v)
                        .max(v.saturating_sub(start + length - 1))
                };
                let ring = gap(c.column, walls.origin.column, walls.columns).max(gap(
                    c.row,
                    walls.origin.row,
                    walls.rows,
                ));
                (GARRISON_RING.0..=GARRISON_RING.1).contains(&ring)
                    && self.island_at(c) == Some(island)
                    && self.terrain
                        [usize::from(c.row) * usize::from(self.columns()) + usize::from(c.column)]
                    .biome
                    .is_walkable()
                    && occupancy.is_free_for(c, None)
            })
            .collect();
        posts.sort_by_key(|c| {
            worldgen::mix(
                self.seed ^ STRONGHOLD_SALT,
                u64::from(c.row) * 65536 + u64::from(c.column),
            )
        });
        let (low, high) = GARRISON_SIZE;
        let count = low + worldgen::mix(self.seed ^ STRONGHOLD_SALT, 1) % (high - low + 1);
        for (number, cell) in posts.into_iter().take(count as usize).enumerate() {
            self.animals
                .push(war_band_member(format!("garrison-{number}"), number, cell));
        }
    }

    /// Raiders posted on the stronghold's island (`fort`, from
    /// `stronghold_island`) guard its ground instead of roaming, even after it
    /// falls. Derived, so saves carry no flag.
    pub(super) fn garrisoned(&self, fort: Option<usize>, animal: &Animal) -> bool {
        animal.kind.raids() && fort.is_some_and(|f| self.island_at(animal.home) == Some(f))
    }

    pub(super) fn attack_building(
        &mut self,
        ids: &[String],
        building_id: &str,
    ) -> Result<(), CommandError> {
        if ids.is_empty() {
            return Err(CommandError::EmptyUnitGroup);
        }
        if ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
            return Err(CommandError::DuplicateUnit);
        }
        let target = self
            .buildings
            .iter()
            .find(|b| {
                b.id == building_id
                    && b.footprint()
                        .cells()
                        .any(|c| self.explored_cells.contains(&c))
            })
            .ok_or(CommandError::BuildingNotFound)?;
        if target.kind != BuildingKind::Stronghold {
            return Err(CommandError::NotAttackable);
        }
        let footprint = target.footprint();
        for id in ids {
            let index = self
                .units
                .iter()
                .position(|u| &u.id == id)
                .ok_or(CommandError::UnitNotFound)?;
            if self.units[index].kind == UnitKind::Healer {
                return Err(CommandError::CannotAttack);
            }
            if !self.can_reach_beside(index, footprint) {
                return Err(CommandError::TargetUnreachable);
            }
        }
        for id in ids {
            let index = self.ordered_unit(id)?;
            self.units[index].action = UnitAction::AttackBuilding {
                building_id: building_id.into(),
                elapsed_seconds: 0.0,
            };
        }
        Ok(())
    }

    pub(super) fn tick_attack_building(&mut self, unit: usize, id: &str, elapsed: f64, dt: f64) {
        let Some(target) = self.buildings.iter().position(|b| b.id == id) else {
            self.units[unit].action = UnitAction::Idle;
            return;
        };
        match self.travel(unit, Goal::Beside(self.buildings[target].footprint()), dt) {
            Travel::Arrived { remaining } => {
                let elapsed = elapsed + remaining;
                let strike = elapsed >= 1.0;
                self.units[unit].action = UnitAction::AttackBuilding {
                    building_id: id.into(),
                    elapsed_seconds: if strike { 0.0 } else { elapsed },
                };
                if strike {
                    let building = &mut self.buildings[target];
                    building.damage += contact_damage(self.units[unit].kind);
                    if building.damage >= building.kind.max_health() {
                        self.destroy_building(target);
                    }
                }
            }
            Travel::Unreachable => self.units[unit].action = UnitAction::Idle,
            Travel::EnRoute => {}
        }
    }

    pub(super) fn validate_stronghold(&self) -> Result<(), String> {
        let strongholds = self
            .buildings
            .iter()
            .filter(|b| b.kind == BuildingKind::Stronghold)
            .count();
        if strongholds > 1 || (strongholds == 1 && self.raid == Raid::Ended) {
            return Err("invalid barbarian stronghold".into());
        }
        for unit in self
            .units
            .iter()
            .chain(self.ships.iter().flat_map(|s| &s.passengers))
        {
            if let UnitAction::AttackBuilding {
                building_id,
                elapsed_seconds,
            } = &unit.action
                && (!self
                    .buildings
                    .iter()
                    .any(|b| &b.id == building_id && b.kind == BuildingKind::Stronghold)
                    || !(0.0..1.0).contains(elapsed_seconds)
                    || unit.kind == UnitKind::Healer)
            {
                return Err("invalid building attack order".into());
            }
        }
        Ok(())
    }
}

/// Damage per one-second blow a friendly unit deals in melee contact.
pub(super) fn contact_damage(kind: UnitKind) -> f64 {
    match kind {
        UnitKind::Guard => 25.0,
        UnitKind::Archer => 18.0,
        UnitKind::SiegeCart => 35.0,
        _ => 10.0,
    }
}
