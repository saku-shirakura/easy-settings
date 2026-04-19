use serde::de::DeserializeOwned;
use sqlx::FromRow;
use tracing_unwrap::ResultExt;

#[derive(FromRow)]
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
pub async fn migrate(conn: &mut sqlx::SqliteConnection) -> Result<(), sqlx::migrate::MigrateError> {
     sqlx::migrate!().run(conn).await
}
