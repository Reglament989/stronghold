pub mod message;
pub mod room;
pub mod space;

pub async fn services(server: &mut tonic::transport::Server) -> tonic::transport::server::Router {
    let ctx = entity::loader::load().await;
    server
        .add_service(room::svc(ctx.clone()).await)
        .add_service(message::svc(ctx.clone()).await)
        .add_service(space::svc(ctx).await)
}
