#![allow(dead_code)]
use crate::db::{migrate, SettingRow};
use crate::registry::{Registry, SettingValue};
use derive_builder::Builder;
use sqlx::migrate::MigrateError;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{query, Acquire, SqlitePool};
use std::collections::HashMap;
use std::ops::{AddAssign, Sub};
use std::sync::{Arc, LazyLock, OnceLock, RwLock};
use tracing::{warn, warn_span};
use tracing_unwrap::ResultExt;

const DEFAULT_TABLE_NAME: &str = "settings__easy_settings_Yz4Gc";

static APPLICABLE_STATUS: LazyLock<RwLock<HashMap<u64, bool>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static NEXT_MANAGER_ID: LazyLock<RwLock<u64>> = LazyLock::new(|| RwLock::new(0));

#[tracing::instrument]
fn update_applicable_status(id: u64, is_saved: bool) {
    APPLICABLE_STATUS
        .write()
        .unwrap_or_log()
        .insert(id, is_saved);
}

#[tracing::instrument]
fn get_applicable_status(id: &u64) -> bool {
    APPLICABLE_STATUS
        .read()
        .unwrap_or_log()
        .get(id)
        .cloned()
        .unwrap_or_default()
}

#[tracing::instrument]
fn issue_manager_id() -> u64 {
    match NEXT_MANAGER_ID.write().ok_or_log().as_mut() {
        None => {
            panic!("[issue_manager_id] Lock error")
        }
        Some(x) => {
            x.add_assign(1);
            x.sub(1)
        }
    }
}

static DEFAULT_DATABASE_POOL: OnceLock<Arc<SqlitePool>> = OnceLock::new();
static DATABASE_POOLS: LazyLock<RwLock<HashMap<u64, Arc<SqlitePool>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static NEXT_POOL_ID: LazyLock<RwLock<u64>> = LazyLock::new(|| RwLock::new(0));

fn register_database_pool(pool: Arc<SqlitePool>) -> u64 {
    match NEXT_POOL_ID.write().ok_or_log().as_mut() {
        None => {
            panic!("[easy_settings::sqlite::add_database_pool] Lock Failed")
        }
        Some(x) => {
            DATABASE_POOLS.write().unwrap_or_log().insert(**x, pool);
            x.add_assign(1);
            x.sub(1)
        }
    }
}

async fn get_database_pool(id: &Option<u64>) -> Arc<SqlitePool> {
    match id.as_ref().and_then(|id| {
        DATABASE_POOLS
            .read()
            .ok_or_log()
            .and_then(|x| x.get(id).cloned())
    }) {
        None => default_database_pool().await,
        Some(x) => x,
    }
}

#[tracing::instrument]
async fn default_database_pool() -> Arc<SqlitePool> {
    match DEFAULT_DATABASE_POOL.get() {
        None => {
            let x = Arc::new(
                SqlitePool::connect_with(
                    SqliteConnectOptions::new()
                        .filename("settings.db")
                        .foreign_keys(true)
                        .create_if_missing(true),
                )
                .await
                .unwrap_or_log(),
            );
            DEFAULT_DATABASE_POOL.set(x.clone()).unwrap_or_log();
            migrate_pool(x.clone()).await.unwrap_or_log();
            x
        }
        Some(x) => x.clone(),
    }
}

#[tracing::instrument]
async fn migrate_pool(pool: Arc<SqlitePool>) -> Result<(), MigrateError> {
    migrate(&mut pool.acquire().await.unwrap_or_log()).await
}

#[derive(Builder, Clone)]
#[doc = include_str!("../../docs/en/SettingManager/details.md")]
#[builder(build_fn(private, name = "build_"))]
pub struct SettingManager<R>
where
    R: Registry,
{
    #[builder(default = R::default(), setter(skip))]
    registry: R,
    #[builder(default = R::default(), setter(skip))]
    registry_tmp: R,
    #[builder(default = DEFAULT_TABLE_NAME.into())]
    #[builder(setter(custom))]
    tablename: String,
    #[builder(default = Default::default(), setter(custom))]
    pool_id: Option<u64>,
    #[builder(setter(skip))]
    manager_id: u64,
}

impl<R> SettingManagerBuilder<R>
where
    R: Registry,
{
    #[doc = include_str!("../../docs/en/SettingManagerBuilder/tablename.md")]
    #[doc = "```sql"]
    #[doc = include_str!("../../migrations/20260412100221_easy_settings_create_settings_table_Yz4Gc.sql")]
    #[doc = "```"]
    pub fn tablename(&mut self, tablename: impl Into<String>, pool: Arc<SqlitePool>) -> &mut Self {
        self.tablename = Some(tablename.into());
        self.pool_id = Some(Some(register_database_pool(pool)));
        self
    }

    #[doc = include_str!("../../docs/en/SettingManagerBuilder/db_pool.md")]
    pub async fn db_pool(&mut self, pool: Option<Arc<SqlitePool>>) -> &mut Self {
        self.pool_id = Some(pool.map(|x| register_database_pool(x)));
        if self.tablename.is_none() {
            migrate_pool(get_database_pool(&self.pool_id.clone().and_then(|x| x)).await)
                .await
                .unwrap_or_log();
        }
        self
    }

    pub fn build(&self) -> Result<SettingManager<R>, SettingManagerBuilderError> {
        let mut manager = self.build_()?;
        manager.manager_id = issue_manager_id();
        Ok(manager)
    }
}

impl<R> SettingManager<R>
where
    R: Registry,
{
    #[doc = include_str!("../../docs/en/SettingManager/load_all.md")]
    pub async fn load_all(&mut self) -> sqlx::Result<()> {
        update_applicable_status(self.manager_id, false);
        let reg: Vec<SettingRow> = sqlx::query_as(&format!("SELECT * FROM {}", self.tablename))
            .fetch_all(&mut *self.get_pool().await.acquire().await?)
            .await?;
        self.registry.set_from_row_vec(reg);
        self.reset_tmp();
        Ok(())
    }

    #[doc = include_str!("../../docs/en/SettingManager/load.md")]
    pub async fn load(&mut self, key: &str) -> sqlx::Result<()> {
        update_applicable_status(self.manager_id, false);
        let reg: Vec<SettingRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE setting_key = ?",
            self.tablename
        ))
        .bind(key)
        .fetch_all(&mut *self.get_pool().await.acquire().await?)
        .await?;
        self.registry.set_from_row_vec(reg);
        self.reset_tmp();
        Ok(())
    }

    #[doc = include_str!("../../docs/en/SettingManager/save_and_apply.md")]
    pub async fn save_and_apply(&mut self) -> sqlx::Result<()> {
        self.save().await?;
        self.apply();
        Ok(())
    }

    #[doc = include_str!("../../docs/en/SettingManager/save.md")]
    pub async fn save(&self) -> sqlx::Result<()> {
        update_applicable_status(self.manager_id, false);
        let x: Vec<(&str, SettingValue)> = self
            .registry_tmp
            .items()
            .into_iter()
            .filter(|(k, v)| match self.registry.get(k) {
                None => true,
                Some(x) => x != *v,
            })
            .collect();
        let mut conn = self.get_pool().await.acquire().await?;
        let mut tx = conn.begin().await?;
        for (k, v) in x {
            query(&format!(
                "INSERT INTO {} (setting_key, value) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET setting_key = ?1, value = ?2;",
                self.tablename)
            ).bind(k).bind(v.raw_string()).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        update_applicable_status(self.manager_id, true);
        Ok(())
    }

    #[doc = include_str!("../../docs/en/SettingManager/apply.md")]
    pub fn apply(&mut self) -> bool {
        let status = get_applicable_status(&self.manager_id);
        update_applicable_status(self.manager_id, false);
        if status {
            self.registry = self.registry_tmp.clone();
            true
        } else {
            warn_span!("SettingManager<R>::apply")
                .in_scope(|| warn!("It is not applicable, but apply was called."));
            false
        }
    }

    #[doc = include_str!("../../docs/en/SettingManager/reset_tmp.md")]
    pub fn reset_tmp(&mut self) {
        self.registry_tmp = self.registry.clone();
        update_applicable_status(self.manager_id, false);
    }

    #[doc = include_str!("../../docs/en/SettingManager/get_tmp_registry.md")]
    pub fn get_tmp_registry(&mut self) -> &mut R {
        update_applicable_status(self.manager_id, false);
        &mut self.registry_tmp
    }

    #[doc = include_str!("../../docs/en/SettingManager/get_registry.md")]
    pub fn get_registry(&self) -> &R {
        &self.registry
    }

    #[doc = include_str!("../../docs/en/SettingManager/set_pool.md")]
    pub async fn set_pool(&mut self, pool: Option<Arc<SqlitePool>>) {
        self.pool_id = pool.map(|x| register_database_pool(x));
        if self.tablename == DEFAULT_TABLE_NAME {
            migrate_pool(self.get_pool().await).await.unwrap_or_log();
        }
        update_applicable_status(self.manager_id, false);
    }

    async fn get_pool(&self) -> Arc<SqlitePool> {
        get_database_pool(&self.pool_id).await
    }
}
