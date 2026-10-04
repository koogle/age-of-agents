//! Frame the arriving vessel on a new island, or the settlement on initial load.
use super::*;
impl App {
    pub(super) fn frame_town_center(&mut self) {
        let Some(snapshot) = self.view.snapshot.as_ref() else {
            return;
        };
        if let Some(ship) = snapshot
            .ships
            .iter()
            .find(|s| Some(&s.id) == self.selection.ship.as_ref())
            .or_else(|| snapshot.ships.first())
        {
            let center = ship.cell.center();
            let at = terrain::world_of(center.x, center.y);
            self.rig.look_at(at.x, at.y);
            self.framed = true;
        } else if let Some(building) = snapshot.buildings.first() {
            let center = view::footprint_center(&self.view.heights, building);
            self.rig.look_at(center.x, center.z);
            self.framed = true;
        }
    }
}
