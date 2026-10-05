use super::*;
use aoa_game::{Command as GameCommand, EconomyRules, GameWorld, ProductKind};

fn overlaps(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[0] + b[2] - 0.01
        && b[0] < a[0] + a[2] - 0.01
        && a[1] < b[1] + b[3] - 0.01
        && b[1] < a[1] + a[3] - 0.01
}

#[test]
fn minimap_clicks_and_camera_marker_share_the_displayed_projection() {
    let assets = pollster::block_on(crate::assets::Assets::load());
    let atlas = build_atlas(&assets);
    let snapshot = GameWorld::default().snapshot();
    let camera = Vec2::new(36.0, 18.0);
    let model = Model {
        snapshot: Some(&snapshot),
        units: &[],
        building: None,
        ship: None,
        build: BuildUi::Off,
        show_grid: false,
        toast: None,
        camera,
    };
    let map = minimap::Minimap::new(
        Vec2::new(snapshot.columns as f32, snapshot.rows as f32) * crate::terrain::CELL,
    );
    for (width, height, scale) in [(1280.0, 800.0, 1.0), (390.0, 844.0, 2.0)] {
        let mut hud = Hud::new();
        hud.layout(&atlas, &model, width * scale, height * scale, scale);
        let globe = hud.quads.iter().find(|q| q.params[0] == 3.0).unwrap().rect;
        let screen_of = |world| {
            Vec2::new(globe[0], globe[1]) + map.local_of(world) * Vec2::new(globe[2], globe[3])
        };
        let marker = hud
            .quads
            .iter()
            .find(|q| q.params[0] == 2.0 && (q.rect[2] - 10.0 * scale).abs() < 1e-5)
            .unwrap();
        assert!(
            (Vec2::new(marker.rect[0], marker.rect[1]) + Vec2::splat(5.0 * scale))
                .abs_diff_eq(screen_of(camera), 1e-4)
        );
        for point in [camera, Vec2::new(30.0, 20.0), Vec2::new(15.0, 12.0)] {
            assert!(hud.press(screen_of(point)));
            let Some(Action::LookAt(actual)) = hud.release() else {
                panic!("minimap press must navigate");
            };
            assert!(actual.abs_diff_eq(point, 1e-4));
        }
    }
}

#[test]
fn mobile_controls_stay_separate_and_hit_the_actions_they_display() {
    let assets = pollster::block_on(crate::assets::Assets::load());
    let atlas = build_atlas(&assets);
    let mut world = GameWorld::default();
    world.economy_rules = EconomyRules::Unrestricted;
    world.stockpile.wood = 1000.0;
    world.stockpile.food = 1000.0;
    world.ships.push(aoa_game::TransportShip {
        id: "layout-ship".into(),
        cell: aoa_game::CellCoordinate::new(0, 0),
        step: None,
        destination: None,
        heading: [1, 0],
        passengers: vec![],
        home_dock_id: None,
    });
    let snapshot = world.snapshot();
    let units = [snapshot.units[0].unit.id.clone()];
    let building = &snapshot.buildings[0].building.id;
    let mut states = vec![
        (BuildUi::Off, false, false),
        (BuildUi::Off, false, true),
        (BuildUi::Off, true, false),
        (BuildUi::Categories, false, false),
    ];
    states.extend(BuildingGroup::ALL.map(|g| (BuildUi::Group(g), false, false)));
    states.push((BuildUi::Placing(BuildingKind::House), false, false));
    for (width, height) in [
        (320.0, 844.0),
        (360.0, 844.0),
        (390.0, 844.0),
        (430.0, 844.0),
        (599.0, 844.0),
        (844.0, 390.0),
        (1440.0, 900.0),
    ] {
        for scale in [1.0, 2.0] {
            for &(build, town, ship) in &states {
                let model = Model {
                    snapshot: Some(&snapshot),
                    units: if town || ship { &[] } else { &units },
                    building: town.then_some(building.as_str()),
                    ship: ship.then_some("layout-ship"),
                    build,
                    show_grid: false,
                    toast: None,
                    camera: Vec2::new(15.0, 10.0),
                };
                let mut hud = Hud::new();
                hud.layout(&atlas, &model, width * scale, height * scale, scale);
                let regions: Vec<_> = hud
                    .regions
                    .iter()
                    .map(|r| (r.rect, r.action.clone()))
                    .collect();
                for (i, (rect, action)) in regions.iter().enumerate() {
                    assert!(
                        rect[0] >= 0.0 && rect[1] >= 0.0,
                        "{width}: {action:?} {rect:?}"
                    );
                    assert!(rect[0] + rect[2] <= width * scale + 0.1);
                    assert!(rect[1] + rect[3] <= height * scale + 0.1);
                    for (other, other_action) in &regions[i + 1..] {
                        // Desktop speed coins retain their existing circular shoulder
                        // arrangement; their rectangles can touch the map's empty corner.
                        let desktop_map_corner = width >= 600.0
                            && height >= 500.0
                            && matches!(
                                (action, other_action),
                                (Action::LookAt(_), Action::Speed(_))
                                    | (Action::Speed(_), Action::LookAt(_))
                            );
                        assert!(
                            desktop_map_corner || !overlaps(*rect, *other),
                            "{width}: {action:?} overlaps {other_action:?}"
                        );
                    }
                    let center = Vec2::new(rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0);
                    assert!(hud.press(center));
                    if matches!(action, Action::LookAt(_)) {
                        assert_eq!(
                            hud.release(),
                            Some(Action::LookAt(
                                Vec2::new(snapshot.columns as f32, snapshot.rows as f32)
                                    * crate::terrain::CELL
                                    / 2.0
                            ))
                        );
                    } else {
                        assert_eq!(hud.release(), Some(action.clone()));
                    }
                    if (width < 600.0 || height < 500.0) && matches!(action, Action::Speed(_)) {
                        assert!(rect[2] >= 44.0 * scale && rect[3] >= 44.0 * scale);
                    }
                }
                // The normal phone controls share the lowest 140 logical pixels.
                if width >= 390.0
                    && (width < 600.0 || height < 500.0)
                    && build == BuildUi::Off
                    && !ship
                {
                    for (rect, action) in &regions {
                        if !matches!(action, Action::Reset | Action::Grid) {
                            assert!(rect[1] >= (height - 140.0) * scale - 0.1, "{action:?}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn full_mobile_queue_wraps_without_covering_navigation_or_commands() {
    let assets = pollster::block_on(crate::assets::Assets::load());
    let atlas = build_atlas(&assets);
    let mut world = GameWorld::default();
    world.economy_rules = EconomyRules::Unrestricted;
    world.stockpile.wood = 1000.0;
    world.buildings[0].kind = BuildingKind::LumberMill;
    world.buildings[0].produces = BuildingKind::LumberMill.products().to_vec();
    let id = world.buildings[0].id.clone();
    for _ in 0..6 {
        world
            .apply_command(GameCommand::Produce {
                building_id: id.clone(),
                product: ProductKind::Timber,
            })
            .unwrap();
    }
    let snapshot = world.snapshot();
    let model = Model {
        snapshot: Some(&snapshot),
        units: &[],
        building: Some(&id),
        ship: None,
        build: BuildUi::Off,
        show_grid: false,
        toast: None,
        camera: Vec2::new(15.0, 10.0),
    };
    for width in [320.0, 390.0, 599.0] {
        let mut hud = Hud::new();
        hud.layout(&atlas, &model, width, 844.0, 1.0);
        let queued: Vec<_> = hud
            .regions
            .iter()
            .filter(|r| matches!(r.action, Action::CancelQueuedJob(_)))
            .collect();
        assert_eq!(queued.len(), 5);
        for q in queued {
            assert!(q.rect[0] >= 0.0 && q.rect[1] > 0.0 && q.rect[0] + q.rect[2] <= width);
            for other in &hud.regions {
                if q.action != other.action {
                    assert!(
                        !overlaps(q.rect, other.rect),
                        "{width}: {:?} overlaps {:?}",
                        q.action,
                        other.action
                    );
                }
            }
        }
    }
}

#[test]
fn completed_research_coin_explains_instead_of_dispatching_research() {
    let assets = pollster::block_on(crate::assets::Assets::load());
    let atlas = build_atlas(&assets);
    let mut world = GameWorld::default();
    world.stockpile.food = 1000.0;
    world.stockpile.wood = 1000.0;
    world.researched_technologies.push(TechnologyKind::Masonry);
    let snapshot = world.snapshot();
    let model = Model {
        snapshot: Some(&snapshot),
        units: &[],
        building: Some(&snapshot.buildings[0].building.id),
        build: BuildUi::Off,
        ship: None,
        show_grid: false,
        toast: None,
        camera: Vec2::ZERO,
    };
    for (width, height, scale) in [
        (390.0, 844.0, 1.0),
        (780.0, 1688.0, 2.0),
        (1280.0, 800.0, 1.0),
    ] {
        let mut hud = Hud::new();
        hud.layout(&atlas, &model, width, height, scale);
        let rect = hud.regions.iter().find_map(|r| {
            matches!(&r.action, Action::Explain(reason) if reason.starts_with("Masonry: Research complete")).then_some(r.rect)
        }).expect("completed research remains visible and explains its status");
        assert!(hud.press(Vec2::new(rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0)));
        assert!(matches!(hud.release(), Some(Action::Explain(_))));
    }
}
