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