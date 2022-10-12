use coap::Server;
use coap_lite::{CoapRequest, CoapResponse, RequestType as Method};
use grpc::MyLord;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

use std::sync::Arc;
use std::time;
use tokio::sync::Mutex;

mod minion;

use minion::Minion;

mod grpc;

#[derive(Debug)]
pub struct State {
    pub minions: HashMap<IpAddr, Minion>,
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

async fn wait() {
    tokio::signal::ctrl_c().await.ok();
}

async fn coap_request_handler(
    state: Arc<Mutex<State>>,
    request: CoapRequest<SocketAddr>,
) -> std::option::Option<CoapResponse> {
    match request.get_method() {
        &Method::Get => println!("request by get {}", &request.get_path()),
        &Method::Post => match request.get_path().as_str() {
            "reg" => {
                let mut state = state.lock().await;
                let msg = serde_cbor::from_slice::<messages::Register>(
                    request.message.payload.as_slice(),
                );
                if let Ok(msg) = msg {
                    if let Some(source) = request.source {
                        println!("register from {}, token = {}", source.ip(), msg.token);
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(State::new()));
    let lord_state = state.clone();

    let coap_addr = "[::1]:5683";
    let grpc_addr = "[::1]:4321";

    // spawn CoAP server
    let coap_handle = tokio::spawn(async move {
        let state = state.clone();
        let mut server = Server::new(coap_addr).unwrap();
        println!("Coap Server up on {}", coap_addr);

        server
            .run(move |request| coap_request_handler(state.clone(), request))
            .await
    });

    // spawn gRPC server
    let lord = MyLord { state: lord_state };
    let grpc_handle = tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(grpc::lord::lord_server::LordServer::new(lord))
            //.serve(grpc_addr.parse()?),
            .serve_with_shutdown(grpc_addr.parse()?, wait()),
    );

    // wait for sigint.
    // the grpc server will exit on sigint, too, due to `serve_with_shutdown(..., wait())`.
    // the coap server won't, so we'll abort() it later.
    // TODO: this might not handle any errors that cause abortion of the tasks :(
    tokio::signal::ctrl_c().await?;

    grpc_handle.await??;
    coap_handle.abort();

    Ok(())
}
