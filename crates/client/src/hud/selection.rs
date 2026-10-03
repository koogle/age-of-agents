//! What the selection shows: the info pill's portrait, title and detail, and
//! the command coins for a villager (build menu, stop) or a building (train,
//! research). Pure functions of the snapshot, tested without a GPU.
use aoa_game::{
    BuildingKind, ProductKind, RESEARCH_FOOD_COST, RESEARCH_WOOD_COST, ResourceKind,
    TechnologyKind, UnitAction, WorldSnapshot,
};

use super::{Action, BuildUi, Model};

/// Coin icon, sheet row, name and what it is for.
pub(super) fn building_info(
    kind: BuildingKind,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match kind {
        BuildingKind::MiningCamp => (
            "building_mining_camp",
            "mining_camp",
            "Mining camp",
            "Mineral drop-off; nearby gathering +25%",
        ),
        BuildingKind::Farm => (
            "building_farm",
            "farm",
            "Farm",
            "Unlocks food fields; nearby gathering +25%",
        ),
        BuildingKind::LumberMill => (
            "building_lumber_mill",
            "lumber_mill",
            "Lumber mill",
            "Turns wood into timber",
        ),
        BuildingKind::Smelter => (
            "building_smelter",
            "smelter",
            "Smelter",
            "Turns iron and coal into steel",
        ),
        BuildingKind::Kiln => (
            "building_kiln",
            "kiln",
            "Kiln",
            "Turns clay and wood into bricks",
        ),
        BuildingKind::Weaver => (
            "building_weaver",
            "weaver",
            "Weaver",
            "Turns fiber into cloth",
        ),
        BuildingKind::Kitchen => (
            "building_kitchen",
            "kitchen",
            "Kitchen",
            "Turns food into rations",
        ),
        BuildingKind::Barracks => (
            "building_barracks",
            "barracks",
            "Barracks",
            "Trains guards; combat comes later",
        ),
        BuildingKind::Range => (
            "building_range",
            "range",
            "Range",
            "Trains archers; combat comes later",
        ),
        BuildingKind::Workshop => (
            "building_workshop",
            "workshop",
            "Workshop",
            "Builds siege carts; combat comes later",
        ),
        BuildingKind::Infirmary => (
            "building_infirmary",
            "infirmary",
            "Infirmary",
            "Trains healers; healing comes later",
        ),
        BuildingKind::Monument => (
            "building_monument",
            "monument",
            "Monument",
            "A landmark with sight radius 24",
        ),
        BuildingKind::House => ("building_house", "house", "House", "Room for 5 villagers"),
        BuildingKind::Granary => (
            "building_granary",
            "granary",
            "Granary",
            "Drop-off for food and fiber",
        ),
        BuildingKind::Watchtower => (
            "building_watchtower",
            "watchtower",
            "Watchtower",
            "Sees far across the island",
        ),
        BuildingKind::Dock => ("building_dock", "dock", "Dock", "Must touch the sea"),
        _ => (
            "building_towncenter",
            "towncenter",
            "Town center",
            "Trains villagers, houses 5",
        ),
    }
}

/// "15 wood, 15 stone".
pub(super) fn cost_text(cost: &[(ResourceKind, f64)]) -> String {
    cost.iter()
        .map(|(kind, amount)| format!("{amount} {}", kind.name()))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn tech_info(tech: TechnologyKind) -> (&'static str, &'static str, &'static str) {
    match tech {
        TechnologyKind::Forestry => ("tech_forestry", "Forestry", "Wood +20%"),
        TechnologyKind::Agriculture => ("tech_agriculture", "Agriculture", "Food +20%"),
        TechnologyKind::Masonry => ("tech_masonry", "Masonry", "Stone and clay +20%"),
        TechnologyKind::Mining => ("tech_mining", "Mining", "Gold and iron +20%"),
        TechnologyKind::Textiles => ("tech_textiles", "Textiles", "Fiber +20%"),
    }
}

fn product_label(product: ProductKind) -> String {
    if let Some(kind) = product.unit_kind() {
        format!("Train {}", kind.name().to_lowercase())
    } else if let Some((kind, amount)) = product.output() {
        format!("Make {amount} {}", kind.name())
    } else {
        unreachable!("every product has an output")
    }
}

pub(super) struct Command {
    pub(super) icon: &'static str,
    pub(super) label: String,
    pub(super) detail: String,
    pub(super) enabled: bool,
    pub(super) action: Action,
}

pub(super) type Selected = (&'static str, String, String, Option<f32>, Vec<Command>);

pub(super) fn selection_model(snapshot: &WorldSnapshot, model: &Model) -> Option<Selected> {
    let stock = &snapshot.stockpile;
    if !model.units.is_empty() {
        let busy = snapshot
            .units
            .iter()
            .any(|u| model.units.contains(&u.unit.id) && u.unit.action != UnitAction::Idle);
        let workers = snapshot
            .units
            .iter()
            .filter(|u| model.units.contains(&u.unit.id))
            .all(|u| u.unit.kind == aoa_game::UnitKind::Villager);
        let mut commands = match model.build {
            BuildUi::PlacingField => vec![Command {
                icon: "command_cancel",
                label: "Place field".into(),
                detail: format!(
                    "{} · {} s work · {} food",
                    cost_text(aoa_game::FIELD_COST),
                    aoa_game::FIELD_WORK_SECONDS,
                    aoa_game::FIELD_FOOD
                ),
                enabled: true,
                action: Action::Cancel,
            }],
            BuildUi::Placing(kind) => vec![Command {
                icon: "command_cancel",
                label: format!("Place the {}", building_info(kind).2.to_lowercase()),
                detail: "Tap a clear site · Esc to cancel".into(),
                enabled: true,
                action: Action::Cancel,
            }],
            BuildUi::Categories | BuildUi::Group(_) => super::build_menu::commands(
                model.build,
                stock,
                snapshot
                    .buildings
                    .iter()
                    .any(|b| b.building.kind == BuildingKind::Farm && b.building.is_complete()),
                &snapshot.available_buildings,
            ),
            BuildUi::Off => vec![Command {
                icon: "command_build",
                label: "Build".into(),
                detail: "Choose a building".into(),
                enabled: true,
                action: Action::Build,
            }],
        };
        if workers && matches!(model.build, BuildUi::Categories | BuildUi::Group(_)) {
            let (title, detail) = match model.build {
                BuildUi::Group(group) => (
                    group.name(),
                    "Choose a building · Costs shown on hover or tap",
                ),
                _ if snapshot.available_buildings.len() < aoa_game::BUILDABLE.len() => {
                    ("Build", "Later islands will unlock metal, bricks and cloth")
                }
                _ => ("Build", "Choose a building type"),
            };
            return Some(("command_build", title.into(), detail.into(), None, commands));
        }
        if !workers {
            commands.clear();
        }
        if busy && model.build == BuildUi::Off {
            commands.push(Command {
                icon: "command_cancel",
                label: "Stop".into(),
                detail: "Drop the current task · X".into(),
                enabled: true,
                action: Action::Stop,
            });
        }
        if model.units.len() > 1 {
            let idle = snapshot
                .units
                .iter()
                .filter(|u| model.units.contains(&u.unit.id) && u.unit.action == UnitAction::Idle)
                .count();
            return Some((
                "portrait_group",
                format!("{} units", model.units.len()),
                format!("{idle} awaiting orders"),
                None,
                commands,
            ));
        }
        let unit = snapshot
            .units
            .iter()
            .find(|u| u.unit.id == model.units[0])?;
        let activity = match &unit.unit.action {
            UnitAction::Idle => "Awaiting orders".to_string(),
            UnitAction::Move { .. } => "Walking".into(),
            UnitAction::Build { .. } if unit.unit.cargo.is_some() => {
                format!(
                    "Unloading {} before building",
                    resource_name(unit.unit.cargo.as_ref().unwrap().kind)
                )
            }
            UnitAction::Build { .. } => "Building".into(),
            UnitAction::Cultivate { .. } if unit.unit.cargo.is_some() => {
                format!(
                    "Unloading {} before preparing field",
                    resource_name(unit.unit.cargo.as_ref().unwrap().kind)
                )
            }
            UnitAction::Cultivate { .. } => "Preparing field".into(),
            UnitAction::Deposit { .. } => "Taking goods to unload".into(),
            UnitAction::Gather { resource_id, phase } => match phase {
                aoa_game::GatherPhase::ToResource => "Heading out to gather".into(),
                aoa_game::GatherPhase::Gathering => "Gathering".into(),
                aoa_game::GatherPhase::Returning | aoa_game::GatherPhase::Depositing
                    if unit.unit.cargo.is_some() =>
                {
                    let carried = resource_name(unit.unit.cargo.as_ref().unwrap().kind);
                    match snapshot.resources.iter().find(|r| &r.id == resource_id) {
                        Some(resource) => format!(
                            "Unloading {carried} before gathering {}",
                            resource_name(resource.kind)
                        ),
                        None => format!("Unloading {carried} first"),
                    }
                }
                aoa_game::GatherPhase::Returning => "Carrying goods home".into(),
                aoa_game::GatherPhase::Depositing => "Unloading".into(),
            },
        };
        let cargo = unit
            .unit
            .cargo
            .as_ref()
            .map(|c| format!(" · {} carried", c.amount.floor()))
            .unwrap_or_default();
        let title = format!(
            "{} {}",
            unit.unit.kind.name(),
            unit.unit.id.rsplit('-').next().unwrap_or("")
        );
        return Some((
            "portrait_villager",
            title,
            format!("{activity}{cargo}"),
            None,
            commands,
        ));
    }
    let building = snapshot
        .buildings
        .iter()
        .find(|b| Some(b.building.id.as_str()) == model.building)?;
    let kind = building.building.kind;
    let (portrait, _, name, purpose) = building_info(kind);
    if let Some(work) = building.building.construction {
        return Some((
            portrait,
            format!("{name} foundation"),
            "Villagers can help build it".into(),
            Some((work / kind.build_seconds()) as f32),
            Vec::new(),
        ));
    }
    let job = building.building.job.as_ref();
    let busy = job.is_some();
    let (detail, progress) = match job {
        Some(aoa_game::BuildingJob::Produce {
            product,
            elapsed_seconds,
        }) => (
            if *elapsed_seconds >= product.seconds() && product.unit_kind().is_some() {
                "Waiting for a free spawn cell".to_string()
            } else {
                format!("Producing {}", product_label(*product))
            },
            Some((elapsed_seconds / product.seconds()).min(1.0) as f32),
        ),
        Some(aoa_game::BuildingJob::Research {
            technology,
            elapsed_seconds,
        }) => (
            format!("Researching {}", tech_info(*technology).1),
            Some((elapsed_seconds / aoa_game::RESEARCH_SECONDS) as f32),
        ),
        None => (purpose.to_string(), None),
    };
    let housing: usize = snapshot
        .buildings
        .iter()
        .filter(|b| b.building.is_complete())
        .map(|b| b.building.kind.housing())
        .sum();
    let population = snapshot.units.len() + snapshot.buildings.iter().filter(|b| matches!(b.building.job, Some(aoa_game::BuildingJob::Produce { product, .. }) if product.unit_kind().is_some())).count();
    let mut commands = Vec::new();
    for &product in &building.building.produces {
        let crowded = product.unit_kind().is_some() && population >= housing;
        commands.push(Command {
            icon: "command_train",
            label: product_label(product),
            detail: if crowded {
                "Needs a house first".into()
            } else {
                format!(
                    "{} · {} seconds",
                    cost_text(product.cost()),
                    product.seconds()
                )
            },
            enabled: !busy && !crowded && stock.affords(product.cost()),
            action: Action::Produce(product),
        });
    }
    let known = &snapshot.researched_technologies;
    for &tech in &building.building.researches {
        let (icon, name, effect) = tech_info(tech);
        let done = known.contains(&tech);
        let queued = snapshot.buildings.iter().any(|b| matches!(b.building.job, Some(aoa_game::BuildingJob::Research { technology, .. }) if technology == tech));
        let blocked = tech.prerequisite().filter(|p| !known.contains(p));
        let detail = if done {
            "researched".to_string()
        } else if queued {
            "research in progress".to_string()
        } else if let Some(p) = blocked {
            format!("needs {}", tech_info(p).1)
        } else {
            format!(
                "{effect} · {} food, {} wood",
                RESEARCH_FOOD_COST, RESEARCH_WOOD_COST
            )
        };
        commands.push(Command {
            icon,
            label: name.into(),
            detail,
            enabled: !done
                && !queued
                && blocked.is_none()
                && !busy
                && stock.food >= RESEARCH_FOOD_COST
                && stock.wood >= RESEARCH_WOOD_COST,
            action: Action::Research(tech),
        });
    }
    Some((portrait, name.into(), detail, progress, commands))
}

fn resource_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Wood => "wood",
        ResourceKind::Food => "food",
        ResourceKind::Stone => "stone",
        ResourceKind::Gold => "gold",
        ResourceKind::Iron => "iron",
        ResourceKind::Coal => "coal",
        ResourceKind::Clay => "clay",
        ResourceKind::Fiber => "fiber",
        ResourceKind::Timber => "timber",
        ResourceKind::Steel => "steel",
        ResourceKind::Bricks => "bricks",
        ResourceKind::Cloth => "cloth",
        ResourceKind::Rations => "rations",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{BUILDABLE, GameWorld};
    use glam::Vec2;

    #[test]
    fn reassignment_status_explains_unloading_and_then_the_new_task() {
        let mut world = GameWorld::default();
        world.economy_rules = aoa_game::EconomyRules::Unrestricted;
        let visible = world.snapshot().resources[0].id.clone();
        world
            .resources
            .iter_mut()
            .find(|r| r.id == visible)
            .unwrap()
            .kind = ResourceKind::Food;
        world.units[0].cargo = Some(aoa_game::CarriedResource {
            kind: ResourceKind::Wood,
            amount: 7.0,
        });
        world.units[0].action = UnitAction::Gather {
            resource_id: visible,
            phase: aoa_game::GatherPhase::Returning,
        };
        assert!(unit_detail(&world).starts_with("Unloading wood before gathering food"));
        world.units[0].cargo = None;
        world.units[0].action = UnitAction::Gather {
            resource_id: "outside-fog".into(),
            phase: aoa_game::GatherPhase::ToResource,
        };
        assert_eq!(unit_detail(&world), "Heading out to gather");
    }

    fn unit_detail(world: &GameWorld) -> String {
        let snapshot = world.snapshot();
        let units = [snapshot.units[0].unit.id.clone()];
        let model = Model {
            snapshot: Some(&snapshot),
            units: &units,
            building: None,
            build: BuildUi::Off,
            show_grid: false,
            toast: None,
            camera: Vec2::ZERO,
        };
        selection_model(&snapshot, &model).unwrap().2
    }

    #[test]
    fn grouped_menu_exposes_every_building_once() {
        let mut world = GameWorld::default();
        world.economy_rules = aoa_game::EconomyRules::Unrestricted;
        let snapshot = world.snapshot();
        let units = [snapshot.units[0].unit.id.clone()];
        let mut seen = Vec::new();
        for group in super::super::BuildingGroup::ALL {
            let model = Model {
                snapshot: Some(&snapshot),
                units: &units,
                building: None,
                build: BuildUi::Group(group),
                show_grid: false,
                toast: None,
                camera: Vec2::ZERO,
            };
            let commands = selection_model(&snapshot, &model).unwrap().4;
            assert!(commands.len() <= 8);
            for command in commands {
                if let Action::Place(kind) = command.action {
                    seen.push(kind);
                }
            }
        }
        assert_eq!(seen.len(), BUILDABLE.len());
        for kind in BUILDABLE {
            assert_eq!(seen.iter().filter(|&&item| item == kind).count(), 1);
        }
    }

    #[test]
    fn processor_commands_use_recipe_costs_and_show_blocked_jobs() {
        let mut world = GameWorld::default();
        world.economy_rules = aoa_game::EconomyRules::Unrestricted;
        world.buildings[0].kind = BuildingKind::Smelter;
        world.buildings[0].produces = BuildingKind::Smelter.products().to_vec();
        world.buildings[0].researches.clear();
        let poor = commands_for_town_center(&world);
        assert_eq!(poor.len(), 1);
        assert_eq!(poor[0].action, Action::Produce(ProductKind::Steel));
        assert!(!poor[0].enabled);
        world.stockpile.iron = 5.0;
        world.stockpile.coal = 5.0;
        assert!(commands_for_town_center(&world)[0].enabled);
        world.buildings[0].job = Some(aoa_game::BuildingJob::Produce {
            product: ProductKind::Steel,
            elapsed_seconds: 1.0,
        });
        assert!(!commands_for_town_center(&world)[0].enabled);
    }

    fn commands_for_town_center(world: &GameWorld) -> Vec<Command> {
        let snapshot = world.snapshot();
        let model = Model {
            snapshot: Some(&snapshot),
            units: &[],
            building: Some(&snapshot.buildings[0].building.id),
            build: BuildUi::Off,
            show_grid: false,
            toast: None,
            camera: Vec2::ZERO,
        };
        selection_model(&snapshot, &model).unwrap().4
    }

    #[test]
    fn town_center_coins_follow_costs_and_prerequisites() {
        let mut world = GameWorld::default();
        world.economy_rules = aoa_game::EconomyRules::Unrestricted;
        world.stockpile.food = 0.0;
        world.stockpile.wood = 0.0;
        let poor = commands_for_town_center(&world);
        assert!(poor.iter().all(|c| !c.enabled), "nothing is affordable");
        world.stockpile.food = 100.0;
        world.stockpile.wood = 100.0;
        let rich = commands_for_town_center(&world);
        let enabled = |action: Action| rich.iter().find(|c| c.action == action).unwrap().enabled;
        assert!(enabled(Action::Produce(ProductKind::Villager)));
        assert!(enabled(Action::Research(TechnologyKind::Masonry)));
        assert!(
            !enabled(Action::Research(TechnologyKind::Mining)),
            "mining needs masonry"
        );
    }

    #[test]
    fn build_menu_greys_out_what_the_stockpile_cannot_cover() {
        let mut world = GameWorld::default();
        world.economy_rules = aoa_game::EconomyRules::Unrestricted;
        world.stockpile = Default::default();
        world.stockpile.wood = 15.0;
        let snapshot = world.snapshot();
        let units = [snapshot.units[0].unit.id.clone()];
        let model = Model {
            snapshot: Some(&snapshot),
            units: &units,
            building: None,
            build: BuildUi::Group(super::super::BuildingGroup::Town),
            show_grid: false,
            toast: None,
            camera: Vec2::ZERO,
        };
        let commands = selection_model(&snapshot, &model).unwrap().4;
        let enabled = |kind| {
            commands
                .iter()
                .find(|c| c.action == Action::Place(kind))
                .unwrap()
                .enabled
        };
        assert!(enabled(BuildingKind::House));
        assert!(!enabled(BuildingKind::TownCenter));
        assert!(!enabled(BuildingKind::Granary));
        assert!(!enabled(BuildingKind::Monument));
        assert!(!enabled(BuildingKind::Dock));
        assert!(commands.iter().any(|c| c.action == Action::Build));
    }
}
