use coap::CoAPClient;
use coap_lite::{CoapRequest, RequestType as Method};
use std::net::ToSocketAddrs;

mod coreconf;

#[derive(Debug)]
struct SidDecodeError {}
impl std::error::Error for SidDecodeError {}

impl std::fmt::Display for SidDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid encoding")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "coap://[fe80::1462:8cff:feac:d5f4%tap0]/.well-known/core?ct=core.c.dn";
    println!("Client request: {}", url);

    let mut sidmap = coreconf::SidMap::new();
    sidmap
        .add_file("../sid/ietf-interfaces@2018-02-20.sid")?
        .add_file("../sid/ietf-ip@2018-02-22.sid")?
        .add_file("../sid/ietf-system@2014-08-06.sid")?;

    let response = CoAPClient::get(url);
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
                "{path} {sid} {}",
                sidmap.map.get(&(sid as usize)).map_or("unknown", |s| &s)
            );
        }
    }
    let response = CoAPClient::get("coap://[fe80::1462:8cff:feac:d5f4%tap0]/c/Xh");
    let mut parsed = coreconf::cbor::Entry::parse(&response?.message.payload)?;
    parsed.apply_sidmap(&sidmap);

    println!("{:#?}", parsed);
    Ok(())
}

fn decode_sid(sid_b64_compressed: &str) -> Result<u64, Box<dyn std::error::Error>> {
    use base64::CharacterSet;
    let sid_b64: String = std::iter::repeat('A')
        .take(12 - (sid_b64_compressed.len() % 12))
        .chain(sid_b64_compressed.chars())
        .collect();

    let config = base64::Config::new(CharacterSet::UrlSafe, false);
    let sid = base64::decode_config(sid_b64, config)?;

    // unwrap never fails
    Ok(u64::from_be_bytes((&sid[1..]).try_into().unwrap()))
}
