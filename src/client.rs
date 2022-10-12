use lord::lord_client::LordClient;
use lord::{MinionsRequest, VersionRequest};

pub mod lord {
    tonic::include_proto!("lord");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = LordClient::connect("http://[::1]:4321").await?;

    let request = tonic::Request::new(VersionRequest {});

    let response = client.version(request).await?;

    println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(MinionsRequest {});
    let response = client.get_minions(request).await?;

    for minion in response.into_inner().minions {
        println!("minion ip: {} last_seen: {}", minion.ip, minion.last_seen);
    }

    Ok(())
}
