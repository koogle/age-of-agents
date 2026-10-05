use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use aoa_game::GameWorld;

// No save migrations during development. Bump this when the persisted model changes.
const STORE_VERSION: u32 = 14;

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
        let transaction = connection.transaction()?;
        if schema_version != STORE_VERSION {
            transaction.execute_batch("DROP TABLE IF EXISTS world_state;")?;
        }
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS world_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                world_json TEXT NOT NULL
            );",
        )?;
        transaction.pragma_update(None, "user_version", STORE_VERSION)?;
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
        let world: GameWorld = serde_json::from_str(&json)?;
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
    fn current_store_rejects_obsolete_fields_and_missing_required_fields() {
        for (index, field) in [
            "island_origins",
            "economy_rules",
            "ships",
            "queue",
            "islands",
        ]
        .iter()
        .enumerate()
        {
            let path = temporary_db(&format!("schema-{index}"));
            let mut value = serde_json::to_value(GameWorld::default()).unwrap();
            match *field {
                "queue" => {
                    value["buildings"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("queue");
                }
                "islands" => {
                    value["islands"] = serde_json::json!([]);
                }
                _ => {
                    value.as_object_mut().unwrap().remove(*field);
                }
            }
            let store = store_raw(&path, &value);
            assert!(store.load().is_err(), "accepted obsolete schema: {field}");
            store.initialize().unwrap();
            assert!(
                store.load().is_err(),
                "reset corrupt current store: {field}"
            );
            std::fs::remove_file(path).unwrap();
        }
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
    fn incompatible_store_versions_reset_before_deserialization() {
        for version in [0, 2, 10, 11, 12, 13, STORE_VERSION + 1] {
            let path = temporary_db(&format!("version-{version}"));
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE world_state (id INTEGER PRIMARY KEY, world_json TEXT NOT NULL);
                 INSERT INTO world_state VALUES (1, 'incompatible snapshot');",
                )
                .unwrap();
            connection
                .pragma_update(None, "user_version", version)
                .unwrap();
            drop(connection);

            let store = Store::from_path(&path);
            store.initialize().unwrap();
            assert_eq!(store.load().unwrap(), None);
            let world = GameWorld::default();
            store.save(&world).unwrap();
            store.initialize().unwrap();
            assert_eq!(store.load().unwrap(), Some(world));
            let connection = Connection::open(&path).unwrap();
            let version: u32 = connection
                .query_row("PRAGMA user_version", [], |row| row.get(0))
                .unwrap();
            assert_eq!(version, STORE_VERSION);
            drop(connection);
            std::fs::remove_file(path).unwrap();
        }
    }
}
