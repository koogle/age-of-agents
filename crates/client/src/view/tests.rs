use super::*;
#[test]
fn drawn_direction_flips_only_past_the_margin() {
    let front_left = View {
        toward_viewer: true,
        screen_right: false,
    };
    // A heading wobbling just across the left/right boundary keeps its side.
    assert_eq!(front_left.follow(0.2, 0.9), front_left);
    assert_eq!(front_left.follow(-0.2, -0.2), front_left);
    // Clearly across, it flips.
    let back_right = front_left.follow(0.7, -0.7);
    assert!(back_right.screen_right && !back_right.toward_viewer);
    assert_eq!(back_right.follow(-0.2, 0.2), back_right);
}

#[test]
fn walking_uses_only_two_stride_pictures_in_every_direction() {
    let sheet: VillagerSheet =
        serde_json::from_str(include_str!("../../../../assets/sprites/villager.json")).unwrap();
    for activity in ["walk", "carry"] {
        for frames in sheet.animations[activity].values() {
            assert_ne!(frames[0], frames[frames.len() / 2]);
            for step in 0..20 {
                let distance = (step as f32 + 0.1) * STRIDE_DISTANCE;
                assert_eq!(
                    walking_frame(distance, frames.len()),
                    (step % 2) * (frames.len() / 2)
                );
            }
        }
    }
}

#[test]
fn displayed_movement_starts_walking_immediately_and_stops_without_a_tail() {
    let initial = GameWorld::default().snapshot();
    for direction in [-1.0, 1.0] {
        let mut view = WorldView::new();
        view.sync(initial.clone());
        let mut moved = initial.clone();
        moved.tick = 1;
        moved.units[0].position.y += direction;
        view.sync(moved);
        view.frame(0.3);
        let entry = &view.units["villager-1"];
        assert_eq!(entry.velocity.z.signum(), direction as f32);
        assert!(entry.walked > 0.0);
        let walked = entry.walked;
        view.frame(0.016);
        let entry = &view.units["villager-1"];
        assert_eq!(entry.velocity, Vec3::ZERO);
        assert_eq!(entry.walked, walked);
        // Camera motion never advances a unit's gait.
        let mut rig = Rig::new();
        rig.nudge(1.0, 1.0);
        view.frame(0.1);
        assert_eq!(view.units["villager-1"].walked, walked);
    }
}

#[test]
fn villagers_on_a_new_island_follow_its_snapshots() {
    let mut view = WorldView::new();
    let mut old = GameWorld::default().snapshot();
    for tick in 500..506 {
        old.tick = tick;
        view.sync(old.clone());
    }
    view.frame(1.0);
    let mut fresh = GameWorld::generate(9).snapshot();
    assert!(view.sync(fresh.clone()));
    for _ in 0..4 {
        fresh.tick += 1;
        fresh.units[0].position.y += 1.0;
        view.sync(fresh.clone());
        view.frame(0.1);
    }
    for _ in 0..20 {
        view.frame(0.1);
    }
    let at = &fresh.units[0].position;
    let at = terrain::world_of(at.x, at.y);
    let shown = view.units["villager-1"].position;
    assert!((shown.x - at.x).abs() < 1e-4 && (shown.z - at.y).abs() < 1e-4);
}

#[test]
fn additive_click_keeps_members_and_plain_click_replaces() {
    let mut selection = Selection {
        building: Some("base-1".into()),
        ..Selection::default()
    };
    selection.select_units(vec!["villager-1".into()], false);
    selection.select_units(vec!["villager-2".into()], true);
    assert_eq!(selection.units, ["villager-1", "villager-2"]);
    assert!(selection.building.is_none());
    selection.select_units(vec!["villager-1".into()], true);
    assert_eq!(selection.units, ["villager-1", "villager-2"]);
    selection.select_units(vec!["villager-1".into()], false);
    assert_eq!(selection.units, ["villager-1"]);
    selection.select_units(vec!["villager-1".into()], false);
    assert_eq!(selection.units, ["villager-1"]);
}

use aoa_game::GameWorld;

fn sheet() -> TownCenterSheet {
    serde_json::from_str(include_str!("../../../../assets/sprites/towncenter.json")).unwrap()
}

#[test]
fn paving_covers_exact_claims_through_construction_without_repainting_biomes() {
    let mut snapshot = GameWorld::default().snapshot();
    let mut house = snapshot.buildings[0].clone();
    house.building.kind = aoa_game::BuildingKind::House;
    house.building.origin = aoa_game::CellCoordinate::new(30, 25);
    house.building.construction = Some(0.0);
    house.columns = 3;
    house.rows = 3;
    snapshot.buildings.push(house.clone());
    house.building.origin.column += 3;
    snapshot.buildings.push(house);
    let mut view = WorldView::new();
    view.sync(snapshot.clone());
    let (_, before) = view.cell_data().unwrap();
    let at = |column: usize, row: usize| {
        before[(row * usize::from(aoa_game::WORLD_COLUMNS) + column) * 2 + 1]
    };
    assert_eq!(at(29, 25), 0);
    assert_eq!(at(30, 24), 0);
    assert_eq!(at(30, 25), 255);
    assert_eq!(at(32, 27), 255);
    assert_eq!(at(33, 27), 255);
    assert_eq!(at(35, 27), 255);
    assert_eq!(at(36, 27), 0);
    assert_eq!(at(35, 28), 0);
    for (cell, encoded) in snapshot.terrain.iter().zip(before.as_chunks::<2>().0) {
        assert_eq!(
            encoded[0],
            cell.biome.map(terrain::biome_layer).unwrap_or(255)
        );
    }
    snapshot.buildings[1].building.construction = None;
    snapshot.buildings[2].building.construction = None;
    view.sync(snapshot);
    assert_eq!(view.cell_data().unwrap().1, before);
}

#[test]
fn the_temple_rises_through_its_drawn_stages() {
    let sheet = sheet();
    let at = |seconds: f64| town_center_frame(&sheet, Some(seconds), false);
    assert_eq!(at(0.0), "foundation");
    assert_eq!(
        at(aoa_game::BuildingKind::TownCenter.build_seconds() * 0.2),
        "build33"
    );
    assert_eq!(
        at(aoa_game::BuildingKind::TownCenter.build_seconds() * 0.6),
        "build66"
    );
    assert_eq!(town_center_frame(&sheet, None, false), "complete");
    assert_eq!(town_center_frame(&sheet, None, true), "working");
    for frame in ["foundation", "build33", "build66", "complete", "working"] {
        assert!(sheet.frames.contains_key(frame));
    }
}

#[test]
fn samples_interpolate_between_ticks_and_hold_at_the_ends() {
    let samples: VecDeque<_> = [(1.0, Vec3::ZERO), (2.0, Vec3::X)].into();
    assert_eq!(sample_at(&samples, 0.5), Vec3::ZERO);
    assert!((sample_at(&samples, 1.25) - Vec3::new(0.25, 0.0, 0.0)).length() < 1e-6);
    assert_eq!(sample_at(&samples, 3.0), Vec3::X);
}

#[test]
fn the_presentation_clock_trails_the_newest_tick_and_never_passes_it() {
    let mut world = GameWorld::default();
    let mut view = WorldView::new();
    for _ in 0..20 {
        world.tick(0.1);
        view.sync(world.snapshot());
        view.frame(0.1);
    }
    let render = view.render_tick.unwrap();
    assert!(render <= view.latest_tick);
    assert!(view.latest_tick - render < PLAYOUT_TICKS + 1.0);
    // A paused simulation stops the clock at the newest tick.
    for _ in 0..50 {
        view.frame(0.1);
    }
    assert_eq!(view.render_tick.unwrap(), view.latest_tick);
}
