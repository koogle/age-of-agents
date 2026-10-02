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
    let row = match kind {
        BuildingKind::House => "house",
        BuildingKind::Granary => "granary",
        BuildingKind::Watchtower => "watchtower",
        BuildingKind::Dock => "dock",
        _ => {
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
    };
    let sheet = &sheets.buildings;
    let stage = construction.map_or(3, |work| {
        ((work / aoa_game::BUILD_SECONDS * 3.0) as usize).min(2)
    });
    (
        SHEET_BUILDINGS,
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
    // Source coordinates are measured at structural wall/platform corners.
    // Pots, steps and scaffolding may overhang; they never change the plot.
    let [left, front, right, rear] =
        corners.map(|[x, y]| Vec2::new(x / cell[0], 1.0 - y / cell[1]));
    let (columns, rows) = kind.size();
    let (width, depth) = (columns as f32 * CELL, rows as f32 * CELL);
    let anchor = ground(heights, center.x + width * 0.5, center.z + depth * 0.5);
    let ground_left = ground(heights, anchor.x - width, anchor.z);
    let ground_right = ground(heights, anchor.x, anchor.z - depth);
    let (camera_right, camera_up) = Rig::new().basis();
    let project = |p: Vec3| Vec2::new(p.dot(camera_right), p.dot(camera_up));
    let ground_rear = ground(heights, anchor.x - width, anchor.z - depth);
    let source_inverse = Mat2::from_cols(left - front, right - front).inverse();
    let target = Mat2::from_cols(
        project(ground_left - anchor),
        project(ground_right - anchor),
    );
    // Map all four corners, including the rear of an exposed foundation.
    // The two coefficients correct perspective and non-planar ground; zero
    // recovers the ordinary affine mapping on a perfectly rectangular source.
    let source_rear = source_inverse * (rear - front);
    let target_rear = target.inverse() * project(ground_rear - anchor);
    let (a, b, c, d) = (source_rear.x, source_rear.y, target_rear.x, target_rear.y);
    let correction = Mat2::from_cols(
        Vec2::new(a * (1.0 - c), -d * a),
        Vec2::new(-c * b, b * (1.0 - d)),
    )
    .inverse()
        * (target_rear - source_rear);
    let transform = Mat2::from_cols(
        target.x_axis * (1.0 + correction.x),
        target.y_axis * (1.0 + correction.y),
    ) * source_inverse;
    let warp = source_inverse.transpose() * correction;
    (
        sheet,
        Sprite {
            anchor: anchor.to_array(),
            size: [transform.x_axis.x, transform.y_axis.y],
            shear: [transform.y_axis.x, transform.x_axis.y],
            warp: warp.to_array(),
            pivot: front.to_array(),
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

    fn sheets() -> Sheets {
        Sheets::parse(
            include_bytes!("../../../../assets/sprites/villager.json"),
            include_bytes!("../../../../assets/sprites/villager_idle_hd.json"),
            include_bytes!("../../../../assets/sprites/resources.json"),
            include_bytes!("../../../../assets/sprites/towncenter.json"),
            include_bytes!("../../../../assets/sprites/buildings_hd.json"),
        )
    }

    fn painted_corners(
        rig: &Rig,
        sheets: &Sheets,
        heights: &Heights,
        kind: BuildingKind,
        center: Vec3,
        work: Option<f64>,
        working: bool,
    ) -> [Vec2; 4] {
        let art = sprite(sheets, heights, kind, center, work, working).1;
        let (_, _, _, cell, corners) = frame(sheets, kind, work, working);
        let (right, up) = rig.basis();
        corners.map(|[x, y]| {
            let q = Vec2::new(x / cell[0], 1.0 - y / cell[1]) - Vec2::from(art.pivot);
            rig.screen_of(
                Vec3::from(art.anchor)
                    + (right * (q.x * art.size[0] + q.y * art.shear[0])
                        + up * (q.y * art.size[1] + q.x * art.shear[1]))
                        / (1.0 + q.dot(Vec2::from(art.warp))),
            )
            .unwrap()
        })
    }

    #[test]
    fn every_stage_pins_painted_base_corners_to_authoritative_plot() {
        let sheets = sheets();
        // Uneven elevations exercise the vertical offset and affine correction.
        let heights = Heights::from_cells(
            (0..2400).map(|i| (Some(0.25 + ((i % 60) as f32 * 0.08).sin() * 0.12), None)),
        );
        let mut rig = Rig::new();
        rig.width = 1200.0;
        rig.height = 900.0;
        for kind in aoa_game::BUILDABLE {
            let (columns, rows) = kind.size();
            let (width, depth) = (columns as f32 * CELL, rows as f32 * CELL);
            let center = ground(&heights, 10.0 + width * 0.5, 12.0 + depth * 0.5);
            let corners = [
                (10.0, 12.0 + depth),
                (10.0 + width, 12.0 + depth),
                (10.0 + width, 12.0),
                (10.0, 12.0),
            ];
            for (work, working) in [
                (Some(0.0), false),
                (Some(aoa_game::BUILD_SECONDS * 0.34), false),
                (Some(aoa_game::BUILD_SECONDS * 0.68), false),
                (None, false),
                (None, true),
            ] {
                for factor in [0.5, 1.0, 2.0] {
                    rig.zoom(factor);
                    rig.nudge(1.0, -1.0);
                    let actual =
                        painted_corners(&rig, &sheets, &heights, kind, center, work, working);
                    for (actual, (x, z)) in actual.into_iter().zip(corners) {
                        let expected = rig.screen_of(ground(&heights, x, z)).unwrap();
                        assert!(
                            actual.distance(expected) < 0.002,
                            "{kind:?} {work:?}: painted {actual:?}, grid {expected:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn adjacent_houses_share_painted_wall_endpoints_on_both_axes() {
        let sheets = sheets();
        let heights = Heights::unknown();
        let mut rig = Rig::new();
        rig.width = 1200.0;
        rig.height = 900.0;
        let size = BuildingKind::House.size().0 as f32 * CELL;
        let center = ground(&heights, 10.0, 12.0);
        let [left, front, right, _] = painted_corners(
            &rig,
            &sheets,
            &heights,
            BuildingKind::House,
            center,
            None,
            false,
        );
        let along_x = painted_corners(
            &rig,
            &sheets,
            &heights,
            BuildingKind::House,
            center + Vec3::X * size,
            None,
            false,
        );
        let along_z = painted_corners(
            &rig,
            &sheets,
            &heights,
            BuildingKind::House,
            center + Vec3::Z * size,
            None,
            false,
        );
        assert!(along_x[0].distance(front) < 0.002);
        assert!(along_z[2].distance(front) < 0.002);
        assert!(along_x[3].distance(right) < 0.002);
        assert!(along_z[3].distance(left) < 0.002);
        assert!(right.x > left.x);
    }

    #[test]
    fn island_build_sites_keep_sprite_projection_finite() {
        let sheets = sheets();
        for seed in [0, 42, 731] {
            let snapshot = aoa_game::GameWorld::generate(seed).snapshot();
            let mut heights =
                Heights::from_cells(snapshot.terrain.iter().map(|c| (c.elevation, c.biome)));
            for kind in aoa_game::BUILDABLE {
                let (columns, rows) = kind.size();
                for row in 1..snapshot.rows - rows {
                    for column in 1..snapshot.columns - columns {
                        if !(row..row + rows).all(|r| {
                            (column..column + columns).all(|c| {
                                let cell = &snapshot.terrain
                                    [r as usize * snapshot.columns as usize + c as usize];
                                cell.elevation.is_some()
                                    && cell.biome.is_some_and(|b| b.is_walkable())
                            })
                        }) {
                            continue;
                        }
                        let center = ground(
                            &heights,
                            (column as f32 + columns as f32 * 0.5) * CELL,
                            (row as f32 + rows as f32 * 0.5) * CELL,
                        );
                        heights.set_plots(
                            vec![[
                                column as f32 * CELL,
                                row as f32 * CELL,
                                columns as f32 * CELL,
                                rows as f32 * CELL,
                            ]],
                            false,
                        );
                        for work in [Some(0.0), None] {
                            let art = sprite(&sheets, &heights, kind, center, work, false).1;
                            for q in [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE] {
                                let denominator =
                                    1.0 + (q - Vec2::from(art.pivot)).dot(Vec2::from(art.warp));
                                assert!(
                                    denominator.is_finite() && denominator > 0.0,
                                    "folded sprite: seed {seed}, {kind:?}, {column}/{row}, {work:?}, {denominator}"
                                );
                            }
                        }
                    }
                }
            }
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
                assert_eq!(actual.shear, initial.shear);
                assert_eq!(actual.warp, initial.warp);
            }
        }
    }
}
