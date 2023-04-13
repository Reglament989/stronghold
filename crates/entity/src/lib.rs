#[cfg(feature = "server")]
pub mod loader;
pub mod proto;

#[cfg(feature = "server")]
pub mod config;
pub mod helpers;

use eyre::{Result, WrapErr};

pub use mongodb::{
    self,
    bson::doc,
    bson::Document,
    options::{FindOneOptions, FindOptions},
};

use serde::{de::DeserializeOwned, Serialize};

use futures::stream::TryStreamExt;

#[cfg(feature = "server")]
pub type EntityContext = std::sync::Arc<futures::lock::Mutex<mongodb::Database>>;

#[cfg(feature = "server")]
#[tonic::async_trait]
pub trait Entity<T: Serialize + DeserializeOwned + Unpin + Send + Sync + Clone> {
    const COLLECTION: &'static str;

    async fn collection(ctx: &EntityContext) -> mongodb::Collection<T> {
        ctx.lock().await.collection::<T>(Self::COLLECTION)
    }

    async fn find<
        F: Into<Option<Document>> + std::marker::Send,
        O: Into<Option<FindOptions>> + std::marker::Send,
    >(
        ctx: &EntityContext,
        filter: F,
        options: O,
    ) -> Result<Vec<T>> {
        let cursor = Self::collection(ctx).await.find(filter, options).await?;
        Ok(cursor
            .try_collect::<Vec<_>>()
            .await
            .with_context(|| format!("Failed to find {}", Self::COLLECTION))?)
    }

    async fn find_one<
        F: Into<Option<Document>> + std::marker::Send,
        O: Into<Option<FindOneOptions>> + std::marker::Send,
    >(
        ctx: &EntityContext,
        filter: F,
        options: O,
    ) -> Result<Option<T>> {
        Self::collection(ctx)
            .await
            .find_one(filter, options)
            .await
            .with_context(|| format!("Failed to find {}", Self::COLLECTION))
    }

    async fn create(ctx: &EntityContext, payload: &T) -> Result<String> {
        Ok(Self::collection(ctx)
            .await
            .insert_one(payload, None)
            .await
            .with_context(|| format!("Failed to create {}", Self::COLLECTION))?
            .inserted_id
            .as_object_id()
            .unwrap()
            .to_hex())
    }

    async fn update_one<F: Into<Document> + std::marker::Send>(
        ctx: &EntityContext,
        filter: F,
        payload: F,
    ) -> Result<()> {
        Self::collection(ctx)
            .await
            .update_one(
                filter.into(),
                payload.into(), // doc! {"$set": mongodb::bson::to_document(&payload)?},
                None,
            )
            .await?;
        Ok(())
    }

    async fn delete_one<F: Into<Document> + std::marker::Send>(
        ctx: &EntityContext,
        filter: F,
    ) -> Result<()> {
        Self::collection(ctx)
            .await
            .delete_one(filter.into(), None)
            .await?;
        Ok(())
    }
}
