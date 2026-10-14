//! Lairs on later islands refill: a fallen animal rises again at its home once
//! nobody stands nearby, unless a building has claimed the land around it.
use super::*;

/// Lairs refill from the third island in the danger ranking onwards.
const RESPAWN_LEVEL: u8 = 2;
/// A lair stays empty while a land unit stands this close.
const UNIT_CLEARANCE: f64 = 12.0;
/// A building this close claims the land: its lairs stay empty for good.
const BUILDING_CLEARANCE: f64 = 16.0;

/// A fallen animal waiting to rise again at its home.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Respawn {
    pub id: String,
    pub kind: AnimalKind,
    pub home: CellCoordinate,
    pub level: u8,
    pub seconds_left: f64,
}

impl GameWorld {
    /// Remove a slain animal, queueing its return when its lair refills.
    pub(super) fn animal_fell(&mut self, index: usize) {
        let animal = self.animals.remove(index);
        if animal.level >= RESPAWN_LEVEL
            && let Some(seconds) = animal.kind.respawn_seconds()
        {
            self.respawns.push(Respawn {
                id: animal.id,
                kind: animal.kind,
                home: animal.home,
                level: animal.level,
                seconds_left: seconds,
            });
        }
    }

    pub(super) fn tick_respawns(&mut self, dt: f64) {
        let mut index = 0;
        while index < self.respawns.len() {
            let respawn = &mut self.respawns[index];
            respawn.seconds_left = (respawn.seconds_left - dt).max(0.0);
            if respawn.seconds_left == 0.0 && self.lair_clear(self.respawns[index].home) {
                let Respawn {
                    id,
                    kind,
                    home,
                    level,
                    ..
                } = self.respawns.remove(index);
                self.animals.push(Animal::spawn(id, kind, home, level));
            } else {
                index += 1;
            }
        }
    }

    fn lair_clear(&self, home: CellCoordinate) -> bool {
        self.units
            .iter()
            .all(|u| u.cell.center().distance(home.center()) > UNIT_CLEARANCE)
            && self.buildings.iter().all(|b| {
                b.id == TEMPLE_ID
                    || b.footprint().center().distance(home.center()) > BUILDING_CLEARANCE
            })
            && self.occupancy().is_free_for(home, None)
    }

    pub(super) fn validate_respawns(&self) -> Result<(), String> {
        let mut ids: BTreeSet<&String> = self.animals.iter().map(|a| &a.id).collect();
        for respawn in &self.respawns {
            if !ids.insert(&respawn.id)
                || !self.in_bounds(respawn.home)
                || respawn.level < RESPAWN_LEVEL
                || respawn.kind.respawn_seconds().is_none()
                || !respawn.seconds_left.is_finite()
                || respawn.seconds_left < 0.0
            {
                return Err("invalid respawn".into());
            }
        }
        Ok(())
    }
}
