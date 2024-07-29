use serde::Serialize;
use serde_with::skip_serializing_none;
use time::OffsetDateTime;
use uuid::Uuid;

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonPublic {
    pub creator_uuid: Uuid,
    pub guid: Uuid,

    pub name: String,
    pub tag_line: String,
    pub description: String,
    pub icon: Option<String>,
    pub gallery: Option<Vec<String>>,
    pub version: String,

    pub is_visible: bool,
    pub is_accepted: bool,

    pub install_count: i32,

    pub delete_reason: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperPublic {
    pub guid: Uuid,

    pub name: String,
    pub description: String,
    pub icon: Option<String>,

    pub addon_count: i32,
    pub delete_reason: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}
