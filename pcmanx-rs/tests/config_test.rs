use std::time::{SystemTime, UNIX_EPOCH};

use pcmanx_rs::config::{AppConfig, SiteConfig};

#[test]
fn site_config_json_round_trip() {
    let site = SiteConfig {
        name: "PTT".to_string(),
        url: "ptt.cc:23".to_string(),
        ..SiteConfig::default()
    };

    let json = serde_json::to_string(&site).unwrap();
    let decoded: SiteConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(site, decoded);
}

#[test]
fn app_config_default_sites_empty() {
    let config = AppConfig::default();
    assert!(config.sites.is_empty());
}

#[test]
fn app_config_load_missing_returns_default() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("pcmanx-rs-test-{nonce}"));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let old = std::env::var_os("XDG_CONFIG_HOME");
    std::env::set_var("XDG_CONFIG_HOME", &temp_dir);

    let loaded = AppConfig::load().unwrap();

    if let Some(val) = old {
        std::env::set_var("XDG_CONFIG_HOME", val);
    } else {
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    assert_eq!(loaded, AppConfig::default());
}
