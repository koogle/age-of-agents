//! Presentation and input for island-local inventories and ship holds.
use super::*;
use aoa_game::{CargoDirection, ResourceKind, island_at};

/// Selected carriers eligible for a manual unload when selecting a building.
/// Gathering loops keep their existing automatic delivery and resumption.
pub(super) fn carriers_for(
    snapshot: Option<&aoa_game::WorldSnapshot>,
    selected: &[String],
    id: &str,
) -> Vec<String> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    let Some(building) = snapshot
        .buildings
        .iter()
        .find(|b| b.building.id == id && b.building.construction.is_none())
    else {
        return Vec::new();
    };
    snapshot
        .units
        .iter()
        .filter(|u| selected.contains(&u.unit.id))
        .filter(|u| !matches!(u.unit.action, aoa_game::UnitAction::Gather { .. }))
        .filter(|u| {
            u.unit
                .cargo
                .as_ref()
                .is_some_and(|cargo| building.building.kind.accepts(cargo.kind))
        })
        .map(|u| u.unit.id.clone())
        .collect()
}

impl App {
    pub(crate) fn select_storage_ship(&mut self, id: String) {
        self.selection.units.clear();
        self.selection.building = None;
        self.selection.ship = Some(id);
    }

    pub(crate) fn update_resource_island(&mut self) {
        let Some(snapshot) = &self.view.snapshot else {
            return;
        };
        let point = if self.hud.covers(self.cursor) {
            self.hud.map_point(self.cursor)
        } else {
            self.rig
                .ground_at(self.cursor, |x, z| self.view.heights.placement_at(x, z))
                .map(|p| Vec2::new(p.x, p.z))
        };
        if let Some(point) = point {
            let cell = aoa_game::CellCoordinate::new(
                (point.x / terrain::CELL).max(0.0) as u16,
                (point.y / terrain::CELL).max(0.0) as u16,
            );
            if let Some(island) = island_at(&snapshot.island_origins, cell) {
                self.resource_island = island;
            }
        }
        self.resource_island = self.resource_island.min(snapshot.inventories.len() - 1);
    }

    pub(crate) fn transfer_cargo(
        &mut self,
        kind: ResourceKind,
        direction: CargoDirection,
        amount: f64,
    ) {
        if let Some(ship_id) = self.selection.ship.clone() {
            self.send(Command::TransferShipCargo {
                ship_id,
                kind,
                amount,
                direction,
            });
        }
    }

    pub(crate) fn order_ship_storage(&mut self, ship_id: &str) {
        for unit_id in self.selection.units.clone() {
            let carrying = self.view.snapshot.as_ref().is_some_and(|s| {
                s.units
                    .iter()
                    .any(|u| u.unit.id == unit_id && u.unit.cargo.is_some())
            });
            self.send(if carrying {
                Command::Deposit {
                    unit_id,
                    storage_id: ship_id.into(),
                }
            } else {
                Command::Board {
                    unit_id,
                    ship_id: ship_id.into(),
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{BuildingKind, CarriedResource, GameWorld, GatherPhase, UnitAction};

    #[test]
    fn building_selection_preserves_every_gathering_phase_in_mixed_groups() {
        let mut snapshot = GameWorld::default().snapshot();
        let building = &mut snapshot.buildings[0].building;
        building.kind = BuildingKind::TownCenter;
        building.construction = None;
        let building_id = building.id.clone();
        let mut carrier = snapshot.units[0].clone();
        carrier.unit.action = UnitAction::Idle;
        carrier.unit.cargo = Some(CarriedResource {
            kind: ResourceKind::Wood,
            amount: 7.0,
        });
        carrier.unit.id = "stopped-carrier".into();
        snapshot.units = vec![carrier.clone()];
        for (i, phase) in [
            GatherPhase::ToResource,
            GatherPhase::Gathering,
            GatherPhase::Returning,
            GatherPhase::Depositing,
        ]
        .into_iter()
        .enumerate()
        {
            let mut gatherer = carrier.clone();
            gatherer.unit.id = format!("gatherer-{i}");
            gatherer.unit.action = UnitAction::Gather {
                resource_id: "wood".into(),
                phase,
            };
            snapshot.units.push(gatherer);
        }
        let selected = snapshot
            .units
            .iter()
            .map(|u| u.unit.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            carriers_for(Some(&snapshot), &selected, &building_id),
            vec![carrier.unit.id.clone()]
        );
        assert!(carriers_for(Some(&snapshot), &selected[1..], &building_id).is_empty());
        assert!(carriers_for(Some(&snapshot), &[], &building_id).is_empty());
        snapshot.buildings[0].building.kind = BuildingKind::Granary;
        assert!(carriers_for(Some(&snapshot), &selected, &building_id).is_empty());
        snapshot.buildings[0].building.kind = BuildingKind::LumberMill;
        assert_eq!(
            carriers_for(Some(&snapshot), &selected, &building_id),
            vec![carrier.unit.id]
        );
        snapshot.units[0].unit.cargo = None;
        assert!(carriers_for(Some(&snapshot), &selected, &building_id).is_empty());
        assert!(carriers_for(None, &selected, &building_id).is_empty());
    }
}
