# easy-settings

derive マクロを使用し、設定値を`Sqlite Database`に保存する、構造的な設定プロファイルを簡潔に定義できるようにします。

[![easy-settings at crates.io](https://img.shields.io/crates/v/easy-settings.svg)](https://crates.io/crates/easy-settings)
[![easy-settings at docs.rs](https://docs.rs/easy-settings/badge.svg)](https://docs.rs/easy-settings)
[![CI](https://github.com/saku-shirakura/easy-settings/actions/workflows/ci.yml/badge.svg)](https://github.com/saku-shirakura/easy-settings/actions/workflows/ci.yml)

## 概要

easy-settingsは、以下のような構造を持った設定を定義するために設計されています。

```
カテゴリ                       | 項目
 Root                         ┄ AAA, BBB
  ├ Category1                 ┄ CCC
  │       ├ Category2         ┄ DDD
  │       └ Category3
  │               └ Category4 ┄ EEE
  └ Category5
          └ Category4         ┄ EEE
```

上記のような構造をeasy-settingsでは以下のように定義します。

```rust
#[derive(Clone, Registry)]
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

### 使用例

以下の例では、 `SettingManager` を使用し、 `Sqlite Database` に設定値を永続化しています。

別のストアを使用する場合には、 `SettingManager` を使用せず、`Registry` のみで運用することもできます。

```rust
#[derive(Clone, Registry)]
pub struct RegistryExample {
    aaa: Option<i64>,
}

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // 設定マネージャ(設定レジストリはこれを経由することで、SqliteDBに保存する事ができます。)
    let mut manager: SettingManager<RegistryExample> =
        SettingManagerBuilder::<RegistryExample>::default().build()?;

    // 設定の初期値はNoneです。
    assert_eq!(manager.get_registry().get_aaa(), None);
    // 設定値を変更する場合、まずは `tmp_registry` を変更します。
    manager.get_tmp_registry().set_aaa(Some(0));
    // 変更後、`save()`を実行することで変更を適用できます。
    manager.save().await?;
    // 適用した場合、設定値もSome(0)になります。
    assert_eq!(manager.get_registry().get_aaa(), Some(0));
    Ok(())
}
```

## Registry

`#[derive(Clone, Registry)]`を使用することで、Registryトレイトを自動で実装できます。

- `Registry` には、`Clone`トレイトの実装が必須です。
- `Default`トレイトは、`derive(Registry)`により、自動的に実装されます。
- `Registry` のフィールドは `std::option::Option<T>` でなければなりません。
- `Registry` では、[`serde::Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) 及び [
  `serde::Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html)を実装している型を使用できます。

### 属性

easy-settingsで使用可能な属性を以下に示します。

#### 構造体

```rust
// カテゴリのリストを定義します。
#[easy_settings(categories("AAA", "BBB", "..."))]
// カテゴリ間の関係を定義します。
// parents: カテゴリ間の関係定義において、 `children` に対する親であることを表します。
// children: カテゴリ間の関係定義において、 `parents` に対する子であることを表します。
#[easy_settings(rel(parents(), children(..)))]
struct Example{}
```

#### フィールド

```rust
#[easy_settings(categories("AAA", "BBB", "..."))]
struct Example{
  // 設定の初期値を指定します。
  // get_ex_default(&self) が`Option<i64>`の代わりに、`i64`を返すようになります。
  // 設定値が`None`の場合、`default`にフォールバックします。
  // defaultを指定しない場合、以下の実装と同等です。
  // `get_ex_default(&self).unwrap_or_else(|| 10)`
  #[easy_settings(default = 10)]
  ex_default: Option<i64>,
  // フィールドの名称(設定キー)を変更します。
  // この設定は以下の場所に影響を及ぼします。
  // - DB上での設定キー
  // - setter, getterの名称 (set_*, get_*)
  // - set, get関数のキー
  #[easy_settings(name = "alphabet")]
  ex_name: Option<i64>,
  // 設定が属するカテゴリを指定します。
  // カテゴリは複数指定できます。
  // カテゴリのリストで定義されていないものを指定した場合、無視されます。
  #[easy_settings(categories("AAA"))]
  ex_categories: Option<i64>
}
```

