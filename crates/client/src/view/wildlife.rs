//! Authored wolf and bear frames; snapshots already exclude hidden animals.
use super::*;

pub(super) fn draw(
    snapshot: &WorldSnapshot,
    heights: &Heights,
    time: f32,
    sprites: &mut Vec<(usize, Sprite)>,
    decals: &mut Vec<Decal>,
    picks: &mut Vec<Pickable>,
) {
    for animal in &snapshot.animals {
        let at = animal.position();
        let center = ground(
            heights,
            at.x as f32 * terrain::CELL,
            at.y as f32 * terrain::CELL,
        );
        let bear = animal.kind == aoa_game::AnimalKind::Bear;
        let moving = animal.step.is_some();
        let stride = moving && (time * 5.0) as u32 % 2 == 1;
        let size = if bear { 1.3 } else { 0.95 };
        let mirror = animal.heading[0] < animal.heading[1];
        // Measured opaque paw baselines in the original 627px frames.
        let foot = match (bear, stride) {
            (false, false) => 611.0,
            (false, true) => 589.0,
            (true, false) => 541.0,
            (true, true) => 545.0,
        };
        let sprite = Sprite {
            anchor: center.to_array(),
            size: [size, size],
            pivot: [0.5, 1.0 - foot / 627.0],
            uv: uv(
                [
                    if stride { 0.5 } else { 0.0 },
                    if bear { 0.5 } else { 0.0 },
                    0.5,
                    0.5,
                ],
                [1.0, 1.0],
                mirror,
            ),
            pull: 0.3 * size,
            tint: [1.0; 4],
            footprint: [0.0; 2],
        };
        sprites.push((13, sprite));
        picks.push(Pickable {
            pick: Pick::Animal(animal.id.clone()),
            sprite,
        });
        decals.push(Decal {
            center: [center.x, center.y + 0.02, center.z],
            radius: if bear { 0.35 } else { 0.25 },
            color: [0.8, 0.12, 0.08, 0.7],
            ring: 1.0,
        });
    }
}
