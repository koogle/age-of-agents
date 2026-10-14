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
    /// Spartoi: squads of risen dead that share one alert and rise again
    /// quickly after falling.
    Skeleton,
    /// Centaur archers in bands: they shoot from range instead of closing in.
    Centaur,
    /// A one-eyed giant whose club sweeps every unit around him.
    Cyclops,
    /// The cyclops warlord guarding the temple: the cyclops's heavier form.
    CyclopsGeneral,
    /// The temple's guardian sleeps until the artifact leaves the temple or he
    /// is struck, then hunts the bearer across the whole island.
    Minotaur,
}

/// Pack members (a pride, a squad, a band) whose homes lie this close share alerts.
const PRIDE_RADIUS: f64 = 5.0;
/// Centaurs shoot at units this close.
const ARROW_RANGE: f64 = 4.0;
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
            Self::Skeleton => "Skeleton warrior",
            Self::Centaur => "Centaur archer",
            Self::Cyclops => "Cyclops",
            Self::CyclopsGeneral => "Cyclops general",
            Self::Minotaur => "Minotaur",
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
            Self::Skeleton => 150.0,
            Self::Centaur => 300.0,
            Self::Cyclops => 1000.0,
            Self::CyclopsGeneral => 2000.0,
            Self::Minotaur => 1800.0,
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
            Self::Skeleton => 5.0,
            Self::Centaur => 7.0,
            Self::Cyclops | Self::CyclopsGeneral => 5.0,
            Self::Minotaur => 6.0,
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
            Self::Skeleton => 9.0,
            Self::Centaur => 12.0,
            Self::Cyclops | Self::CyclopsGeneral => 8.0,
            // Awake, he roams the whole temple island.
            Self::Minotaur => f64::INFINITY,
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
            Self::Skeleton => 2.0,
            Self::Centaur => 2.6,
            Self::Cyclops => 1.5,
            Self::CyclopsGeneral => 1.4,
            Self::Minotaur => 2.6,
        }
    }
    /// Kinds that hunt together: one member's sighting alerts its whole pack.
    fn pack(self) -> Option<u8> {
        match self {
            Self::Lioness | Self::Lion => Some(0),
            Self::Skeleton => Some(1),
            Self::Centaur => Some(2),
            _ => None,
        }
    }
    /// Temple guardians lie close together around the sanctuary.
    fn guards_temple(self) -> bool {
        matches!(self, Self::Viper | Self::CyclopsGeneral | Self::Minotaur)
    }
    /// Seconds before a fallen lair animal on a later island rises again;
    /// raiders and temple bosses never return.
    pub(super) fn respawn_seconds(self) -> Option<f64> {
        match self {
            Self::Skeleton => Some(60.0),
            Self::Barbarian
            | Self::Chieftain
            | Self::Torchbearer
            | Self::CyclopsGeneral
            | Self::Minotaur => None,
            _ => Some(240.0),
        }
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
            Self::Skeleton => 20.0,
            Self::Centaur => 25.0,
            Self::Cyclops => 60.0,
            Self::CyclopsGeneral => 80.0,
            Self::Minotaur => 90.0,
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
    /// The danger rank of its island (0 home, rising toward the temple);
    /// health and damage grow with it. Raiders stay at 1.
    pub level: u8,
}
impl Animal {
    pub fn spawn(id: String, kind: AnimalKind, cell: CellCoordinate, level: u8) -> Self {
        let mut animal = Self {
            id,
            kind,
            home: cell,
            cell,
            step: None,
            health: 0.0,
            attack_seconds: 0.0,
            heading: [1, 0],
            level,
        };
        animal.health = animal.max_health();
        animal
    }
    /// Health and damage rise by a quarter and a tenth per level beyond the second island.
    fn scale(&self, per_level: f64) -> f64 {
        1.0 + per_level * f64::from(self.level.saturating_sub(1))
    }
    pub fn max_health(&self) -> f64 {
        self.kind.max_health() * self.scale(0.25)
    }
    pub(super) fn damage(&self) -> f64 {
        self.kind.damage() * self.scale(0.1)
    }
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
    /// The island's danger rank: its place in the planned archipelago, which
    /// orders sites by crossings from home, so the temple island ranks highest.
    pub(super) fn island_level(&self, island: usize) -> u8 {
        archipelago_plan(self.seed)
            .iter()
            .position(|site| *site == self.islands[island])
            .unwrap_or(island) as u8
    }

    pub(super) fn populate_wildlife(&mut self, island: usize) {
        use AnimalKind::*;
        let roll = |salt: u64, sides: u64| {
            (worldgen::mix(self.seed ^ salt, island as u64) % sides) as usize
        };
        let level = self.island_level(island);
        // Each entry is a kind and whether it joins the last pack leader placed.
        let mut roster = Vec::new();
        let mut add = |kind, count: usize, joins: bool| roster.extend(vec![(kind, joins); count]);
        if island == 0 {
            add(Wolf, 1, false);
            add(Boar, 1, false);
        } else {
            let temple_island =
                self.islands[island] == *archipelago_plan(self.seed).last().expect("planned sites");
            let deeper = usize::from(level.saturating_sub(1));
            add(Wolf, 1 + roll(0x7769_6c64_6c69_6665, 3) + deeper / 2, false);
            add(Bear, if level >= 4 { 2 } else { 1 }, false);
            add(Boar, 2 + roll(0x626f_6172, 2), false);
            add(Lion, 1, false);
            add(Lioness, 2 + roll(0x6c69_6f6e, 2), true);
            add(Python, 1 + roll(0x7079_7468_6f6e, 2), false);
            add(SeaSerpent, 1, false);
            if level >= 2 {
                // A squad: its leader, then the rest beside him.
                add(Skeleton, 1, false);
                add(Skeleton, 1 + usize::from(level), true);
            }
            if level >= 3 {
                add(Centaur, 1, false);
                add(Centaur, 1 + deeper / 3, true);
            }
            if temple_island {
                add(Viper, 2, false);
                add(CyclopsGeneral, 1, false);
                add(Minotaur, 1, false);
            } else if level >= 4 {
                add(Cyclops, 1, false);
            }
        }
        let mut leader = None;
        for (number, (kind, joins)) in roster.into_iter().enumerate() {
            if joins && leader.is_none() {
                // Without a leader there is no pack to join.
                continue;
            }
            let beside = if joins { leader } else { None };
            let cell = self.wildlife_cell(island, beside, kind);
            if !joins {
                leader = cell;
            }
            if let Some(cell) = cell {
                self.animals.push(Animal::spawn(
                    format!("animal-{island}-{number}"),
                    kind,
                    cell,
                    level,
                ));
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
            AnimalKind::Viper | AnimalKind::CyclopsGeneral | AnimalKind::Minotaur => {
                self.buildings.iter().any(|b| {
                    b.id == TEMPLE_ID && b.origin.center().distance(cell.center()) <= REACH + 2.0
                })
            }
            _ => true,
        }
    }

    fn wildlife_cell(
        &self,
        island: usize,
        pride: Option<CellCoordinate>,
        kind: AnimalKind,
    ) -> Option<CellCoordinate> {
        let region = self.islands[island];
        let occupancy = self.occupancy();
        self.terrain
            .iter()
            .filter_map(|t| {
                let c = t.coordinate();
                (region.contains(c)
                    && t.biome.is_walkable()
                    && occupancy.is_free_for(c, None)
                    && self
                        .buildings
                        .iter()
                        // The temple and stronghold are not settlements to keep clear of.
                        .filter(|b| !b.kind.is_generated())
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
                        // Temple guardians crowd the sanctuary, a little apart.
                        None if kind.guards_temple() => self
                            .animals
                            .iter()
                            .all(|a| a.home.center().distance(c.center()) >= 2.0),
                        // A stronghold garrison holds posts, not territories.
                        None => self
                            .animals
                            .iter()
                            .filter(|a| !a.kind.raids())
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
        for id in ids {
            let index = self
                .units
                .iter()
                .position(|u| &u.id == id)
                .ok_or(CommandError::UnitNotFound)?;
            if self.units[index].kind == UnitKind::Healer {
                return Err(CommandError::CannotAttack);
            }
            if !self.can_reach_attack(index, animal) {
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
        let archer = self.units[unit].kind == UnitKind::Archer;
        if !archer && from.touches(to) && !self.melee_clear(from, to) {
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
        let goal = if archer {
            Goal::ArcherRange(to)
        } else {
            Goal::Beside(footprint)
        };
        match self.travel(unit, goal, dt) {
            Travel::Arrived { remaining } => {
                let target = &self.animals[animal];
                let ready = if archer {
                    // Both ends of a moving target's step must stay in a clear shot.
                    let occupancy = self.occupancy();
                    let from = self.units[unit].cell;
                    self.archer_shot_clear(from, target.cell, &occupancy)
                        && target
                            .step
                            .is_none_or(|step| self.archer_shot_clear(from, step.to, &occupancy))
                } else {
                    target.step.is_none() && self.melee_clear(self.units[unit].cell, target.cell)
                };
                if !ready {
                    if archer {
                        self.reset_windup(unit, id);
                    }
                    return;
                }
                let elapsed = elapsed + remaining;
                if elapsed >= 1.0 {
                    self.animals[animal].health -=
                        stronghold::contact_damage(self.units[unit].kind);
                }
                self.units[unit].action = UnitAction::AttackAnimal {
                    animal_id: id.into(),
                    elapsed_seconds: if elapsed >= 1.0 { 0.0 } else { elapsed },
                };
                if self.animals[animal].health <= 0.0 {
                    self.animal_fell(animal);
                    // Its hunters move on to the next monster in their own sight, if any.
                    for hunter in 0..self.units.len() {
                        if matches!(&self.units[hunter].action, UnitAction::AttackAnimal { animal_id, .. } if animal_id == id)
                        {
                            self.units[hunter].action = match self.next_quarry(hunter) {
                                Some(animal_id) => UnitAction::AttackAnimal {
                                    animal_id,
                                    elapsed_seconds: 0.0,
                                },
                                None => UnitAction::Idle,
                            };
                        }
                    }
                }
            }
            Travel::Unreachable => self.units[unit].action = UnitAction::Idle,
            // An archer that has to move again draws its next shot from scratch.
            Travel::EnRoute if archer => self.reset_windup(unit, id),
            Travel::EnRoute => {}
        }
    }

    fn reset_windup(&mut self, unit: usize, id: &str) {
        self.units[unit].action = UnitAction::AttackAnimal {
            animal_id: id.into(),
            elapsed_seconds: 0.0,
        };
    }

    /// The nearest visible, reachable monster within the unit's own sight, the
    /// way a gatherer moves on to the next node of a patch.
    fn next_quarry(&self, unit: usize) -> Option<String> {
        let visible = self.visible_cells();
        let at = self.units[unit].cell.center();
        self.animals
            .iter()
            .filter(|a| {
                !a.concealed(&self.units)
                    && visible.contains(&a.cell)
                    && a.step.is_none_or(|s| visible.contains(&s.to))
                    && a.cell.center().distance(at) <= UNIT_SIGHT_RADIUS
            })
            .min_by(|a, b| {
                a.cell
                    .center()
                    .distance(at)
                    .total_cmp(&b.cell.center().distance(at))
                    .then_with(|| a.id.cmp(&b.id))
            })
            .filter(|a| self.can_reach_attack(unit, a))
            .map(|a| a.id.clone())
    }

    /// Archers need a reachable cell with a clear shot; everyone else a free
    /// cell beside the animal.
    fn can_reach_attack(&self, unit: usize, animal: &Animal) -> bool {
        if self.units[unit].kind == UnitKind::Archer {
            let occupancy = self.occupancy();
            self.static_paths(unit, &occupancy)
                .nearest(self.archer_positions(animal.cell, &occupancy))
                .is_some()
        } else {
            self.can_reach_beside(unit, animal.footprint())
        }
    }

    pub(super) fn tick_wildlife(&mut self, dt: f64) {
        let fort = self.stronghold_island();
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
            if animal.kind.raids() && !self.garrisoned(fort, animal) {
                self.tick_raider(index, dt);
                continue;
            }
            let home = animal.home;
            let cell = animal.cell;
            let kind = animal.kind;
            let territory = self.territory(fort, animal);
            let sighted = |a: &Animal| {
                self.units.iter().any(|u| {
                    u.health > 0.0
                        && u.cell.center().distance(a.home.center()) <= self.territory(fort, a)
                        && u.cell.center().distance(a.cell.center()) <= a.kind.aggro()
                })
            };
            // One member's sighting sends its whole pack after intruders in reach;
            // one sentry's alarm turns out the whole garrison.
            let garrison = self.garrisoned(fort, animal);
            let alerted = (kind.pack().is_some()
                && self.animals.iter().any(|a| {
                    a.kind.pack() == kind.pack()
                        && a.home.center().distance(home.center()) <= PRIDE_RADIUS
                        && sighted(a)
                }))
                || (garrison
                    && self
                        .animals
                        .iter()
                        .any(|a| self.garrisoned(fort, a) && sighted(a)));
            let nearest = |(_, a): &(usize, &Unit), (_, b): &(usize, &Unit)| {
                a.cell
                    .center()
                    .distance(cell.center())
                    .total_cmp(&b.cell.center().distance(cell.center()))
                    .then_with(|| a.id.cmp(&b.id))
            };
            let target = if kind == AnimalKind::Minotaur {
                // Asleep until the artifact leaves the temple or he is struck;
                // then the bearer first, else anyone on his island.
                let island = self.island_at(home);
                let bearer = self.artifact_bearer.as_deref();
                let awake = bearer.is_some() || animal.health < animal.max_health();
                self.units
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| awake && u.health > 0.0 && self.island_at(u.cell) == island)
                    .min_by(|a, b| {
                        (Some(a.1.id.as_str()) != bearer)
                            .cmp(&(Some(b.1.id.as_str()) != bearer))
                            .then_with(|| nearest(a, b))
                    })
                    .map(|(i, _)| i)
            } else {
                self.units
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| {
                        u.health > 0.0
                            && u.cell.center().distance(home.center()) <= territory
                            && (alerted
                                || u.cell.center().distance(cell.center()) <= kind.aggro()
                                // Anyone attacking it from inside its territory draws it out, so
                                // archers cannot freely shoot an idle animal.
                                || matches!(&u.action, UnitAction::AttackAnimal { animal_id, .. } if animal_id == &animal.id))
                    })
                    .min_by(nearest)
                    .map(|(i, _)| i)
            };
            if let Some(unit) = target {
                let target_cell = self.units[unit].cell;
                // Centaurs shoot from range; everyone else needs clear contact
                // and cannot bite diagonally through two touching obstacles.
                let shot = kind == AnimalKind::Centaur
                    && target_cell.center().distance(cell.center()) <= ARROW_RANGE;
                let contact = self.units[unit].step.is_none()
                    && cell.touches(target_cell)
                    && self.melee_clear(cell, target_cell);
                if shot || contact {
                    self.animals[index].heading = raid::heading(cell, target_cell);
                    self.animals[index].attack_seconds += dt;
                    if self.animals[index].attack_seconds >= 1.0 {
                        let damage = self.animals[index].damage();
                        if matches!(kind, AnimalKind::Cyclops | AnimalKind::CyclopsGeneral) {
                            // The club sweeps every unit standing around him.
                            for u in 0..self.units.len() {
                                let at = self.units[u].cell;
                                if self.units[u].step.is_none()
                                    && cell.touches(at)
                                    && self.melee_clear(cell, at)
                                {
                                    self.units[u].health -= damage;
                                }
                            }
                        } else {
                            self.units[unit].health -= damage;
                        }
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

    /// How far an animal strays from home: its kind's territory, or the
    /// stronghold's ground for a garrison raider.
    pub(super) fn territory(&self, fort: Option<usize>, animal: &Animal) -> f64 {
        if self.garrisoned(fort, animal) {
            stronghold::GARRISON_TERRITORY
        } else {
            animal.kind.territory()
        }
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
        let fort = self.stronghold_island();
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
                // Seven planned sites at most, so ranks stop at six.
                || animal.level > 6
                || !animal.health.is_finite()
                || animal.health <= 0.0
                || animal.health > animal.max_health()
                || !animal.attack_seconds.is_finite()
                || !(0.0..1.0).contains(&animal.attack_seconds)
                || animal.cell.center().distance(animal.home.center())
                    > self.territory(fort, animal)
                || animal.step.is_some_and(|s| {
                    !animal.cell.touches(s.to)
                        || !(0.0..1.0).contains(&s.progress)
                        || s.to.center().distance(animal.home.center())
                            > self.territory(fort, animal)
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
