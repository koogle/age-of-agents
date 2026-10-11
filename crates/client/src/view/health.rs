use super::*;

impl WorldView {
    pub fn draw_health(
        &self,
        hud: &mut crate::hud::Hud,
        atlas: &crate::hud::Atlas,
        rig: &Rig,
        scale: f32,
        selection: &Selection,
    ) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let (_, up) = rig.basis();
        for unit in &snapshot.units {
            let Some(entry) = self.units.get(&unit.unit.id) else {
                continue;
            };
            let anchor = ground(&self.heights, entry.position.x, entry.position.z);
            if let Some(top) = rig.screen_offset(anchor, up * VILLAGER_HEIGHT) {
                hud.health_bar(top, (unit.unit.health / 100.0) as f32, scale);
                if snapshot.artifact_bearer.as_ref() == Some(&unit.unit.id) {
                    hud.artifact_marker(atlas, top, scale);
                }
            }
        }
        for pick in &self.pickables {
            let id = match &pick.pick {
                Pick::Animal(id) => id,
                Pick::Building(id) => {
                    // A damaged building shows its bar above the roof only while a
                    // raider stands against it or the player has it selected.
                    if let Some(b) = snapshot.buildings.iter().find(|b| &b.building.id == id)
                        && b.building.damage > 0.0
                        && (selection.building.as_ref() == Some(id) || under_attack(snapshot, b))
                    {
                        let sprite = &pick.sprite;
                        let height = sprite.size[1] * (1.0 - sprite.pivot[1]) * 0.85;
                        if let Some(top) = rig.screen_offset(Vec3::from(sprite.anchor), up * height)
                        {
                            let health = 1.0 - b.building.damage / b.building.kind.max_health();
                            hud.health_bar(top, health as f32, scale);
                        }
                    }
                    continue;
                }
                _ => continue,
            };
            let Some(animal) = snapshot.animals.iter().find(|a| &a.id == id) else {
                continue;
            };
            let sprite = &pick.sprite;
            // Authored species occupy different heights within their atlas cells.
            let top = match animal.kind {
                aoa_game::AnimalKind::Wolf => 0.75,
                aoa_game::AnimalKind::Bear => 0.8,
                aoa_game::AnimalKind::Boar => 0.55,
                aoa_game::AnimalKind::Lioness => 0.75,
                aoa_game::AnimalKind::Lion => 0.84,
                aoa_game::AnimalKind::Python => 0.76,
                aoa_game::AnimalKind::SeaSerpent => 0.48,
                aoa_game::AnimalKind::Viper => 0.43,
                aoa_game::AnimalKind::Barbarian
                | aoa_game::AnimalKind::Chieftain
                | aoa_game::AnimalKind::Torchbearer => 0.9,
            };
            let height = sprite.size[1] * (top - sprite.pivot[1]);
            if let Some(top) = rig.screen_offset(Vec3::from(sprite.anchor), up * height) {
                hud.health_bar(
                    top,
                    (animal.health / animal.kind.max_health()) as f32,
                    scale,
                );
            }
        }
    }
}

/// A raider standing still against the footprint is striking it.
fn under_attack(snapshot: &WorldSnapshot, building: &aoa_game::BuildingView) -> bool {
    let o = building.building.origin;
    let (columns, rows) = (i32::from(building.columns), i32::from(building.rows));
    snapshot.animals.iter().any(|a| {
        let dx = i32::from(a.cell.column) - i32::from(o.column);
        let dy = i32::from(a.cell.row) - i32::from(o.row);
        a.kind.raids()
            && a.step.is_none()
            && (-1..=columns).contains(&dx)
            && (-1..=rows).contains(&dy)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{Animal, AnimalKind, CellCoordinate, GameWorld, Step};

    #[test]
    fn a_building_bar_follows_raiders_striking_it() {
        let snapshot = GameWorld::default().snapshot();
        let building = snapshot.buildings[0].clone();
        let o = building.building.origin;
        let raider = |kind, cell, step| Animal {
            id: "raider-0".into(),
            kind,
            home: cell,
            cell,
            step,
            health: 100.0,
            attack_seconds: 0.0,
            heading: [1, 0],
        };
        let beside = CellCoordinate::new(o.column + building.columns, o.row);
        let far = CellCoordinate::new(o.column + building.columns + 2, o.row);
        let moving = Some(Step {
            to: far,
            progress: 0.5,
        });
        for (animal, attacked) in [
            (raider(AnimalKind::Torchbearer, beside, None), true),
            (raider(AnimalKind::Barbarian, beside, moving), false),
            (raider(AnimalKind::Chieftain, far, None), false),
            (raider(AnimalKind::Wolf, beside, None), false),
        ] {
            let mut snapshot = snapshot.clone();
            snapshot.animals = vec![animal];
            assert_eq!(under_attack(&snapshot, &building), attacked);
        }
    }
}
