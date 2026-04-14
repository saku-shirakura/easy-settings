use crate::db::SettingRow;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing_unwrap::ResultExt;

#[derive(PartialEq, Debug)]
pub struct SettingValue(Option<String>);

impl<'a, T> From<Option<T>> for SettingValue
where
    T: Serialize,
{
    fn from(value: Option<T>) -> Self {
        SettingValue(
            value
                .as_ref()
                .and_then(|x| serde_json::to_string(x).ok_or_log()),
        )
    }
}

impl<T> From<SettingValue> for Option<T>
where
    T: DeserializeOwned,
{
    fn from(value: SettingValue) -> Self {
        value
            .0
            .and_then(|x| serde_json::from_str(x.as_str()).ok_or_log())
    }
}

impl SettingValue {
    pub fn from_raw_string(value: Option<String>) -> Self {
        Self(value)
    }

    pub fn raw_string(&self) -> &Option<String> {
        &self.0
    }
}

pub enum RegistryNode {
    Category(&'static str),
    SettingItem(&'static str),
}

impl RegistryNode {
    pub fn is_category(&self) -> bool {
        match self {
            RegistryNode::Category(_) => true,
            RegistryNode::SettingItem(_) => false,
        }
    }

    pub fn is_setting_item(&self) -> bool {
        match self {
            RegistryNode::Category(_) => false,
            RegistryNode::SettingItem(_) => true,
        }
    }

    pub fn value(&self) -> &'static str {
        match self {
            RegistryNode::Category(x) => x,
            RegistryNode::SettingItem(x) => x,
        }
    }
}

pub trait Registry: Default + Clone {
    fn set(&mut self, key: &str, value: SettingValue);

    fn set_from_row(&mut self, row: SettingRow);

    fn set_from_row_vec(&mut self, row: Vec<SettingRow>) {
        row.into_iter().for_each(|x| self.set_from_row(x))
    }

    fn get(&self, key: &str) -> Option<SettingValue>;

    fn items(&self) -> Vec<(&str, SettingValue)> {
        Self::keys()
            .iter()
            .map(|x| (*x, self.get(x).unwrap()))
            .collect()
    }

    fn keys() -> &'static [&'static str];

    fn categories() -> &'static [&'static str];

    fn child_nodes(parent_node: Option<&str>) -> &'static [RegistryNode];
}
