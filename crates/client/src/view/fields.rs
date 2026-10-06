//! Dedicated field art: cleared soil, tilling, seedlings, and ripe wheat.
use super::{Sheets, buildings, ground};
use crate::{
    render::Sprite,
    terrain::{CELL, Heights},
};
use aoa_game::{FIELD_WORK_SECONDS, ResourceNode};
use glam::{Vec2, Vec3};

pub(super) fn center(resource: &ResourceNode) -> Vec2 {
    let center = resource.footprint().center();
    Vec2::new(center.x as f32, center.y as f32) * CELL
}

fn stage(resource: &ResourceNode) -> usize {
    let field = resource.field.as_ref().expect("cultivated field");
    if let Some(work) = field.work {
        ((work / FIELD_WORK_SECONDS * 3.0) as usize).min(2)
    } else if resource.amount <= 0.0 {
        0
    } else {
        3
    }
}

pub(super) fn sprite(
    sheets: &Sheets,
    heights: &Heights,
    resource: &ResourceNode,
) -> (usize, Sprite) {
    let center = center(resource);
    plot(
        sheets,
        heights,
        ground(heights, center.x, center.y),
        stage(resource),
    )
}

pub(crate) fn preview(sheets: &Sheets, heights: &Heights, center: Vec3) -> (usize, Sprite) {
    plot(sheets, heights, center, 3)
}

fn plot(sheets: &Sheets, heights: &Heights, center: Vec3, stage: usize) -> (usize, Sprite) {
    let (sheet, art) = sheets.catalog.field();
    buildings::on_plot(
        buildings::Frame {
            sheet,
            rect: art.frames["field"][stage],
            atlas: art.size,
            cell: art.cell,
            corners: art.footprints["field"][stage],
            mirror: false,
        },
        heights,
        center,
        Vec2::splat(3.0 * CELL),
        0.94,
    )
}
