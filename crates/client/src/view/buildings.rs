//! Structural base corners, rather than image bounds, determine building geometry.
use aoa_game::BuildingKind;
use glam::{Mat2, Vec2, Vec3};

use super::{SHEET_BUILDINGS, SHEET_TOWN_CENTER, Sheets, ground, town_center_frame, uv};
use crate::{
    camera::Rig,
    render::Sprite,
    terrain::{CELL, Heights},
};

type Corners = [[f32; 2]; 4];

fn frame(
    sheets: &Sheets,
    kind: BuildingKind,
    construction: Option<f64>,
    working: bool,
) -> (usize, [f32; 4], [f32; 2], [f32; 2], Corners) {
    if kind == BuildingKind::TownCenter {
        let tc = &sheets.town_center;
        let frame = town_center_frame(tc, construction, working);
        return (
            SHEET_TOWN_CENTER,
            tc.frames[frame],
            tc.size,
            tc.cell,
            tc.footprints[frame],
        );
    }
    let (index, sheet, row) = match kind {
        BuildingKind::House => (SHEET_BUILDINGS, &sheets.buildings, "house"),
        BuildingKind::Granary => (SHEET_BUILDINGS, &sheets.buildings, "granary"),
        BuildingKind::Watchtower => (SHEET_BUILDINGS, &sheets.buildings, "watchtower"),
        BuildingKind::Dock => (SHEET_BUILDINGS, &sheets.buildings, "dock"),
        _ => sheets.catalog.building(kind),
    };
    let stage = construction.map_or(3, |work| {
        ((work / kind.build_seconds() * 3.0) as usize).min(2)
    });
    (
        index,
        sheet.frames[row][stage],
        sheet.size,
        sheet.cell,
        sheet.footprints[row][stage],
    )
}

pub(crate) fn sprite(
    sheets: &Sheets,
    heights: &Heights,
    kind: BuildingKind,
    center: Vec3,
    construction: Option<f64>,
    working: bool,
) -> (usize, Sprite) {
    let (sheet, rect, atlas, cell, corners) = frame(sheets, kind, construction, working);
    let (columns, rows) = kind.size();
    let (width, depth) = (columns as f32 * CELL, rows as f32 * CELL);
    let (right, up) = Rig::new().basis();
    let project = |p: Vec3| Vec2::new(p.dot(right), p.dot(up));
    let ground_projection = Mat2::from_cols(project(Vec3::X), project(Vec3::Z));
    // Measure the painted base in ground coordinates, then fit its bounding
    // rectangle uniformly. The image keeps its original angles and proportions.
    let points = corners.map(|[x, y]| ground_projection.inverse() * Vec2::new(x, -y));
    let low = points.into_iter().reduce(Vec2::min).unwrap();
    let high = points.into_iter().reduce(Vec2::max).unwrap();
    // Initial construction plots fill the claim, independently of the smaller
    // finished house. Later walls/roofs and completed placement ghosts retain
    // the approved building scale.
    let foundation = construction.is_some_and(|work| {
        work < kind.build_seconds()
            * if kind == BuildingKind::TownCenter {
                0.15
            } else {
                1.0 / 3.0
            }
    });
    let plot_fill = if foundation {
        1.0
    } else if kind == BuildingKind::House {
        0.72
    } else {
        0.94
    };
    let scale = (Vec2::new(width, depth) / (high - low)).min_element() * plot_fill;
    let base_center = ground_projection * ((low + high) * 0.5);
    // Take depth from the plot's front, while centering the unmodified art on
    // the plot. This keeps the front wall clear of the ground's depth buffer.
    let anchor = ground(heights, center.x + width * 0.5, center.z + depth * 0.5);
    let pixel_anchor = base_center + project(anchor - center) / scale;
    (
        sheet,
        Sprite {
            anchor: anchor.to_array(),
            size: [cell[0] * scale, cell[1] * scale],
            pivot: [pixel_anchor.x / cell[0], 1.0 + pixel_anchor.y / cell[1]],
            uv: uv(rect, atlas, false),
            pull: 0.08 * width,
            tint: [1.0; 4],
            footprint: [width, depth],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Rig;
    use crate::view::{Sheets, footprint_center};

    fn sheets() -> Sheets {
        Sheets::parse(
            include_bytes!("../../../../assets/sprites/villager.json"),
            include_bytes!("../../../../assets/sprites/villager_idle_hd.json"),
            include_bytes!("../../../../assets/sprites/resources.json"),
            include_bytes!("../../../../assets/sprites/towncenter.json"),
            include_bytes!("../../../../assets/sprites/buildings_hd.json"),
            [
                include_bytes!("../../../../assets/sprites/buildings_economy.json"),
                include_bytes!("../../../../assets/sprites/buildings_crafts.json"),
                include_bytes!("../../../../assets/sprites/buildings_civic.json"),
                include_bytes!("../../../../assets/sprites/units.json"),
            ],
        )
    }

    #[test]
    fn every_stage_preserves_art_proportions_and_fits_the_plot() {
        let sheets = sheets();
        let mut heights = Heights::unknown();
        let mut rig = Rig::new();
        rig.width = 1200.0;
        rig.height = 900.0;
        let (right, up) = rig.basis();
        let project = |p: Vec3| Vec2::new(p.dot(right), p.dot(up));
        let ground_inverse = Mat2::from_cols(project(Vec3::X), project(Vec3::Z)).inverse();
        for kind in aoa_game::BUILDABLE {
            let (columns, rows) = kind.size();
            let extent = Vec2::new(columns as f32, rows as f32) * CELL;
            heights.set_plots(vec![[10.0, 12.0, extent.x, extent.y]], false);
            let center = ground(&heights, 10.0 + extent.x * 0.5, 12.0 + extent.y * 0.5);
            for (work, working) in [
                (Some(0.0), false),
                (Some(kind.build_seconds() * 0.34), false),
                (Some(kind.build_seconds() * 0.68), false),
                (None, false),
                (None, true),
            ] {
                let art = sprite(&sheets, &heights, kind, center, work, working).1;
                let (_, _, _, cell, corners) = frame(&sheets, kind, work, working);
                assert!(
                    (art.size[0] / cell[0] - art.size[1] / cell[1]).abs() < 1e-6,
                    "{kind:?} must use the same scale on both image axes"
                );
                for [x, y] in corners {
                    let q = Vec2::new(x / cell[0], 1.0 - y / cell[1]) - Vec2::from(art.pivot);
                    let offset =
                        project(Vec3::from(art.anchor) - center) + q * Vec2::from(art.size);
                    let ground_offset = ground_inverse * offset;
                    assert!(
                        ground_offset
                            .abs()
                            .cmple(extent * 0.5 + Vec2::splat(1e-5))
                            .all(),
                        "{kind:?} {work:?}: base corner {ground_offset:?} outside plot {extent:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn house_foundation_reaches_plot_edges_while_finished_house_stays_small() {
        let sheets = sheets();
        let heights = Heights::unknown();
        let center = ground(&heights, 12.0, 12.0);
        let (right, up) = Rig::new().basis();
        let project = |p: Vec3| Vec2::new(p.dot(right), p.dot(up));
        let inverse = Mat2::from_cols(project(Vec3::X), project(Vec3::Z)).inverse();
        for (work, expected_fill) in [(Some(0.0), 1.0), (None, 0.72)] {
            let art = sprite(&sheets, &heights, BuildingKind::House, center, work, false).1;
            let (_, _, _, cell, corners) = frame(&sheets, BuildingKind::House, work, false);
            let max_extent = corners
                .into_iter()
                .map(|[x, y]| {
                    let q = Vec2::new(x / cell[0], 1.0 - y / cell[1]) - Vec2::from(art.pivot);
                    let offset =
                        project(Vec3::from(art.anchor) - center) + q * Vec2::from(art.size);
                    (inverse * offset).abs().max_element() / (3.0 * CELL * 0.5)
                })
                .fold(0.0_f32, f32::max);
            assert!((max_extent - expected_fill).abs() < 1e-5);
        }
    }

    #[test]
    fn building_geometry_is_fixed_during_camera_navigation() {
        let sheets = sheets();
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
