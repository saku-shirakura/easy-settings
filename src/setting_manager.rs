#![allow(dead_code)]
use crate::db::{migrate, SettingRow};
use crate::registry::{Registry, SettingValue};
use derive_builder::Builder;
use sqlx::migrate::MigrateError;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{query, Acquire, SqlitePool};
use std::error::Error;
use std::sync::{Arc, OnceLock};
use tracing_unwrap::ResultExt;

static DEFAULT_DATABASE_POOL: OnceLock<Arc<SqlitePool>> = OnceLock::new();

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

#[derive(Builder)]
pub struct SettingManager<R>
where
    R: Registry,
{
    #[builder(default = R::default(), setter(skip))]
    registry: R,
    #[builder(default = R::default(), setter(skip))]
    registry_tmp: R,
    #[builder(default = "settings__easy_settings_Yz4Gc".into())]
    #[builder(setter(into))]
    tablename: String,
    #[builder(default = Default::default(), setter(custom))]
    pool: Option<Arc<SqlitePool>>,
}

impl<R> SettingManagerBuilder<R>
where
    R: Registry,
{
    pub async fn db_pool(&mut self, pool: Option<Arc<SqlitePool>>) -> &mut Self {
        self.pool = Some(pool);
        migrate_pool(match self.pool.as_ref() {
            None | Some(None) => default_database_pool().await,
            Some(Some(x)) => x.clone(),
        })
        .await
        .unwrap_or_log();
        self
    }
}

impl<R> SettingManager<R>
where
    R: Registry,
{
    pub async fn load_all(&mut self) -> sqlx::Result<()> {
        let reg: Vec<SettingRow> = sqlx::query_as(&format!("SELECT * FROM {}", self.tablename))
            .fetch_all(&mut *self.get_pool().await.acquire().await?)
            .await?;
        self.registry.set_from_row_vec(reg);
        self.reset_tmp();
        Ok(())
    }

    pub async fn load(&mut self, key: &str) -> sqlx::Result<()> {
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

    pub async fn save(&mut self) -> Result<(), Box<dyn Error>> {
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
            ).bind(k).bind(v.raw_string()).execute(&mut  *tx).await?;
        }
        tx.commit().await?;
        self.load_all().await?;
        Ok(())
    }

    pub fn reset_tmp(&mut self) {
        self.registry_tmp = self.registry.clone();
    }

    pub fn get_tmp_registry(&mut self) -> &mut R {
        &mut self.registry_tmp
    }

    pub fn get_registry(&self) -> &R {
        &self.registry
    }

    pub async fn set_pool(&mut self, pool: Option<Arc<SqlitePool>>) {
        self.pool = pool;
        migrate_pool(self.get_pool().await).await.unwrap_or_log();
    }

    async fn get_pool(&self) -> Arc<SqlitePool> {
        match self.pool.clone() {
            None => default_database_pool().await,
            Some(x) => x,
        }
    }
}
