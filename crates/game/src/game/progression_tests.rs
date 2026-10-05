use super::tests::{cell, free_site, run};
use super::*;

#[test]
fn new_games_only_offer_the_starter_economy_and_old_saves_keep_the_catalog() {
    let w = GameWorld::generate(1);
    assert_eq!(w.economy_rules, EconomyRules::IslandProgression);
    assert_eq!(w.available_buildings().len(), STARTER_BUILDINGS.len());
    assert!(STARTER_BUILDINGS.iter().all(|&k| w.building_available(k)));
    assert!(
        w.resources
            .iter()
            .all(|r| STARTER_RESOURCES.contains(&r.kind))
    );
    assert_eq!(
        w.snapshot().buildings[0].building.researches,
        vec![
            TechnologyKind::Forestry,
            TechnologyKind::Agriculture,
            TechnologyKind::Masonry
        ]
    );
    let mut old = serde_json::to_value(&w).unwrap();
    old.as_object_mut().unwrap().remove("economy_rules");
    let old: GameWorld = serde_json::from_value(old).unwrap();
    old.validate().unwrap();
    assert_eq!(old.economy_rules, EconomyRules::Unrestricted);
    assert_eq!(old.available_buildings(), BUILDABLE);
    assert_eq!(old.resources, w.resources);
    let roundtrip: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    assert_eq!(roundtrip, w);
}

#[test]
fn locked_build_production_and_research_are_atomic_even_with_free_materials() {
    let mut w = fixture::fixture();
    w.economy_rules = EconomyRules::IslandProgression;
    w.resources.retain(|r| STARTER_RESOURCES.contains(&r.kind));
    w.inventories[0].wood = 1000.0;
    w.inventories[0].stone = 1000.0;
    w.inventories[0].food = 1000.0;
    let origin = free_site(&mut w, BuildingKind::Kitchen);
    let before = w.clone();
    assert_eq!(
        w.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            kind: BuildingKind::Kitchen,
            origin
        }),
        Err(CommandError::NotBuildable)
    );
    assert_eq!(w, before);
    assert_eq!(
        w.apply_command(Command::Research {
            building_id: "base-1".into(),
            technology: TechnologyKind::Mining
        }),
        Err(CommandError::TechnologyUnavailable)
    );
    assert_eq!(w, before);
    // An existing or injected processor cannot bypass authoritative progression.
    w.buildings
        .push(building(BuildingKind::Kitchen, "kitchen", origin, None));
    let before = w.clone();
    assert_eq!(
        w.apply_command(Command::Produce {
            building_id: "kitchen".into(),
            product: ProductKind::Rations
        }),
        Err(CommandError::ProductUnavailable)
    );
    assert_eq!(w, before);
}

#[test]
fn discovering_complementary_materials_unlocks_industries_and_persists_after_depletion() {
    let mut w = fixture::fixture();
    w.economy_rules = EconomyRules::IslandProgression;
    w.resources.clear();
    w.explored_cells.clear();
    for (i, kind) in [
        ResourceKind::Iron,
        ResourceKind::Coal,
        ResourceKind::Clay,
        ResourceKind::Fiber,
    ]
    .into_iter()
    .enumerate()
    {
        w.resources.push(ResourceNode {
            id: format!("discovery-{i}"),
            kind,
            cell: cell(i as u16 + 2, 2),
            amount: 40.0,
            capacity: 40.0,
            field: None,
        });
    }
    assert!(
        !w.building_available(BuildingKind::Smelter),
        "hidden nodes cannot unlock buildings"
    );
    w.explored_cells.push(cell(2, 2));
    assert!(w.building_available(BuildingKind::MiningCamp));
    assert!(w.technology_available(TechnologyKind::Mining));
    assert!(
        !w.building_available(BuildingKind::Smelter),
        "iron alone lacks fuel"
    );
    w.explored_cells.push(cell(3, 2));
    assert!(w.building_available(BuildingKind::Smelter));
    assert!(!w.building_available(BuildingKind::Kiln));
    w.explored_cells.extend([cell(4, 2), cell(5, 2)]);
    assert!(w.building_available(BuildingKind::Kiln));
    assert!(w.building_available(BuildingKind::Weaver));
    assert!(w.technology_available(TechnologyKind::Textiles));
    for r in &mut w.resources {
        r.amount = 0.0;
    }
    let restored: GameWorld = serde_json::from_value(serde_json::to_value(&w).unwrap()).unwrap();
    assert!(restored.building_available(BuildingKind::Smelter));
    assert!(!restored.building_available(BuildingKind::Kitchen));
}

#[test]
fn starter_materials_can_fund_settlement_processing_and_the_reserved_transport_budget() {
    let mut w = fixture::fixture();
    w.economy_rules = EconomyRules::IslandProgression;
    for (kind, amount) in STARTER_RESOURCE_BUDGET {
        w.inventories[0].add(kind, amount);
    }
    let origin = free_site(&mut w, BuildingKind::LumberMill);
    w.apply_command(Command::Build {
        unit_id: "villager-1".into(),
        kind: BuildingKind::LumberMill,
        origin,
    })
    .unwrap();
    run(&mut w, 120.0);
    let mill = w.buildings.last().unwrap().id.clone();
    assert!(w.buildings.last().unwrap().is_complete());
    for _ in 0..4 {
        w.apply_command(Command::Produce {
            building_id: mill.clone(),
            product: ProductKind::Timber,
        })
        .unwrap();
        run(&mut w, ProductKind::Timber.seconds() + 0.1);
    }
    for kind in [
        BuildingKind::TownCenter,
        BuildingKind::House,
        BuildingKind::Farm,
        BuildingKind::Granary,
        BuildingKind::Dock,
    ] {
        assert!(w.inventories[0].affords(kind.cost()));
        for &(r, amount) in kind.cost() {
            w.inventories[0].add(r, -amount);
        }
    }
    for technology in [
        TechnologyKind::Forestry,
        TechnologyKind::Agriculture,
        TechnologyKind::Masonry,
    ] {
        w.apply_command(Command::Research {
            building_id: "base-1".into(),
            technology,
        })
        .unwrap();
        run(&mut w, RESEARCH_SECONDS + 0.1);
    }
    w.apply_command(Command::Produce {
        building_id: "base-1".into(),
        product: ProductKind::Villager,
    })
    .unwrap();
    run(&mut w, VILLAGER_PRODUCTION_SECONDS + 0.1);
    for &(r, amount) in FIELD_COST {
        w.inventories[0].add(r, -amount);
    }
    assert!(w.inventories[0].affords(&FIRST_TRANSPORT_COST));
    w.validate().unwrap();
}
