use super::*;

impl WorldView {
    pub fn draw_health(
        &self,
        hud: &mut crate::hud::Hud,
        atlas: &crate::hud::Atlas,
        rig: &Rig,
        scale: f32,
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
                    // Only buildings raiders have damaged carry a bar, above the roof.
                    if let Some(b) = snapshot.buildings.iter().find(|b| &b.building.id == id)
                        && b.building.damage > 0.0
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
                aoa_game::AnimalKind::Barbarian | aoa_game::AnimalKind::Chieftain => 0.9,
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
