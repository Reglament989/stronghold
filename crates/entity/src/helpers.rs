use mongodb::bson::Document;
use prost_types::FieldMask;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "prost_types::FieldMask")]
pub struct FieldMaskDef {
    pub paths: Vec<String>,
}

pub mod field_mask_ref {
    use super::FieldMaskDef;
    use prost_types::FieldMask;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<FieldMask>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "FieldMaskDef")] &'a FieldMask);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<FieldMask>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "FieldMaskDef")] FieldMask);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

impl From<Option<FieldMask>> for FieldMaskDef {
    fn from(s: Option<FieldMask>) -> FieldMaskDef {
        FieldMaskDef {
            paths: s.unwrap_or_default().paths,
        }
    }
}

impl From<FieldMaskDef> for Document {
    fn from(value: FieldMaskDef) -> Self {
        let mut doc = mongodb::bson::Document::new();
        for path in value.paths {
            doc.insert(path, 1);
        }
        doc
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "prost_types::Timestamp")]
pub struct TimestampDef {
    pub seconds: i64,
    pub nanos: i32,
}

pub mod timestamp_ref {
    use super::TimestampDef;
    use prost_types::Timestamp;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Timestamp>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "TimestampDef")] &'a Timestamp);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "TimestampDef")] Timestamp);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

// impl From<PacketType> for Packet {
//     fn from(value: PacketType) -> Self {
//         todo!()
//     }
// }
