use crate::Entity;

#[cfg(feature = "client")]
tonic::include_proto!("room");

#[cfg(feature = "federation")]
tonic::include_proto!("federation");

#[cfg(feature = "server")]
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("room_descriptor");

#[cfg(feature = "client")]
impl Entity<Room> for Room {
    const COLLECTION: &'static str = "rooms";
}

#[cfg(feature = "client")]
impl Entity<Message> for Message {
    const COLLECTION: &'static str = "messages";
}
