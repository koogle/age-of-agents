//! Fold the former island inventories and transport holds into the shared pool.
//! Removed fields are consumed once; saving writes only the current schema.
use super::StoreResult;
use aoa_game::{GameWorld, Stockpile};
use serde_json::Value;

pub(super) fn load_world(json: &str) -> StoreResult<GameWorld> {
    let mut value: Value = serde_json::from_str(json)?;
    let mut stock = value["stockpile"].clone();
    validate_stock(&stock)?;
    pool_ship_holds(&mut value, &mut stock)?;
    if let Some(islands) = value.get_mut("islands").and_then(Value::as_array_mut) {
        for island in islands {
            if let Some(local) = island.as_object_mut().and_then(|i| i.remove("stockpile")) {
                pool(&mut stock, local, false)?;
            }
            pool_ship_holds(island, &mut stock)?;
        }
    }
    value["stockpile"] = stock;
    let mut world: GameWorld = serde_json::from_value(value)?;
    world.unify_islands()?;
    Ok(world)
}

fn pool_ship_holds(map: &mut Value, stock: &mut Value) -> StoreResult<()> {
    if let Some(ships) = map.get_mut("ships").and_then(Value::as_array_mut) {
        for ship in ships {
            if let Some(goods) = ship.as_object_mut().and_then(|s| s.remove("goods")) {
                pool(stock, goods, true)?;
            }
        }
    }
    Ok(())
}

fn validate_stock(value: &Value) -> StoreResult<()> {
    // Retain required-field/type checks before inspecting or merging amounts.
    let _: Stockpile = serde_json::from_value(value.clone())?;
    for amount in value.as_object().ok_or("invalid stockpile")?.values() {
        if !amount.as_f64().is_some_and(|n| n.is_finite() && n >= 0.0) {
            return Err("persisted world is corrupt: invalid resource amount".into());
        }
    }
    Ok(())
}

fn pool(stock: &mut Value, source: Value, ship_hold: bool) -> StoreResult<()> {
    validate_stock(&source)?;
    let entries = source.as_object().ok_or("invalid stockpile")?;
    if ship_hold && entries.values().map(|n| n.as_f64().unwrap()).sum::<f64>() > 200.0 {
        return Err("persisted world is corrupt: overfull legacy transport hold".into());
    }
    for (kind, amount) in entries {
        let current = stock
            .get(kind)
            .and_then(Value::as_f64)
            .ok_or("unknown resource")?;
        let total = current + amount.as_f64().unwrap();
        if !total.is_finite() {
            return Err("persisted world is corrupt: resource total overflow".into());
        }
        stock[kind] = Value::from(total);
    }
    Ok(())
}
