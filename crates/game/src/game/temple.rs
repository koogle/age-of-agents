//! The run's goal: the Sanctuary of the Gods on the final island holds the
//! Artifact of the Gods. A unit claims it there and wins the run by bringing
//! it within three cells of a completed town center on the home island.
use super::movement::{Goal, Travel};
use super::*;

pub const TEMPLE_ID: &str = "temple";
/// The bearer wins within this many cells of a completed home town center.
const HOME_REACH: u16 = 3;

impl GameWorld {
    /// Place the sanctuary near the middle of a freshly generated final island:
    /// the free, fully walkable 5×5 plot nearest the centre with an open ring
    /// around it, so landing parties can always walk up to it.
    pub(super) fn place_temple(&mut self, site: CellCoordinate) {
        let (columns, rows) = BuildingKind::Temple.size();
        let center =
            CellCoordinate::new(site.column + WORLD_COLUMNS / 2, site.row + WORLD_ROWS / 2);
        let walkable = |world: &Self, cell: CellCoordinate| {
            world.in_bounds(cell)
                && world.terrain[usize::from(cell.row) * usize::from(world.columns())
                    + usize::from(cell.column)]
                .biome
                .is_walkable()
        };
        let occupancy = self.occupancy();
        let origin = (site.row + 2..site.row + WORLD_ROWS - rows - 2)
            .flat_map(|row| {
                (site.column + 2..site.column + WORLD_COLUMNS - columns - 2)
                    .map(move |column| CellCoordinate::new(column, row))
            })
            .filter(|&origin| {
                // The plot plus a one-cell ring must be open land.
                let ring = Footprint {
                    origin: CellCoordinate::new(origin.column - 1, origin.row - 1),
                    columns: columns + 2,
                    rows: rows + 2,
                };
                ring.cells()
                    .all(|cell| walkable(self, cell) && occupancy.is_free_for(cell, None))
            })
            .min_by_key(|origin| {
                let dx = u32::from((origin.column + columns / 2).abs_diff(center.column));
                let dy = u32::from((origin.row + rows / 2).abs_diff(center.row));
                (dx * dx + dy * dy, origin.row, origin.column)
            });
        if let Some(origin) = origin {
            self.buildings
                .push(building(BuildingKind::Temple, TEMPLE_ID, origin, None));
        }
    }

    pub(super) fn claim_artifact(
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
        let temple = self
            .buildings
            .iter()
            .find(|b| b.id == building_id && b.kind == BuildingKind::Temple)
            .ok_or(CommandError::BuildingNotFound)?
            .footprint();
        if self.artifact_bearer.is_some() {
            return Err(CommandError::ArtifactUnavailable);
        }
        for id in ids {
            let unit = self
                .units
                .iter()
                .position(|u| &u.id == id)
                .ok_or(CommandError::UnitNotFound)?;
            if !self.can_reach_beside(unit, temple) {
                return Err(CommandError::TargetUnreachable);
            }
        }
        for id in ids {
            let unit = self.ordered_unit(id)?;
            self.units[unit].action = UnitAction::ClaimArtifact {
                building_id: building_id.into(),
            };
        }
        Ok(())
    }

    /// Walk beside the temple; the first unit to arrive takes the artifact.
    pub(super) fn tick_claim_artifact(&mut self, unit: usize, building_id: &str, dt: f64) {
        let Some(temple) = self
            .buildings
            .iter()
            .find(|b| b.id == building_id)
            .map(|b| b.footprint())
        else {
            self.units[unit].action = UnitAction::Idle;
            return;
        };
        match self.travel(unit, Goal::Beside(temple), dt) {
            Travel::EnRoute => {}
            Travel::Unreachable => self.units[unit].action = UnitAction::Idle,
            Travel::Arrived { .. } => {
                if self.artifact_bearer.is_none() {
                    self.artifact_bearer = Some(self.units[unit].id.clone());
                }
                self.units[unit].action = UnitAction::Idle;
            }
        }
    }

    /// A fallen bearer returns the artifact to the temple; a bearer standing
    /// beside a completed town center on the home island wins the run.
    pub(super) fn tick_artifact(&mut self) {
        let Some(bearer) = self.artifact_bearer.clone() else {
            return;
        };
        let aboard = self
            .ships
            .iter()
            .any(|s| s.passengers.iter().any(|u| u.id == bearer));
        let Some(unit) = self.units.iter().find(|u| u.id == bearer) else {
            if !aboard {
                self.artifact_bearer = None;
            }
            return;
        };
        let home = self.buildings.iter().any(|b| {
            let f = b.footprint();
            let gap = |value: u16, start: u16, length: u16| {
                start
                    .saturating_sub(value)
                    .max(value.saturating_sub(start + length - 1))
            };
            b.kind == BuildingKind::TownCenter
                && b.is_complete()
                && self.island_at(b.origin) == Some(0)
                && gap(unit.cell.column, f.origin.column, f.columns).max(gap(
                    unit.cell.row,
                    f.origin.row,
                    f.rows,
                )) <= HOME_REACH
        });
        if home && self.scenario.outcome != ScenarioOutcome::Won {
            self.scenario.outcome = ScenarioOutcome::Won;
            self.scenario.objective_progress = ScenarioObjectiveProgress {
                completed: 1,
                total: 1,
            };
        }
    }

    pub(super) fn validate_artifact(&self) -> Result<(), String> {
        let temples = self
            .buildings
            .iter()
            .filter(|b| b.kind == BuildingKind::Temple)
            .count();
        let bearer_exists = self.artifact_bearer.as_ref().is_none_or(|id| {
            self.units.iter().any(|u| &u.id == id)
                || self
                    .ships
                    .iter()
                    .any(|s| s.passengers.iter().any(|u| &u.id == id))
        });
        if temples > 1 || (temples == 0 && self.artifact_bearer.is_some()) || !bearer_exists {
            return Err("invalid temple or artifact bearer".into());
        }
        Ok(())
    }
}
