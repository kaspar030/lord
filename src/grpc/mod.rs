use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::sync::Mutex;

use tonic::{Request, Response, Status};

pub mod lord {
    tonic::include_proto!("lord");
}

pub use lord::lord_server::Lord;
pub use lord::{Minion, MinionsReply, MinionsRequest};
pub use lord::{VersionReply, VersionRequest};

#[derive(Debug)]
pub struct MyLord {
    pub state: Arc<Mutex<crate::State>>,
}

#[tonic::async_trait]
impl Lord for MyLord {
    async fn version(
        &self,
        request: Request<VersionRequest>, // Accept request of type VersionRequest
    ) -> Result<Response<VersionReply>, Status> {
        // Return an instance of type VersionReply
        println!("Got a request: {:?}", request);

        let reply = VersionReply {
            version: format!("0.1.0").into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }

    async fn get_minions(
        &self,
        request: Request<MinionsRequest>,
    ) -> Result<Response<MinionsReply>, Status> {
        println!("Got a request: {:?}", request);

        let minions: Vec<Minion> = {
            let state = self.state.lock().await;
            state
                .minions
                .iter()
                .map(|(ip, minion)| Minion {
                    ip: format!("{ip}").into(),
                    last_seen: std::time::Instant::now()
                        .duration_since(minion.last_seen)
                        .as_secs(),
                })
                .collect()
        };

        let reply = MinionsReply { minions };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}
