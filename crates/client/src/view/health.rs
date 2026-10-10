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
            let Pick::Animal(id) = &pick.pick else {
                continue;
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
