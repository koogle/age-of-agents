//! Purpose-based build groups; every building keeps its own authored portrait.
use super::{
    Action, BuildUi,
    selection::{Command, building_info, cost_text},
};
use aoa_game::{BuildingKind, Stockpile};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuildingGroup {
    Town,
    Gathering,
    Production,
    Military,
    Roads,
}

impl BuildingGroup {
    pub const ALL: [Self; 5] = [
        Self::Town,
        Self::Gathering,
        Self::Production,
        Self::Military,
        Self::Roads,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Town => "Town",
            Self::Gathering => "Gathering",
            Self::Production => "Production",
            Self::Military => "Military",
            Self::Roads => "Roads",
        }
    }
    fn buildings(self) -> &'static [BuildingKind] {
        use BuildingKind::*;
        match self {
            Self::Roads => &[],
            Self::Town => &[TownCenter, House, Granary, Dock, Monument],
            Self::Gathering => &[Farm, MiningCamp],
            Self::Production => &[LumberMill, Smelter, Kiln, Weaver, Kitchen],
            Self::Military => &[Watchtower, Barracks, Range, Workshop, Infirmary],
        }
    }
}

pub(super) fn atlas(kind: BuildingKind) -> &'static str {
    use BuildingKind::*;
    match kind {
        TownCenter => "towncenter",
        House | Granary | Watchtower | Dock => "buildings_hd",
        MiningCamp | Farm | LumberMill | Smelter => "buildings_economy",
        Kiln | Weaver | Kitchen | Monument => "buildings_crafts",
        Barracks | Range | Workshop | Infirmary => "buildings_civic",
    }
}

pub(super) fn commands(
    build: BuildUi,
    stock: &Stockpile,
    has_farm: bool,
    available: &[BuildingKind],
) -> Vec<Command> {
    let mut commands = if let BuildUi::Group(group) = build {
        group
            .buildings()
            .iter()
            .filter(|kind| available.contains(kind))
            .map(|&kind| {
                let (icon, _, name, _) = building_info(kind);
                Command {
                    icon,
                    label: name.into(),
                    detail: cost_text(kind.cost()),
                    enabled: stock.affords(kind.cost()),
                    action: Action::Place(kind),
                }
            })
            .collect()
    } else {
        BuildingGroup::ALL
            .into_iter()
            .filter(|group| {
                *group == BuildingGroup::Roads
                    || group
                        .buildings()
                        .iter()
                        .any(|kind| available.contains(kind))
            })
            .map(|group| Command {
                icon: group
                    .buildings()
                    .first()
                    .map_or("command_build", |&k| building_info(k).0),
                label: group.name().into(),
                detail: group
                    .buildings()
                    .iter()
                    .filter(|kind| available.contains(kind))
                    .map(|&kind| building_info(kind).2)
                    .collect::<Vec<_>>()
                    .join(", "),
                enabled: true,
                action: Action::BuildGroup(group),
            })
            .collect::<Vec<_>>()
    };
    if build == BuildUi::Group(BuildingGroup::Roads) {
        for kind in [aoa_game::RoadKind::Dirt, aoa_game::RoadKind::Stone] {
            commands.push(Command {
                icon: if kind == aoa_game::RoadKind::Dirt {
                    "command_build"
                } else {
                    "resource_stone"
                },
                label: kind.name().into(),
                detail: format!(
                    "{} · 2 s work per cell · +50% movement · straight lines",
                    if kind == aoa_game::RoadKind::Dirt {
                        "Labour only"
                    } else {
                        "1 stone per cell"
                    }
                ),
                enabled: stock.stone >= kind.stone_per_cell(),
                action: Action::PlaceRoad(kind),
            });
        }
    }
    if build == BuildUi::Group(BuildingGroup::Gathering) {
        commands.push(Command {
            icon: "field",
            label: "Field".into(),
            detail: if has_farm {
                format!(
                    "{} · {} s work · {} food; tap depleted fields to replenish",
                    cost_text(aoa_game::FIELD_COST),
                    aoa_game::FIELD_WORK_SECONDS,
                    aoa_game::FIELD_FOOD
                )
            } else {
                "Build a farm first".into()
            },
            enabled: has_farm && stock.affords(aoa_game::FIELD_COST),
            action: Action::PlaceField,
        });
    }
    commands.push(Command {
        icon: "command_cancel",
        label: if build == BuildUi::Categories {
            "Close"
        } else {
            "All types"
        }
        .into(),
        detail: "Back to building types".into(),
        enabled: true,
        action: if build == BuildUi::Categories {
            Action::Cancel
        } else {
            Action::Build
        },
    });
    commands
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn starter_menu_only_shows_relevant_groups_and_buildings() {
        let stock = Stockpile::default();
        let categories = commands(
            BuildUi::Categories,
            &stock,
            false,
            &aoa_game::STARTER_BUILDINGS,
        );
        assert_eq!(categories.len(), 6); // All five groups and Close.
        assert!(
            categories
                .iter()
                .any(|c| c.action == Action::BuildGroup(BuildingGroup::Military))
        );
        let military = commands(
            BuildUi::Group(BuildingGroup::Military),
            &stock,
            false,
            &aoa_game::STARTER_BUILDINGS,
        );
        assert_eq!(
            military
                .iter()
                .map(|c| c.action.clone())
                .collect::<Vec<_>>(),
            vec![
                Action::Place(BuildingKind::Watchtower),
                Action::Place(BuildingKind::Barracks),
                Action::Place(BuildingKind::Range),
                Action::Build
            ]
        );
        assert!(!military[1].enabled, "unaffordable barracks stays visible");
        let production = commands(
            BuildUi::Group(BuildingGroup::Production),
            &stock,
            false,
            &aoa_game::STARTER_BUILDINGS,
        );
        assert_eq!(production.len(), 6);
        assert_eq!(
            production[0].action,
            Action::Place(BuildingKind::LumberMill)
        );
        let gathering = commands(
            BuildUi::Group(BuildingGroup::Gathering),
            &stock,
            false,
            &aoa_game::STARTER_BUILDINGS,
        );
        assert_eq!(gathering.len(), 4); // Farm, Mining Camp, Field, All types.
        assert_eq!(gathering[2].action, Action::PlaceField);
        assert!(!gathering[2].enabled);
    }

    #[test]
    fn roads_are_basic_choices_with_labour_only_dirt_and_stone_costs() {
        let menu = commands(
            BuildUi::Group(BuildingGroup::Roads),
            &Stockpile::default(),
            false,
            &[],
        );
        assert_eq!(menu[0].action, Action::PlaceRoad(aoa_game::RoadKind::Dirt));
        assert!(menu[0].enabled);
        assert_eq!(menu[1].action, Action::PlaceRoad(aoa_game::RoadKind::Stone));
        assert!(!menu[1].enabled);
        assert!(
            commands(BuildUi::Categories, &Stockpile::default(), false, &[])
                .iter()
                .any(|c| c.action == Action::BuildGroup(BuildingGroup::Roads))
        );
    }

    #[test]
    fn fields_need_a_farm_and_both_materials() {
        let mut stock = Stockpile {
            wood: 10.0,
            stone: 5.0,
            ..Default::default()
        };
        let field = |stock: &Stockpile, farm| {
            commands(
                BuildUi::Group(BuildingGroup::Gathering),
                stock,
                farm,
                &aoa_game::BUILDABLE,
            )
            .into_iter()
            .find(|c| c.action == Action::PlaceField)
            .unwrap()
        };
        assert!(!field(&stock, false).enabled);
        assert!(field(&stock, true).enabled);
        stock.stone = 4.0;
        assert!(!field(&stock, true).enabled);
        stock.stone = 5.0;
        stock.wood = 9.0;
        assert!(!field(&stock, true).enabled);
    }

    #[test]
    fn every_building_has_a_distinct_authored_portrait_in_the_runtime_atlas() {
        let assets = pollster::block_on(crate::assets::Assets::load());
        let atlas = super::super::build_atlas(&assets);
        let mut icons = HashSet::new();
        for kind in aoa_game::BUILDABLE {
            let icon = building_info(kind).0;
            assert!(
                icons.insert(icon),
                "placeholder portrait reused for {kind:?}"
            );
            let &(uv, size) = &atlas.content[icon];
            assert!(uv.iter().all(|v| (0.0..=1.0).contains(v)));
            assert!(size.x > 0.0 && size.y > 0.0);
        }
    }

    #[test]
    fn categories_and_back_navigation_are_available_without_resources() {
        let stock = Stockpile::default();
        let categories = commands(BuildUi::Categories, &stock, false, &aoa_game::BUILDABLE);
        assert_eq!(categories.len(), 6);
        for (command, group) in categories.iter().zip(BuildingGroup::ALL) {
            assert_eq!(command.action, Action::BuildGroup(group));
            assert!(command.enabled);
            let menu = commands(BuildUi::Group(group), &stock, false, &aoa_game::BUILDABLE);
            assert!(menu.len() <= 6);
            assert_eq!(menu.last().unwrap().action, Action::Build);
        }
        assert_eq!(categories.last().unwrap().action, Action::Cancel);
    }
}
