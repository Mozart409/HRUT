use proto::greet_server::{Greet, GreetServer};
use tonic::codec::CompressionEncoding;
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("greet.v1");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("greet_descriptor");
}

#[derive(Debug, Default)]
struct GreetService {}

#[tonic::async_trait]
impl Greet for GreetService {
    async fn greet(
        &self,
        request: tonic::Request<proto::GreetRequest>,
    ) -> Result<tonic::Response<proto::GreetResponse>, tonic::Status> {
        let input = request.get_ref();

        let response = proto::GreetResponse {
            greeting: "Hello ".to_owned() + &input.name.to_owned(),
        };

        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

    let addr = "[::1]:50051".parse()?;

    let greet = GreetService::default();

    health_reporter
        .set_serving::<GreetServer<GreetService>>()
        .await;
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    tracing::info!(message = "Starting server", %addr);

    Server::builder()
        .accept_http1(true)
        .layer(tower_http::cors::CorsLayer::permissive())
        .add_service(service)
        .add_service(health_service)
        .add_service(tonic_web::enable(
            GreetServer::new(greet)
                .send_compressed(CompressionEncoding::Gzip)
                .accept_compressed(CompressionEncoding::Gzip),
        ))
        .serve(addr)
        .await?;

    Ok(())
}
