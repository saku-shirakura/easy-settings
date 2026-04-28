use crate::IntoSettingRow;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::any::TypeId;
use tracing_unwrap::ResultExt;

#[derive(PartialEq, Debug)]
#[doc=include_str!("../docs/en/SettingValue/details.md")]
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
    #[doc=include_str!("../docs/en/SettingValue/from_raw_string.md")]
    pub fn from_raw_string(value: Option<String>) -> Self {
        Self(value)
    }

    #[doc=include_str!("../docs/en/SettingValue/raw_string.md")]
    pub fn raw_string(&self) -> &Option<String> {
        &self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[doc=include_str!("../docs/en/RegistryNode/details.md")]
pub enum RegistryNode {
    Category(&'static str),
    SettingItem(&'static str),
}

impl RegistryNode {
    #[doc=include_str!("../docs/en/RegistryNode/is_category.md")]
    pub fn is_category(&self) -> bool {
        match self {
            RegistryNode::Category(_) => true,
            RegistryNode::SettingItem(_) => false,
        }
    }

    #[doc=include_str!("../docs/en/RegistryNode/is_setting_item.md")]
    pub fn is_setting_item(&self) -> bool {
        match self {
            RegistryNode::Category(_) => false,
            RegistryNode::SettingItem(_) => true,
        }
    }

    #[doc=include_str!("../docs/en/RegistryNode/value.md")]
    pub fn value(&self) -> &'static str {
        match self {
            RegistryNode::Category(x) => x,
            RegistryNode::SettingItem(x) => x,
        }
    }
}

#[doc=include_str!("../docs/en/Registry_Trait/details.md")]
pub trait Registry: Default + Clone {
    #[doc=include_str!("../docs/en/Registry_Trait/set.md")]
    fn set(&mut self, key: &str, value: SettingValue);

    #[doc=include_str!("../docs/en/Registry_Trait/set_from_row.md")]
    fn set_from_row<T>(&mut self, row: T)
    where
        T: IntoSettingRow,
    {
        let row = row.into_setting_row();
        self.set(&*row.setting_key, SettingValue::from_raw_string(row.value))
    }

    #[doc=include_str!("../docs/en/Registry_Trait/set_from_row_vec.md")]
    fn set_from_row_vec<T>(&mut self, row: Vec<T>)
    where
        T: IntoSettingRow,
    {
        row.into_iter().for_each(|x| self.set_from_row(x))
    }

    #[doc=include_str!("../docs/en/Registry_Trait/get.md")]
    fn get(&self, key: &str) -> Option<SettingValue>;

    #[doc=include_str!("../docs/en/Registry_Trait/items.md")]
    fn items(&self) -> Vec<(&str, SettingValue)> {
        Self::keys()
            .iter()
            .map(|x| (*x, self.get(x).unwrap()))
            .collect()
    }

    #[doc=include_str!("../docs/en/Registry_Trait/get_item_type.md")]
    fn get_item_type(key: &str) -> Option<TypeId>;

    #[doc=include_str!("../docs/en/Registry_Trait/keys.md")]
    fn keys() -> &'static [&'static str];

    #[doc=include_str!("../docs/en/Registry_Trait/categories.md")]
    fn categories() -> &'static [&'static str];

    #[doc=include_str!("../docs/en/Registry_Trait/child_nodes.md")]
    fn child_nodes(parent_node: Option<&str>) -> &'static [RegistryNode];
}
