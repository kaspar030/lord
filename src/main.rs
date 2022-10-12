use coap::Server;
use coap_lite::{CoapResponse, RequestType as Method};
use tokio::runtime::Runtime;

use std::collections::HashMap;
use std::net::IpAddr;

use std::sync::Arc;
use std::time;
use tokio::sync::Mutex;

pub struct MinionId(pub usize);
pub struct MinionKey(pub String);

#[derive(Debug)]
enum MinionState {
    Pending,
    Adopted,
    Denied,
}

#[derive(Debug)]
struct Minion {
    // id: MinionId,
    // key: MinionKey,
    state: MinionState,
    last_seen: time::Instant,
}

impl Minion {
    pub fn new() -> Self {
        Self {
            state: MinionState::Pending,
            last_seen: time::Instant::now(),
        }
    }
}

struct State {
    minions: HashMap<IpAddr, Minion>,
}

impl State {
    pub fn new() -> Self {
        State {
            minions: HashMap::new(),
        }
    }
}

pub mod messages {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Register {
        pub token: String,
    }
}

fn main() {
    let addr = "127.0.0.1:5683";

    let state = Arc::new(Mutex::new(State::new()));

    Runtime::new().unwrap().block_on(async move {
        let mut server = Server::new(addr).unwrap();
        println!("Server up on {}", addr);

        server
            .run(|request| async {
                match request.get_method() {
                    &Method::Get => println!("request by get {}", &request.get_path()),
                    &Method::Post => match request.get_path().as_str() {
                        "reg" => {
                            println!("reg");
                            let newcount = {
                                let mut state = state.lock().await;
                                let msg = serde_cbor::from_slice::<messages::Register>(
                                    request.message.payload.as_slice(),
                                );
                                if let Ok(msg) = msg {
                                    if let Some(source) = request.source {
                                        println!(
                                            "register from {}, token = {}",
                                            source.ip(),
                                            msg.token
                                        );
                                        if let Some(minion) = state.minions.get_mut(&source.ip()) {
                                            println!("existing state: {:?}", minion);
                                            minion.last_seen = time::Instant::now();
                                        } else {
                                            state.minions.insert(source.ip(), Minion::new());
                                        }
                                    }
                                } else {
                                    if let Some(source) = request.source {
                                        println!("register from {} with bad content", source.ip());
                                        let mut response = request.response.unwrap();
                                        response.set_status(coap_lite::ResponseType::BadOption);
                                        return Some(response);
                                    }
                                }
                            };
                        }
                        _ => println!(
                            "request by post to {}: {}",
                            request.get_path(),
                            String::from_utf8(request.message.payload).unwrap()
                        ),
                    },
                    &Method::Put => println!(
                        "request by put {}",
                        String::from_utf8(request.message.payload).unwrap()
                    ),
                    _ => println!("request by other method"),
                };

                return match request.response {
                    Some(mut message) => {
                        message.message.payload = b"OK".to_vec();
                        Some(message)
                    }
                    _ => None,
                };
            })
            .await
            .unwrap();
    });
}
