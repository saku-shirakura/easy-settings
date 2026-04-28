use serde::de::DeserializeOwned;
use tracing_unwrap::ResultExt;

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Clone)]
#[doc=include_str!("../docs/en/SettingRow/details.md")]
#[doc = "```sql"]
#[doc=include_str!("../migrations/20260412100221_easy_settings_create_settings_table_Yz4Gc.sql")]
#[doc = "```"]
pub struct SettingRow {
    pub setting_key: String,
    pub value: Option<String>,
}

impl SettingRow {
    #[doc=include_str!("../docs/en/SettingRow/value.md")]
    pub fn value<T>(&self) -> Option<T>
    where
        T: DeserializeOwned,
    {
        self.value
            .as_ref()
            .and_then(|x| serde_json::from_str(x.as_str()).ok_or_log())
    }
}

#[cfg(feature = "sqlite")]
pub fn migrate(
    conn: std::sync::Arc<sqlx::SqlitePool>,
) -> impl Future<Output = Result<(), sqlx::migrate::MigrateError>> + Send {
    async move { sqlx::migrate!().run(std::ops::Deref::deref(&conn)).await }
}
