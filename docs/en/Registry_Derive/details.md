## Registry

The Registry trait can be automatically implemented using `#[derive(Clone, Registry)]`.

- [`Registry`] must implement the [`Clone`] trait.

- The [`Default`] trait is automatically implemented by `derive(Registry)`.

- Fields in [`Registry`] must be [`Option<T>`].

- In [`Registry`], you can use types that implement [
  `serde::Deserialize`] and [
  `serde::Serialize`].

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