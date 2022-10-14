use super::SidMap;
use anyhow::anyhow;
use smol_str::SmolStr;
use std::fmt::Display;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Sid {
    Absolute(usize),
    String(SmolStr),
}

impl Sid {
    pub fn plus(&self, rel: usize) -> Self {
        if let Sid::Absolute(abs) = self {
            return Sid::Absolute(abs + rel);
        }
        panic!("Sid::from_base() called with non-absolute self");
    }

    pub fn apply_sidmap(&mut self, sidmap: &SidMap) {
        match self {
            Sid::Absolute(value) => {
                if let Some(identifier) = sidmap.sid2string.get(value) {
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
            Sid::Absolute(val) => f.write_fmt(format_args!("{val}")),
            Sid::String(smolstr) => f.write_fmt(format_args!("{}", smolstr)),
        }
    }
}
