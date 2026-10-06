//! Walkable road surfaces and explicitly assigned, paid-once construction.
use super::*;

pub const ROAD_WORK_SECONDS: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoadKind {
    Dirt,
    Stone,
}

impl RoadKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dirt => "Dirt road",
            Self::Stone => "Stone road",
        }
    }
    pub fn stone_per_cell(self) -> f64 {
        match self {
            Self::Dirt => 0.0,
            Self::Stone => 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Road {
    pub cell: CellCoordinate,
    pub kind: RoadKind,
    /// None means complete; unfinished surfaces are walkable at ordinary speed.
    pub work: Option<f64>,
}

impl Road {
    pub fn footprint(&self) -> Footprint {
        Footprint {
            origin: self.cell,
            columns: 1,
            rows: 1,
        }
    }
}

/// Axis-aligned, inclusive line. Bent and diagonal orders are rejected.
pub fn road_line(start: CellCoordinate, end: CellCoordinate) -> Option<Vec<CellCoordinate>> {
    if start.column != end.column && start.row != end.row {
        return None;
    }
    let count = start
        .column
        .abs_diff(end.column)
        .max(start.row.abs_diff(end.row));
    let dx = (i32::from(end.column) - i32::from(start.column)).signum();
    let dy = (i32::from(end.row) - i32::from(start.row)).signum();
    Some(
        (0..=count)
            .map(|n| {
                CellCoordinate::new(
                    (i32::from(start.column) + dx * i32::from(n)) as u16,
                    (i32::from(start.row) + dy * i32::from(n)) as u16,
                )
            })
            .collect(),
    )
}

impl GameWorld {
    pub(super) fn completed_road_cells(&self) -> BTreeSet<CellCoordinate> {
        self.roads
            .iter()
            .filter(|r| r.work.is_none())
            .map(|r| r.cell)
            .collect()
    }

    pub(super) fn road_weight(&self, cell: CellCoordinate) -> u32 {
        if self
            .roads
            .iter()
            .any(|r| r.cell == cell && r.work.is_none())
        {
            2
        } else {
            3
        }
    }

    pub(super) fn next_road(&self, cells: &[CellCoordinate]) -> Option<&Road> {
        cells
            .iter()
            .find_map(|c| self.roads.iter().find(|r| r.cell == *c && r.work.is_some()))
    }

    pub(super) fn order_road(
        &mut self,
        id: &str,
        start: CellCoordinate,
        end: CellCoordinate,
        kind: RoadKind,
    ) -> Result<(), CommandError> {
        let unit = self.ordered_unit(id)?;
        if self.units[unit].kind != UnitKind::Villager {
            return Err(CommandError::VillagerRequired);
        }
        if !self.in_bounds(start) || !self.in_bounds(end) {
            return Err(CommandError::InvalidBuildSite);
        }
        let cells = road_line(start, end).ok_or(CommandError::InvalidBuildSite)?;
        let island =
            island_at(&self.island_origins, start).ok_or(CommandError::InvalidBuildSite)?;
        let visible = self.visible_cells();
        let occupancy = self.occupancy();
        let paths = self.static_paths(unit, &occupancy);
        let mut new = Vec::new();
        for &cell in &cells {
            if island_at(&self.island_origins, cell) != Some(island)
                || !visible.contains(&cell)
                || occupancy.is_static(cell)
            {
                return Err(CommandError::InvalidBuildSite);
            }
            let road = Road {
                cell,
                kind,
                work: Some(0.0),
            };
            if paths
                .nearest(movement::interaction_cells(road.footprint()))
                .is_none()
            {
                return Err(CommandError::TargetUnreachable);
            }
            // Crossings retain existing surfaces and never charge twice.
            if !self.roads.iter().any(|r| r.cell == cell) {
                new.push(road);
            }
        }
        let cost = kind.stone_per_cell() * new.len() as f64;
        if cost > 0.0 {
            self.spend_at(start, &[(ResourceKind::Stone, cost)])?;
        }
        self.roads.extend(new);
        if self.next_road(&cells).is_some() {
            self.units[unit].action = UnitAction::BuildRoad { cells };
        }
        Ok(())
    }

    pub(super) fn tick_road(&mut self, unit: usize, cells: Vec<CellCoordinate>, mut dt: f64) {
        if self.drop_off_before_building(unit, dt) {
            return;
        }
        loop {
            let Some(road) = self.next_road(&cells) else {
                self.units[unit].action = UnitAction::Idle;
                return;
            };
            let cell = road.cell;
            match self.travel(unit, Goal::Beside(road.footprint()), dt) {
                Travel::EnRoute => return,
                Travel::Unreachable => {
                    self.units[unit].action = UnitAction::Idle;
                    return;
                }
                Travel::Arrived { remaining } => dt = remaining,
            }
            let road = self.roads.iter_mut().find(|r| r.cell == cell).unwrap();
            let needed = ROAD_WORK_SECONDS - road.work.unwrap();
            if dt + f64::EPSILON < needed {
                road.work = Some(road.work.unwrap() + dt);
                return;
            }
            road.work = None;
            dt = (dt - needed).max(0.0);
        }
    }

    pub(super) fn validate_roads(&self) -> Result<(), String> {
        let mut cells = BTreeSet::new();
        for road in &self.roads {
            if !self.in_bounds(road.cell)
                || !cells.insert(road.cell)
                || !self.terrain[usize::from(road.cell.row) * usize::from(self.columns())
                    + usize::from(road.cell.column)]
                .biome
                .is_walkable()
                || road
                    .work
                    .is_some_and(|w| !w.is_finite() || !(0.0..ROAD_WORK_SECONDS).contains(&w))
            {
                return Err("invalid road surface or construction progress".into());
            }
        }
        for unit in &self.units {
            if let UnitAction::BuildRoad { cells: line } = &unit.action
                && (line.is_empty()
                    || !line.iter().all(|c| cells.contains(c))
                    || road_line(line[0], *line.last().unwrap()).as_ref() != Some(line))
            {
                return Err("invalid road construction task".into());
            }
        }
        Ok(())
    }
}
