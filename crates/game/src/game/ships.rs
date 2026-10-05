//! Explicit transport. Passengers live in exactly one collection: land units
//! or a ship manifest. Passengers can land at clear shore without a dock.
use super::*;
use crate::navigation::PathTree;
use movement::interaction_cells;

pub const TRANSPORT_PASSENGERS: usize = 4;
const SAIL_SPEED: f64 = 4.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportShip {
    pub id: String,
    pub cell: CellCoordinate,
    pub step: Option<Step>,
    pub destination: Option<CellCoordinate>,
    /// Last travel vector, retained when stopped for sprite facing.
    pub heading: [i8; 2],
    pub passengers: Vec<Unit>,
    /// The dock that built this ship; older saves learn it when departing a dock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_dock_id: Option<String>,
}

impl TransportShip {
    pub fn position(&self) -> Position {
        let from = self.cell.center();
        self.step.map_or(from, |s| Position {
            x: from.x + (s.to.center().x - from.x) * s.progress,
            y: from.y + (s.to.center().y - from.y) * s.progress,
        })
    }
    pub fn stopped(&self) -> bool {
        self.step.is_none() && self.destination.is_none()
    }
    pub fn beside(&self, footprint: Footprint) -> bool {
        footprint
            .cells()
            .any(|c| c.column.abs_diff(self.cell.column) + c.row.abs_diff(self.cell.row) == 1)
    }
    pub fn footprint(&self) -> Footprint {
        Footprint {
            origin: self.cell,
            columns: 1,
            rows: 1,
        }
    }
}

impl GameWorld {
    pub(super) fn ship_index(&self, id: &str) -> Result<usize, CommandError> {
        self.ships
            .iter()
            .position(|s| s.id == id)
            .ok_or(CommandError::ShipNotFound)
    }
    fn water(&self, cell: CellCoordinate) -> bool {
        occupancy::in_bounds(cell)
            && self.terrain
                [usize::from(cell.row) * usize::from(WORLD_COLUMNS) + usize::from(cell.column)]
            .biome
                == TerrainBiome::Water
    }
    pub(super) fn water_free(&self, cell: CellCoordinate, ship: Option<usize>) -> bool {
        self.water(cell)
            && self.ships.iter().enumerate().all(|(i, s)| {
                Some(i) == ship
                    || (s.cell != cell
                        && s.step.is_none_or(|step| step.to != cell)
                        && s.destination != Some(cell))
            })
    }
    fn sea_paths(&self, ship: usize, from: CellCoordinate) -> PathTree {
        PathTree::search(WORLD_COLUMNS, WORLD_ROWS, from, |c| {
            self.water_free(c, Some(ship))
        })
    }
    pub(super) fn spawn_ship(&mut self, building: usize) -> bool {
        let Some(cell) = interaction_cells(self.buildings[building].footprint()).find(|&c| {
            self.water_free(c, None)
                && self.buildings[building]
                    .footprint()
                    .cells()
                    .any(|land| land.column.abs_diff(c.column) + land.row.abs_diff(c.row) == 1)
        }) else {
            return false;
        };
        self.ships.push(TransportShip {
            id: format!("transport-{}", self.next_unit_id),
            cell,
            step: None,
            destination: None,
            heading: [1, 0],
            passengers: Vec::new(),
            home_dock_id: Some(self.buildings[building].id.clone()),
        });
        self.next_unit_id += 1;
        if self.islands.is_empty() {
            self.discover_island();
        }
        true
    }
    pub(super) fn sail(&mut self, id: &str, to: CellCoordinate) -> Result<(), CommandError> {
        let i = self.ship_index(id)?;
        if !self.water_free(to, Some(i)) {
            return Err(CommandError::DestinationOccupied);
        }
        let from = self.ships[i].step.map_or(self.ships[i].cell, |s| s.to);
        if self.sea_paths(i, from).cost(to).is_none() {
            return Err(CommandError::TargetUnreachable);
        }
        self.ships[i].destination = (from != to).then_some(to);
        // Departure cancels outstanding boarding orders without touching cargo.
        for unit in &mut self.units {
            if matches!(&unit.action, UnitAction::Board { ship_id } if ship_id == id) {
                unit.action = UnitAction::Idle;
            }
        }
        Ok(())
    }
    pub(super) fn sail_to_dock(&mut self, id: &str, building_id: &str) -> Result<(), CommandError> {
        let ship = self.ship_index(id)?;
        let dock = self
            .buildings
            .iter()
            .find(|b| b.id == building_id && b.kind == BuildingKind::Dock && b.is_complete())
            .ok_or(CommandError::DockRequired)?;
        let from = self.ships[ship]
            .step
            .map_or(self.ships[ship].cell, |step| step.to);
        let goals = interaction_cells(dock.footprint()).filter(|&c| {
            self.water_free(c, Some(ship))
                && dock
                    .footprint()
                    .cells()
                    .any(|land| land.column.abs_diff(c.column) + land.row.abs_diff(c.row) == 1)
        });
        let to = self
            .sea_paths(ship, from)
            .nearest(goals)
            .ok_or(CommandError::TargetUnreachable)?;
        self.sail(id, to)
    }
    pub(super) fn tick_ships(&mut self, dt: f64) {
        for i in 0..self.ships.len() {
            let mut remaining = dt;
            while remaining > 0.0 {
                if self.ships[i].step.is_none() {
                    let Some(to) = self.ships[i].destination else {
                        break;
                    };
                    if to == self.ships[i].cell {
                        self.ships[i].destination = None;
                        break;
                    }
                    let Some(next) = self.sea_paths(i, self.ships[i].cell).first_step(to) else {
                        break;
                    };
                    let from = self.ships[i].cell;
                    self.ships[i].heading = [
                        (i32::from(next.column) - i32::from(from.column)) as i8,
                        (i32::from(next.row) - i32::from(from.row)) as i8,
                    ];
                    self.ships[i].step = Some(Step {
                        to: next,
                        progress: 0.0,
                    });
                }
                let ship = &mut self.ships[i];
                let step = ship.step.as_mut().unwrap();
                let duration = ship.cell.center().distance(step.to.center()) / SAIL_SPEED;
                let used = remaining.min((1.0 - step.progress) * duration);
                step.progress += used / duration;
                remaining -= used;
                if step.progress >= 1.0 - 1e-9 {
                    ship.cell = step.to;
                    ship.step = None;
                    if ship.destination == Some(ship.cell) {
                        ship.destination = None;
                    }
                } else {
                    break;
                }
            }
        }
    }
    pub(super) fn dock_for_ship(&self, ship: usize) -> Option<&Building> {
        self.buildings.iter().find(|b| {
            b.kind == BuildingKind::Dock
                && b.is_complete()
                && self.ships[ship].beside(b.footprint())
        })
    }
    /// A dock bridges its footprint; elsewhere a ship can land directly at shore.
    fn landing_cells(&self, ship: usize) -> Vec<CellCoordinate> {
        let mut cells: BTreeSet<_> = interaction_cells(self.ships[ship].footprint()).collect();
        if let Some(dock) = self.dock_for_ship(ship) {
            cells.extend(interaction_cells(dock.footprint()));
        }
        cells.into_iter().collect()
    }
    pub(super) fn boarding_goal(&self, unit: usize, id: &str) -> Option<CellCoordinate> {
        let ship = self.ship_index(id).ok()?;
        if !self.ships[ship].stopped() {
            return None;
        }
        let occupancy = self.occupancy();
        let from = self.units[unit]
            .step
            .map_or(self.units[unit].cell, |step| step.to);
        let paths = PathTree::search(WORLD_COLUMNS, WORLD_ROWS, from, |c| !occupancy.is_static(c));
        paths.nearest(
            self.landing_cells(ship)
                .into_iter()
                .filter(|&c| occupancy.is_free_for(c, Some(unit))),
        )
    }
    pub(super) fn board(&mut self, unit_id: &str, ship_id: &str) -> Result<(), CommandError> {
        let ship = self.ship_index(ship_id)?;
        if !self.ships[ship].stopped() {
            return Err(CommandError::ShipMustBeStopped);
        }
        let unit = self.ordered_unit(unit_id)?;
        let waiting = self
            .units
            .iter()
            .filter(|u| matches!(&u.action, UnitAction::Board { ship_id: id } if id == ship_id))
            .count();
        if self.ships[ship].passengers.len() + waiting >= TRANSPORT_PASSENGERS {
            return Err(CommandError::ShipFull);
        }
        if self.boarding_goal(unit, ship_id).is_none() {
            return Err(CommandError::ShoreBlocked);
        }
        self.units[unit].action = UnitAction::Board {
            ship_id: ship_id.into(),
        };
        Ok(())
    }
    pub(super) fn tick_board(&mut self, unit: usize, id: &str, dt: f64) {
        let Some(goal) = self.boarding_goal(unit, id) else {
            self.units[unit].action = UnitAction::Idle;
            return;
        };
        match self.travel(unit, Goal::Cell(goal), dt) {
            Travel::Arrived { .. } => {
                let ship = self.ship_index(id).unwrap();
                let mut passenger = self.units.remove(unit);
                passenger.action = UnitAction::Idle;
                passenger.step = None;
                self.ships[ship].passengers.push(passenger);
            }
            Travel::Unreachable => self.units[unit].action = UnitAction::Idle,
            Travel::EnRoute => {}
        }
    }
    pub(super) fn disembark(&mut self, id: &str) -> Result<(), CommandError> {
        let ship = self.ship_index(id)?;
        if !self.ships[ship].stopped() {
            return Err(CommandError::ShipMustBeStopped);
        }
        let occupancy = self.occupancy();
        let mut cells: Vec<_> = self
            .landing_cells(ship)
            .into_iter()
            .filter(|&c| occupancy.is_free_for(c, None))
            .collect();
        // Spread a full manifest inland from clear shore cells. Only traverse
        // free cardinal neighbors; blocked shore never teleports passengers.
        let mut cursor = 0;
        while cells.len() < self.ships[ship].passengers.len() && cursor < cells.len() {
            let from = cells[cursor];
            for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
                if let Some(next) =
                    crate::navigation::offset(from, dx, dy, WORLD_COLUMNS, WORLD_ROWS)
                    && occupancy.is_free_for(next, None)
                    && !cells.contains(&next)
                {
                    cells.push(next);
                }
            }
            cursor += 1;
        }
        cells.truncate(self.ships[ship].passengers.len());
        if cells.len() != self.ships[ship].passengers.len() {
            return Err(CommandError::ShoreBlocked);
        }
        for (mut unit, cell) in self.ships[ship].passengers.drain(..).zip(cells) {
            unit.cell = cell;
            self.units.push(unit);
        }
        Ok(())
    }
    pub(super) fn validate_ships(&self) -> Result<(), String> {
        let mut ids: BTreeSet<_> = self.units.iter().map(|u| u.id.as_str()).collect();
        let mut claims = BTreeSet::new();
        let mut destinations = BTreeSet::new();
        for ship in &self.ships {
            if !ids.insert(&ship.id)
                || ship.id == format!("transport-{}", self.next_unit_id)
                || !self.water(ship.cell)
                || !claims.insert(ship.cell)
                || ship.heading.iter().any(|v| !(-1..=1).contains(v))
            {
                return Err("invalid transport identity, heading or water claim".into());
            }
            if let Some(step) = ship.step {
                if !ship.cell.touches(step.to)
                    || !self.water(step.to)
                    || !claims.insert(step.to)
                    || !(0.0..1.0).contains(&step.progress)
                {
                    return Err("invalid transport step".into());
                }
                if ship.cell.column != step.to.column
                    && ship.cell.row != step.to.row
                    && (!self.water(CellCoordinate::new(ship.cell.column, step.to.row))
                        || !self.water(CellCoordinate::new(step.to.column, ship.cell.row)))
                {
                    return Err("transport cuts a land corner".into());
                }
            }
            if let Some(to) = ship.destination
                && (!self.water(to) || !destinations.insert(to))
            {
                return Err("invalid transport destination".into());
            }
            let waiting = self
                .units
                .iter()
                .filter(
                    |u| matches!(&u.action, UnitAction::Board { ship_id } if ship_id == &ship.id),
                )
                .count();
            if ship.passengers.len() + waiting > TRANSPORT_PASSENGERS {
                return Err("invalid transport manifest".into());
            }
            for unit in &ship.passengers {
                if !ids.insert(&unit.id)
                    || unit.id == self.next_unit_name()
                    || unit.step.is_some()
                    || unit.action != UnitAction::Idle
                    || !occupancy::in_bounds(unit.cell)
                    || unit
                        .cargo
                        .as_ref()
                        .is_some_and(|c| !(c.amount > 0.0 && c.amount <= VILLAGER_CARRY_CAPACITY))
                {
                    return Err("invalid transport passenger".into());
                }
            }
        }
        for unit in &self.units {
            if let UnitAction::Board { ship_id } = &unit.action
                && !self.ships.iter().any(|s| &s.id == ship_id && s.stopped())
            {
                return Err("boarding a missing or moving ship".into());
            }
        }
        Ok(())
    }
}
