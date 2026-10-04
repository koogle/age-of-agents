//! Unit selection uses the displayed ground anchors, including terrain height.
use super::WorldView;
use crate::camera::Rig;
use glam::Vec2;

impl WorldView {
    pub fn units_in_box(&self, rig: &Rig, from: Vec2, to: Vec2) -> Vec<String> {
        let min = from.min(to).max(Vec2::ZERO);
        let max = from.max(to).min(Vec2::new(rig.width, rig.height));
        let mut ids: Vec<_> = self
            .units
            .iter()
            .filter_map(|(id, unit)| {
                let foot = rig.screen_of(unit.position)?;
                (foot.cmpge(min).all() && foot.cmple(max).all()).then(|| id.clone())
            })
            .collect();
        ids.sort();
        ids
    }
}

#[derive(Default)]
pub struct Selection {
    pub units: Vec<String>,
    pub building: Option<String>,
    pub ship: Option<String>,
    pub cargo_index: usize,
}

impl Selection {
    pub fn select_unit(&mut self, id: String, additive: bool) {
        if !additive {
            self.units.clear();
        }
        if let Some(index) = self.units.iter().position(|unit| unit == &id) {
            self.units.remove(index);
        } else {
            self.units.push(id);
        }
        self.building = None;
        self.ship = None;
    }
}

impl Selection {
    pub fn add_units(&mut self, ids: Vec<String>) {
        if ids.is_empty() {
            return;
        }
        self.building = None;
        self.ship = None;
        for id in ids {
            if !self.units.contains(&id) {
                self.units.push(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::GameWorld;
    use glam::Vec3;

    #[test]
    fn box_uses_projected_feet_in_either_direction_and_excludes_offscreen_units() {
        let mut view = WorldView::new();
        view.sync(GameWorld::default().snapshot());
        let mut rig = Rig::new();
        rig.width = 800.0;
        rig.height = 600.0;
        rig.target = view.units["villager-1"].position;
        view.units.get_mut("villager-2").unwrap().position =
            rig.target + Vec3::new(100.0, 0.0, 0.0);
        let foot = rig.screen_of(rig.target).unwrap();
        let from = foot - Vec2::splat(5.0);
        let to = foot + Vec2::splat(5.0);
        assert_eq!(view.units_in_box(&rig, from, to), ["villager-1"]);
        assert_eq!(view.units_in_box(&rig, to, from), ["villager-1"]);
        assert!(
            view.units_in_box(&rig, from - Vec2::splat(50.0), from)
                .is_empty()
        );
        assert_eq!(
            view.units_in_box(&rig, Vec2::splat(-10000.0), Vec2::splat(10000.0)),
            ["villager-1"]
        );
    }

    #[test]
    fn box_adds_without_toggling_existing_units() {
        let mut selection = Selection {
            building: Some("base-1".into()),
            ..Selection::default()
        };
        selection.add_units(vec![]);
        assert!(selection.building.is_some());
        selection.add_units(vec!["villager-1".into(), "villager-2".into()]);
        selection.add_units(vec!["villager-1".into()]);
        assert_eq!(selection.units, ["villager-1", "villager-2"]);
        assert!(selection.building.is_none());
    }
}
