//! Isolated farm research fixture; writes JSON without touching a saved game.
use aoa_game::*;
fn main() {
    let mut world = GameWorld::generate(123);
    world.economy_rules = EconomyRules::Unrestricted;
    world.simulation_speed = 0.0;
    world.units.clear();
    world.animals.clear();
    world.resources.clear();
    let farm = &mut world.buildings[0];
    farm.kind = BuildingKind::Farm;
    farm.produces = farm.kind.products().to_vec();
    farm.researches = farm.kind.technologies().to_vec();
    world.inventories[0].food = 100.0;
    world.inventories[0].wood = 100.0;
    world.inventories[0].bricks = 100.0;
    world.inventories[0].timber = 100.0;
    world.validate().unwrap();
    println!("{}", serde_json::to_string(&world).unwrap());
}
