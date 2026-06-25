#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Eq, Serialize, Deserialize)]
pub enum Charset {
    #[default]
    Ascii,
    Mbcs1,
    Mbcs2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharAttr {
    pub fg: u8,
    pub bg: u8,
    pub bright: bool,
    pub blink: bool,
    pub underline: bool,
    pub inverse: bool,
    pub invisible: bool,
    pub hyperlink: bool,
    pub charset: Charset,
    pub need_update: bool,
}

impl Default for CharAttr {
    fn default() -> Self {
        Self {
            fg: 7,
            bg: 0,
            bright: false,
            blink: false,
            underline: false,
            inverse: false,
            invisible: false,
            hyperlink: false,
            charset: Charset::Ascii,
            need_update: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermCell {
    pub ch: char,
    pub attr: CharAttr,
}

impl Default for TermCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            attr: CharAttr::default(),
        }
    }
}

pub const COLOR_TABLE: [[u8; 3]; 16] = [
    [0, 0, 0],
    [128, 0, 0],
    [0, 128, 0],
    [128, 128, 0],
    [0, 0, 128],
    [128, 0, 128],
    [0, 128, 128],
    [192, 192, 192],
    [128, 128, 128],
    [255, 0, 0],
    [0, 255, 0],
    [255, 255, 0],
    [0, 0, 255],
    [255, 0, 255],
    [0, 255, 255],
    [255, 255, 255],
];
