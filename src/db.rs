use serde::de::DeserializeOwned;
use sqlx::{migrate, FromRow, SqliteConnection};
use tracing_unwrap::ResultExt;

#[derive(FromRow)]
pub struct SettingRow {
    pub setting_key: String,
    pub value: Option<String>,
}

impl SettingRow {
    pub fn value<T>(&self) -> Option<T>
    where
        T: DeserializeOwned,
    {
        self.value
            .as_ref()
            .and_then(|x| serde_json::from_str(x.as_str()).ok_or_log())
    }
}

pub async fn migrate(conn: &mut SqliteConnection) -> Result<(), migrate::MigrateError> {
    migrate!().run(conn).await
}
