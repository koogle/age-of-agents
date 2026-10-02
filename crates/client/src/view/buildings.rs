//! Building art uses a fixed ground anchor, shared by real buildings and previews.
use aoa_game::BuildingKind;
use glam::{Vec2, Vec3};

use super::{SHEET_BUILDINGS, SHEET_TOWN_CENTER, Sheets, ground, town_center_frame, uv};
use crate::{render::Sprite, terrain::Heights};

/// Sheet row, width in world units, and footprint centre as a fraction of cell height.
fn art(kind: BuildingKind) -> Option<(&'static str, f32, f32)> {
    Some(match kind {
        BuildingKind::House => ("house", 3.2, 205.0 / 256.0),
        BuildingKind::Granary => ("granary", 3.3, 199.0 / 256.0),
        BuildingKind::Watchtower => ("watchtower", 4.25, 228.0 / 256.0),
        BuildingKind::Dock => ("dock", 3.8, 193.0 / 256.0),
        _ => return None,
    })
}

pub(crate) fn sprite(
    sheets: &Sheets,
    heights: &Heights,
    kind: BuildingKind,
    center: Vec3,
    construction: Option<f64>,
    working: bool,
) -> (usize, Sprite) {
    let (sheet, width, ratio, pivot, rect, atlas, drop) =
        if let Some((row, width, center_y)) = art(kind) {
            let sheet = &sheets.buildings;
            let stage = construction.map_or(3, |work| {
                ((work / aoa_game::BUILD_SECONDS * 3.0) as usize).min(2)
            });
            let base = 248.0 / 256.0;
            (
                SHEET_BUILDINGS,
                width,
                sheet.cell[1] / sheet.cell[0],
                [0.5, 1.0 - base],
                sheet.frames[row][stage],
                sheet.size,
                (base - center_y) * width,
            )
        } else {
            let tc = &sheets.town_center;
            let frame = town_center_frame(tc, construction, working);
            let width = tc.cell[0] * tc.units_per_pixel * kind.size().0 as f32 / 4.0;
            (
                SHEET_TOWN_CENTER,
                width,
                tc.cell[1] / tc.cell[0],
                [
                    tc.anchor[0] / tc.cell[0],
                    1.0 - tc.base_bottom[1] / tc.cell[1],
                ],
                tc.frames[frame],
                tc.size,
                (tc.base_bottom[1] - tc.anchor[1]) / tc.cell[1] * width,
            )
        };
    // Painted front edges extend below the footprint centre. Seat that front
    // edge on the terrain in the fixed isometric direction, never toward the
    // camera's position (which changes when panning and zooming).
    let front = Vec2::splat(std::f32::consts::FRAC_1_SQRT_2) * (drop / crate::camera::PITCH.sin());
    (
        sheet,
        Sprite {
            anchor: ground(heights, center.x + front.x, center.z + front.y).to_array(),
            size: [width, width * ratio],
            pivot,
            uv: uv(rect, atlas, false),
            pull: 0.08 * width,
            tint: [1.0; 4],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Rig;
    use crate::view::{Sheets, footprint_center};

    #[test]
    fn building_geometry_is_fixed_during_camera_navigation() {
        let sheets = Sheets::parse(
            include_bytes!("../../../../assets/sprites/villager.json"),
            include_bytes!("../../../../assets/sprites/villager_idle_hd.json"),
            include_bytes!("../../../../assets/sprites/resources.json"),
            include_bytes!("../../../../assets/sprites/towncenter.json"),
            include_bytes!("../../../../assets/sprites/buildings_hd.json"),
        );
        let heights = Heights::unknown();
        let snapshot = aoa_game::GameWorld::default().snapshot();
        let building = &snapshot.buildings[0];
        let center = footprint_center(&heights, building);
        let mut rig = Rig::new();
        let before = rig.basis();
        let sprites: Vec<_> = aoa_game::BUILDABLE
            .iter()
            .map(|kind| sprite(&sheets, &heights, *kind, center, None, false).1)
            .collect();
        for factor in [0.5, 2.0, 4.0] {
            rig.zoom(factor);
            rig.nudge(1.0, -1.0);
            let after = rig.basis();
            assert!(before.0.abs_diff_eq(after.0, 1e-6));
            assert!(before.1.abs_diff_eq(after.1, 1e-6));
            for (kind, initial) in aoa_game::BUILDABLE.iter().zip(&sprites) {
                let actual = sprite(&sheets, &heights, *kind, center, None, false).1;
                assert_eq!(actual.anchor, initial.anchor);
                assert_eq!(actual.size, initial.size);
                assert_eq!(actual.uv, initial.uv);
            }
        }
    }
}
