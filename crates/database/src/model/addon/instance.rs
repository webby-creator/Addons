/// Instances of addons used on websites
use common::{AddonInstanceId, WebsiteId};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct NewAddonInstance {
    pub website_id: WebsiteId,
    pub website_uuid: Uuid,
}

pub struct AddonInstance {
    pub id: AddonInstanceId,
    pub public_id: Uuid,

    pub website_id: WebsiteId,
    pub website_uuid: Uuid,

    // TODO: Should I store some sort of settings here? Not related to addon, but the instance itself.
    pub delete_reason: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}
