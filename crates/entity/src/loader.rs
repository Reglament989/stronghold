use crate::{config::SETTINGS, EntityContext};

pub async fn load() -> EntityContext {
    let client = mongodb::Client::with_uri_str(&SETTINGS.database.uri)
        .await
        .unwrap();
    let db = client.database("test");
    std::sync::Arc::new(futures::lock::Mutex::new(db))
}
