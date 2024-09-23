// TODO: Global Common

use std::{
    collections::HashMap,
    fmt::Display,
    hash::{Hash, Hasher},
    time::Duration,
};

use eyre::{anyhow, bail, Result};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use time::{macros::format_description, Date, OffsetDateTime, PrimitiveDateTime, Time};
use url::Url;
use uuid::Uuid;

//
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Number {
    Byte(u8),
    Integer(i64),
    Float(f64),
}

impl Number {
    pub fn into_u8(self) -> eyre::Result<u8> {
        if let Self::Byte(v) = self {
            Ok(v)
        } else {
            eyre::bail!("Not u8")
        }
    }

    // TODO: Impl Into
    pub fn convert_f64(self) -> f64 {
        match self {
            Number::Byte(v) => v as f64,
            Number::Integer(v) => v as f64,
            Number::Float(v) => v,
        }
    }
}

impl From<u8> for Number {
    fn from(value: u8) -> Self {
        Self::Byte(value)
    }
}

impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Self::Integer(value as i64)
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<Number> for i32 {
    fn from(val: Number) -> Self {
        match val {
            Number::Byte(v) => v as i32,
            Number::Integer(v) => v as i32,
            Number::Float(v) => v as i32,
        }
    }
}

impl From<Number> for i64 {
    fn from(val: Number) -> Self {
        match val {
            Number::Byte(v) => v as i64,
            Number::Integer(v) => v as i64,
            Number::Float(v) => v as i64,
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Byte(v) => v.fmt(f),
            Number::Integer(v) => v.fmt(f),
            Number::Float(v) => v.fmt(f),
        }
    }
}

impl Default for Number {
    fn default() -> Self {
        Self::Integer(0)
    }
}

// schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SimpleValue {
    Text(String),
    Number(Number),
    Boolean(bool),

    DateTime(OffsetDateTime),
    Date(Date),
    Time(Time),

    ListString(Vec<String>),
    ListNumber(Vec<Number>),

    ArrayUnknown(Vec<serde_json::Value>),
    ObjectUnknown(serde_json::Value),
}

impl SimpleValue {
    pub fn any_as_text(&self) -> Result<String> {
        Ok(match self {
            Self::Text(s) => s.to_string(),
            Self::Number(n) => n.to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::DateTime(dt) => dt.to_string(),
            Self::Date(d) => d.to_string(),
            Self::Time(t) => t.to_string(),
            Self::ListString(_)
            | Self::ListNumber(_)
            | Self::ArrayUnknown(_)
            | Self::ObjectUnknown(_) => return Err(anyhow!("Unable to convert to String"))?,
        })
    }

    pub fn try_as_text(self) -> Result<String> {
        if let Self::Text(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to Text"))?;
        }
    }

    pub fn try_as_number(&self) -> Result<Number> {
        if let Self::Number(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to Number"))?;
        }
    }

    pub fn try_as_boolean(&self) -> Result<bool> {
        if let Self::Boolean(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to Boolean"))?;
        }
    }

    pub fn try_as_date_time(&self) -> Result<OffsetDateTime> {
        if let Self::DateTime(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to DateTime"))?;
        }
    }

    pub fn try_as_date(&self) -> Result<Date> {
        if let Self::Date(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to Date"))?;
        }
    }

    pub fn try_as_time(&self) -> Result<Time> {
        if let Self::Time(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to Time"))?;
        }
    }

    pub fn try_as_list_string(self) -> Result<Vec<String>> {
        if let Self::ListString(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to String List"))?;
        }
    }

    pub fn try_as_list_number(self) -> Result<Vec<Number>> {
        if let Self::ListNumber(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to Number List"))?;
        }
    }

    pub fn try_as_bytes(self) -> Result<Vec<u8>> {
        if let Self::ListNumber(v) = self {
            Ok(v.into_iter().map(|v| v.into_u8()).collect::<Result<_>>()?)
        } else {
            return Err(anyhow!("Unable to convert to Number List"))?;
        }
    }

    pub fn ensure_text(self) -> Result<Self> {
        if matches!(self, Self::Text(_)) {
            Ok(self)
        } else {
            bail!("Not Text")
        }
    }

    pub fn ensure_number(self) -> Result<Self> {
        if matches!(self, Self::Number(_)) {
            Ok(self)
        } else {
            bail!("Not Number")
        }
    }

    pub fn ensure_boolean(self) -> Result<Self> {
        if matches!(self, Self::Boolean(_)) {
            Ok(self)
        } else {
            bail!("Not Boolean")
        }
    }

    pub fn ensure_date_time(self) -> Result<Self> {
        if matches!(self, Self::DateTime(_)) {
            Ok(self)
        } else {
            bail!("Not Date Time")
        }
    }

    pub fn ensure_date(self) -> Result<Self> {
        if matches!(self, Self::Date(_)) {
            Ok(self)
        } else {
            bail!("Not Date")
        }
    }

    pub fn ensure_time(self) -> Result<Self> {
        if matches!(self, Self::Time(_)) {
            Ok(self)
        } else {
            bail!("Not Time")
        }
    }

    pub fn ensure_list_string(self) -> Result<Self> {
        if matches!(self, Self::ListString(_)) {
            Ok(self)
        } else {
            bail!("Not String List")
        }
    }

    pub fn ensure_list_number(self) -> Result<Self> {
        if matches!(self, Self::ListNumber(_)) {
            Ok(self)
        } else {
            bail!("Not Number List")
        }
    }
}

pub type SchemaFieldMap = HashMap<SchematicFieldKey, SchematicField>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schematic {
    pub id: String,
    /// What the schema is for: Forms, Members, Marketing, Billing, etc.
    pub namespace: String,
    /// The field to display if it's being referenced.
    pub primary_field: String,
    /// The name of the schema.
    pub display_name: String,
    /// The capabilities of the schema.
    pub permissions: SchematicPermissions,
    pub version: f64,
    /// The operations allowed on the schema.
    pub allowed_operations: Vec<String>,
    pub is_deleted: bool,
    pub owner_app_id: String,
    pub fields: SchemaFieldMap,
    // pub storage: String,
    /// Time to live
    pub ttl: Option<Duration>,
    pub default_sort: Option<DefaultSort>,
    // pub paging_mode: Vec<String>,
    pub views: Vec<SchemaView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaView {
    pub name: String,
    pub query: SchemaViewQuery,
    pub view_type: SchemaViewTypes,
}

impl Default for SchemaView {
    fn default() -> Self {
        Self {
            name: String::from("Default View"),
            query: SchemaViewQuery::default(),
            view_type: SchemaViewTypes::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SchemaViewTypes {
    pub form: SchemaViewItem,
    pub gallery: SchemaViewItem,
    pub list: SchemaViewItem,
    pub table: SchemaViewItem,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaViewItem {
    #[serde(default)]
    pub hidden_fields: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SchemaViewQuery {
    pub sort: Vec<DefaultSort>,
    pub filter: Vec<SchemaFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchematicPermissions {
    pub insert: PermissionsUser,
    pub update: PermissionsUser,
    pub remove: PermissionsUser,
    pub read: PermissionsUser,
}

impl Default for SchematicPermissions {
    fn default() -> Self {
        Self {
            insert: PermissionsUser::Admin,
            update: PermissionsUser::Admin,
            remove: PermissionsUser::Admin,
            read: PermissionsUser::Admin,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionsUser {
    Anyone,
    Admin,
    Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operations {
    BulkInsert,
    BulkSave,
    QueryReferenced,
    Truncate,
    ReplaceReferences,
    Count,
    Get,
    Find,
    RemoveReference,
    IsReferenced,
    Distinct,
    Remove,
    BulkUpdate,
    Insert,
    Save,
    Update,
    BulkRemove,
    Aggregate,
    InsertReference,
}

#[derive(Debug, Clone, Eq)]
pub enum SchematicFieldKey {
    Id,
    Owner,
    CreatedAt,
    UpdatedAt,
    Other(String),
    OtherStatic(&'static str),
}

impl SchematicFieldKey {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Id => "_id",
            Self::Owner => "_owner",
            Self::CreatedAt => "_createdAt",
            Self::UpdatedAt => "_updatedAt",
            Self::Other(s) => s,
            Self::OtherStatic(s) => s,
        }
    }

    pub fn is_other(&self) -> bool {
        matches!(self, Self::Other(_) | Self::OtherStatic(_))
    }
}

impl Hash for SchematicFieldKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Display for SchematicFieldKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl PartialEq for SchematicFieldKey {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&str> for SchematicFieldKey {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for SchematicFieldKey {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl Serialize for SchematicFieldKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SchematicFieldKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        Ok(match s.as_str() {
            "_id" => Self::Id,
            "_owner" => Self::Owner,
            "_createdAt" => Self::CreatedAt,
            "_updatedAt" => Self::UpdatedAt,
            _ => Self::Other(s),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchematicField {
    pub display_name: String,
    pub sortable: bool,
    pub is_deleted: bool,
    pub system_field: bool,
    pub field_type: SchematicFieldType,
    pub index: u16,

    // Reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referenced_schema: Option<String>,
    // TODO: Default value setter - used for when "duplicating another field"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaFilter {
    pub field: String,
    pub condition: String,
    pub value: SchematicFieldValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSort {
    pub field: String,
    pub order: SortOrder,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchematicFieldBasicType {
    Text,
    Number,
    Boolean,
    DateTime,
    Date,
    Time,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TryFromPrimitive, IntoPrimitive,
)]
#[repr(i32)]
pub enum SchematicFieldType {
    /// A string of text.
    Text,
    /// A number.
    Number,
    /// A URL.
    URL,
    /// An email address.
    Email,
    /// An address.
    Address,
    /// A phone number.
    Phone,
    /// A boolean.
    Boolean,
    /// A date and time.
    DateTime,
    /// A date.
    Date,
    /// A time.
    Time,
    /// Rich content.
    RichContent,
    /// Rich text.
    RichText,
    /// A reference to another schema item.
    Reference,
    /// A reference to multiple schema items.
    MultiReference,
    /// A media gallery.
    MediaGallery,
    /// A document.
    Document,
    /// A multi-document.
    MultiDocument,
    /// An image.
    Image,
    /// A video.
    Video,
    /// An audio.
    Audio,
    /// An array of tags.
    Tags,
    /// An array
    Array,
    /// An object.
    Object,
}

impl SchematicFieldType {
    // TODO: Better Name. Used to determine if bytes being uploaded are a file or not.
    pub fn is_upload_file_type(&self) -> bool {
        matches!(
            self,
            SchematicFieldType::Document
                | SchematicFieldType::MultiDocument
                | SchematicFieldType::Audio
                | SchematicFieldType::Image
                | SchematicFieldType::Video
        )
    }

    pub fn max_bytes_length(&self) -> Option<usize> {
        match self {
            Self::Text => Some(1024 * 1024 * 1024),
            Self::Email => Some(100),
            Self::Number => Some(10),
            Self::URL => Some(1024),
            Self::Address => Some(1024),
            Self::Phone => Some(50),
            Self::Boolean => Some(1),
            Self::DateTime => Some(50),
            Self::Date => Some(50),
            Self::Time => Some(50),
            Self::RichContent => Some(1024 * 1024 * 10),
            Self::RichText => Some(1024 * 1024 * 10),
            Self::Reference => None,
            Self::MultiReference => None,
            Self::MediaGallery => Some(1024 * 1024 * 100),
            Self::Document => Some(1024 * 1024 * 100),
            Self::MultiDocument => Some(1024 * 1024 * 100),
            Self::Image => Some(1024 * 1024 * 100),
            Self::Video => Some(1024 * 1024 * 100),
            Self::Audio => Some(1024 * 1024 * 100),
            Self::Tags => None, // TODO
            Self::Array => None,
            Self::Object => None,
        }
    }

    pub fn parse_value_bytes(self, bytes: Vec<u8>) -> eyre::Result<SimpleValue> {
        match self {
            SchematicFieldType::Number => Ok(serde_json::from_slice(&bytes)?),
            SchematicFieldType::Text
            | SchematicFieldType::URL
            | SchematicFieldType::Email
            | SchematicFieldType::Address
            | SchematicFieldType::Phone
            | SchematicFieldType::Boolean
            | SchematicFieldType::DateTime
            | SchematicFieldType::Date
            | SchematicFieldType::Time
            | SchematicFieldType::RichContent
            | SchematicFieldType::RichText
            | SchematicFieldType::Reference
            | SchematicFieldType::Array
            | SchematicFieldType::Object => Ok(SimpleValue::Text(String::from_utf8(bytes)?)),
            SchematicFieldType::Document
            | SchematicFieldType::Image
            | SchematicFieldType::Video
            | SchematicFieldType::Audio => Ok(SimpleValue::ListNumber(
                bytes.into_iter().map(|v| v.into()).collect(),
            )),
            SchematicFieldType::MultiReference
            | SchematicFieldType::MediaGallery
            | SchematicFieldType::MultiDocument
            | SchematicFieldType::Tags => {
                todo!("{:?} {bytes:?}", String::from_utf8_lossy(&bytes));
            }
        }
    }

    pub fn parse_value(self, received: SimpleValue) -> eyre::Result<SchematicFieldValue> {
        Ok(match self {
            Self::Text => SchematicFieldValue::Text(received.try_as_text()?),
            Self::Number => SchematicFieldValue::Number(received.try_as_number()?),
            Self::URL => SchematicFieldValue::Url(Url::parse(&received.try_as_text()?)?),
            Self::Email => SchematicFieldValue::Email(received.try_as_text()?),
            Self::Phone => SchematicFieldValue::Phone(received.try_as_text()?),
            Self::Address => SchematicFieldValue::Address(received.try_as_text()?),
            Self::Boolean => SchematicFieldValue::Boolean(match received.try_as_text()?.as_str() {
                "1" | "on" | "true" => true,
                "0" | "off" | "false" => false,
                v => v.parse()?,
            }),
            Self::DateTime => SchematicFieldValue::DateTime(
                PrimitiveDateTime::parse(
                    &received.any_as_text()?,
                    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
                )?
                .assume_utc(),
            ),
            Self::Date => SchematicFieldValue::Date(Date::parse(
                &received.any_as_text()?,
                format_description!("[year]-[month]-[day]"),
            )?),
            Self::Time => SchematicFieldValue::Time(Time::parse(
                &received.any_as_text()?,
                format_description!("[hour]:[minute]:[second]"),
            )?),
            Self::RichContent => SchematicFieldValue::Text(received.try_as_text()?),
            Self::RichText => SchematicFieldValue::Text(received.try_as_text()?),
            Self::Reference => SchematicFieldValue::Reference(received.try_as_text()?.parse()?),
            Self::MultiReference => SchematicFieldValue::MultiReference(
                received
                    .try_as_list_string()?
                    .into_iter()
                    .map(|v| v.parse())
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            ),
            Self::MediaGallery => SchematicFieldValue::MultiReference(
                received
                    .try_as_list_string()?
                    .into_iter()
                    .map(|v| v.parse())
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            ),
            Self::Document | Self::Image | Self::Video | Self::Audio => {
                SchematicFieldValue::ListNumber(received.try_as_list_number()?)
            }
            Self::MultiDocument => todo!("Multi Document"),
            Self::Tags => SchematicFieldValue::ListNumber(received.try_as_list_number()?),
            Self::Array => SchematicFieldValue::Text(received.try_as_text()?),
            Self::Object => SchematicFieldValue::Text(received.try_as_text()?),
        })
    }

    pub fn as_name(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Number => "Number",
            Self::URL => "URL",
            Self::Email => "Email",
            Self::Address => "Address",
            Self::Phone => "Phone",
            Self::Boolean => "True/False",
            Self::DateTime => "Date & Time",
            Self::Date => "Date",
            Self::Time => "Time",
            Self::RichContent => "Rich Content",
            Self::RichText => "Rich Text",
            Self::Reference => "Reference",
            Self::MultiReference => "Multi Reference",
            Self::MediaGallery => "Media Gallery",
            Self::Document => "Document",
            Self::MultiDocument => "Multi Document",
            Self::Image => "Image",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Tags => "Tags",
            Self::Array => "Array",
            Self::Object => "Object",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SchematicFieldValue {
    // Url gets serialized/deserialized to/from a String
    Text(String),
    Number(Number),
    Boolean(bool),

    Url(Url),
    Email(String),
    Phone(String),
    Address(String),

    DateTime(OffsetDateTime),
    Date(Date),
    Time(Time),
    Reference(Uuid),
    MultiReference(Vec<Uuid>),
    ListString(Vec<String>),
    ListNumber(Vec<Number>),

    Array(Vec<serde_json::Value>),
    Object(serde_json::Value),
}

impl SchematicFieldValue {
    pub fn try_as_reference(self) -> Result<Uuid> {
        if let Self::Reference(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to Reference"))?;
        }
    }

    pub fn try_as_text(self) -> Result<String> {
        if let Self::Text(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to Text"))?;
        }
    }

    pub fn try_as_number(&self) -> Result<Number> {
        if let Self::Number(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to Number"))?;
        }
    }

    pub fn try_as_boolean(&self) -> Result<bool> {
        if let Self::Boolean(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to Boolean"))?;
        }
    }

    pub fn try_as_url(self) -> Result<Url> {
        if let Self::Url(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to Url"))?;
        }
    }

    pub fn try_as_email(self) -> Result<String> {
        if let Self::Email(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to Email"))?;
        }
    }

    pub fn try_as_phone(self) -> Result<String> {
        if let Self::Phone(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to Phone"))?;
        }
    }

    pub fn try_as_address(self) -> Result<String> {
        if let Self::Address(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to Address"))?;
        }
    }

    pub fn try_as_date_time(&self) -> Result<OffsetDateTime> {
        if let Self::DateTime(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to DateTime"))?;
        }
    }

    pub fn try_as_date(&self) -> Result<Date> {
        if let Self::Date(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to Date"))?;
        }
    }

    pub fn try_as_time(&self) -> Result<Time> {
        if let Self::Time(v) = self {
            Ok(*v)
        } else {
            return Err(anyhow!("Unable to convert to Time"))?;
        }
    }

    pub fn try_as_list_string(self) -> Result<Vec<String>> {
        if let Self::ListString(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to String List"))?;
        }
    }

    pub fn try_as_list_number(self) -> Result<Vec<Number>> {
        if let Self::ListNumber(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to String List"))?;
        }
    }

    pub fn try_as_list_reference(self) -> Result<Vec<Uuid>> {
        if let Self::MultiReference(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to String List"))?;
        }
    }

    pub fn try_as_array(self) -> Result<Vec<serde_json::Value>> {
        if let Self::Array(v) = self {
            Ok(v)
        } else {
            return Err(anyhow!("Unable to convert to String List"))?;
        }
    }
}

mod _backend {
    use std::result::Result;

    use sqlx::{
        encode::IsNull,
        error::BoxDynError,
        sqlite::{SqliteRow, SqliteTypeInfo},
        Decode, Encode, FromRow, Row, Sqlite, Type,
    };

    use super::SchematicFieldType;

    impl FromRow<'_, SqliteRow> for SchematicFieldType {
        fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
            Ok(Self::try_from(row.try_get::<i32, _>(0)?).unwrap())
        }
    }

    impl Encode<'_, Sqlite> for SchematicFieldType {
        fn encode_by_ref(
            &self,
            buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
        ) -> Result<IsNull, BoxDynError> {
            Encode::<Sqlite>::encode_by_ref(&(*self as i32), buf)
        }
    }

    impl Decode<'_, Sqlite> for SchematicFieldType {
        fn decode(value: <Sqlite as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
            Ok(Self::try_from(<i32 as Decode<Sqlite>>::decode(value)?)?)
        }
    }

    impl Type<Sqlite> for SchematicFieldType {
        fn type_info() -> SqliteTypeInfo {
            <i32 as Type<Sqlite>>::type_info()
        }
    }
}
