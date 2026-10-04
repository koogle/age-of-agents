//! Ship art uses two authored views, mirrored for the other two quadrants.
use super::*;

pub(super) fn draw(
    snapshot: &WorldSnapshot,
    selection: &Selection,
    sprites: &mut Vec<(usize, Sprite)>,
    decals: &mut Vec<Decal>,
    picks: &mut Vec<Pickable>,
) {
    for ship in &snapshot.ships {
        let position = ship.position();
        let anchor = [
            position.x as f32 * terrain::CELL,
            terrain::SEA_LEVEL + 0.06,
            position.y as f32 * terrain::CELL,
        ];
        let [dx, dy] = ship.heading;
        let rear = dx + dy < 0;
        let mirror = dx > dy;
        let sprite = Sprite {
            anchor,
            // Match the modest boat fitted beside the dock sprite.
            size: [1.6, 1.6],
            pivot: [0.5, 0.2],
            uv: uv(
                [if rear { 512.0 } else { 0.0 }, 0.0, 512.0, 512.0],
                [1024.0, 512.0],
                mirror,
            ),
            pull: 0.0,
            tint: [1.0; 4],
            footprint: [0.0; 2],
        };
        picks.push(Pickable {
            pick: Pick::Ship(ship.id.clone()),
            sprite,
        });
        sprites.push((11, sprite));
        if selection.ship.as_deref() == Some(&ship.id) {
            decals.push(Decal {
                center: anchor,
                radius: 0.4,
                color: TEAM_BLUE,
                ring: 3.0,
            });
        }
    }
}
