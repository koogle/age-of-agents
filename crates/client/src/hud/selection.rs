//! What the selection shows: the info pill's portrait, title and detail, and
//! the command coins for a villager (build menu, stop) or a building (train,
//! research). Pure functions of the snapshot, tested without a GPU.
use aoa_game::{
    BUILDABLE, BuildingKind, ProductKind, RESEARCH_FOOD_COST, RESEARCH_WOOD_COST, ResourceKind,
    TechnologyKind, UnitAction, WorldSnapshot,
};

use super::{Action, BuildUi, Model};

/// Coin icon, sheet row, name and what it is for.
pub(super) fn building_info(
    kind: BuildingKind,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match kind {
        BuildingKind::MiningCamp => (
            "building_towncenter",
            "towncenter",
            "Mining camp",
            "Mineral drop-off; nearby gathering +25%",
        ),
        BuildingKind::Farm => (
            "building_granary",
            "granary",
            "Farm",
            "Food/fiber drop-off; nearby gathering +25%",
        ),
        BuildingKind::LumberMill => (
            "building_towncenter",
            "towncenter",
            "Lumber mill",
            "Turns wood into timber",
        ),
        BuildingKind::Smelter => (
            "building_towncenter",
            "towncenter",
            "Smelter",
            "Turns iron and coal into steel",
        ),
        BuildingKind::Kiln => (
            "building_towncenter",
            "towncenter",
            "Kiln",
            "Turns clay and wood into bricks",
        ),
        BuildingKind::Weaver => (
            "building_towncenter",
            "towncenter",
            "Weaver",
            "Turns fiber into cloth",
        ),
        BuildingKind::Kitchen => (
            "building_granary",
            "granary",
            "Kitchen",
            "Turns food into rations",
        ),
        BuildingKind::Barracks => (
            "building_watchtower",
            "watchtower",
            "Barracks",
            "Trains guards; combat comes later",
        ),
        BuildingKind::Range => (
            "building_watchtower",
            "watchtower",
            "Range",
            "Trains archers; combat comes later",
        ),
        BuildingKind::Workshop => (
            "building_towncenter",
            "towncenter",
            "Workshop",
            "Builds siege carts; combat comes later",
        ),
        BuildingKind::Infirmary => (
            "building_towncenter",
            "towncenter",
            "Infirmary",
            "Trains healers; healing comes later",
        ),
        BuildingKind::Monument => (
            "building_towncenter",
            "towncenter",
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
            BuildUi::Placing(kind) => vec![Command {
                icon: "command_cancel",
                label: format!("Place the {}", building_info(kind).2.to_lowercase()),
                detail: "Tap a clear site · Esc to cancel".into(),
                enabled: true,
                action: Action::Cancel,
            }],
            BuildUi::Menu(page) => {
                let mut menu: Vec<Command> = BUILDABLE
                    .into_iter()
                    .skip(page * 5)
                    .take(5)
                    .map(|kind| {
                        let (icon, _, name, _) = building_info(kind);
                        Command {
                            icon,
                            label: name.into(),
                            detail: cost_text(kind.cost()),
                            enabled: stock.affords(kind.cost()),
                            action: Action::Place(kind),
                        }
                    })
                    .collect();
                if page > 0 {
                    menu.push(Command {
                        icon: "command_cancel",
                        label: "Previous buildings".into(),
                        detail: format!("Page {} of 4", page),
                        enabled: true,
                        action: Action::BuildPage(page - 1),
                    });
                }
                if (page + 1) * 5 < BUILDABLE.len() {
                    menu.push(Command {
                        icon: "command_build",
                        label: "More buildings".into(),
                        detail: format!("Page {} of 4", page + 2),
                        enabled: true,
                        action: Action::BuildPage(page + 1),
                    });
                }
                menu.push(Command {
                    icon: "command_cancel",
                    label: "Back".into(),
                    detail: "Close the build menu".into(),
                    enabled: true,
                    action: Action::Cancel,
                });
                menu
            }
            BuildUi::Off => vec![Command {
                icon: "command_build",
                label: "Build".into(),
                detail: "Choose a building".into(),
                enabled: true,
                action: Action::Build,
            }],
        };
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
                "Dropping off goods first".into()
            }
            UnitAction::Build { .. } => "Building".into(),
            UnitAction::Deposit { .. } => "Taking goods to unload".into(),
            UnitAction::Gather { phase, .. } => match phase {
                aoa_game::GatherPhase::ToResource => "Heading out to gather".into(),
                aoa_game::GatherPhase::Gathering => "Gathering".into(),
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
    for &product in kind.products() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::GameWorld;
    use glam::Vec2;

    #[test]
    fn paged_menu_exposes_every_building_once() {
        let world = GameWorld::default();
        let snapshot = world.snapshot();
        let units = [snapshot.units[0].unit.id.clone()];
        let mut seen = Vec::new();
        for page in 0..BUILDABLE.len().div_ceil(5) {
            let model = Model {
                snapshot: Some(&snapshot),
                units: &units,
                building: None,
                build: BuildUi::Menu(page),
                reset_armed: false,
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
        assert_eq!(seen, BUILDABLE);
    }

    #[test]
    fn processor_commands_use_recipe_costs_and_show_blocked_jobs() {
        let mut world = GameWorld::default();
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
            reset_armed: false,
            show_grid: false,
            toast: None,
            camera: Vec2::ZERO,
        };
        selection_model(&snapshot, &model).unwrap().4
    }

    #[test]
    fn town_center_coins_follow_costs_and_prerequisites() {
        let mut world = GameWorld::default();
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
        world.stockpile = Default::default();
        world.stockpile.wood = 15.0;
        let snapshot = world.snapshot();
        let units = [snapshot.units[0].unit.id.clone()];
        let model = Model {
            snapshot: Some(&snapshot),
            units: &units,
            building: None,
            build: BuildUi::Menu(0),
            reset_armed: false,
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
        assert!(!enabled(BuildingKind::Watchtower));
        assert!(!enabled(BuildingKind::Dock));
        assert!(commands.iter().any(|c| c.action == Action::Cancel));
    }
}
