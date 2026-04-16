#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
pub struct Object {
    id: i64,
}

impl Object {
    pub fn new(id: i64) -> Self {
        Object { id }
    }

    pub fn new8000() -> Self {
        Self::new(8000)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum Combo {
    A,
    B,
    C,
}
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use easy_settings::{Registry, RegistryNode, SettingValue};

#[derive(Clone, Debug, PartialEq, Default)]
// Root ┄ object, array, datetime, combo
//  ├ Category1 ┄ abc
//  │       ├ Category2 ┄ integer
//  │       └ Category3
//  │               └ Category4 ┄ float
//  ├ Category5 ┄ abc
//  │       └ Category6
//  ├ Category7
//  │       └ Category4 ┄ float
//  ├ Category8
//  │       ├ Category9
//  │       └ Category10 ┄ bool
//  └ Category11
//          ├ Category9
//          └ Category10 ┄ bool
//
pub struct RegistryExample {
    pub integer: Option<i64>,
    pub float: Option<f64>,
    pub string: Option<String>,
    pub object: Option<Object>,
    pub array: Option<Vec<Object>>,
    pub datetime: Option<DateTime<Local>>,
    pub bool: Option<bool>,
    pub combo: Option<Combo>,
}

impl RegistryExample {
    pub fn set_integer(&mut self, value: Option<i64>) {
        self.integer = value;
    }

    pub fn get_integer(&self) -> Option<i64> {
        self.integer.clone()
    }

    pub fn set_float(&mut self, value: Option<f64>) {
        self.float = value;
    }

    pub fn get_float(&self) -> f64 {
        self.float.clone().unwrap_or_else(|| 100.0f64)
    }

    pub fn set_abc(&mut self, value: Option<String>) {
        self.string = value;
    }

    pub fn get_abc(&self) -> Option<String> {
        self.string.clone()
    }

    pub fn set_object(&mut self, value: Option<Object>) {
        self.object = value;
    }

    pub fn get_object(&self) -> Object {
        self.object.clone().unwrap_or_else(|| Object::new8000())
    }

    pub fn set_array(&mut self, value: Option<Vec<Object>>) {
        self.array = value;
    }

    pub fn get_array(&self) -> Option<Vec<Object>> {
        self.array.clone()
    }

    pub fn set_datetime(&mut self, value: Option<DateTime<Local>>) {
        self.datetime = value;
    }

    pub fn get_datetime(&self) -> Option<DateTime<Local>> {
        self.datetime.clone()
    }

    pub fn set_bool(&mut self, value: Option<bool>) {
        self.bool = value;
    }

    pub fn get_bool(&self) -> Option<bool> {
        self.bool.clone()
    }

    pub fn set_combo(&mut self, value: Option<Combo>) {
        self.combo = value;
    }

    pub fn get_combo(&self) -> Option<Combo> {
        self.combo.clone()
    }
}

impl Registry for RegistryExample {
    fn set(&mut self, key: &str, value: SettingValue) {
        match key {
            "integer" => self.set_integer(value.into()),
            "float" => self.set_float(value.into()),
            "abc" => self.set_abc(value.into()),
            "object" => self.set_object(value.into()),
            "array" => self.set_array(value.into()),
            "datetime" => self.set_datetime(value.into()),
            "bool" => self.set_bool(value.into()),
            "combo" => self.set_combo(value.into()),
            &_ => {}
        }
    }

    fn get(&self, key: &str) -> Option<SettingValue> {
        Some(match key {
            "integer" => SettingValue::from(self.integer.as_ref()),
            "float" => SettingValue::from(self.float.as_ref()),
            "abc" => SettingValue::from(self.string.as_ref()),
            "object" => SettingValue::from(self.object.as_ref()),
            "array" => SettingValue::from(self.array.as_ref()),
            "datetime" => SettingValue::from(self.datetime.as_ref()),
            "bool" => SettingValue::from(self.bool.as_ref()),
            "combo" => SettingValue::from(self.combo.as_ref()),
            &_ => return None,
        })
    }

    fn keys() -> &'static [&'static str] {
        &[
            "abc", "array", "bool", "combo", "datetime", "float", "integer", "object",
        ]
    }

    fn categories() -> &'static [&'static str] {
        &[
            "Category1",
            "Category10",
            "Category11",
            "Category2",
            "Category3",
            "Category4",
            "Category5",
            "Category6",
            "Category7",
            "Category8",
            "Category9",
        ]
    }

    fn child_nodes(parent_node: Option<&str>) -> &'static [RegistryNode] {
        match parent_node {
            None => &[
                RegistryNode::Category("Category1"),
                RegistryNode::Category("Category11"),
                RegistryNode::Category("Category5"),
                RegistryNode::Category("Category7"),
                RegistryNode::Category("Category8"),
                RegistryNode::SettingItem("array"),
                RegistryNode::SettingItem("combo"),
                RegistryNode::SettingItem("datetime"),
                RegistryNode::SettingItem("object"),
            ],
            Some(x) => match x {
                "Category1" => &[
                    RegistryNode::Category("Category2"),
                    RegistryNode::Category("Category3"),
                    RegistryNode::SettingItem("abc"),
                ],
                "Category2" => &[RegistryNode::SettingItem("integer")],
                "Category3" => &[RegistryNode::Category("Category4")],
                "Category4" => &[RegistryNode::SettingItem("float")],
                "Category5" => &[
                    RegistryNode::Category("Category6"),
                    RegistryNode::SettingItem("abc"),
                ],
                "Category6" => &[],
                "Category7" => &[RegistryNode::Category("Category4")],
                "Category8" => &[
                    RegistryNode::Category("Category10"),
                    RegistryNode::Category("Category9"),
                ],
                "Category9" => &[],
                "Category10" => &[RegistryNode::SettingItem("bool")],
                "Category11" => &[
                    RegistryNode::Category("Category10"),
                    RegistryNode::Category("Category9"),
                ],
                &_ => &[],
            },
        }
    }
}
