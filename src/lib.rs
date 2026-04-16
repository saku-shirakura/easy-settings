#![doc=include_str!("../README.md")]

mod db;
mod registry;
mod setting_manager;

#[doc= include_str!("../docs/en/Registry_Derive/details.md")]
pub use easy_settings_derive::Registry;

pub use setting_manager::SettingManager;
pub use setting_manager::SettingManagerBuilder;
pub use setting_manager::SettingManagerBuilderError;

pub use db::SettingRow;

pub use registry::Registry;
pub use registry::RegistryNode;
pub use registry::SettingValue;
