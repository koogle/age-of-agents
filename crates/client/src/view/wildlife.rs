//! Authored wolf, bear and boar frames; snapshots already exclude hidden animals.
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
        let (row, size, radius) = match animal.kind {
            aoa_game::AnimalKind::Wolf => (0, 0.95, 0.25),
            aoa_game::AnimalKind::Bear => (1, 1.3, 0.35),
            aoa_game::AnimalKind::Boar => (2, 0.95, 0.25),
        };
        let moving = animal.step.is_some();
        let stride = moving && (time * 5.0) as u32 % 2 == 1;
        // The authoritative contact timer freezes with pause and resets on retreat.
        // Its reset after a hit naturally gives one idle/recovery beat.
        let frame = if !moving && animal.attack_seconds > 0.0 {
            if animal.attack_seconds < 0.6 { 2 } else { 3 }
        } else {
            usize::from(stride)
        };
        let mirror = animal.heading[0] < animal.heading[1];
        // Attacks register the planted rear paw against this idle baseline.
        let foot = 590.0;
        let sprite = Sprite {
            anchor: center.to_array(),
            size: [size, size],
            pivot: [0.5, 1.0 - foot / 627.0],
            uv: uv(
                [frame as f32 * 627.0, row as f32 * 627.0, 627.0, 627.0],
                [2508.0, 1881.0],
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
            radius,
            color: [0.8, 0.12, 0.08, 0.7],
            ring: 1.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{AnimalKind, GameWorld, Step};

    #[test]
    fn combat_poses_follow_snapshot_phase_and_use_the_authored_cells() {
        let world = GameWorld::default();
        let mut snapshot = world.snapshot();
        snapshot.animals = world.animals.clone();
        snapshot.animals.truncate(1);
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../../../assets/sprites/wildlife.json")).unwrap();
        for (kind, name) in [
            (AnimalKind::Wolf, "wolf"),
            (AnimalKind::Bear, "bear"),
            (AnimalKind::Boar, "boar"),
        ] {
            for heading in [[1, 0], [-1, 0]] {
                for (phase, pose) in [
                    (0.0, "idle"),
                    (0.3, "attack_windup"),
                    (0.8, "attack_strike"),
                ] {
                    snapshot.animals[0].kind = kind;
                    snapshot.animals[0].heading = heading;
                    snapshot.animals[0].attack_seconds = phase;
                    snapshot.animals[0].step = None;
                    let rect: [f32; 4] = serde_json::from_value(
                        manifest["frames"][format!("{name}_{pose}")].clone(),
                    )
                    .unwrap();
                    let size: [f32; 2] = serde_json::from_value(manifest["size"].clone()).unwrap();
                    // A frozen snapshot must retain its attack pose even as wall time passes.
                    for time in [0.0, 0.25, 9.0] {
                        let mut sprites = Vec::new();
                        draw(
                            &snapshot,
                            &Heights::unknown(),
                            time,
                            &mut sprites,
                            &mut Vec::new(),
                            &mut Vec::new(),
                        );
                        assert_eq!(sprites[0].1.uv, uv(rect, size, heading[0] < heading[1]));
                    }
                }
            }
        }
        // Starting a step must select locomotion rather than a stale combat phase.
        let animal = &mut snapshot.animals[0];
        animal.step = Some(Step {
            to: animal.home,
            progress: 0.2,
        });
        animal.attack_seconds = 0.8;
        animal.heading = [1, 0];
        let mut sprites = Vec::new();
        draw(
            &snapshot,
            &Heights::unknown(),
            0.25,
            &mut sprites,
            &mut Vec::new(),
            &mut Vec::new(),
        );
        assert_eq!(sprites[0].1.uv, [0.25, 1254.0 / 1881.0, 0.5, 1.0]);
    }
}
