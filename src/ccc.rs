use anyhow::{anyhow, Error};
use base64::CharacterSet;
use clap::{Parser, Subcommand};
use coap::CoAPClient;

mod coreconf;

use coreconf::{cbor::Entry, SIDMAP};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// get a CORECONF resource
    #[allow(non_camel_case_types)]
    get_sid {
        /// host address of CORECONF device
        host: String,
        /// CORECONF resource as SID or identifier
        sid: String,
        /// optional k parameter
        k: Option<Vec<String>>,
    },
    /// get a list of exported SIDs/identifiers
    #[allow(non_camel_case_types)]
    get_sid_list {
        /// host address of CORECONF device
        host: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::get_sid { host, sid, k }) => {
            let sid = match usize::from_str_radix(sid, 10) {
                Ok(v) => v,
                Err(_) => {
                    *(SIDMAP
                        .string2sid
                        .get(sid)
                        .ok_or(anyhow!("unknown SID identifier"))?)
                }
            };

            println!("{:#?}", get_sid(host, sid, k)?)
        }
        Some(Commands::get_sid_list { host }) => get_sid_list(host)?,
        None => {}
    }
    Ok(())
}

#[derive(Debug)]
struct SidDecodeError {}
impl std::error::Error for SidDecodeError {}

impl std::fmt::Display for SidDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid encoding")
    }
}

fn get_sid(
    host: &str,
    sid: usize,
    k: &Option<Vec<String>>,
) -> Result<Entry, Box<dyn std::error::Error>> {
    let query = if let Some(k_vec) = k {
        let mut k = "?k=".to_string();
        k.push_str(&k_vec.join(",")[..]);
        k
    } else {
        "".to_string()
    };
    let url = format!("coap://{}/c/{}{}", host, encode_sid(sid), query);
    println!("coap url: {}", &url);

    let response = CoAPClient::get(&url)?;
    let mut parsed = coreconf::cbor::Entry::parse(&response.message.payload)?;
    parsed.apply_sidmap(&coreconf::SIDMAP);
    Ok(parsed)
}

fn get_sid_list(host: &str) -> Result<(), Error> {
    let response = CoAPClient::get(&format!("coap://{}/.well-known/core?ct=core.c.dn", host));
    if let Ok(response) = response {
        let parser = coap_lite::link_format::LinkFormatParser::new(
            std::str::from_utf8(&response.message.payload).unwrap(),
        );
        for entry in parser
            .filter_map(|e| e.ok())
            .filter(|(path, mut link_attrs)| {
                path.starts_with("/c/")
                    && link_attrs.any(|attr| attr.0 == "ct" && attr.1.to_cow() == "core.c.dn")
            })
        {
            let (path, _) = entry;
            let sid = decode_sid(&path[3..]).unwrap();
            println!(
                "{sid} {}",
                SIDMAP
                    .sid2string
                    .get(&(sid as usize))
                    .map_or("unknown", |s| &s)
            );
        }
    }
    Ok(())
}

fn encode_sid(sid: usize) -> String {
    let sid = sid as u64;

    // turn 8 bytes of sid u64 to 9 bytes for base64 without trailing chars
    let sid_bytes: Vec<u8> = std::iter::once(0 as u8).chain(sid.to_be_bytes()).collect();

    // encode with base64
    let config = base64::Config::new(CharacterSet::UrlSafe, false);
    let encoded: String = base64::encode_config(sid_bytes, config)
        .chars()
        // drop leading 'A'
        .skip_while(|c| *c == 'A')
        .collect();

    encoded
}

fn decode_sid(sid_b64_compressed: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let sid_b64: String = std::iter::repeat('A')
        .take(12 - (sid_b64_compressed.len() % 12))
        .chain(sid_b64_compressed.chars())
        .collect();

    let config = base64::Config::new(CharacterSet::UrlSafe, false);
    let sid = base64::decode_config(sid_b64, config)?;

    // unwrap never fails
    Ok(u64::from_be_bytes((&sid[1..]).try_into().unwrap()))
}
