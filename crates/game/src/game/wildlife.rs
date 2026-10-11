//! Territorial wildlife and explicit hunting orders, without a general combat engine.
use super::*;
use crate::navigation::{PathTree, offset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimalKind {
    Wolf,
    Bear,
    Boar,
    /// A pride hunter; pride members share one alert.
    Lioness,
    /// The maned pride leader: the lioness's heavier form.
    Lion,
    /// Ambushers: concealed until a unit comes close, then a heavy strike.
    /// A rock python lies up near wild resources.
    Python,
    /// A sea serpent waits on the shoreline.
    SeaSerpent,
    /// A marble viper guards the temple island.
    Viper,
    /// Raiders land on the second island, hunt its units and tear down its
    /// buildings; they roam the whole island instead of a territory.
    Barbarian,
    /// The raid's leader: the barbarian's heavier form.
    Chieftain,
    /// A lightly clad fire-raider: burns buildings fast, fights people poorly.
    Torchbearer,
}

/// Lions whose homes lie this close belong to one pride.
const PRIDE_RADIUS: f64 = 5.0;
/// An ambusher stays out of snapshots until a land unit is this close.
const REVEAL_RADIUS: f64 = 3.0;

impl AnimalKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Wolf => "Wolf",
            Self::Bear => "Bear",
            Self::Boar => "Boar",
            Self::Lioness => "Lioness",
            Self::Lion => "Lion",
            Self::Python => "Rock python",
            Self::SeaSerpent => "Sea serpent",
            Self::Viper => "Marble viper",
            Self::Barbarian => "Barbarian raider",
            Self::Chieftain => "Barbarian chieftain",
            Self::Torchbearer => "Barbarian torchbearer",
        }
    }
    pub fn max_health(self) -> f64 {
        match self {
            Self::Wolf => 300.0,
            Self::Bear => 600.0,
            Self::Boar => 60.0,
            Self::Lioness => 250.0,
            Self::Lion => 500.0,
            Self::Python => 400.0,
            Self::SeaSerpent => 350.0,
            Self::Viper => 250.0,
            Self::Barbarian => 150.0,
            Self::Chieftain => 600.0,
            Self::Torchbearer => 100.0,
        }
    }
    pub(super) fn aggro(self) -> f64 {
        match self {
            Self::Wolf => 6.0,
            Self::Bear => 4.0,
            Self::Boar => 4.0,
            Self::Lioness | Self::Lion => 6.0,
            Self::Python | Self::SeaSerpent | Self::Viper => 2.5,
            Self::Barbarian | Self::Chieftain | Self::Torchbearer => 7.0,
        }
    }
    fn territory(self) -> f64 {
        match self {
            Self::Wolf => 10.0,
            Self::Bear => 6.0,
            Self::Boar => 8.0,
            Self::Lioness | Self::Lion => 12.0,
            Self::Python | Self::SeaSerpent | Self::Viper => 3.0,
            Self::Barbarian | Self::Chieftain | Self::Torchbearer => f64::INFINITY,
        }
    }
    fn speed(self) -> f64 {
        match self {
            Self::Wolf => 2.5,
            Self::Bear => 1.8,
            Self::Boar => 2.2,
            Self::Lioness => 2.8,
            Self::Lion => 2.4,
            Self::Python => 1.2,
            Self::SeaSerpent => 1.6,
            Self::Viper => 1.4,
            Self::Barbarian => 2.4,
            Self::Chieftain => 2.2,
            Self::Torchbearer => 2.6,
        }
    }
    pub(super) fn is_lion(self) -> bool {
        matches!(self, Self::Lioness | Self::Lion)
    }
    pub fn raids(self) -> bool {
        matches!(self, Self::Barbarian | Self::Chieftain | Self::Torchbearer)
    }
    pub fn ambushes(self) -> bool {
        matches!(self, Self::Python | Self::SeaSerpent | Self::Viper)
    }
    pub(super) fn damage(self) -> f64 {
        match self {
            Self::Wolf => 35.0,
            Self::Bear => 50.0,
            Self::Boar => 10.0,
            Self::Lioness => 30.0,
            Self::Lion => 45.0,
            Self::Python => 45.0,
            Self::SeaSerpent => 35.0,
            Self::Viper => 60.0,
            Self::Barbarian => 15.0,
            Self::Chieftain => 30.0,
            Self::Torchbearer => 6.0,
        }
    }
    /// Damage per second against buildings; animals leave buildings alone.
    pub(super) fn building_damage(self) -> f64 {
        match self {
            Self::Barbarian => 10.0,
            Self::Chieftain => 25.0,
            Self::Torchbearer => 30.0,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animal {
    pub id: String,
    pub kind: AnimalKind,
    pub home: CellCoordinate,
    pub cell: CellCoordinate,
    pub step: Option<Step>,
    pub health: f64,
    pub attack_seconds: f64,
    pub heading: [i8; 2],
}
impl Animal {
    pub fn position(&self) -> Position {
        Position::on_step(self.cell, self.step)
    }
    /// An idle ambusher at home with no land unit within reach stays hidden:
    /// out of snapshots and untargetable. Derived, so saves carry no extra state.
    pub fn concealed(&self, units: &[Unit]) -> bool {
        self.kind.ambushes()
            && self.step.is_none()
            && self.attack_seconds == 0.0
            && !units
                .iter()
                .any(|u| u.cell.center().distance(self.cell.center()) <= REVEAL_RADIUS)
    }
    pub(super) fn footprint(&self) -> Footprint {
        Footprint {
            origin: self.cell,
            columns: 1,
            rows: 1,
        }
    }
}

impl GameWorld {
    pub(super) fn populate_wildlife(&mut self, island: usize) {
        let count = if island == 0 {
            1
        } else {
            2 + (worldgen::mix(self.seed ^ 0x7769_6c64_6c69_6665, island as u64) % 3) as usize
        };
        let boars = if island == 0 {
            1
        } else {
            2 + (worldgen::mix(self.seed ^ 0x626f_6172, island as u64) % 2) as usize
        };
        let pride = if island == 0 {
            0
        } else {
            3 + (worldgen::mix(self.seed ^ 0x6c69_6f6e, island as u64) % 2) as usize
        };
        let temple_island = self.island_origins[island]
            == *archipelago_plan(self.seed).last().expect("planned sites");
        let snakes: Vec<AnimalKind> = if island == 0 {
            Vec::new()
        } else {
            let pythons =
                1 + (worldgen::mix(self.seed ^ 0x7079_7468_6f6e, island as u64) % 2) as usize;
            let mut kinds = vec![AnimalKind::Python; pythons];
            kinds.push(AnimalKind::SeaSerpent);
            if temple_island {
                kinds.extend([AnimalKind::Viper, AnimalKind::Viper]);
            }
            kinds
        };
        let mut leader = None;
        for number in 0..count + boars + pride + snakes.len() {
            let kind = if number >= count + boars + pride {
                snakes[number - count - boars - pride]
            } else if number == count + boars {
                AnimalKind::Lion
            } else if number > count + boars {
                AnimalKind::Lioness
            } else if number >= count {
                AnimalKind::Boar
            } else if island > 0 && number == count - 1 {
                AnimalKind::Bear
            } else {
                AnimalKind::Wolf
            };
            // Only lionesses join the leader; everything else keeps its own spacing.
            let beside = (kind == AnimalKind::Lioness).then_some(leader).flatten();
            if let Some(cell) = self.wildlife_cell(island, beside, kind) {
                if kind == AnimalKind::Lion {
                    leader = Some(cell);
                }
                self.animals.push(Animal {
                    id: format!("animal-{island}-{number}"),
                    kind,
                    home: cell,
                    cell,
                    step: None,
                    health: kind.max_health(),
                    attack_seconds: 0.0,
                    heading: [1, 0],
                });
            } else if kind == AnimalKind::Lion {
                // Without a leader there is no pride to join.
                break;
            }
        }
    }

    /// A free starting cell on the island well away from the settlement: apart from
    /// other territories, or beside `pride` for a pride member.
    /// Whether an ambusher may lie up at `cell`: pythons near wild resources, sea
    /// serpents on the shoreline, vipers around the temple. Other kinds lie anywhere.
    pub(super) fn habitat(&self, kind: AnimalKind, cell: CellCoordinate) -> bool {
        const REACH: f64 = 4.0;
        match kind {
            AnimalKind::Python => self
                .resources
                .iter()
                .any(|r| r.field.is_none() && r.cell.center().distance(cell.center()) <= REACH),
            AnimalKind::SeaSerpent => {
                let columns = self.columns();
                let rows = self.rows();
                (-4..=4).any(|dy| {
                    (-4..=4).any(|dx| {
                        offset(cell, dx, dy, columns, rows).is_some_and(|c| {
                            c.center().distance(cell.center()) <= REACH
                                && self.terrain[usize::from(c.row) * usize::from(columns)
                                    + usize::from(c.column)]
                                .biome
                                    == TerrainBiome::Water
                        })
                    })
                })
            }
            AnimalKind::Viper => self.buildings.iter().any(|b| {
                b.id == TEMPLE_ID && b.origin.center().distance(cell.center()) <= REACH + 2.0
            }),
            _ => true,
        }
    }

    fn wildlife_cell(
        &self,
        island: usize,
        pride: Option<CellCoordinate>,
        kind: AnimalKind,
    ) -> Option<CellCoordinate> {
        let origin = self.island_origins[island];
        let occupancy = self.occupancy();
        self.terrain
            .iter()
            .filter_map(|t| {
                let c = t.coordinate();
                (c.column >= origin.column
                    && c.column < origin.column + WORLD_COLUMNS
                    && c.row >= origin.row
                    && c.row < origin.row + WORLD_ROWS
                    && t.biome.is_walkable()
                    && occupancy.is_free_for(c, None)
                    && self
                        .buildings
                        .iter()
                        // The temple is a lair, not a settlement to keep clear of.
                        .filter(|b| b.id != TEMPLE_ID)
                        .all(|b| b.footprint().center().distance(c.center()) >= 26.0)
                    && self
                        .units
                        .iter()
                        .all(|u| u.cell.center().distance(c.center()) >= 26.0)
                    && self.habitat(kind, c)
                    && match pride {
                        Some(leader) => {
                            leader.center().distance(c.center()) <= 3.0
                                && self.animals.iter().all(|a| a.home != c)
                        }
                        None => self
                            .animals
                            .iter()
                            .all(|a| a.home.center().distance(c.center()) >= 18.0),
                    })
                .then_some(c)
            })
            .min_by_key(|c| {
                worldgen::mix(
                    self.seed ^ island as u64,
                    u64::from(c.row) * 65536 + u64::from(c.column),
                )
            })
    }

    pub(super) fn attack_animal(
        &mut self,
        ids: &[String],
        animal_id: &str,
    ) -> Result<(), CommandError> {
        if ids.is_empty() {
            return Err(CommandError::EmptyUnitGroup);
        }
        if ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
            return Err(CommandError::DuplicateUnit);
        }
        let visible = self.visible_cells();
        let animal = self
            .animals
            .iter()
            .find(|a| {
                a.id == animal_id
                    && !a.concealed(&self.units)
                    && visible.contains(&a.cell)
                    && a.step.is_none_or(|s| visible.contains(&s.to))
            })
            .ok_or(CommandError::AnimalNotVisible)?;
        let footprint = animal.footprint();
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
            self.units[index].action = UnitAction::AttackAnimal {
                animal_id: animal_id.into(),
                elapsed_seconds: 0.0,
            };
        }
        Ok(())
    }

    pub(super) fn tick_attack_animal(&mut self, unit: usize, id: &str, elapsed: f64, dt: f64) {
        let visible = self.visible_cells();
        let Some(animal) = self.animals.iter().position(|a| {
            a.id == id
                && !a.concealed(&self.units)
                && visible.contains(&a.cell)
                && a.step.is_none_or(|s| visible.contains(&s.to))
        }) else {
            self.units[unit].action = UnitAction::Idle;
            return;
        };
        let footprint = self.animals[animal].footprint();
        let from = self.units[unit].cell;
        let to = footprint.origin;
        if from.touches(to) && !self.melee_clear(from, to) {
            let occupancy = self.occupancy();
            let clear = |c| occupancy.is_free_for(c, Some(unit));
            let roads = self.completed_road_cells();
            let goals: Vec<_> = [(0, -1), (-1, 0), (1, 0), (0, 1)]
                .into_iter()
                .filter_map(|(x, y)| offset(to, x, y, self.columns(), self.rows()))
                .filter(|c| clear(*c))
                .collect();
            if let Some((_, path)) = PathTree::nearest_weighted(
                self.columns(),
                self.rows(),
                from,
                &goals,
                clear,
                |cell| if roads.contains(&cell) { 2 } else { 3 },
            ) && let Some(&goal) = path.last()
            {
                self.travel(unit, Goal::Cell(goal), dt);
            }
            return;
        }
        match self.travel(unit, Goal::Beside(footprint), dt) {
            Travel::Arrived { remaining } => {
                if self.animals[animal].step.is_some()
                    || !self.melee_clear(self.units[unit].cell, self.animals[animal].cell)
                {
                    return;
                }
                let elapsed = elapsed + remaining;
                if elapsed >= 1.0 {
                    let damage = match self.units[unit].kind {
                        UnitKind::Guard => 25.0,
                        UnitKind::Archer => 18.0,
                        UnitKind::SiegeCart => 35.0,
                        _ => 10.0,
                    };
                    self.animals[animal].health -= damage;
                }
                self.units[unit].action = UnitAction::AttackAnimal {
                    animal_id: id.into(),
                    elapsed_seconds: if elapsed >= 1.0 { 0.0 } else { elapsed },
                };
                if self.animals[animal].health <= 0.0 {
                    self.animals.remove(animal);
                    for u in &mut self.units {
                        if matches!(&u.action, UnitAction::AttackAnimal { animal_id, .. } if animal_id == id)
                        {
                            u.action = UnitAction::Idle;
                        }
                    }
                }
            }
            Travel::Unreachable => self.units[unit].action = UnitAction::Idle,
            Travel::EnRoute => {}
        }
    }

    pub(super) fn tick_wildlife(&mut self, dt: f64) {
        for index in 0..self.animals.len() {
            let animal = &self.animals[index];
            if let Some(step) = animal.step {
                let progress = step.progress
                    + dt * animal.kind.speed() / animal.cell.center().distance(step.to.center());
                if progress >= 1.0 {
                    self.animals[index].cell = step.to;
                    self.animals[index].step = None;
                } else {
                    self.animals[index].step = Some(Step { progress, ..step });
                }
                continue;
            }
            if animal.kind.raids() {
                self.tick_raider(index, dt);
                continue;
            }
            let home = animal.home;
            let cell = animal.cell;
            let territory = animal.kind.territory();
            let sighted = |a: &Animal| {
                self.units.iter().any(|u| {
                    u.health > 0.0
                        && u.cell.center().distance(a.home.center()) <= a.kind.territory()
                        && u.cell.center().distance(a.cell.center()) <= a.kind.aggro()
                })
            };
            // One lion's sighting sends its whole pride after intruders in reach.
            let alerted = animal.kind.is_lion()
                && self.animals.iter().any(|a| {
                    a.kind.is_lion()
                        && a.home.center().distance(home.center()) <= PRIDE_RADIUS
                        && sighted(a)
                });
            let target = self
                .units
                .iter()
                .enumerate()
                .filter(|(_, u)| {
                    u.health > 0.0
                        && u.cell.center().distance(home.center()) <= territory
                        && (alerted
                            || u.cell.center().distance(cell.center()) <= animal.kind.aggro())
                })
                .min_by(|(_, a), (_, b)| {
                    a.cell
                        .center()
                        .distance(cell.center())
                        .total_cmp(&b.cell.center().distance(cell.center()))
                        .then_with(|| a.id.cmp(&b.id))
                })
                .map(|(i, _)| i);
            if let Some(unit) = target
                && self.units[unit].step.is_none()
                && cell.touches(self.units[unit].cell)
            {
                // An animal cannot bite diagonally through two touching obstacles.
                let clear = self.melee_clear(cell, self.units[unit].cell);
                if clear {
                    let target_cell = self.units[unit].cell;
                    self.animals[index].heading = [
                        (i32::from(target_cell.column) - i32::from(cell.column)) as i8,
                        (i32::from(target_cell.row) - i32::from(cell.row)) as i8,
                    ];
                    self.animals[index].attack_seconds += dt;
                    if self.animals[index].attack_seconds >= 1.0 {
                        self.units[unit].health -= self.animals[index].kind.damage();
                        self.animals[index].attack_seconds = 0.0;
                    }
                    continue;
                }
            }
            self.animals[index].attack_seconds = 0.0;
            if target.is_none() && cell == home {
                continue;
            }
            let occupancy = self.occupancy();
            let clear = |c: CellCoordinate| {
                c.center().distance(home.center()) <= territory && occupancy.is_free_for(c, None)
            };
            let goals: Vec<_> = if let Some(unit) = target {
                movement::interaction_cells(Footprint {
                    origin: self.units[unit].cell,
                    columns: 1,
                    rows: 1,
                })
                .filter(|c| clear(*c))
                .collect()
            } else if cell != home && clear(home) {
                vec![home]
            } else {
                Vec::new()
            };
            if let Some((_, path)) =
                PathTree::route_to_nearest(self.columns(), self.rows(), cell, &goals, clear)
                && let Some(&to) = path.first()
            {
                self.animals[index].heading = [
                    (i32::from(to.column) - i32::from(cell.column)) as i8,
                    (i32::from(to.row) - i32::from(cell.row)) as i8,
                ];
                self.animals[index].step = Some(Step { to, progress: 0.0 });
            }
        }
        self.units.retain(|u| u.health > 0.0);
    }

    pub(super) fn melee_clear(&self, from: CellCoordinate, to: CellCoordinate) -> bool {
        if from.column == to.column || from.row == to.row {
            return true;
        }
        let occupancy = self.occupancy();
        occupancy.is_free_for(CellCoordinate::new(from.column, to.row), None)
            && occupancy.is_free_for(CellCoordinate::new(to.column, from.row), None)
    }

    pub(super) fn validate_wildlife(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        for animal in &self.animals {
            if !ids.insert(&animal.id)
                || animal.id.is_empty()
                || !self.in_bounds(animal.home)
                || self
                    .terrain
                    .get(
                        usize::from(animal.home.row) * usize::from(self.columns())
                            + usize::from(animal.home.column),
                    )
                    .is_none_or(|t| !t.biome.is_walkable())
                || animal.heading.iter().any(|v| !(-1..=1).contains(v))
                || !animal.health.is_finite()
                || animal.health <= 0.0
                || animal.health > animal.kind.max_health()
                || !animal.attack_seconds.is_finite()
                || !(0.0..1.0).contains(&animal.attack_seconds)
                || animal.cell.center().distance(animal.home.center()) > animal.kind.territory()
                || animal.step.is_some_and(|s| {
                    !animal.cell.touches(s.to)
                        || !(0.0..1.0).contains(&s.progress)
                        || s.to.center().distance(animal.home.center()) > animal.kind.territory()
                })
            {
                return Err("invalid wildlife state".into());
            }
        }
        for unit in self
            .units
            .iter()
            .chain(self.ships.iter().flat_map(|s| &s.passengers))
        {
            if !unit.health.is_finite() || unit.health <= 0.0 || unit.health > UNIT_HEALTH {
                return Err("invalid unit health".into());
            }
            if let UnitAction::AttackAnimal {
                animal_id,
                elapsed_seconds,
            } = &unit.action
                && (!self.animals.iter().any(|a| &a.id == animal_id)
                    || !elapsed_seconds.is_finite()
                    || !(0.0..1.0).contains(elapsed_seconds)
                    || unit.kind == UnitKind::Healer)
            {
                return Err("invalid animal attack order".into());
            }
        }
        Ok(())
    }
}
