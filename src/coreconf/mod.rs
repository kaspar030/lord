pub mod cbor;
mod sidfile;
mod sidmap;
use std::fmt::Display;

use anyhow::anyhow;
pub use sidmap::SidMap;
use smol_str::SmolStr;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Sid {
    Relative(usize),
    Absolute(usize),
    String(SmolStr),
}

impl Sid {
    pub fn plus(&self, rel: usize) -> Self {
        if let Sid::Absolute(abs) = self {
            return Sid::Absolute(abs + rel);
        }
        panic!("Sid::from_base() called with non-Absoluteself");
    }

    pub fn apply_sidmap(&mut self, sidmap: &SidMap) {
        match self {
            Sid::Absolute(value) => {
                if let Some(identifier) = sidmap.map.get(value) {
                    *self = Sid::String(smol_str::SmolStr::new(identifier));
                }
            }
            _ => (),
        };
    }
}
impl Display for Sid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sid::Absolute(val) => f.write_fmt(format_args!("val")),
            Sid::Relative(val) => f.write_fmt(format_args!("+val")),
            Sid::String(smolstr) => f.write_fmt(format_args!("{}", smolstr)),
        }
    }
}
