use tonic::{Request, Response, Status};

pub mod lord {
    tonic::include_proto!("lord");
}

pub use lord::lord_server::Lord;
pub use lord::{VersionReply, VersionRequest};

#[derive(Debug, Default)]
pub struct MyLord {}

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
}
