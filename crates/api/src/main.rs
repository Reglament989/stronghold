use tonic::{metadata::MetadataValue, transport::Server, Request, Status};

pub mod services;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(entity::proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let addr = "127.0.0.1:50051".parse()?;
    let mut server = Server::builder();

    server.add_service(reflector);

    let router = services::services(&mut server).await;

    router.serve(addr).await?;
    Ok(())
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
