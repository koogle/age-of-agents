//! Fields reuse the authored HD farm's soil, growing and ripe crop frames.
use super::{Sheets, buildings, ground};
use crate::{
    render::Sprite,
    terrain::{CELL, Heights},
};
use aoa_game::{BuildingKind, FIELD_WORK_SECONDS, ResourceNode};
use glam::Vec2;

pub(super) fn center(resource: &ResourceNode) -> Vec2 {
    let center = resource.footprint().center();
    Vec2::new(center.x as f32, center.y as f32) * CELL
}

pub(super) fn sprite(
    sheets: &Sheets,
    heights: &Heights,
    resource: &ResourceNode,
) -> (usize, Sprite) {
    let field = resource.field.as_ref().expect("cultivated field");
    let construction = if let Some(work) = field.work {
        Some(work / FIELD_WORK_SECONDS * BuildingKind::Farm.build_seconds())
    } else if resource.amount <= 0.0 {
        Some(0.0)
    } else {
        None
    };
    let center = center(resource);
    buildings::sprite(
        sheets,
        heights,
        BuildingKind::Farm,
        ground(heights, center.x, center.y),
        construction,
        false,
    )
}
