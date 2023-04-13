use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use tonic::{codegen::InterceptedService, Request, Response, Status};

use entity::{
    doc,
    helpers::{FieldMaskDef, TimestampDef},
    mongodb::bson,
    proto::{
        message,
        message_service_server::{MessageService as IMessageService, MessageServiceServer},
    },
    Document, Entity, EntityContext, FindOneOptions, FindOptions,
};

use crate::check_auth;
pub struct MessageService {
    ctx: EntityContext,
}

impl MessageService {
    async fn new(ctx: EntityContext) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl IMessageService for MessageService {
    async fn list_messages(
        &self,
        request: Request<entity::proto::ListMessagesRequest>,
    ) -> Result<Response<entity::proto::ListMessagesResponse>, Status> {
        let body = request.into_inner();
        let options = FindOptions::builder()
            .projection(Into::<Document>::into(Into::<FieldMaskDef>::into(
                body.field_mask,
            )))
            .limit(Some(body.page_size as i64))
            .build();
        let filter = doc! {"room_id": bson::oid::ObjectId::from_str(&body.room_id).unwrap(),
        "created_at.seconds": { "$gte": TimestampDef::from(body.from_date).seconds}}; // FUCK YOU MONGO
        let messages = entity::proto::Message::find(&self.ctx, Some(filter), Some(options))
            .await
            .map_err(|err| {
                Status::internal(format!("Failed to find messages, Report: {:#?}", err))
            })?;
        Ok(Response::new(entity::proto::ListMessagesResponse {
            messages,
            next_page_token: "".to_string(),
        }))
    }

    async fn list_thread_messages(
        &self,
        request: Request<entity::proto::ListThreadMessagesRequest>,
    ) -> Result<Response<entity::proto::ListMessagesResponse>, Status> {
        let body = request.into_inner();
        if let Some(req) = body.request {
            let options = FindOptions::builder()
                .projection(Into::<Document>::into(Into::<FieldMaskDef>::into(
                    req.field_mask,
                )))
                .limit(Some(req.page_size as i64))
                .build();
            let filter = doc! {"room_id": bson::oid::ObjectId::from_str(&req.room_id).unwrap(),
            "thread_id": bson::oid::ObjectId::from_str(&body.thread_id).unwrap()}; // FUCK YOU MONGO
            let messages = entity::proto::Message::find(&self.ctx, Some(filter), Some(options))
                .await
                .map_err(|err| {
                    Status::internal(format!("Failed to find messages, Report: {:#?}", err))
                })?;
            return Ok(Response::new(entity::proto::ListMessagesResponse {
                messages,
                next_page_token: "".to_string(),
            }));
        }
        Err(Status::invalid_argument("request is required"))
    }

    async fn send_message(
        &self,
        request: Request<entity::proto::SendMessageRequest>,
    ) -> Result<Response<()>, Status> {
        if let Some(message) = request.into_inner().message {
            let message_id = entity::proto::Message::create(&self.ctx, &message)
                .await
                .map_err(|err| {
                    Status::internal(format!(
                        "Failed to create message: {:?}, Report: {:#?}",
                        message, err
                    ))
                })?;
            if let Some(message::Body::KeysRotation(_)) = message.body {
                entity::proto::Room::update_one(
                    &self.ctx,
                    doc! {"_id": bson::oid::ObjectId::from_str(&message.room_id).unwrap()},
                    doc! {"$push": {"keys_rotation": message_id}},
                )
                .await
                .map_err(|err| {
                    Status::internal(format!(
                        "Failed to record rotation into room, Report: {:#?}",
                        err
                    ))
                })?;
            }
            return Ok(Response::new(()));
        }
        Err(Status::invalid_argument("message is required"))
    }

    async fn acknowledge_message(
        &self,
        request: Request<entity::proto::AcknowledgeMessageRequest>,
    ) -> Result<Response<()>, Status> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let user_id = ""; // TODO: get user_id from context
        entity::proto::Message::update_one(
            &self.ctx,
            doc! {"_id": bson::oid::ObjectId::from_str(&request.into_inner().message_id).unwrap()},
            doc! {"$set": {format!("read_by.{}", user_id): since_the_epoch.as_secs() as i64}},
        )
        .await
        .map_err(|err| {
            Status::internal(format!("Failed to acknowledge_message, Report: {:#?}", err))
        })?;
        Err(Status::invalid_argument("message is required"))
    }

    async fn update_message(
        &self,
        request: Request<entity::proto::UpdateMessageRequest>,
    ) -> Result<Response<entity::proto::Message>, Status> {
        if let Some(message) = request.into_inner().message {
            entity::proto::Message::update_one(
                &self.ctx,
                doc! {"_id": bson::oid::ObjectId::from_str(&message.id).unwrap()}, // FUCK YOU MONGO
                doc! {"$set": bson::to_document(&message).unwrap()},
            )
            .await
            .map_err(|err| {
                Status::internal(format!(
                    "Failed to update message: {:?}, Report: {:#?}",
                    message, err
                ))
            })?;
            return Ok(Response::new(message));
        }
        Err(Status::invalid_argument("message is required"))
    }

    async fn delete_message(
        &self,
        request: Request<entity::proto::DeleteMessageRequest>,
    ) -> Result<Response<()>, Status> {
        let message = entity::proto::Message::delete_one(
            &self.ctx,
            doc! {"_id": bson::oid::ObjectId::from_str(&request.into_inner().id).unwrap()},
        )
        .await
        .map_err(|err| Status::internal(format!("Failed to delete_message, Report: {:#?}", err)))?;
        return Ok(Response::new(message));
    }
}

pub async fn svc(
    entity: EntityContext,
) -> InterceptedService<
    MessageServiceServer<MessageService>,
    fn(Request<()>) -> Result<Request<()>, Status>,
> {
    let server = MessageService::new(entity).await;

    MessageServiceServer::with_interceptor(server, check_auth)
}
