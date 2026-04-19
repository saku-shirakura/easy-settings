mod auto_impl;
mod manual_impl;
use chrono::DateTime;
use easy_settings::sqlite::{SettingManager, SettingManagerBuilder};
use easy_settings::{Registry, SettingValue};
use sqlx::{query, Row, SqlitePool};
use std::str::FromStr;
use std::sync::Arc;

fn auto_to_manual_object(obj: auto_impl::Object) -> manual_impl::Object {
    manual_impl::Object::new(obj.id)
}

fn auto_to_manual_conbo(combo: auto_impl::Combo) -> manual_impl::Combo {
    match combo {
        auto_impl::Combo::A => manual_impl::Combo::A,
        auto_impl::Combo::B => manual_impl::Combo::B,
        auto_impl::Combo::C => manual_impl::Combo::C,
    }
}

fn auto_to_manual(reg: auto_impl::RegistryExample) -> manual_impl::RegistryExample {
    manual_impl::RegistryExample {
        integer: reg.integer,
        float: reg.float,
        string: reg.string,
        object: reg.object.map(auto_to_manual_object),
        array: reg
            .array
            .map(|x| x.into_iter().map(auto_to_manual_object).collect()),
        datetime: reg.datetime,
        bool: reg.bool,
        combo: reg.combo.map(auto_to_manual_conbo),
    }
}

#[tokio::test]
async fn derive_implementation_test() {
    let pool = Arc::new(SqlitePool::connect(":memory:").await.unwrap());

    query("CREATE TABLE IF NOT EXISTS settings (setting_key TEXT PRIMARY KEY CHECK ( setting_key <> '' ), value TEXT)")
        .execute(&mut *pool.acquire().await.unwrap()).await.unwrap();

    let mut manual_manager: SettingManager<manual_impl::RegistryExample> =
        SettingManagerBuilder::default()
            .db_pool(Some(pool.clone()))
            .await
            .build()
            .unwrap();
    let mut auto_manager: SettingManager<auto_impl::RegistryExample> =
        SettingManagerBuilder::default()
            .tablename("settings", pool.clone())
            .build()
            .unwrap();

    // Default value
    assert_eq!(
        manual_manager.get_registry().clone(),
        auto_to_manual(auto_manager.get_registry().clone())
    );
    assert_eq!(
        manual_manager.get_tmp_registry().clone(),
        auto_to_manual(auto_manager.get_tmp_registry().clone())
    );

    // Save And Load test

    // manual
    manual_manager.get_tmp_registry().set_integer(Some(90));
    manual_manager.get_tmp_registry().set_float(Some(90f64));
    manual_manager
        .get_tmp_registry()
        .set_abc(Some("90f64".into()));
    manual_manager
        .get_tmp_registry()
        .set_object(Some(manual_impl::Object::new(500)));
    manual_manager.get_tmp_registry().set_array(Some(vec![
        manual_impl::Object::new(600),
        manual_impl::Object::new(700),
        manual_impl::Object::new(800),
    ]));
    manual_manager
        .get_tmp_registry()
        .set_datetime(Some(DateTime::from_str("2026-01-01T00:00:00Z").unwrap()));
    manual_manager.get_tmp_registry().set_bool(Some(true));
    manual_manager
        .get_tmp_registry()
        .set_combo(Some(manual_impl::Combo::A));
    let manual_before_apply = manual_manager.get_tmp_registry().clone();
    manual_manager.save().await.unwrap();

    // derive
    auto_manager.get_tmp_registry().set_integer(Some(90));
    auto_manager.get_tmp_registry().set_float(Some(90f64));
    auto_manager
        .get_tmp_registry()
        .set_abc(Some("90f64".into()));
    auto_manager
        .get_tmp_registry()
        .set_object(Some(auto_impl::Object::new(500)));
    auto_manager.get_tmp_registry().set_array(Some(vec![
        auto_impl::Object::new(600),
        auto_impl::Object::new(700),
        auto_impl::Object::new(800),
    ]));
    auto_manager
        .get_tmp_registry()
        .set_datetime(Some(DateTime::from_str("2026-01-01T00:00:00Z").unwrap()));
    auto_manager.get_tmp_registry().set_bool(Some(true));
    auto_manager
        .get_tmp_registry()
        .set_combo(Some(auto_impl::Combo::A));
    let auto_before_apply = auto_manager.get_tmp_registry().clone();
    auto_manager.save().await.unwrap();

    // Check Db

    assert_eq!(query("SELECT COUNT(*) AS c FROM (SELECT setting_key FROM settings UNION ALL SELECT setting_key FROM settings__easy_settings_Yz4Gc) x LEFT OUTER JOIN settings__easy_settings_Yz4Gc sesY4G on x.setting_key = sesY4G.setting_key LEFT OUTER JOIN settings on x.setting_key = settings.setting_key WHERE sesY4G.value <> settings.value OR (sesY4G.value IS NOT NULL AND settings.value IS NULL) OR (sesY4G.value IS NULL AND settings.value IS NOT NULL);")
                   .fetch_one(&mut *pool.acquire().await.unwrap()).await.unwrap().get::<i64, _>("c"),
               0);

    // Applied
    assert_eq!(
        manual_manager.get_tmp_registry().clone(),
        auto_to_manual(auto_manager.get_tmp_registry().clone())
    );
    assert_eq!(
        manual_manager.get_registry().clone(),
        auto_to_manual(auto_manager.get_registry().clone())
    );
    assert_eq!(
        manual_manager.get_registry().clone(),
        manual_manager.get_tmp_registry().clone()
    );
    assert_eq!(manual_manager.get_registry().clone(), manual_before_apply);
    assert_eq!(auto_manager.get_registry().clone(), auto_before_apply);

    assert_eq!(
        manual_impl::RegistryExample::keys(),
        auto_impl::RegistryExample::keys()
    );
    assert_eq!(
        manual_impl::RegistryExample::categories(),
        auto_impl::RegistryExample::categories()
    );
    assert_eq!(
        manual_impl::RegistryExample::child_nodes(None),
        auto_impl::RegistryExample::child_nodes(None)
    );
    manual_impl::RegistryExample::categories()
        .iter()
        .for_each(|x| {
            assert_eq!(
                manual_impl::RegistryExample::child_nodes(Some(x)),
                auto_impl::RegistryExample::child_nodes(Some(x))
            );
        });

    // trait Registry Impl Test

    // get
    let auto_reg = auto_manager.get_registry().clone();
    let man_reg = manual_manager.get_registry().clone();
    assert_eq!(
        SettingValue::from(auto_reg.integer.as_ref()),
        auto_reg.get("integer").unwrap()
    );
    assert_eq!(
        SettingValue::from(auto_reg.float.as_ref()),
        auto_reg.get("float").unwrap()
    );
    assert_eq!(
        SettingValue::from(auto_reg.string.as_ref()),
        auto_reg.get("abc").unwrap()
    );
    assert_eq!(
        SettingValue::from(auto_reg.object.as_ref()),
        auto_reg.get("object").unwrap()
    );
    assert_eq!(
        SettingValue::from(auto_reg.array.as_ref()),
        auto_reg.get("array").unwrap()
    );
    assert_eq!(
        SettingValue::from(auto_reg.datetime.as_ref()),
        auto_reg.get("datetime").unwrap()
    );
    assert_eq!(
        SettingValue::from(auto_reg.bool.as_ref()),
        auto_reg.get("bool").unwrap()
    );
    assert_eq!(
        SettingValue::from(auto_reg.combo.as_ref()),
        auto_reg.get("combo").unwrap()
    );
    assert_eq!(
        SettingValue::from(man_reg.integer.as_ref()),
        man_reg.get("integer").unwrap()
    );
    assert_eq!(
        SettingValue::from(man_reg.float.as_ref()),
        man_reg.get("float").unwrap()
    );
    assert_eq!(
        SettingValue::from(man_reg.string.as_ref()),
        man_reg.get("abc").unwrap()
    );
    assert_eq!(
        SettingValue::from(man_reg.object.as_ref()),
        man_reg.get("object").unwrap()
    );
    assert_eq!(
        SettingValue::from(man_reg.array.as_ref()),
        man_reg.get("array").unwrap()
    );
    assert_eq!(
        SettingValue::from(man_reg.datetime.as_ref()),
        man_reg.get("datetime").unwrap()
    );
    assert_eq!(
        SettingValue::from(man_reg.bool.as_ref()),
        man_reg.get("bool").unwrap()
    );
    assert_eq!(
        SettingValue::from(man_reg.combo.as_ref()),
        man_reg.get("combo").unwrap()
    );
    // getter
    let mut auto_reg = auto_reg.clone();
    let mut man_reg = man_reg.clone();

    // auto
    assert_eq!(auto_reg.get_integer(), Some(90));
    auto_reg.set_integer(None);
    assert_eq!(auto_reg.get_integer(), None);
    assert_eq!(auto_reg.get_float(), 90f64);
    auto_reg.set_float(None);
    assert_eq!(auto_reg.get_float(), 100f64);
    assert_eq!(auto_reg.get_abc(), Some("90f64".into()));
    auto_reg.set_abc(None);
    assert_eq!(auto_reg.get_abc(), None);
    assert_eq!(auto_reg.get_object(), auto_impl::Object::new(500));
    auto_reg.set_object(None);
    assert_eq!(auto_reg.get_object(), auto_impl::Object::new8000());
    assert_eq!(
        auto_reg.get_array(),
        Some(vec![
            auto_impl::Object::new(600),
            auto_impl::Object::new(700),
            auto_impl::Object::new(800),
        ])
    );
    auto_reg.set_array(None);
    assert_eq!(auto_reg.get_array(), None);
    assert_eq!(
        auto_reg.get_datetime(),
        Some(DateTime::from_str("2026-01-01T00:00:00Z").unwrap())
    );
    auto_reg.set_datetime(None);
    assert_eq!(auto_reg.get_datetime(), None);
    assert_eq!(auto_reg.get_bool(), Some(true));
    auto_reg.set_bool(None);
    assert_eq!(auto_reg.get_bool(), None);
    assert_eq!(auto_reg.get_combo(), Some(auto_impl::Combo::A));
    auto_reg.set_combo(None);
    assert_eq!(auto_reg.get_combo(), None);
    // manual
    assert_eq!(man_reg.get_integer(), Some(90));
    man_reg.set_integer(None);
    assert_eq!(man_reg.get_integer(), None);
    assert_eq!(man_reg.get_float(), 90f64);
    man_reg.set_float(None);
    assert_eq!(man_reg.get_float(), 100f64);
    assert_eq!(man_reg.get_abc(), Some("90f64".into()));
    man_reg.set_abc(None);
    assert_eq!(man_reg.get_abc(), None);
    assert_eq!(man_reg.get_object(), manual_impl::Object::new(500));
    man_reg.set_object(None);
    assert_eq!(man_reg.get_object(), manual_impl::Object::new8000());
    assert_eq!(
        man_reg.get_array(),
        Some(vec![
            manual_impl::Object::new(600),
            manual_impl::Object::new(700),
            manual_impl::Object::new(800),
        ])
    );
    man_reg.set_array(None);
    assert_eq!(man_reg.get_array(), None);
    assert_eq!(
        man_reg.get_datetime(),
        Some(DateTime::from_str("2026-01-01T00:00:00Z").unwrap())
    );
    man_reg.set_datetime(None);
    assert_eq!(man_reg.get_datetime(), None);
    assert_eq!(man_reg.get_bool(), Some(true));
    man_reg.set_bool(None);
    assert_eq!(man_reg.get_bool(), None);
    assert_eq!(man_reg.get_combo(), Some(manual_impl::Combo::A));
    man_reg.set_combo(None);
    assert_eq!(man_reg.get_combo(), None);
}
