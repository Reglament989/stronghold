use std::str::FromStr;

use tonic::{metadata::MetadataValue, transport::Server, Request, Response, Status};

use entity::{
    doc,
    helpers::FieldMaskDef,
    mongodb::bson,
    proto::room_service_server::{RoomService as IRoomService, RoomServiceServer},
    Document, Entity, EntityContext, FindOneOptions,
};

pub struct RoomService {
    ctx: EntityContext,
}

impl RoomService {
    async fn new() -> Self {
        let ctx = entity::loader::load().await;
        Self { ctx }
    }
}

#[tonic::async_trait]
impl IRoomService for RoomService {
    async fn list_rooms(
        &self,
        request: Request<entity::proto::ListRoomsRequest>,
    ) -> Result<Response<entity::proto::ListRoomsResponse>, Status> {
        let rooms = entity::proto::Room::find(&self.ctx, None, None)
            .await
            .map_err(|err| Status::internal(format!("Failed to find rooms, Report: {:#?}", err)))?;
        Ok(Response::new(entity::proto::ListRoomsResponse {
            rooms,
            next_page_token: "".to_string(),
        }))
    }

    async fn get_room(
        &self,
        request: Request<entity::proto::GetRoomRequest>,
    ) -> Result<Response<entity::proto::Room>, Status> {
        // let filter = doc!();
        let options = FindOneOptions::builder()
            .projection(Into::<Document>::into(Into::<FieldMaskDef>::into(
                request.into_inner().field_mask,
            )))
            .build();
        // .unwrap().paths;
        let room = entity::proto::Room::find_one(&self.ctx, None, options)
            .await
            .map_err(|err| {
                Status::internal(format!("Failed to find_one room, Report: {:#?}", err))
            })?
            .ok_or(Status::not_found("room not found"))?;
        dbg!(&room);
        return Ok(Response::new(room));
    }

    async fn create_room(
        &self,
        request: Request<entity::proto::CreateRoomRequest>,
    ) -> Result<Response<entity::proto::Room>, Status> {
        if let Some(room) = request.into_inner().room {
            entity::proto::Room::create(&self.ctx, &room)
                .await
                .map_err(|err| {
                    Status::internal(format!(
                        "Failed to create room: {:?}, Report: {:#?}",
                        room, err
                    ))
                })?;
            return Ok(Response::new(room));
        }
        Err(Status::invalid_argument("room is required"))
    }

    async fn update_room(
        &self,
        request: Request<entity::proto::UpdateRoomRequest>,
    ) -> Result<Response<entity::proto::Room>, Status> {
        if let Some(room) = request.into_inner().room {
            entity::proto::Room::update_one(
                &self.ctx,
                doc! {"_id": bson::oid::ObjectId::from_str(&room.id).unwrap()}, // FUCK YOU MONGO
                &room,
            )
            .await
            .map_err(|err| {
                Status::internal(format!(
                    "Failed to update room: {:?}, Report: {:#?}",
                    room, err
                ))
            })?;
            return Ok(Response::new(room));
        }
        Err(Status::invalid_argument("room is required"))
    }

    async fn delete_room(
        &self,
        request: Request<entity::proto::DeleteRoomRequest>,
    ) -> Result<Response<()>, Status> {
        let room = entity::proto::Room::delete_one(
            &self.ctx,
            doc! {"_id": bson::oid::ObjectId::from_str(&request.into_inner().id).unwrap()},
        )
        .await
        .map_err(|err| Status::internal(format!("Failed to find_one room, Report: {:#?}", err)))?;
        return Ok(Response::new(room));
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(entity::proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let addr = "127.0.0.1:50051".parse()?;
    let server = RoomService::new().await;

    let svc = RoomServiceServer::with_interceptor(server, check_auth);

    Server::builder()
        .add_service(reflector)
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer some-secret-token".parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
