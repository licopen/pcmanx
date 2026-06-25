#![allow(dead_code)]

use encoding_rs::BIG5;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Encoding {
    #[default]
    Big5,
    Uao241,
    Uao250,
    Utf8,
}

pub fn decode(bytes: &[u8], enc: Encoding) -> String {
    match enc {
        Encoding::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
        Encoding::Big5 => {
            let (decoded, _, _) = BIG5.decode(bytes);
            decoded.into_owned()
        }
        // TODO: Replace with dedicated UAO tables. UAO is a Big5 superset.
        Encoding::Uao241 | Encoding::Uao250 => {
            let (decoded, _, _) = BIG5.decode(bytes);
            decoded.into_owned()
        }
    }
}

pub fn encode(s: &str, enc: Encoding) -> Vec<u8> {
    match enc {
        Encoding::Utf8 => s.as_bytes().to_vec(),
        Encoding::Big5 => {
            let (encoded, _, _) = BIG5.encode(s);
            encoded.into_owned()
        }
        // TODO: Replace with dedicated UAO tables. UAO is a Big5 superset.
        Encoding::Uao241 | Encoding::Uao250 => {
            let (encoded, _, _) = BIG5.encode(s);
            encoded.into_owned()
        }
    }
}
