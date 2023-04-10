use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let _ = tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("room_descriptor.bin"))
        .build_client(false)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .message_attribute(".", "#[serde(default)]")
        // .field_attribute(".", "#[serde(skip_serializing_if = \"crate::helpers::is_default\")]")
        .field_attribute("field_mask", "#[serde(with = \"crate::helpers::field_mask_ref\")]")
        .field_attribute("last_seen", "#[serde(with = \"crate::helpers::timestamp_ref\")]")
        .field_attribute("created_at", "#[serde(with = \"crate::helpers::timestamp_ref\")]")
        .field_attribute("updated_at", "#[serde(with = \"crate::helpers::timestamp_ref\")]")
        .field_attribute("room.Room.id", 
        "#[serde(rename(deserialize = \"_id\"), deserialize_with = \"mongodb::bson::serde_helpers::deserialize_hex_string_from_object_id\")]")
        .compile(&["./protos/room.proto", "./protos/federation.proto"], &["./protos", "./protos/lib"]);
    Ok(())
}
