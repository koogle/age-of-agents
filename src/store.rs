use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use aoa_game::GameWorld;

mod migration;

const DEFAULT_DB_PATH: &str = "age_of_agents.db";

type StoreResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn configured() -> Self {
        Self::from_path(
            std::env::var_os("AGE_OF_AGENTS_DB")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_DB_PATH)),
        )
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn initialize(&self) -> StoreResult<()> {
        let mut connection = Connection::open(&self.path)?;
        let schema_version: u32 =
            connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        let legacy_schema = {
            let mut statement = connection.prepare("PRAGMA table_info(world_state)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            columns
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .any(|column| column == "saved_at")
        };
        let transaction = connection.transaction()?;
        if legacy_schema || schema_version < 10 {
            // The pre-milestone prototype stored a fundamentally different world
            // model; versions through 3 predate typed cargo, building jobs,
            // technologies, and the seven-resource stockpile, and version 4 stored
            // free-floating positions rather than exclusive cell claims and
            // building footprints; version 5 used a coarser 30 by 20 grid; version 6 had an all-land
            // map with no seed, elevation or water; versions 7 and 8 had 4×4 and 6×6 town centers and smaller buildings,
            // whose footprints would now overlap their neighbours; version 9 had a
            // 60 by 40 map. Those snapshots cannot be translated safely
            // into the current deterministic world.
            transaction.execute_batch("DROP TABLE IF EXISTS world_state;")?;
        }
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS world_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                world_json TEXT NOT NULL
            );
            PRAGMA user_version = 10;",
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn load(&self) -> StoreResult<Option<GameWorld>> {
        let connection = Connection::open(&self.path)?;
        let json: Option<String> = connection
            .query_row(
                "SELECT world_json FROM world_state WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let Some(json) = json else {
            return Ok(None);
        };
        let world = migration::load_world(&json)?;
        world
            .validate()
            .map_err(|error| format!("persisted world is corrupt: {error}"))?;
        Ok(Some(world))
    }

    pub fn save(&self, world: &GameWorld) -> StoreResult<()> {
        let json = serde_json::to_string(world)?;
        let connection = Connection::open(&self.path)?;
        connection.execute(
            "INSERT INTO world_state (id, world_json) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET world_json = excluded.world_json",
            params![json],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn temporary_db(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "age-of-agents-{name}-{}-{nonce}.db",
            std::process::id()
        ))
    }

    #[test]
    fn persistence_round_trips_mid_step_cargo_and_gather_phase() {
        let path = temporary_db("roundtrip");
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let mut world = GameWorld::default();
        world
            .apply_command(aoa_game::Command::Gather {
                unit_id: "villager-1".into(),
                resource_id: "berries-1".into(),
            })
            .unwrap();
        for _ in 0..7 {
            world.tick(0.1);
        }
        assert!(world.units[0].step.is_some());
        store.save(&world).unwrap();
        assert_eq!(store.load().unwrap().unwrap(), world);

        while world.units[0].cargo.is_none() {
            world.tick(0.1);
        }
        store.save(&world).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, world);
        assert!(matches!(
            loaded.units[0].action,
            aoa_game::UnitAction::Gather {
                phase: aoa_game::GatherPhase::Gathering,
                ..
            }
        ));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn configured_path_is_used_for_every_save() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = temporary_db("configured");
        let old_value = std::env::var_os("AGE_OF_AGENTS_DB");
        // SAFETY: this module serializes its only environment-mutating test, and
        // the value is restored before releasing the lock.
        unsafe { std::env::set_var("AGE_OF_AGENTS_DB", &path) };

        let store = Store::configured();
        store.initialize().unwrap();
        store.save(&GameWorld::default()).unwrap();
        assert_eq!(store.path(), path);
        assert!(path.exists());

        match old_value {
            Some(value) => {
                // SAFETY: guarded and restored as described above.
                unsafe { std::env::set_var("AGE_OF_AGENTS_DB", value) };
            }
            None => {
                // SAFETY: guarded and restored as described above.
                unsafe { std::env::remove_var("AGE_OF_AGENTS_DB") };
            }
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn corrupt_snapshot_is_an_error() {
        let path = temporary_db("corrupt");
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let connection = Connection::open(&path).unwrap();
        connection
            .execute(
                "INSERT INTO world_state (id, world_json) VALUES (1, 'not json')",
                [],
            )
            .unwrap();
        drop(connection);

        assert!(store.load().is_err());
        std::fs::remove_file(path).unwrap();
    }

    fn store_raw(path: &Path, value: &serde_json::Value) -> Store {
        let store = Store::from_path(path);
        store.initialize().unwrap();
        let connection = Connection::open(path).unwrap();
        connection
            .execute(
                "INSERT INTO world_state (id, world_json) VALUES (1, ?1)",
                params![serde_json::to_string(value).unwrap()],
            )
            .unwrap();
        store
    }

    fn legacy_inventories() -> serde_json::Value {
        let mut value = serde_json::to_value(GameWorld::default()).unwrap();
        value.as_object_mut().unwrap().remove("inventories");
        let stock = |wood| {
            serde_json::to_value(aoa_game::Stockpile {
                wood,
                ..Default::default()
            })
            .unwrap()
        };
        let ship = |id, wood| {
            serde_json::json!({
                "id": id, "cell": {"column": 0, "row": 0},
                "step": null, "destination": null, "heading": [1, 0],
                "passengers": [], "goods": stock(wood)
            })
        };
        value["stockpile"] = stock(10.0);
        value["ships"] = serde_json::json!([ship("transport-home", 20.0)]);
        value["islands"] = serde_json::json!([{
            "id": 1, "terrain": value["terrain"], "explored_cells": [],
            "units": [], "ships": [ship("transport-away", 40.0)],
            "resources": [], "buildings": [], "stockpile": stock(30.0)
        }]);
        value
    }

    #[test]
    fn legacy_island_and_ship_resources_are_pooled_once_and_saved() {
        let path = temporary_db("shared-resources");
        let store = store_raw(&path, &legacy_inventories());
        let world = store.load().unwrap().unwrap();
        assert_eq!(world.inventories[0].wood, 100.0);
        assert_eq!(world.ships.len(), 2);
        assert!(world.islands.is_empty());
        assert_eq!(world.island_origins.len(), 2);
        // Repeated reads and a save/reload must never credit resources twice.
        assert_eq!(store.load().unwrap().unwrap(), world);
        store.save(&world).unwrap();
        assert_eq!(store.load().unwrap().unwrap(), world);
        let json = serde_json::to_value(&world).unwrap();
        assert!(json["islands"][0].get("stockpile").is_none());
        assert!(json["ships"][0].get("goods").is_none());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn corrupt_legacy_resources_are_rejected_before_pooling() {
        for (index, pointer, amount) in [
            (0, "/islands/0/stockpile/wood", -1.0),
            (1, "/ships/0/goods/wood", -1.0),
            (2, "/islands/0/ships/0/goods/wood", 201.0),
            (3, "/stockpile/wood", -1.0),
        ] {
            let path = temporary_db(&format!("bad-inventory-{index}"));
            let mut value = legacy_inventories();
            *value.pointer_mut(pointer).unwrap() = serde_json::json!(amount);
            let store = store_raw(&path, &value);
            assert!(store.load().unwrap_err().to_string().contains("corrupt"));
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn legacy_pooling_rejects_overflow_and_malformed_stockpiles() {
        for mode in 0..3 {
            let path = temporary_db(&format!("malformed-inventory-{mode}"));
            let mut value = legacy_inventories();
            match mode {
                0 => {
                    value["stockpile"]["wood"] = serde_json::json!(1e308);
                    value["islands"][0]["stockpile"]["wood"] = serde_json::json!(1e308);
                }
                1 => {
                    value["ships"][0]["goods"]
                        .as_object_mut()
                        .unwrap()
                        .remove("food");
                }
                _ => {
                    value["islands"][0]["stockpile"]["wood"] = serde_json::Value::Null;
                }
            }
            assert!(store_raw(&path, &value).load().is_err());
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn save_missing_a_field_is_an_explicit_load_error() {
        let path = temporary_db("missing-field");
        let mut value = serde_json::to_value(GameWorld::default()).unwrap();
        value["units"][0].as_object_mut().unwrap().remove("cell");
        let error = store_raw(&path, &value).load().unwrap_err().to_string();
        assert!(error.contains("missing field `cell`"), "{error}");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn overlapping_persisted_world_is_corrupt_not_reset() {
        let path = temporary_db("overlap");
        let mut value = serde_json::to_value(GameWorld::default()).unwrap();
        value["units"][1]["cell"] = value["units"][0]["cell"].clone();
        let error = store_raw(&path, &value).load().unwrap_err().to_string();
        assert!(error.contains("persisted world is corrupt"), "{error}");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn unknown_persisted_enum_value_is_an_explicit_load_error() {
        let path = temporary_db("unknown-enum");
        let store = Store::from_path(&path);
        store.initialize().unwrap();
        let mut value = serde_json::to_value(GameWorld::default()).unwrap();
        value["resources"][0]["kind"] = serde_json::Value::String("unobtainium".into());
        let connection = Connection::open(&path).unwrap();
        connection
            .execute(
                "INSERT INTO world_state (id, world_json) VALUES (1, ?1)",
                params![serde_json::to_string(&value).unwrap()],
            )
            .unwrap();
        drop(connection);

        let error = store.load().unwrap_err().to_string();
        assert!(error.contains("unknown variant `unobtainium`"), "{error}");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn legacy_schema_is_migrated_to_an_empty_current_store() {
        let path = temporary_db("legacy");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE world_state (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    world_json TEXT NOT NULL,
                    saved_at TEXT NOT NULL
                );
                INSERT INTO world_state VALUES (1, '{\"legacy\":true}', '2026-08-01');",
            )
            .unwrap();
        drop(connection);

        let store = Store::from_path(&path);
        store.initialize().unwrap();
        assert_eq!(store.load().unwrap(), None);
        store.save(&GameWorld::default()).unwrap();
        assert!(store.load().unwrap().is_some());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn version_two_world_is_migrated_before_deserialization() {
        let path = temporary_db("version-two");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE world_state (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    world_json TEXT NOT NULL
                );
                INSERT INTO world_state VALUES (1, '{\"width\":1200,\"height\":800}');
                PRAGMA user_version = 2;",
            )
            .unwrap();
        drop(connection);

        let store = Store::from_path(&path);
        store.initialize().unwrap();
        assert_eq!(store.load().unwrap(), None);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn version_three_world_is_migrated_before_deserialization() {
        let path = temporary_db("version-three");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE world_state (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    world_json TEXT NOT NULL
                );
                INSERT INTO world_state VALUES (1, '{\"width\":2400,\"height\":1600}');
                PRAGMA user_version = 3;",
            )
            .unwrap();
        drop(connection);

        let store = Store::from_path(&path);
        store.initialize().unwrap();
        assert_eq!(store.load().unwrap(), None);
        std::fs::remove_file(path).unwrap();
    }
}
