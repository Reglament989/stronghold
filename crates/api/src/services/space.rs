use std::str::FromStr;

use tonic::{codegen::InterceptedService, Request, Response, Status};

use entity::{
    doc,
    helpers::FieldMaskDef,
    mongodb::bson,
    proto::space_service_server::{SpaceService as ISpaceService, SpaceServiceServer},
    Document, Entity, EntityContext, FindOneOptions,
};

use crate::check_auth;
pub struct SpaceService {
    ctx: EntityContext,
}

impl SpaceService {
    async fn new(ctx: EntityContext) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl ISpaceService for SpaceService {
    async fn list_spaces(
        &self,
        request: Request<entity::proto::ListSpacesRequest>,
    ) -> Result<Response<entity::proto::ListSpacesResponse>, Status> {
        let spaces = entity::proto::Space::find(&self.ctx, None, None)
            .await
            .map_err(|err| {
                Status::internal(format!("Failed to find spaces, Report: {:#?}", err))
            })?;
        Ok(Response::new(entity::proto::ListSpacesResponse {
            spaces,
            next_page_token: "".to_string(),
        }))
    }

    async fn get_space(
        &self,
        request: Request<entity::proto::GetSpaceRequest>,
    ) -> Result<Response<entity::proto::Space>, Status> {
        // let filter = doc!();
        let options = FindOneOptions::builder()
            .projection(Into::<Document>::into(Into::<FieldMaskDef>::into(
                request.into_inner().field_mask,
            )))
            .build();
        // .unwrap().paths;
        let space = entity::proto::Space::find_one(&self.ctx, None, options)
            .await
            .map_err(|err| {
                Status::internal(format!("Failed to find_one space, Report: {:#?}", err))
            })?
            .ok_or(Status::not_found("space not found"))?;
        dbg!(&space);
        return Ok(Response::new(space));
    }

    async fn create_space(
        &self,
        request: Request<entity::proto::CreateSpaceRequest>,
    ) -> Result<Response<entity::proto::Space>, Status> {
        if let Some(space) = request.into_inner().space {
            entity::proto::Space::create(&self.ctx, &space)
                .await
                .map_err(|err| {
                    Status::internal(format!(
                        "Failed to create space: {:?}, Report: {:#?}",
                        space, err
                    ))
                })?;
            return Ok(Response::new(space));
        }
        Err(Status::invalid_argument("space is required"))
    }

    async fn update_space(
        &self,
        request: Request<entity::proto::UpdateSpaceRequest>,
    ) -> Result<Response<entity::proto::Space>, Status> {
        if let Some(space) = request.into_inner().space {
            entity::proto::Space::update_one(
                &self.ctx,
                doc! {"_id": bson::oid::ObjectId::from_str(&space.id).unwrap()}, // FUCK YOU MONGO
                doc! {"$set": bson::to_document(&space).unwrap()},
            )
            .await
            .map_err(|err| {
                Status::internal(format!(
                    "Failed to update space: {:?}, Report: {:#?}",
                    space, err
                ))
            })?;
            return Ok(Response::new(space));
        }
        Err(Status::invalid_argument("space is required"))
    }

    async fn delete_space(
        &self,
        request: Request<entity::proto::DeleteSpaceRequest>,
    ) -> Result<Response<()>, Status> {
        let space = entity::proto::Space::delete_one(
            &self.ctx,
            doc! {"_id": bson::oid::ObjectId::from_str(&request.into_inner().id).unwrap()},
        )
        .await
        .map_err(|err| Status::internal(format!("Failed to find_one space, Report: {:#?}", err)))?;
        return Ok(Response::new(space));
    }
}

pub async fn svc(
    entity: EntityContext,
) -> InterceptedService<
    SpaceServiceServer<SpaceService>,
    fn(Request<()>) -> Result<Request<()>, Status>,
> {
    let server = SpaceService::new(entity).await;

    SpaceServiceServer::with_interceptor(server, check_auth)
}
