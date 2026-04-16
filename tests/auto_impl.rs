#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
pub struct Object {
    pub id: i64,
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
use easy_settings::Registry;

#[derive(Clone, Registry, PartialEq, Debug)]
#[easy_settings(categories(
    "Category1",
    "Category2",
    "Category3",
    "Category4",
    "Category5",
    "Category6",
    "Category7",
    "Category8",
))]
#[easy_settings(rel(parents("Category1"), children("Category2", "Category3")))]
#[easy_settings(
    rel(parents("Category5"), children("Category6")),
    rel(parents("Category3", "Category7"), children("Category4"))
)]
#[easy_settings(
    categories("Category9", "Category10", "Category11"),
    rel(
        parents("Category8"),
        parents("Category11"),
        children("Category9"),
        children("Category10")
    )
)]
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
    #[easy_settings(categories("Category2"))]
    pub integer: Option<i64>,
    #[easy_settings(categories("Category4"), default = 100.0f64)]
    pub float: Option<f64>,
    #[easy_settings(name = "abc", categories("Category1", "Category5"))]
    pub string: Option<String>,
    #[easy_settings(default = Object::new8000())]
    pub object: Option<Object>,
    pub array: Option<Vec<Object>>,
    pub datetime: Option<DateTime<Local>>,
    #[easy_settings(categories("Category10"))]
    pub bool: Option<bool>,
    pub combo: Option<Combo>,
}
