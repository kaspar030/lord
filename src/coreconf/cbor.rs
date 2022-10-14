use std::collections::HashMap;
use std::fmt::Display;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Sid, SidMap};

pub enum Entry {
    Map(IndexMap<Sid, Entry>),
    Array(Vec<Entry>),
    String(String),
    Bool(bool),
    Integer(i128),
}

#[derive(Default, Debug, Error)]
pub enum CoreconfCborParseError {
    #[default]
    Unknown,
    UnexpectedStructure,
    SidNotInteger,
}

impl Display for CoreconfCborParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unexpected cbor structure")
    }
}

use serde_cbor::value::Value;
impl TryFrom<(&Sid, serde_cbor::Value)> for Entry {
    type Error = CoreconfCborParseError;
    fn try_from(base_sid_value: (&Sid, serde_cbor::Value)) -> Result<Self, Self::Error> {
        let (base_sid, cbor_value) = base_sid_value;
        match cbor_value {
            Value::Map(cbor_map) => Ok(Entry::Map({
                let mut map = IndexMap::new();
                for (k, v) in cbor_map {
                    let sid = match k {
                        Value::Integer(cbor_sid) => base_sid.plus(cbor_sid as usize),
                        _ => return Err(CoreconfCborParseError::SidNotInteger),
                    };
                    let v = match v {
                        Value::Text(s) => Entry::String(s),
                        Value::Integer(i) => Entry::Integer(i),
                        Value::Bool(b) => Entry::Bool(b),
                        Value::Map(_) => Entry::try_from((&sid, v))?,
                        Value::Array(_) => Entry::try_from((&sid, v))?,
                        _ => {
                            println!("{:#?}", v);
                            Entry::String("unimplemented".to_string())
                        }
                    };
                    map.insert(sid.clone(), v);
                }
                map
            })),
            Value::Array(cbor_array) => Ok(Entry::Array({
                let mut array = Vec::with_capacity(cbor_array.len());
                for entry in cbor_array {
                    array.push(Entry::try_from((base_sid, entry))?)
                }
                array
            })),
            _ => Err(CoreconfCborParseError::UnexpectedStructure),
        }
    }
}

impl Entry {
    pub fn parse(cbor: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let cbor: serde_cbor::Value = serde_cbor::from_slice(cbor)?;

        Ok(Entry::try_from((&Sid::Absolute(0), cbor))?)
    }

    pub fn apply_sidmap(&mut self, sidmap: &SidMap) {
        match self {
            Entry::Map(map) => {
                *self = Entry::Map(
                    map.drain(..)
                        .map(|(mut sid, mut v)| {
                            sid.apply_sidmap(sidmap);
                            v.apply_sidmap(sidmap);
                            (sid, v)
                        })
                        .collect(),
                );
            }
            Entry::Array(array) => array
                .iter_mut()
                .for_each(|entry| entry.apply_sidmap(sidmap)),
            _ => (),
        }
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::Map(v) => {
                f.debug_map()
                    .entries(v.iter().map(|(k, v)| ("a", "b")))
                    .finish()?;
            }
            Entry::String(s) => f.write_str(&s)?,
            Entry::Bool(v) => f.write_fmt(format_args!("{}", v))?,
            Entry::Integer(v) => f.write_fmt(format_args!("{}", v))?,
            Entry::Array(v) => f.write_fmt(format_args!("{:?}", v))?,
        }
        Ok(())
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::Map(v) => {
                f.debug_map()
                    .entries(v.iter().map(|(k, v)| (format!("{}", k), v)))
                    .finish()?;
            }
            Entry::String(s) => f.write_str(&s)?,
            Entry::Bool(v) => f.write_fmt(format_args!("{}", v))?,
            Entry::Integer(v) => f.write_fmt(format_args!("{}", v))?,
            Entry::Array(v) => f.write_fmt(format_args!("{:#?}", v))?,
        }
        Ok(())
    }
}
