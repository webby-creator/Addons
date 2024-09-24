#[macro_use]
extern crate log;

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub mod api;
pub mod generate;
mod id;
pub mod upload;
mod widget;

pub use id::*;
pub use widget::*;

#[derive(Serialize, Deserialize)]
pub struct DashboardPageInfo {
    #[serde(rename = "type")]
    pub type_of: String,
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct MemberModel {
    pub id: MemberId,

    pub uuid: Uuid,

    pub role: i64,

    pub tag: String,
    pub display_name: String,

    pub email: String,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct WebsiteModel {
    pub id: WebsiteId,

    pub owner_id: MemberId,

    pub public_id: String,

    pub name: String,
    /// If URL starts with '/' it is relative to the domain
    pub url: Option<String>,
    pub theme_id: i32,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub struct AddonPermission {
    pub scope: String,
    pub category: String,
    pub operation: Option<String>,
    pub info: Option<String>,
}

impl ToString for AddonPermission {
    fn to_string(&self) -> String {
        let mut value = format!("{}.{}", self.scope, self.category);

        if let Some(val) = self.operation.as_deref() {
            write!(&mut value, ".{val}").unwrap();
        }

        if let Some(val) = self.info.as_deref() {
            write!(&mut value, ".{val}").unwrap();
        }

        value
    }
}
