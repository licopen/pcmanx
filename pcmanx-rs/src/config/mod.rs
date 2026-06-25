#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::charset::Encoding;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CrLfMode {
    Cr,
    Lf,
    CrLf,
}

impl Default for CrLfMode {
    fn default() -> Self {
        Self::Cr
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProxyType {
    None,
    Socks4,
    Socks5,
    Http,
}

impl Default for ProxyType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub addr: String,
    pub port: String,
    pub user: String,
    pub pass: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AutoLogin {
    pub pre_login: String,
    pub pre_login_prompt: String,
    pub login: String,
    pub login_prompt: String,
    pub passwd: String,
    pub passwd_prompt: String,
    pub post_login: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SiteConfig {
    pub name: String,
    pub url: String,
    pub encoding: Encoding,
    pub auto_reconnect: u32,
    pub anti_idle: u32,
    pub anti_idle_str: String,
    pub detect_dbchar: bool,
    pub rows_per_page: u32,
    pub cols_per_page: u32,
    pub auto_wrap_on_paste: u32,
    pub term_type: String,
    pub crlf: CrLfMode,
    pub auto_login: AutoLogin,
    pub uao: u32,
    pub proxy: Option<ProxyConfig>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            url: String::new(),
            encoding: Encoding::Big5,
            auto_reconnect: 20,
            anti_idle: 180,
            anti_idle_str: "^[OB".to_string(),
            detect_dbchar: true,
            rows_per_page: 24,
            cols_per_page: 80,
            auto_wrap_on_paste: 78,
            term_type: "vt100".to_string(),
            crlf: CrLfMode::Cr,
            auto_login: AutoLogin::default(),
            uao: 2,
            proxy: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowConfig {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub state: i32,
    pub maximized: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            x: 40,
            y: 40,
            w: 640,
            h: 480,
            state: 0,
            maximized: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    pub query_on_exit: bool,
    pub query_on_close_conn: bool,
    pub cancel_sel_after_copy: bool,
    pub copy_trim_tail: bool,
    pub opacity: i32,
    pub show_toolbar: bool,
    pub show_statusbar: bool,
    pub show_tabbar: bool,
    pub show_menubar: bool,
    pub popup_notifier: bool,
    pub popup_timeout: i32,
    pub mid_click_as_close: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            query_on_exit: true,
            query_on_close_conn: true,
            cancel_sel_after_copy: true,
            copy_trim_tail: true,
            opacity: 100,
            show_toolbar: true,
            show_statusbar: true,
            show_tabbar: true,
            show_menubar: true,
            popup_notifier: true,
            popup_timeout: 6,
            mid_click_as_close: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalConfig {
    pub rows_per_page: u32,
    pub cols_per_page: u32,
    pub beep_on_bell: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            rows_per_page: 24,
            cols_per_page: 80,
            beep_on_bell: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayConfig {
    pub anti_alias_font: bool,
    pub compact_layout: bool,
    pub char_padding_x: i32,
    pub char_padding_y: i32,
    pub font_size: i32,
    pub font_family: String,
    pub font_size_en: i32,
    pub font_family_en: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            anti_alias_font: true,
            compact_layout: false,
            char_padding_x: 0,
            char_padding_y: 0,
            font_size: 14,
            font_family: "WenQuanYi Micro Hei Mono".to_string(),
            font_size_en: 14,
            font_family_en: "WenQuanYi Micro Hei Mono".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub key_site_list: String,
    pub key_new_conn0: String,
    pub key_new_conn1: String,
    pub key_reconn0: String,
    pub key_reconn1: String,
    pub key_close0: String,
    pub key_close1: String,
    pub key_next_page: String,
    pub key_prev_page: String,
    pub key_first_page: String,
    pub key_last_page: String,
    pub key_copy0: String,
    pub key_copy1: String,
    pub key_paste0: String,
    pub key_paste1: String,
    pub key_paste_clipboard: String,
    pub key_emotions: String,
    pub key_fullscreen: String,
    pub key_show_main_window: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            key_site_list: "<Alt>S".to_string(),
            key_new_conn0: "<Alt>Q".to_string(),
            key_new_conn1: "<Ctrl><Shift>T".to_string(),
            key_reconn0: "<Alt>R".to_string(),
            key_reconn1: "<Ctrl>Insert".to_string(),
            key_close0: "<Alt>W".to_string(),
            key_close1: "<Ctrl>Delete".to_string(),
            key_next_page: "<Alt>X".to_string(),
            key_prev_page: "<Alt>Z".to_string(),
            key_first_page: "<Ctrl>Home".to_string(),
            key_last_page: "<Ctrl>End".to_string(),
            key_copy0: "<Alt>O".to_string(),
            key_copy1: "<Ctrl><Shift>C".to_string(),
            key_paste0: "<Alt>P".to_string(),
            key_paste1: "<Ctrl><Shift>V".to_string(),
            key_paste_clipboard: "<Shift>Insert".to_string(),
            key_emotions: "<Ctrl>Return".to_string(),
            key_fullscreen: "<ALT>Return".to_string(),
            key_show_main_window: "<Alt>M".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub sites: Vec<SiteConfig>,
    pub default_site: SiteConfig,
    pub window: WindowConfig,
    pub general: GeneralConfig,
    pub terminal: TerminalConfig,
    pub display: DisplayConfig,
    pub hotkeys: HotkeyConfig,
    pub web_browser: String,
    pub mail_client: String,
    pub socket_timeout: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sites: Vec::new(),
            default_site: SiteConfig::default(),
            window: WindowConfig::default(),
            general: GeneralConfig::default(),
            terminal: TerminalConfig::default(),
            display: DisplayConfig::default(),
            hotkeys: HotkeyConfig::default(),
            web_browser: "xdg-open".to_string(),
            mail_client: "xdg-email".to_string(),
            socket_timeout: 30,
        }
    }
}

impl AppConfig {
    fn config_path() -> Result<PathBuf> {
        let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config directory available"))?;
        Ok(base.join("pcmanx").join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
