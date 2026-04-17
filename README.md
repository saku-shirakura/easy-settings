# easy-settings

The `derive` macro allows you to concisely define structured configuration profiles that save configuration values ​​to
a `Sqlite Database`.

[![easy-settings at crates.io](https://img.shields.io/crates/v/easy-settings.svg)](https://crates.io/crates/easy-settings)
[![easy-settings at docs.rs](https://docs.rs/easy-settings/badge.svg)](https://docs.rs/easy-settings)
[![CI](https://github.com/saku-shirakura/easy-settings/actions/workflows/ci.yml/badge.svg)](https://github.com/saku-shirakura/easy-settings/actions/workflows/ci.yml)

## Overview

easy-settings is designed to define configurations with the following structure:

```text
Category                      | Item
 Root                         ┄ AAA, BBB
  ├ Category1                 ┄ CCC
  │       ├ Category2         ┄ DDD
  │       └ Category3
  │               └ Category4 ┄ EEE
  └ Category5
          └ Category4         ┄ EEE
```

In easy-settings, the above structure is defined as follows:

```rust
#[derive(Clone, easy_settings::Registry)]
#[easy_settings(categories(
    "Category1",
    "Category2",
    "Category3",
    "Category4",
    "Category5",
))]
#[easy_settings(rel(parents("Category1"), children("Category2", "Category3")))]
#[easy_settings(rel(parents("Category3", "Category5"), children("Category4")))]
pub struct RegistryExample {
    pub aaa: Option<i64>,
    pub bbb: Option<i64>,
    #[easy_settings(categories("Category1"))]
    pub ccc: Option<i64>,
    #[easy_settings(categories("Category2"))]
    pub ddd: Option<i64>,
    #[easy_settings(categories("Category4"))]
    pub eee: Option<i64>,
}
```

### Example Usage

The following example uses `SettingManager` to persist settings to `Sqlite Database`.

If using a different store, you can operate using only `Registry` without using `SettingManager`.

```rust
#[derive(Clone, easy_settings::Registry)]
pub struct RegistryExample {
    aaa: Option<i64>,
}
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Setting Manager (The setting registry can be saved to SqliteDB via this.)
    let mut manager: easy_settings::SettingManager<RegistryExample> =
        easy_settings::SettingManagerBuilder::<RegistryExample>::default().build()?;

    // The initial value of the setting is None.
    assert_eq!(manager.get_registry().get_aaa(), None);

    // To change the setting value, first change `tmp_registry`.
    manager.get_tmp_registry().set_aaa(Some(0));

    // After making changes, you can apply the changes by executing `save()`.
    manager.save().await?;

    // When applied, the setting value will also become Some(0).
    assert_eq!(manager.get_registry().get_aaa(), Some(0));
    Ok(())
}
```

## Registry

The Registry trait can be automatically implemented using `#[derive(Clone, Registry)]`.

- `Registry` must implement the `Clone` trait.

- The `Default` trait is automatically implemented by `derive(Registry)`.

- Fields in `Registry` must be `std::option::Option<T>`.

- In `Registry`, you can use types that implement [
  `serde::Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) and [
  `serde::Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html).

### Attributes

The following attributes are available in easy-settings:

#### Structures

```rust
#[derive(Clone, easy_settings::Registry)]
// Defines a list of categories.
#[easy_settings(categories("AAA", "BBB", "..."))]
// Defines relationships between categories.
// parents: Indicates that a category is a parent to `children` in the relationship definition between categories.
// children: Indicates that a category is a child to `parents` in the relationship definition between categories.
#[easy_settings(rel(parents("AAA"), children("BBB")))]
struct Example {}
```

#### Fields

```rust
#[derive(Clone, easy_settings::Registry)]
#[easy_settings(categories("AAA", "BBB", "..."))]
struct Example {
    // Specifies the initial value of the setting.
    // get_ex_default(&self) will return `i64` instead of `Option<i64>`.
    // If the setting value is `None`, it falls back to `default`.
    // If default is not specified, it is equivalent to the following implementation:
    // `get_ex_default(&self).unwrap_or_else(|| 10)`
    #[easy_settings(default = 10)]
    ex_default: Option<i64>,
    // Changes the name of the field (setting key).
    // This setting affects the following locations:
    // - Setting key in the database
    // - Setter and getter names (set_*, get_*)
    // - Keys for set and get functions
    #[easy_settings(name = "alphabet")]
    ex_name: Option<i64>,
    // Specifies the category to which the setting belongs.

    // Multiple categories can be specified.

    // If a category not defined in the category list is specified, it will be ignored.
    #[easy_settings(categories("AAA"))]
    ex_categories: Option<i64>
}
```

## Additional Information

This file has been automatically translated. The original is written in Japanese and can be found
at [README-ja.md](https://github.com/saku-shirakura/easy-settings/blob/main/README-ja.md).