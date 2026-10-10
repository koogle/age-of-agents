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
        // The front view points screen-left; the reverse view points right.
        let mirror = if rear { dx < dy } else { dx > dy };
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
            // The painted hull extends toward the viewer below its waterline
            // anchor. Give it the same footprint depth as other billboards,
            // so the depth-writing sea does not slice off the lower planks.
            pull: 0.3 * 1.6,
            tint: [1.0; 4],
            footprint: [0.0; 2],
            base: [0.0; 2],
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

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{CellCoordinate, GameWorld, TransportShip};

    fn sprite(heading: [i8; 2]) -> Sprite {
        let mut snapshot = GameWorld::default().snapshot();
        snapshot.ships = vec![TransportShip {
            id: "boat".into(),
            cell: CellCoordinate::new(0, 0),
            step: None,
            destination: None,
            heading,
            passengers: vec![],
            cargo: Default::default(),
            home_dock_id: None,
        }];
        let mut sprites = Vec::new();
        draw(
            &snapshot,
            &Selection::default(),
            &mut sprites,
            &mut Vec::new(),
            &mut Vec::new(),
        );
        sprites[0].1
    }

    #[test]
    fn painted_bow_faces_the_projected_travel_direction() {
        let (right, _) = crate::camera::Rig::new().basis();
        for heading in [[1, 0], [0, 1], [-1, 0], [0, -1], [1, -1], [-1, 1]] {
            let sprite = sprite(heading);
            let rear = sprite.uv[0].min(sprite.uv[2]) >= 0.5;
            let mirrored = sprite.uv[0] > sprite.uv[2];
            let bow_points_right = rear != mirrored;
            let travel_right = right.x * heading[0] as f32 + right.z * heading[1] as f32 > 0.0;
            assert_eq!(bow_points_right, travel_right, "heading {heading:?}");
        }
    }

    #[test]
    fn full_painted_hull_is_in_front_of_the_highest_sea_crest() {
        let sprite = sprite([1, 0]);
        let (right, up) = crate::camera::Rig::new().basis();
        let toward_camera = right.cross(up).normalize();
        // The packer registers the opaque keel at row 480 of each 512px frame.
        // The billboard shader uses a camera-facing plane at the pulled depth.
        let bottom = 1.0 - 480.0 / 512.0;
        let keel_height = sprite.anchor[1]
            + (bottom - sprite.pivot[1]) * sprite.size[1] * up.y
            + sprite.pull * toward_camera.y;
        // sea.wgsl combines two waves, each with a 0.025-unit amplitude.
        assert!(keel_height > terrain::SEA_LEVEL + 0.05);
    }
}
