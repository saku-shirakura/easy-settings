#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc=include_str!("../README.md")]

mod db;
mod registry;
mod setting_managers;

#[doc= include_str!("../docs/en/Registry_Derive/details.md")]
pub use easy_settings_derive::Registry;

#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use crate::setting_managers::sqlite::SettingManager;
    pub use crate::setting_managers::sqlite::SettingManagerBuilder;
    pub use crate::setting_managers::sqlite::SettingManagerBuilderError;
}

pub use db::SettingRow;

pub use registry::Registry;
pub use registry::RegistryNode;
pub use registry::SettingValue;

pub mod re_export {
    pub mod serde {
        pub use serde::de::DeserializeOwned;
        pub use serde::Deserialize;
        pub use serde::Serialize;
    }

    #[cfg(feature = "sqlx")]
    pub mod sqlx {
        #[cfg(feature = "sqlite")]
        pub mod sqlite {
            pub use sqlx::SqlitePool;
        }

        pub use sqlx::Error;
        pub use sqlx::Result;
    }
}
