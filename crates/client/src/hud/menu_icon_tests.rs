use super::*;
use aoa_game::{GameWorld, UnitKind};
use std::collections::HashSet;

#[test]
fn every_resource_has_distinct_loaded_art() {
    let assets = pollster::block_on(Assets::load());
    let atlas = build_atlas(&assets);
    let mut seen = HashSet::new();
    for kind in ResourceKind::ALL {
        let key = resource_icon(kind);
        assert!(seen.insert(key), "shared resource art: {kind:?}");
        assert!(atlas.sprites.contains_key(key), "unloaded icon: {key}");
    }
    for key in ICONS {
        assert!(
            atlas.sprites.contains_key(key),
            "missing atlas entry: {key}"
        );
    }
}

#[test]
fn specialist_selection_uses_its_authored_unit_art() {
    let mut world = GameWorld::default();
    for (kind, expected) in [
        (UnitKind::Villager, "portrait_villager"),
        (UnitKind::Guard, "unit_guard"),
        (UnitKind::Archer, "unit_archer"),
        (UnitKind::Healer, "unit_healer"),
        (UnitKind::SiegeCart, "unit_siege_cart"),
    ] {
        world.units[0].kind = kind;
        let snapshot = world.snapshot();
        let selected = [snapshot.units[0].unit.id.clone()];
        let model = Model {
            resource_island: 0,
            snapshot: Some(&snapshot),
            units: &selected,
            building: None,
            ship: None,
            build: BuildUi::Off,
            show_grid: false,
            toast: None,
            camera: Vec2::ZERO,
        };
        assert_eq!(selection_model(&snapshot, &model).unwrap().0, expected);
    }
}

#[test]
fn category_and_back_icons_preserve_navigation_actions() {
    let stock = aoa_game::Stockpile::default();
    let categories = build_menu::commands(BuildUi::Categories, &stock, true, &BUILDABLE);
    let icons: HashSet<_> = categories
        .iter()
        .filter(|c| matches!(c.action, Action::BuildGroup(_)))
        .map(|c| c.icon)
        .collect();
    assert_eq!(icons.len(), BuildingGroup::ALL.len());
    assert!(icons.iter().all(|key| key.starts_with("category_")));
    let close = categories.last().unwrap();
    assert_eq!(
        (close.icon, &close.action),
        ("command_cancel", &Action::Cancel)
    );
    for group in BuildingGroup::ALL {
        let commands = build_menu::commands(BuildUi::Group(group), &stock, true, &BUILDABLE);
        let back = commands.last().unwrap();
        assert_eq!((back.icon, &back.action), ("command_back", &Action::Build));
    }
}

#[test]
fn category_information_uses_the_same_icon_as_its_command() {
    let assets = pollster::block_on(Assets::load());
    let atlas = build_atlas(&assets);
    let snapshot = GameWorld::default().snapshot();
    let selected = [snapshot.units[0].unit.id.clone()];
    let mut model = Model {
        resource_island: 0,
        snapshot: Some(&snapshot),
        units: &selected,
        building: None,
        ship: None,
        build: BuildUi::Categories,
        show_grid: false,
        toast: None,
        camera: Vec2::ZERO,
    };
    for group in BuildingGroup::ALL {
        model.build = BuildUi::Group(group);
        assert_eq!(selection_model(&snapshot, &model).unwrap().0, group.icon());
        model.build = BuildUi::Categories;
        for (w, h, scale) in [(1280.0, 800.0, 1.0), (390.0, 844.0, 2.0)] {
            let mut hud = Hud::new();
            hud.layout(&atlas, &model, w * scale, h * scale, scale);
            let region = hud
                .regions
                .iter()
                .find(|r| r.action == Action::BuildGroup(group))
                .unwrap();
            hud.hover = Some(Vec2::new(
                region.rect[0] + region.rect[2] / 2.0,
                region.rect[1] + region.rect[3] / 2.0,
            ));
            hud.layout(&atlas, &model, w * scale, h * scale, scale);
            let expected = atlas.sprites[group.icon()];
            assert!(
                hud.quads.iter().any(|q| q.uv == expected),
                "category info icon missing at {w}: {group:?}"
            );
        }
    }
}
