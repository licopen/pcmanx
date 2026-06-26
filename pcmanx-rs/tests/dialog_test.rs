/// Tests for the dialog data models (no GTK widgets needed).
use pcmanx_rs::config::{AutoLogin, CrLfMode, ProxyConfig, ProxyType, SiteConfig};
use pcmanx_rs::charset::Encoding;

fn make_site() -> SiteConfig {
    SiteConfig {
        name: "Test BBS".to_string(),
        url: "bbs.example.com:23".to_string(),
        encoding: Encoding::Big5,
        auto_reconnect: 30,
        anti_idle: 120,
        anti_idle_str: "^[OB".to_string(),
        detect_dbchar: true,
        rows_per_page: 24,
        cols_per_page: 80,
        auto_wrap_on_paste: 78,
        term_type: "vt100".to_string(),
        crlf: CrLfMode::Cr,
        auto_login: AutoLogin {
            pre_login: String::new(),
            pre_login_prompt: String::new(),
            login: "myuser".to_string(),
            login_prompt: "login:".to_string(),
            passwd: "s3cr3t".to_string(),
            passwd_prompt: "password:".to_string(),
            post_login: String::new(),
        },
        uao: 2,
        proxy: Some(ProxyConfig {
            proxy_type: ProxyType::Socks5,
            addr: "proxy.example.com".to_string(),
            port: "1080".to_string(),
            user: "proxyuser".to_string(),
            pass: "proxypass".to_string(),
        }),
    }
}

/// SiteConfig round-trips through JSON serialization (the same persistence
/// path SiteEditDialog::get_site() ultimately relies on).
#[test]
fn site_config_json_round_trip() {
    let original = make_site();
    let json = serde_json::to_string(&original).expect("serialize");
    let restored: SiteConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, restored);
}

#[test]
fn site_config_default_is_valid() {
    let site = SiteConfig::default();
    assert_eq!(site.rows_per_page, 24);
    assert_eq!(site.cols_per_page, 80);
    assert_eq!(site.encoding, Encoding::Big5);
}

#[test]
fn site_config_encoding_variants_round_trip() {
    for enc in &[Encoding::Big5, Encoding::Uao241, Encoding::Uao250, Encoding::Utf8] {
        let mut site = make_site();
        site.encoding = *enc;
        let json = serde_json::to_string(&site).expect("serialize");
        let restored: SiteConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.encoding, *enc);
    }
}

#[test]
fn site_config_proxy_none_round_trip() {
    let mut site = make_site();
    site.proxy = None;
    let json = serde_json::to_string(&site).expect("serialize");
    let restored: SiteConfig = serde_json::from_str(&json).expect("deserialize");
    assert!(restored.proxy.is_none());
}
