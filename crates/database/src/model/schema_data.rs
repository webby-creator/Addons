use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};

use eyre::{ContextCompat, Result};
use global_common::{
    filter::{Filter, FilterConditionType, FilterValue},
    schema::{SchematicFieldKey, SchematicFieldType, SchematicFieldValue},
    value::Number,
};
use local_common::{AddonId, SchemaDataId, SchemaDataTagId, SchemaId};
use serde::Serialize;
use sqlx::{types::Json, FromRow, SqliteConnection};
use time::{Date, OffsetDateTime, Time};
use uuid::Uuid;

use crate::SchemaModel;

#[derive(Debug)]
pub struct NewSchemaDataModel {
    pub addon_id: AddonId,
    pub schema_id: SchemaId,

    pub public_id: Uuid,

    pub field_text: Option<Json<HashMap<String, String>>>,
    pub field_number: Option<Json<HashMap<String, Number>>>,
    pub field_url: Option<Json<HashMap<String, String>>>,
    pub field_email: Option<Json<HashMap<String, String>>>,
    pub field_address: Option<Json<HashMap<String, String>>>,
    pub field_phone: Option<Json<HashMap<String, String>>>,
    pub field_bool: Option<Json<HashMap<String, bool>>>,
    pub field_datetime: Option<Json<HashMap<String, OffsetDateTime>>>,
    pub field_date: Option<Json<HashMap<String, Date>>>,
    pub field_time: Option<Json<HashMap<String, Time>>>,
    pub field_rich_content: Option<Json<HashMap<String, String>>>,
    pub field_rich_text: Option<Json<HashMap<String, String>>>,

    // TODO: At some point I may want to use ids - not UUIDs
    pub field_reference: Option<Json<HashMap<String, Uuid>>>,
    pub field_multi_reference: Option<Json<HashMap<String, Vec<Uuid>>>>,
    pub field_gallery: Option<Json<HashMap<String, Vec<Uuid>>>>,
    pub field_document: Option<Json<HashMap<String, Uuid>>>,
    pub field_multi_document: Option<Json<HashMap<String, Vec<Uuid>>>>,
    pub field_image: Option<Json<HashMap<String, Uuid>>>,
    pub field_video: Option<Json<HashMap<String, Uuid>>>,
    pub field_audio: Option<Json<HashMap<String, Uuid>>>,
    pub field_tags: Option<Json<HashMap<String, Vec<SchemaDataTagId>>>>,
    pub field_array: Option<Json<HashMap<String, Vec<serde_json::Value>>>>,
    pub field_object: Option<Json<HashMap<String, serde_json::Value>>>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SchemaDataModel {
    pub id: SchemaDataId,

    pub addon_id: AddonId,
    pub schema_id: SchemaId,

    pub public_id: Uuid,

    #[sqlx(default)]
    pub field_text: Option<Json<HashMap<String, String>>>,
    #[sqlx(default)]
    pub field_number: Option<Json<HashMap<String, Number>>>,
    #[sqlx(default)]
    pub field_url: Option<Json<HashMap<String, String>>>,
    #[sqlx(default)]
    pub field_email: Option<Json<HashMap<String, String>>>,
    #[sqlx(default)]
    pub field_address: Option<Json<HashMap<String, String>>>,
    #[sqlx(default)]
    pub field_phone: Option<Json<HashMap<String, String>>>,
    #[sqlx(default)]
    pub field_bool: Option<Json<HashMap<String, bool>>>,
    #[sqlx(default)]
    pub field_datetime: Option<Json<HashMap<String, OffsetDateTime>>>,
    #[sqlx(default)]
    pub field_date: Option<Json<HashMap<String, Date>>>,
    #[sqlx(default)]
    pub field_time: Option<Json<HashMap<String, Time>>>,
    #[sqlx(default)]
    pub field_rich_content: Option<Json<HashMap<String, String>>>,
    #[sqlx(default)]
    pub field_rich_text: Option<Json<HashMap<String, String>>>,

    #[sqlx(default)]
    pub field_reference: Option<Json<HashMap<String, Uuid>>>,
    #[sqlx(default)]
    pub field_multi_reference: Option<Json<HashMap<String, Vec<Uuid>>>>,
    #[sqlx(default)]
    pub field_gallery: Option<Json<HashMap<String, Vec<Uuid>>>>,
    #[sqlx(default)]
    pub field_document: Option<Json<HashMap<String, Uuid>>>,
    #[sqlx(default)]
    pub field_multi_document: Option<Json<HashMap<String, Vec<Uuid>>>>,
    #[sqlx(default)]
    pub field_image: Option<Json<HashMap<String, Uuid>>>,
    #[sqlx(default)]
    pub field_video: Option<Json<HashMap<String, Uuid>>>,
    #[sqlx(default)]
    pub field_audio: Option<Json<HashMap<String, Uuid>>>,
    #[sqlx(default)]
    pub field_tags: Option<Json<HashMap<String, Vec<SchemaDataTagId>>>>,
    #[sqlx(default)]
    pub field_array: Option<Json<HashMap<String, Vec<serde_json::Value>>>>,
    #[sqlx(default)]
    pub field_object: Option<Json<HashMap<String, serde_json::Value>>>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

pub struct SchemaDataFieldUpdate {
    pub id: SchemaDataId,

    pub addon_id: AddonId,
    pub schema_id: SchemaId,

    pub public_id: Uuid,

    pub field: SchemaDataFieldUpdateType,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

pub enum SchemaDataFieldUpdateType {
    Text(Option<HashMap<String, String>>),
    Number(Option<HashMap<String, Number>>),
    Url(Option<HashMap<String, String>>),
    Email(Option<HashMap<String, String>>),
    Address(Option<HashMap<String, String>>),
    Phone(Option<HashMap<String, String>>),
    Bool(Option<HashMap<String, bool>>),
    DateTime(Option<HashMap<String, OffsetDateTime>>),
    Date(Option<HashMap<String, Date>>),
    Time(Option<HashMap<String, Time>>),
    RichContent(Option<HashMap<String, String>>),
    RichText(Option<HashMap<String, String>>),
    Reference(Option<HashMap<String, Uuid>>),
    MultiReference(Option<HashMap<String, Vec<Uuid>>>),
    Gallery(Option<HashMap<String, Vec<Uuid>>>),
    Document(Option<HashMap<String, Uuid>>),
    MultiDocument(Option<HashMap<String, Vec<Uuid>>>),
    Image(Option<HashMap<String, Uuid>>),
    Video(Option<HashMap<String, Uuid>>),
    Audio(Option<HashMap<String, Uuid>>),
    Tags(Option<HashMap<String, Vec<SchemaDataTagId>>>),
    Array(Option<HashMap<String, Vec<serde_json::Value>>>),
    Object(Option<HashMap<String, serde_json::Value>>),
}

impl NewSchemaDataModel {
    pub fn new(addon_id: AddonId, schema_id: SchemaId) -> Self {
        let now = OffsetDateTime::now_utc();

        Self {
            addon_id,
            schema_id,
            public_id: Uuid::now_v7(),
            field_text: None,
            field_number: None,
            field_url: None,
            field_email: None,
            field_address: None,
            field_phone: None,
            field_bool: None,
            field_datetime: None,
            field_date: None,
            field_time: None,
            field_rich_content: None,
            field_rich_text: None,
            field_reference: None,
            field_multi_reference: None,
            field_gallery: None,
            field_document: None,
            field_multi_document: None,
            field_image: None,
            field_video: None,
            field_audio: None,
            field_tags: None,
            field_array: None,
            field_object: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn into_self(self, id: SchemaDataId) -> SchemaDataModel {
        SchemaDataModel {
            id,

            addon_id: self.addon_id,
            schema_id: self.schema_id,
            public_id: self.public_id,

            field_text: self.field_text,
            field_number: self.field_number,
            field_url: self.field_url,
            field_email: self.field_email,
            field_address: self.field_address,
            field_phone: self.field_phone,
            field_bool: self.field_bool,
            field_datetime: self.field_datetime,
            field_date: self.field_date,
            field_time: self.field_time,
            field_rich_content: self.field_rich_content,
            field_rich_text: self.field_rich_text,
            field_reference: self.field_reference,
            field_multi_reference: self.field_multi_reference,
            field_gallery: self.field_gallery,
            field_document: self.field_document,
            field_multi_document: self.field_multi_document,
            field_image: self.field_image,
            field_video: self.field_video,
            field_audio: self.field_audio,
            field_tags: self.field_tags,
            field_array: self.field_array,
            field_object: self.field_object,

            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: None,
        }
    }

    pub async fn insert(self, db: &mut SqliteConnection) -> Result<SchemaDataModel> {
        let res = sqlx::query(
            r#"
                INSERT INTO schema_data (
                    addon_id, schema_id, public_id,
                    field_text, field_number, field_url, field_email, field_address, field_phone, field_bool, field_datetime, field_date,
                    field_time, field_rich_content, field_rich_text, field_reference, field_multi_reference, field_gallery, field_document,
                    field_multi_document, field_image, field_video, field_audio, field_tags, field_array, field_object,
                    created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
            "#,
        )
        .bind(self.addon_id)
        .bind(self.schema_id)
        .bind(&self.public_id)
        .bind(&self.field_text)
        .bind(&self.field_number)
        .bind(&self.field_url)
        .bind(&self.field_email)
        .bind(&self.field_address)
        .bind(&self.field_phone)
        .bind(&self.field_bool)
        .bind(&self.field_datetime)
        .bind(&self.field_date)
        .bind(&self.field_time)
        .bind(&self.field_rich_content)
        .bind(&self.field_rich_text)
        .bind(&self.field_reference)
        .bind(&self.field_multi_reference)
        .bind(&self.field_gallery)
        .bind(&self.field_document)
        .bind(&self.field_multi_document)
        .bind(&self.field_image)
        .bind(&self.field_video)
        .bind(&self.field_audio)
        .bind(&self.field_tags)
        .bind(&self.field_array)
        .bind(&self.field_object)
        .bind(self.created_at)
        .bind(self.updated_at)
        .execute(db)
        .await?;

        Ok(self.into_self(SchemaDataId::from(res.last_insert_rowid())))
    }

    pub fn insert_field(
        &mut self,
        field_name: String,
        is_field_in_duplicator: bool,
        field_type: SchematicFieldType,
        value: SchematicFieldValue,
    ) -> Result<()> {
        // Checks to see if the field is in a duplicator. The Value WILL be an Array.
        // We ignore the Field Type.
        if is_field_in_duplicator {
            self.field_array
                .get_or_insert_with(Default::default)
                .insert(field_name, value.try_as_array()?);

            return Ok(());
        }

        match field_type {
            SchematicFieldType::Text => {
                self.field_text
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_text()?);
            }
            SchematicFieldType::Number => {
                self.field_number
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_number()?);
            }
            SchematicFieldType::URL => {
                self.field_url
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_url()?.to_string());
            }
            SchematicFieldType::Email => {
                self.field_email
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_email()?);
            }
            SchematicFieldType::Address => {
                self.field_address
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_address()?);
            }
            SchematicFieldType::Phone => {
                self.field_phone
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_phone()?);
            }
            SchematicFieldType::Boolean => {
                self.field_bool
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_boolean()?);
            }
            SchematicFieldType::DateTime => {
                self.field_datetime
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_date_time()?);
            }
            SchematicFieldType::Date => {
                self.field_date
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_date()?);
            }
            SchematicFieldType::Time => {
                self.field_time
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_time()?);
            }
            SchematicFieldType::RichContent => {
                self.field_rich_content
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_text()?);
            }
            SchematicFieldType::RichText => {
                self.field_rich_text
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_text()?);
            }
            SchematicFieldType::Reference => {
                self.field_reference
                    .get_or_insert_with(Default::default)
                    .insert(field_name, Uuid::parse_str(&value.try_as_text()?)?);
            }
            SchematicFieldType::MultiReference => {
                self.field_multi_reference
                    .get_or_insert_with(Default::default)
                    .insert(
                        field_name,
                        value
                            .try_as_list_string()?
                            .into_iter()
                            .map(|v| Uuid::parse_str(&v))
                            .collect::<Result<Vec<_>, uuid::Error>>()?,
                    );
            }
            SchematicFieldType::MediaGallery => {
                self.field_gallery
                    .get_or_insert_with(Default::default)
                    .insert(
                        field_name,
                        value
                            .try_as_list_string()?
                            .into_iter()
                            .map(|v| Uuid::parse_str(&v))
                            .collect::<Result<Vec<_>, uuid::Error>>()?,
                    );
            }
            SchematicFieldType::Document => {
                self.field_document
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_reference()?);
            }
            SchematicFieldType::MultiDocument => {
                self.field_multi_document
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_list_reference()?);
            }
            SchematicFieldType::Image => {
                self.field_image
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_reference()?);
            }
            SchematicFieldType::Video => {
                self.field_video
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_reference()?);
            }
            SchematicFieldType::Audio => {
                self.field_audio
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_reference()?);
            }
            SchematicFieldType::Tags => {
                self.field_tags.get_or_insert_with(Default::default).insert(
                    field_name,
                    value
                        .try_as_list_number()?
                        .into_iter()
                        .map(|v| i64::from(v).into())
                        .collect(),
                );
            }
            SchematicFieldType::Array => {
                self.field_array
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_array()?);
            }
            SchematicFieldType::Object => {
                self.field_object
                    .get_or_insert_with(Default::default)
                    .insert(field_name, value.try_as_object()?);
            }
        }

        Ok(())
    }
}

impl SchemaDataModel {
    pub fn into_new(self) -> NewSchemaDataModel {
        let now = OffsetDateTime::now_utc();

        NewSchemaDataModel {
            addon_id: self.addon_id,
            schema_id: self.schema_id,
            public_id: Uuid::now_v7(),

            field_text: self.field_text,
            field_number: self.field_number,
            field_url: self.field_url,
            field_email: self.field_email,
            field_address: self.field_address,
            field_phone: self.field_phone,
            field_bool: self.field_bool,
            field_datetime: self.field_datetime,
            field_date: self.field_date,
            field_time: self.field_time,
            field_rich_content: self.field_rich_content,
            field_rich_text: self.field_rich_text,
            field_reference: self.field_reference,
            field_multi_reference: self.field_multi_reference,
            field_gallery: self.field_gallery,
            field_document: self.field_document,
            field_multi_document: self.field_multi_document,
            field_image: self.field_image,
            field_video: self.field_video,
            field_audio: self.field_audio,
            field_tags: self.field_tags,
            field_array: self.field_array,
            field_object: self.field_object,

            created_at: now,
            updated_at: now,
        }
    }

    pub async fn find_by_website_id(
        addon_id: AddonId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, addon_id, schema_id, public_id,
            field_text, field_number, field_url, field_email, field_address, field_phone, field_bool, field_datetime, field_date,
            field_time, field_rich_content, field_rich_text, field_reference, field_multi_reference, field_gallery, field_document,
            field_multi_document, field_image, field_video, field_audio, field_tags, field_array, field_object,
            created_at, updated_at, deleted_at FROM schema_data WHERE addon_id = $1",
        )
        .bind(addon_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn find_by_schema_id(
        schema_id: SchemaId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, addon_id, schema_id, public_id,
            field_text, field_number, field_url, field_email, field_address, field_phone, field_bool, field_datetime, field_date,
            field_time, field_rich_content, field_rich_text, field_reference, field_multi_reference, field_gallery, field_document,
            field_multi_document, field_image, field_video, field_audio, field_tags, field_array, field_object,
            created_at, updated_at, deleted_at FROM schema_data WHERE schema_id = $1",
        )
        .bind(schema_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn find_by_public_id(id: Uuid, db: &mut SqliteConnection) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, addon_id, schema_id, public_id,
            field_text, field_number, field_url, field_email, field_address, field_phone, field_bool, field_datetime, field_date,
            field_time, field_rich_content, field_rich_text, field_reference, field_multi_reference, field_gallery, field_document,
            field_multi_document, field_image, field_video, field_audio, field_tags, field_array, field_object,
            created_at, updated_at, deleted_at FROM schema_data WHERE public_id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_by(
        addon_id: AddonId,
        schema: &SchemaModel,

        filter: Option<&[Filter]>,
        order: Option<HashMap<String, String>>,

        offset: i64,
        limit: i64,

        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        let schema_id = schema.id;

        match (filter, order) {
            (None, None) => {
                Ok(sqlx::query_as(
                    "SELECT
                        id, addon_id, schema_id, public_id,
                        field_text, field_number, field_url, field_email, field_address, field_phone, field_bool, field_datetime, field_date,
                        field_time, field_rich_content, field_rich_text, field_reference, field_multi_reference, field_gallery, field_document,
                        field_multi_document, field_image, field_video, field_audio, field_tags, field_array, field_object,
                        created_at, updated_at, deleted_at
                    FROM schema_data
                    WHERE
                        schema_id = $1
                    LIMIT $2 OFFSET $3",
                )
                .bind(schema_id)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(db)
                .await?)
            }

            (filter, order) => {
                let filters = filter.unwrap_or_default();

                // TODO: Multiple Ordering
                // Order
                let (order_json_tree, order_by) = if let Some(order) = order {
                    let Some((order_field, order_dir)) = order.into_iter().next() else {
                        return Err(eyre::eyre!("Unable to find order"))?;
                    };

                    let order_dir = match order_dir.to_lowercase().as_str() {
                        "desc" => "DESC",
                        _ => "ASC"
                    };

                    let Some((order_field_key, field)) = schema
                        .fields
                        .get_key_value(&SchematicFieldKey::Other(order_field.clone())) else {
                            return Err(eyre::eyre!("Unable to find order field"))?;
                        };

                    let order_name = field_type_to_sql_name(field.field_type);

                    (Some((order_field_key, order_name)), Some((order_field_key, order_dir)))
                } else {
                    (None, None)
                };

                // Field

                let mut sql_building =
                    String::from("SELECT schema_data.id, addon_id, schema_id, public_id,
    field_text, field_number, field_url, field_email, field_address, field_phone, field_bool, field_datetime, field_date,
    field_time, field_rich_content, field_rich_text, field_reference, field_multi_reference, field_gallery, field_document,
    field_multi_document, field_image, field_video, field_audio, field_tags, field_array, field_object,
    created_at, updated_at, deleted_at
FROM schema_data");

                // If we're using custom field ordering
                if let Some((order_field_key, order_name)) = order_json_tree {
                    if let SchematicFieldKey::Other(key) = order_field_key {
                        sql_building.push_str(",\n");

                        writeln!(
                            &mut sql_building,
                            "json_tree(json_patch('{{ \"{key}\": null }}', schema_data.{order_name}), '$.{key}') as json_order",
                        )?;
                    }
                }

                // If we're using custom field filtering
                let mut added_filters = HashSet::new();

                for filter in filters {
                    if added_filters.contains(&filter.name) {
                        continue;
                    }

                    added_filters.insert(filter.name.clone());

                    sql_building.push_str(",\n");

                    let field = schema
                        .fields
                        .get(&SchematicFieldKey::Other(filter.name.clone()))
                        .context("Unable to find primary field")?;

                    let column_name = field_type_to_sql_name(field.field_type);

                    writeln!(
                        &mut sql_building,
                        "json_tree(schema_data.{column_name}, '$.{filter_name}') as json_{filter_name}",
                        filter_name = filter.name,
                    )?;
                }

                writeln!(
                    &mut sql_building,
                    "WHERE addon_id = $1 AND schema_id = $2"
                )?;

                let mut pos = 3;

                // WHERE FILTER
                for filter in filters {
                    sql_building.push_str(" AND\n");

                    let field = schema
                        .fields
                        .get(&SchematicFieldKey::Other(filter.name.clone()))
                        .context("Unable to find primary field")?;

                    let column_name = field_type_to_sql_name(field.field_type);

                    let cond = match filter.cond {
                        // Text
                        FilterConditionType::Cont => "LIKE",
                        FilterConditionType::Dnc => "NOT LIKE",

                        FilterConditionType::Eq => "=",
                        FilterConditionType::Neq => "!=",
                        FilterConditionType::Gte => ">=",
                        FilterConditionType::Gt => ">",
                        FilterConditionType::Lte => "<=",
                        FilterConditionType::Lt => "<",

                        FilterConditionType::Between => "BETWEEN",
                    };

                    if filter.value.is_range() && filter.cond == FilterConditionType::Between {
                        writeln!(
                            &mut sql_building,
                            "({column_name} IS NOT NULL AND json_{filter_name}.value {cond} ${pos} AND ${pos2})",
                            filter_name = filter.name,
                            pos2 = pos + 1,
                        )?;

                        pos += 2;
                    } else {
                        writeln!(
                            &mut sql_building,
                            "({column_name} IS NOT NULL AND json_{filter_name}.value {cond} ${pos})",
                            filter_name = filter.name,
                        )?;

                        pos += 1;
                    }
                }

                // ORDER BY
                if let Some((order_field_key, order_dir)) = order_by {
                    if order_field_key.is_other() {
                        sql_building.push_str("\nORDER BY ");

                        writeln!(
                            &mut sql_building,
                            "json_order.value {order_dir} NULLS LAST",
                        )?;
                    } else {
                        let key = match order_field_key {
                            SchematicFieldKey::Id => "id",
                            SchematicFieldKey::Owner => "creator_id",
                            SchematicFieldKey::CreatedAt => "created_at",
                            SchematicFieldKey::UpdatedAt => "updated_at",
                            _ => unreachable!()
                        };

                        sql_building.push_str("\nORDER BY ");

                        writeln!(
                            &mut sql_building,
                            "{key} {order_dir} NULLS LAST",
                        )?;
                    }
                }

                // LIMIT
                writeln!(
                    &mut sql_building,
                    "LIMIT {limit} OFFSET {offset}",
                )?;

                // Query
                let mut query = sqlx::query_as(&sql_building)
                    .bind(addon_id)
                    .bind(schema_id);

                for filter in filters {
                    // TODO: BUG FIX: Instead of FilterValue being a Number, it'll be Text
                    let field = schema
                        .fields
                        .get(&SchematicFieldKey::Other(filter.name.clone()))
                        .context("Unable to find primary field")?;

                    let mut value = filter.value.clone();

                    if matches!(field.field_type, SchematicFieldType::Number) {
                        value = match value {
                            FilterValue::Text(v) => {
                                if v.contains(".") {
                                    if let Ok(number) = v.parse() {
                                        FilterValue::Number(Number::Float(number))
                                    } else {
                                        FilterValue::Text(v)
                                    }
                                } else if let Ok(number) = v.parse() {
                                    FilterValue::Number(Number::Integer(number))
                                } else {
                                    FilterValue::Text(v)
                                }
                            }

                            v => v
                        };
                    }

                    if value.is_range() && filter.cond == FilterConditionType::Between {
                        if let FilterValue::Range((min, max)) = value {
                            query = query.bind(min.convert_f64()).bind(max.convert_f64());
                        }
                    } else if filter.cond == FilterConditionType::Cont
                        || filter.cond == FilterConditionType::Dnc
                    {
                        query = query.bind(format!("%{value}%"));
                    } else {
                        match &value {
                            &FilterValue::Number(n) => {
                                match n {
                                    Number::Byte(n) => query = query.bind(n),
                                    Number::Integer(n) => query = query.bind(n),
                                    Number::Float(n) => query = query.bind(n),
                                }
                            }

                            v => query = query.bind(v.to_string()),
                        }
                    }
                }

                Ok(query.fetch_all(db).await?)
            }
        }
    }

    pub async fn count_by(
        addon_id: AddonId,
        schema: &SchemaModel,

        filter: Option<&[Filter]>,

        db: &mut SqliteConnection,
    ) -> Result<i64> {
        let schema_id = schema.id;

        match filter {
            Some(filters) => {
                let mut sql_building =
                    String::from("SELECT COUNT(schema_data.id) FROM schema_data");

                let mut added_filters = HashSet::new();

                for filter in filters {
                    if added_filters.contains(&filter.name) {
                        continue;
                    }

                    added_filters.insert(filter.name.clone());

                    sql_building.push_str(",\n");

                    let field = schema
                        .fields
                        .get(&SchematicFieldKey::Other(filter.name.clone()))
                        .context("Unable to find primary field")?;

                    let column_name = field_type_to_sql_name(field.field_type);

                    writeln!(
                        &mut sql_building,
                        "json_tree(schema_data.{column_name}, '$.{filter_name}') as json_{filter_name}",
                        filter_name = filter.name,
                    )?;
                }

                writeln!(&mut sql_building, "WHERE addon_id = $1 AND schema_id = $2")?;

                let mut pos = 3;

                for filter in filters {
                    sql_building.push_str(" AND\n");

                    let field = schema
                        .fields
                        .get(&SchematicFieldKey::Other(filter.name.clone()))
                        .context("Unable to find primary field")?;

                    let column_name = field_type_to_sql_name(field.field_type);

                    let cond = match filter.cond {
                        // Text
                        FilterConditionType::Cont => "LIKE",
                        FilterConditionType::Dnc => "NOT LIKE",

                        FilterConditionType::Eq => "=",
                        FilterConditionType::Neq => "!=",
                        FilterConditionType::Gte => ">=",
                        FilterConditionType::Gt => ">",
                        FilterConditionType::Lte => "<=",
                        FilterConditionType::Lt => "<",

                        FilterConditionType::Between => "BETWEEN",
                    };

                    if filter.value.is_range() && filter.cond == FilterConditionType::Between {
                        writeln!(
                            &mut sql_building,
                            "({column_name} IS NOT NULL AND json_{filter_name}.value {cond} ${pos} AND ${pos2})",
                            filter_name = filter.name,
                            pos2 = pos + 1,
                        )?;

                        pos += 2;
                    } else {
                        writeln!(
                            &mut sql_building,
                            "({column_name} IS NOT NULL AND json_{filter_name}.value {cond} ${pos})",
                            filter_name = filter.name,
                        )?;

                        pos += 1;
                    }
                }

                let mut query = sqlx::query_scalar(&sql_building)
                    .bind(addon_id)
                    .bind(schema_id);

                for filter in filters {
                    if filter.value.is_range() && filter.cond == FilterConditionType::Between {
                        if let FilterValue::Range((min, max)) = filter.value {
                            query = query.bind(min.convert_f64()).bind(max.convert_f64());
                        }
                    } else if filter.cond == FilterConditionType::Cont
                        || filter.cond == FilterConditionType::Dnc
                    {
                        query = query.bind(format!("%{}%", filter.value));
                    } else {
                        query = query.bind(filter.value.to_string());
                    }
                }

                Ok(query.fetch_one(db).await?)
            }

            None => Ok(sqlx::query_scalar(
                "SELECT COUNT(id) FROM schema_data WHERE schema_id = $1",
            )
            .bind(schema_id)
            .fetch_one(db)
            .await?),
        }
    }

    // TODO: Query Data
    // SELECT schema_data.id, addon_id, schema_id, public_id, field_text, created_at, updated_at, deleted_at
    // FROM schema_data, json_tree(schema_data.field_text, '$.text')
    // WHERE addon_id = $1 AND schema_id = $2 AND field_text IS NOT NULL AND json_tree.value LIKE $3'%%'

    pub async fn delete(id: AddonId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("UPDATE schema_data SET deleted_at = $2 WHERE id = $1")
            .bind(id)
            .bind(OffsetDateTime::now_utc())
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn count_by_website_id(addon_id: AddonId, db: &mut SqliteConnection) -> Result<i32> {
        Ok(sqlx::query_scalar(
            "SELECT COUNT(*) FROM schema_data where addon_id = $1 AND deleted_at IS NULL",
        )
        .bind(addon_id)
        .fetch_one(db)
        .await?)
    }

    pub async fn count_by_schema_id(schema_id: SchemaId, db: &mut SqliteConnection) -> Result<i32> {
        Ok(sqlx::query_scalar(
            "SELECT COUNT(*) FROM schema_data where schema_id = $1 AND deleted_at IS NULL",
        )
        .bind(schema_id)
        .fetch_one(db)
        .await?)
    }

    pub async fn get_id_from_public_id(
        uuid: Uuid,
        db: &mut SqliteConnection,
    ) -> Result<Option<SchemaDataId>> {
        Ok(sqlx::query_as::<_, (SchemaDataId,)>(&format!(
            "SELECT id FROM schema_data WHERE public_id = $1"
        ))
        .bind(uuid)
        .fetch_optional(db)
        .await?
        .map(|v| v.0))
    }

    pub async fn get_ids_from_public_ids(
        uuids: Vec<Uuid>,
        db: &mut SqliteConnection,
    ) -> Result<Vec<SchemaDataId>> {
        Ok(sqlx::query_as::<_, (SchemaDataId,)>(&format!(
            "SELECT id FROM schema_data WHERE public_id IN ({})",
            uuids
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ))
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|v| v.0)
        .collect())
    }
}

impl SchemaDataFieldUpdate {
    pub async fn update(
        self,
        field_name: String,
        field_value: Option<SchematicFieldValue>,
        db: &mut SqliteConnection,
    ) -> Result<u64> {
        let sql_field_name = self.sql_field_name();

        let sql =
            format!("UPDATE schema_data SET updated_at = $2, {sql_field_name} = $3 WHERE id = $1");

        let query = sqlx::query(&sql)
            .bind(self.id)
            .bind(OffsetDateTime::now_utc());

        fn update_data<V, F: FnOnce(SchematicFieldValue) -> Result<Option<V>>>(
            field_name: String,
            field_value: Option<SchematicFieldValue>,
            data_value: &mut Option<HashMap<String, V>>,
            func: F,
        ) -> Result<()> {
            if data_value.is_none() && field_value.is_none() {
                // TODO: Change. We return an error if we're not updating anything.
                return Err(eyre::eyre!("None"))?;
            } else if data_value.is_none() {
                *data_value = Some(Default::default());
            }

            let val = data_value.as_mut().unwrap();

            if let Some(field_value) = field_value {
                if let Some(field_value) = func(field_value)? {
                    val.insert(field_name, field_value);
                } else {
                    return Err(eyre::eyre!("Incorrect Field Value Received"))?;
                }
            } else {
                val.remove(&field_name);
            }

            Ok(())
        }

        let query = match self.field {
            SchemaDataFieldUpdateType::Text(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Number(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Number(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Url(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Email(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Address(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Phone(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Bool(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Boolean(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::DateTime(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::DateTime(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Date(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Date(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Time(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Time(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::RichContent(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::RichText(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Reference(mut data_value) => {
                // TODO: Ensure this UUID exists in the referenced schema
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    // String of UUIDs
                    Ok(
                        if let SchematicFieldValue::Reference(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::MultiReference(mut data_value) => {
                // TODO: Ensure this UUID exists in the referenced schema
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    // String of UUIDs
                    Ok(
                        if let SchematicFieldValue::MultiReference(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Gallery(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    // String of UUIDs
                    Ok(
                        // TODO: May use SchematicFieldValue::MultiReference for this
                        if let SchematicFieldValue::ListString(field_value) = field_value {
                            Some(
                                field_value
                                    .into_iter()
                                    .map(|v| Uuid::parse_str(&v))
                                    .collect::<std::result::Result<_, _>>()?,
                            )
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Document(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    // String of UUIDs
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(Uuid::parse_str(&field_value)?)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::MultiDocument(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    // String of UUIDs
                    Ok(
                        if let SchematicFieldValue::ListString(field_value) = field_value {
                            Some(
                                field_value
                                    .into_iter()
                                    .map(|v| Uuid::parse_str(&v))
                                    .collect::<std::result::Result<_, _>>()?,
                            )
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Image(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    // String of UUIDs
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(Uuid::parse_str(&field_value)?)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Video(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    // String of UUIDs
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(Uuid::parse_str(&field_value)?)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Audio(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    // String of UUIDs
                    Ok(
                        if let SchematicFieldValue::Text(field_value) = field_value {
                            Some(Uuid::parse_str(&field_value)?)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Tags(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::ListNumber(field_value) = field_value {
                            Some(
                                field_value
                                    .into_iter()
                                    .map(|v| SchemaDataTagId::from(i64::from(v)))
                                    .collect(),
                            )
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Array(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Array(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
            SchemaDataFieldUpdateType::Object(mut data_value) => {
                update_data(field_name, field_value, &mut data_value, |field_value| {
                    Ok(
                        if let SchematicFieldValue::Object(field_value) = field_value {
                            Some(field_value)
                        } else {
                            None
                        },
                    )
                })?;

                query.bind(data_value.map(Json))
            }
        };

        let res = query.execute(db).await?;

        Ok(res.rows_affected())
    }

    pub fn sql_field_name(&self) -> &'static str {
        match &self.field {
            SchemaDataFieldUpdateType::Text(_) => "field_text",
            SchemaDataFieldUpdateType::Number(_) => "field_number",
            SchemaDataFieldUpdateType::Url(_) => "field_url",
            SchemaDataFieldUpdateType::Email(_) => "field_email",
            SchemaDataFieldUpdateType::Address(_) => "field_address",
            SchemaDataFieldUpdateType::Phone(_) => "field_phone",
            SchemaDataFieldUpdateType::Bool(_) => "field_bool",
            SchemaDataFieldUpdateType::DateTime(_) => "field_datetime",
            SchemaDataFieldUpdateType::Date(_) => "field_date",
            SchemaDataFieldUpdateType::Time(_) => "field_time",
            SchemaDataFieldUpdateType::RichContent(_) => "field_rich_content",
            SchemaDataFieldUpdateType::RichText(_) => "field_rich_text",
            SchemaDataFieldUpdateType::Reference(_) => "field_reference",
            SchemaDataFieldUpdateType::MultiReference(_) => "field_multi_reference",
            SchemaDataFieldUpdateType::Gallery(_) => "field_gallery",
            SchemaDataFieldUpdateType::Document(_) => "field_document",
            SchemaDataFieldUpdateType::MultiDocument(_) => "field_multi_document",
            SchemaDataFieldUpdateType::Image(_) => "field_image",
            SchemaDataFieldUpdateType::Video(_) => "field_video",
            SchemaDataFieldUpdateType::Audio(_) => "field_audio",
            SchemaDataFieldUpdateType::Tags(_) => "field_tags",
            SchemaDataFieldUpdateType::Array(_) => "field_array",
            SchemaDataFieldUpdateType::Object(_) => "field_object",
        }
    }

    pub async fn find_data_field_by_uuid(
        uuid: Uuid,
        field: SchematicFieldType,
        db: &mut SqliteConnection,
    ) -> Result<Option<SchemaDataFieldUpdate>> {
        let field_name = field_type_to_sql_name(field);

        let this: Option<SchemaDataModel> = sqlx::query_as(
            &format!("SELECT id, addon_id, schema_id, public_id, {field_name}, created_at, updated_at, deleted_at FROM schema_data WHERE public_id = $1"),
        )
        .bind(uuid)
        .fetch_optional(db)
        .await?;

        let Some(this) = this else {
            return Ok(None);
        };

        Ok(Some(SchemaDataFieldUpdate {
            id: this.id,
            addon_id: this.addon_id,
            schema_id: this.schema_id,
            public_id: this.public_id,
            field: match field {
                SchematicFieldType::Text => {
                    SchemaDataFieldUpdateType::Text(this.field_text.map(|v| v.0))
                }
                SchematicFieldType::Number => {
                    SchemaDataFieldUpdateType::Number(this.field_number.map(|v| v.0))
                }
                SchematicFieldType::URL => {
                    SchemaDataFieldUpdateType::Url(this.field_url.map(|v| v.0))
                }
                SchematicFieldType::Email => {
                    SchemaDataFieldUpdateType::Email(this.field_email.map(|v| v.0))
                }
                SchematicFieldType::Address => {
                    SchemaDataFieldUpdateType::Address(this.field_address.map(|v| v.0))
                }
                SchematicFieldType::Phone => {
                    SchemaDataFieldUpdateType::Phone(this.field_phone.map(|v| v.0))
                }
                SchematicFieldType::Boolean => {
                    SchemaDataFieldUpdateType::Bool(this.field_bool.map(|v| v.0))
                }
                SchematicFieldType::DateTime => {
                    SchemaDataFieldUpdateType::DateTime(this.field_datetime.map(|v| v.0))
                }
                SchematicFieldType::Date => {
                    SchemaDataFieldUpdateType::Date(this.field_date.map(|v| v.0))
                }
                SchematicFieldType::Time => {
                    SchemaDataFieldUpdateType::Time(this.field_time.map(|v| v.0))
                }
                SchematicFieldType::RichContent => {
                    SchemaDataFieldUpdateType::RichContent(this.field_rich_content.map(|v| v.0))
                }
                SchematicFieldType::RichText => {
                    SchemaDataFieldUpdateType::RichText(this.field_rich_text.map(|v| v.0))
                }
                SchematicFieldType::Reference => {
                    SchemaDataFieldUpdateType::Reference(this.field_reference.map(|v| v.0))
                }
                SchematicFieldType::MultiReference => SchemaDataFieldUpdateType::MultiReference(
                    this.field_multi_reference.map(|v| v.0),
                ),
                SchematicFieldType::MediaGallery => {
                    SchemaDataFieldUpdateType::Gallery(this.field_gallery.map(|v| v.0))
                }
                SchematicFieldType::Document => {
                    SchemaDataFieldUpdateType::Document(this.field_document.map(|v| v.0))
                }
                SchematicFieldType::MultiDocument => {
                    SchemaDataFieldUpdateType::MultiDocument(this.field_multi_document.map(|v| v.0))
                }
                SchematicFieldType::Image => {
                    SchemaDataFieldUpdateType::Image(this.field_image.map(|v| v.0))
                }
                SchematicFieldType::Video => {
                    SchemaDataFieldUpdateType::Video(this.field_video.map(|v| v.0))
                }
                SchematicFieldType::Audio => {
                    SchemaDataFieldUpdateType::Audio(this.field_audio.map(|v| v.0))
                }
                SchematicFieldType::Tags => {
                    SchemaDataFieldUpdateType::Tags(this.field_tags.map(|v| v.0))
                }
                SchematicFieldType::Array => {
                    SchemaDataFieldUpdateType::Array(this.field_array.map(|v| v.0))
                }
                SchematicFieldType::Object => {
                    SchemaDataFieldUpdateType::Object(this.field_object.map(|v| v.0))
                }
            },
            created_at: this.created_at,
            updated_at: this.updated_at,
            deleted_at: this.deleted_at,
        }))
    }
}

fn field_type_to_sql_name(value: SchematicFieldType) -> &'static str {
    match value {
        SchematicFieldType::Text => "field_text",
        SchematicFieldType::Number => "field_number",
        SchematicFieldType::URL => "field_url",
        SchematicFieldType::Email => "field_email",
        SchematicFieldType::Address => "field_address",
        SchematicFieldType::Phone => "field_phone",
        SchematicFieldType::Boolean => "field_bool",
        SchematicFieldType::DateTime => "field_datetime",
        SchematicFieldType::Date => "field_date",
        SchematicFieldType::Time => "field_time",
        SchematicFieldType::RichContent => "field_rich_content",
        SchematicFieldType::RichText => "field_rich_text",
        SchematicFieldType::Reference => "field_reference",
        SchematicFieldType::MultiReference => "field_multi_reference",
        SchematicFieldType::MediaGallery => "field_gallery",
        SchematicFieldType::Document => "field_document",
        SchematicFieldType::MultiDocument => "field_multi_document",
        SchematicFieldType::Image => "field_image",
        SchematicFieldType::Video => "field_video",
        SchematicFieldType::Audio => "field_audio",
        SchematicFieldType::Tags => "field_tags",
        SchematicFieldType::Array => "field_array",
        SchematicFieldType::Object => "field_object",
    }
}
