use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

use addon_common::{JsonListResponse, JsonResponse, ListResponse, WebsiteUuid, WrappingResponse};
use api::{schema::SchematicField, CmsCreateResponse};
use axum::{
    body::Body,
    extract::{self, multipart::Field, Json, Path, State},
    http::HeaderValue,
    response::{IntoResponse, Response},
    routing::{any, delete, get, post},
    Extension, Router,
};
use database::{
    AddonDashboardPage, AddonInstanceModel, AddonModel, AddonPermissionModel,
    AddonTemplatePageContentModel, AddonTemplatePageModel, MediaUploadModel, NewAddonMediaModel,
    NewAddonModel, NewMediaUploadModel, NewSchemaDataModel, NewSchemaModel, SchemaDataFieldUpdate,
    SchemaDataModel, SchemaDataTagModel, SchemaModel,
};
use eyre::{Context, ContextCompat};
use futures::TryStreamExt;
use global_common::{
    id::{AddonInstanceUuid, SchemaDataPublicId},
    request::{
        CmsCreate, CmsCreateDataColumn, CmsCreateDataColumnTag, CmsQuery, CmsUpdate,
        CmsUpdateDataCell,
    },
    response::{BasicCmsInfo, CmsResponse, CmsRowResponse, PublicSchema, SchemaTag},
    schema::{SchematicFieldKey, SchematicFieldType},
    uuid::CollectionName,
    value::SimpleValue,
};
use hyper::header::CONTENT_TYPE;
use lazy_static::lazy_static;
use local_common::{
    api::AddonPublic,
    generate::generate_file_name,
    upload::{
        get_full_file_path, get_next_uploading_file_path, get_thumb_file_path,
        read_and_upload_data, register_b2, StorageService,
    },
    AddonId, DashboardPageInfo, MemberId,
};
use mime_guess::mime::APPLICATION_JSON;
use serde::Deserialize;
use serde_qs::axum::QsQuery;
use sha2::{Digest, Sha256};
use sqlx::{Connection, Pool, Sqlite, SqliteConnection, SqlitePool};
use storage::DisplayStore;
use tokio::{fs::OpenOptions, io::AsyncWriteExt, net::TcpListener};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::Result;

mod addon;
mod vissl;
mod website;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn serve(pool: Pool<Sqlite>) -> Result<()> {
    let port = 5950;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    debug!("addons listening on {addr}");

    let uploader = register_b2().await;

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(
        listener,
        Router::new()
            // API Passthrough
            .route("/_api/:addon_id/*O", any(handle_api))
            .route("/list-active/:website", get(get_active_addon_list))
            .route("/dashboard-pages/:website", get(get_dashboard_pages))
            .route("/list", get(get_addon_list))
            // Update Addon Instance
            .route("/instance/:guid", post(post_addon_instance))
            // Addon
            .route("/addon", post(new_addon))
            .route("/addon/:guid", get(get_addon_public))
            // Get Website Addon Instance info
            .route("/addon/:guid/instance/:website", get(get_addon_instance))
            // Get dashboard page
            .route("/addon/:guid/dashboard/*O", get(get_addon_dashboard_page))
            .route("/addon/:guid/icon", post(upload_icon))
            .route("/addon/:guid/gallery", post(upload_gallery_item))
            .route("/addon/:guid/template/data", get(get_all_template_data))
            .route(
                "/addon/:guid/template/:template",
                get(get_template_page_data).post(update_template_page_data),
            )
            // Private
            .route("/addon/:guid/access/:user", get(get_addon_member_access))
            .route("/addon/:guid/schemas", get(get_addon_schemas))
            .route("/addon/:guid/schema/new", post(new_cms_collection))
            .route(
                "/addon/:guid/schema/:name",
                get(get_cms_info).post(update_cms),
            )
            .route("/addon/:guid/schema/:name/query", get(get_cms_query))
            .route(
                "/addon/:guid/schema/:name/column",
                post(create_new_data_column),
            )
            .route(
                "/addon/:guid/schema/:name/column/:col_id",
                delete(delete_data_column),
            )
            .route(
                "/addon/:guid/schema/:name/column/:col_id/tag",
                post(add_data_column_tag),
            )
            .route("/addon/:guid/schema/:name/row", post(create_new_data_row))
            .route("/addon/:guid/schema/:name/import", post(import_data_rows))
            .route(
                "/addon/:guid/schema/:name/row/:row_id",
                get(get_cms_row).post(update_cms_row_cell),
            )
            .route(
                "/addon/:guid/schema/:name/row/:row_id/duplicate",
                post(duplicate_cms_row_cell),
            )
            //
            .nest("/addon/:guid/vissl", vissl::routes())
            .nest("/website/:website_id", website::routes())
            .nest("/addon/:addon_id", addon::routes())
            .layer(TraceLayer::new_for_http())
            .layer(Extension(uploader.clone()))
            .with_state(pool),
    )
    .await?;

    Ok(())
}

async fn handle_api(
    Path((addon_id, rest)): Path<(Uuid, String)>,
    State(db): State<SqlitePool>,
    req: extract::Request<Body>,
) -> Result<impl IntoResponse> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon_id, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let Some(url) = addon.action_url else {
        return Err(eyre::eyre!("Addon Action URL not found"))?;
    };

    let uri = req.uri().clone();
    let method = req.method().clone();
    let headers = req.headers().clone();

    let mut buf = Vec::new();

    for by in req
        .into_body()
        .into_data_stream()
        .try_collect::<Vec<_>>()
        .await
        .unwrap()
    {
        buf.append(&mut by.to_vec());
    }

    let resp = CLIENT
        .request(
            method,
            format!(
                "{url}/{rest}{}",
                uri.query().map(|v| format!("?{v}")).unwrap_or_default()
            ),
        )
        .headers(headers)
        .body(buf)
        .send()
        .await?;

    let content_type = resp.headers().get(CONTENT_TYPE).cloned();

    // TODO: Ensure response is WrappingResponse at all times

    if resp.status().is_success() {
        Ok(Response::builder()
            .status(resp.status())
            .header(
                CONTENT_TYPE,
                content_type.unwrap_or_else(|| HeaderValue::from_static(APPLICATION_JSON.as_ref())),
            )
            .body(Body::from_stream(resp.bytes_stream()))
            .unwrap())
    } else {
        let status = resp.status();

        let text = resp.text().await?;

        Ok(Response::builder()
            .status(status)
            .body(Body::from(format!("Addon Specific Error Occurred: {text}")))
            .unwrap())
    }
}

async fn get_dashboard_pages(
    Path(website): Path<Uuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonListResponse<serde_json::Value>> {
    let active =
        AddonInstanceModel::find_by_website_uuid(website, &mut *db.acquire().await?).await?;

    let mut items = Vec::new();

    for instance in active {
        let addon = AddonModel::find_one_by_id(instance.addon_id, &mut *db.acquire().await?)
            .await?
            .unwrap();

        if addon.deleted_at.is_some() {
            continue;
        }

        let pages = AddonDashboardPage::find_by_id(addon.id, &mut *db.acquire().await?).await?;

        // TODO: Return if its' an SPA incl. a hash for the page incase we're using multiple SPA's so we know if we have to re-fetch the data.

        items.push(serde_json::json!({
            "name": addon.name,
            "icon": addon.icon,
            "guid": addon.guid,
            "rootPage": addon.root_dashboard_page,
            "pages": pages.into_iter().filter_map(|p| {
                if p.is_sidebar_visible {
                    Some(p.into())
                } else {
                    None
                }
            }).collect::<Vec<DashboardPageInfo>>(),
        }));
    }

    Ok(Json(WrappingResponse::okay(ListResponse::all(items))))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ActiveAddonsResponse {
    instance_guid: AddonInstanceUuid,
    instance_version: String,
    addon: AddonPublic,
}

async fn get_active_addon_list(
    Path(website): Path<WebsiteUuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonListResponse<ActiveAddonsResponse>> {
    Ok(Json(WrappingResponse::okay(ListResponse::all(
        query_active_addon_list(website, &mut *db.acquire().await?).await?,
    ))))
}

pub async fn query_active_addon_list(
    website: WebsiteUuid,
    db: &mut SqliteConnection,
) -> Result<Vec<ActiveAddonsResponse>> {
    let active = AddonInstanceModel::find_by_website_uuid(*website, db).await?;

    let mut items = Vec::new();

    for instance in active {
        let addon = AddonModel::find_one_by_id(instance.addon_id, db)
            .await?
            .context("Addon not found")?;

        items.push(ActiveAddonsResponse {
            instance_guid: instance.public_id,
            instance_version: instance.version,
            addon: addon.into_public(None, None, Vec::new()),
        });
    }

    Ok(items)
}

// TODO: Route: (User) Uninstall
// TODO: Route: (User) Resume Install
// TODO: Route: (Addon) Instance Install Complete

#[derive(Deserialize)]
struct Query {
    pub view: Option<String>,
    pub member: Option<Uuid>,
}

async fn get_addon_list(
    State(db): State<SqlitePool>,
    extract::Query(Query { view, member }): extract::Query<Query>,
) -> Result<Response> {
    match view.as_deref() {
        None | Some("simple") => {
            let addons = if let Some(member) = member {
                AddonModel::find_all_by_member(member, &mut *db.acquire().await?).await?
            } else {
                AddonModel::find_all(&mut *db.acquire().await?).await?
            };

            Ok(Json(WrappingResponse::okay(ListResponse::all(
                addons
                    .into_iter()
                    .map(|a| a.into_public(None, None, Vec::new()))
                    .collect(),
            )))
            .into_response())
        }

        Some("extended") => {
            // TODO: Extended Variant
            let addons = if let Some(member) = member {
                AddonModel::find_all_by_member(member, &mut *db.acquire().await?).await?
            } else {
                AddonModel::find_all(&mut *db.acquire().await?).await?
            };

            Ok(Json(WrappingResponse::okay(ListResponse::all(
                addons
                    .into_iter()
                    .map(|a| a.into_public(None, None, Vec::new()))
                    .collect(),
            )))
            .into_response())
        }

        _ => Ok(Json(WrappingResponse::okay(ListResponse::<()>::empty())).into_response()),
    }
}

async fn get_addon_instance(
    Path((addon_id, website_id)): Path<(Uuid, Uuid)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<serde_json::Value>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon_id, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let Some(inst) =
        AddonInstanceModel::find_by_addon_website_id(addon.id, website_id, &mut acq).await?
    else {
        return Err(eyre::eyre!("Addon Instance not found"))?;
    };

    Ok(Json(WrappingResponse::okay(serde_json::json!({
        "uuid": inst.public_id,
        "isSetup": inst.is_setup,
    }))))
}

#[derive(Deserialize)]
pub struct UpdateAddonInstance {
    pub version: Option<String>,
    pub settings: Option<serde_json::Value>,
}

async fn post_addon_instance(
    Path(instance_id): Path<Uuid>,
    State(db): State<SqlitePool>,
    Json(json): Json<UpdateAddonInstance>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let mut inst = AddonInstanceModel::find_by_uuid(instance_id, &mut acq)
        .await?
        .context("Addon Instance not found")?;

    if let Some(version) = json.version {
        inst.version = version;
    }

    if let Some(settings) = json.settings {
        inst.settings = Some(sqlx::types::Json(settings));
    }

    inst.update(&mut acq).await?;

    Ok(Json(WrappingResponse::okay("ok")))
}

async fn get_addon_public(
    Path(guid): Path<Uuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<AddonPublic>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let perms = AddonPermissionModel::find_by_addon_id(addon.id, &mut acq).await?;

    Ok(Json(WrappingResponse::okay(addon.into_public(
        None,
        None,
        perms.into_iter().map(|p| p.perm.to_string()).collect(),
    ))))
}

async fn get_addon_member_access(
    Path((addon, member)): Path<(Uuid, Uuid)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<bool>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    Ok(Json(WrappingResponse::okay(addon.member_uuid == member)))
}

async fn get_addon_dashboard_page(
    Path((guid, _path)): Path<(Uuid, String)>,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse> {
    let Some(_addon) = AddonModel::find_one_by_guid(guid, &mut *db.acquire().await?).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    // TODO: Upload SPA file instead
    let mut files = tokio::fs::read_dir("../addon-blog/dashboard/dist/assets").await?;

    let resp_builder = axum::response::Response::builder()
        .status(hyper::StatusCode::OK)
        .header(
            hyper::header::CONTENT_TYPE,
            mime_guess::mime::TEXT_PLAIN_UTF_8.as_ref(),
        );

    while let Some(entry) = files.next_entry().await? {
        let meta = entry.metadata().await?;

        if meta.is_file() && entry.file_name().to_string_lossy().ends_with(".js") {
            let contents = tokio::fs::read_to_string(entry.path()).await?;

            return Ok(resp_builder
                .header(
                    hyper::header::CONTENT_TYPE,
                    mime_guess::mime::TEXT_JAVASCRIPT.as_ref(),
                )
                .body(contents)
                .unwrap()
                .into_response());
        }
    }

    Ok(resp_builder.body(String::new()).unwrap().into_response())
}

#[derive(Deserialize)]
pub struct NewAddonJson {
    title: String,
    description: String,
    tagline: String,
    // tags: Vec<String>,
}

async fn new_addon(
    State(db): State<SqlitePool>,
    Json(NewAddonJson {
        title,
        description,
        tagline,
        // tags,
    }): Json<NewAddonJson>,
) -> Result<JsonResponse<AddonPublic>> {
    let addon = NewAddonModel {
        // TODO: Remove
        member_id: MemberId::from(1),
        member_uuid: Uuid::from_bytes([
            0x2c, 0x5e, 0xa4, 0xc0, 0x40, 0x67, 0x11, 0xe9, 0x8b, 0x2d, 0x1b, 0x9d, 0x6b, 0xcd,
            0xbb, 0xfd,
        ]),
        // TODO: Only keep A-Z 0-9 _
        name_id: title.to_lowercase(),
        name: title,
        tag_line: tagline,
        description,
        icon: None,
        version: String::new(),
        action_url: None,
        root_dashboard_page: None,
    }
    .insert(&mut *db.acquire().await?)
    .await?;

    //

    Ok(Json(WrappingResponse::okay(addon.into_public(
        None,
        None,
        Vec::new(),
    ))))
}

async fn upload_icon(
    Path(guid): Path<Uuid>,
    State(db): State<SqlitePool>,
    storage: StorageService,
    mut multipart: extract::Multipart,
) -> Result<JsonResponse<Option<&'static str>>> {
    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut *db.acquire().await?).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    if let Some(field) = multipart.next_field().await? {
        if let Some(model) =
            upload_file(field, addon.member_id, Some((200, 200)), &storage, &db).await?
        {
            NewAddonMediaModel::Upload {
                addon_id: addon.id,
                upload_id: model.id,
            }
            .insert(&mut *db.acquire().await?)
            .await?;

            return Ok(Json(WrappingResponse::okay(Some("ok"))));
        }
    }

    Ok(Json(WrappingResponse::okay(None)))
}

async fn upload_gallery_item(
    Path(guid): Path<Uuid>,
    State(db): State<SqlitePool>,
    storage: StorageService,
    mut multipart: extract::Multipart,
) -> Result<JsonResponse<&'static str>> {
    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut *db.acquire().await?).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let mut models = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if let Some(model) = upload_file(field, addon.member_id, None, &storage, &db).await? {
            let model = NewAddonMediaModel::Upload {
                addon_id: addon.id,
                upload_id: model.id,
            }
            .insert(&mut *db.acquire().await?)
            .await?;

            models.push(model);
        }
    }

    Ok(Json(WrappingResponse::okay("ok")))
}

async fn upload_file(
    mut field: Field<'_>,
    uploader_id: MemberId,
    set_dimensions: Option<(u32, u32)>,
    storage: &StorageService,
    db: &Pool<Sqlite>,
) -> Result<Option<MediaUploadModel>> {
    const MAX_MB_UPLOAD_SIZE: usize = 10;

    let Some(file_name) = field.file_name() else {
        return Err(eyre::eyre!("No file name provided"))?;
    };

    // TODO: Figure out the file type
    if !file_name.contains('.') {
        return Err(eyre::eyre!("No file extension provided"))?;
    }

    // TODO: Better file type detection. Ex: .tar.gz
    let (_, file_type_s) = file_name.rsplit_once('.').unwrap();

    let _meme = mime_guess::from_ext(file_type_s).first_or_text_plain();

    let file_name = file_name.to_string();

    let upload_path = get_next_uploading_file_path();

    // TODO: Later on we'll just want to stream it directly to the storage server
    let mut uploading_file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .read(true)
        .create(true)
        .open(&upload_path)
        .await?;

    let (_file_size, _original_hash) = {
        let mut file_size = 0;
        let mut sha = Sha256::new();

        while let Some(chunk) = field.try_next().await? {
            file_size += chunk.len();
            sha.update(&chunk);
            uploading_file.write_all(&chunk).await?;

            if file_size > 1024 * 1024 * MAX_MB_UPLOAD_SIZE {
                return Err(eyre::eyre!(
                    "File too large MAX Size: {MAX_MB_UPLOAD_SIZE}MB"
                ))?;
            }
        }

        (file_size as i64, format!("{:X}", sha.finalize()))
    };

    let store_path = generate_file_name();

    let mut upload = NewMediaUploadModel::pending(uploader_id, store_path.clone())
        .insert(&mut *db.acquire().await?)
        .await?;

    match read_and_upload_data(
        &store_path,
        file_name,
        upload_path,
        set_dimensions,
        uploading_file,
        storage,
    )
    .await
    {
        Ok(resp) => {
            upload.file_name = resp.file_name;
            upload.file_size = resp.file_size;
            upload.file_type = resp.file_type;
            upload.media_height = resp.media_height;
            upload.media_width = resp.media_width;
            upload.has_thumbnail = resp.has_thumbnail;
            upload.hash = Some(resp.hash);

            upload.update(&mut *db.acquire().await?).await?;

            Ok(Some(upload))
        }

        Err(e) => {
            error!("{e}");

            let full_file_path = get_full_file_path(&store_path);
            let thumb_file_path = get_thumb_file_path(&store_path);

            let _ = storage.hide_file(full_file_path).await;
            let _ = storage.hide_file(thumb_file_path).await;

            Ok(None)
        }
    }
}

//

async fn get_template_page_data(
    Path((addon_id, template_id)): Path<(Uuid, Uuid)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<serde_json::Value>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon_id, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let Some(addon_page) = AddonTemplatePageModel::find_by_public_id(template_id, &mut acq).await?
    else {
        return Err(eyre::eyre!("Addon page not found"))?;
    };

    if addon_page.addon_id != addon.id {
        return Err(eyre::eyre!("Addon page is not valid"))?;
    }

    let page_content = AddonTemplatePageContentModel::find_one_by_page_id(addon_page.id, &mut acq)
        .await?
        .unwrap();

    Ok(Json(WrappingResponse::okay(serde_json::json!({
        "id": addon_page.id,
        "publicId": addon_page.public_id,
        "name": addon_page.display_name,
        "path": addon_page.path,
        "data": page_content.content.0,
    }))))
}

async fn update_template_page_data(
    Path((addon_id, template_id)): Path<(Uuid, Uuid)>,
    State(db): State<SqlitePool>,
    Json(mut page): Json<DisplayStore>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon_id, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let Some(mut addon_page) =
        AddonTemplatePageModel::find_by_public_id(template_id, &mut acq).await?
    else {
        return Err(eyre::eyre!("Addon page not found"))?;
    };

    if addon_page.addon_id != addon.id {
        return Err(eyre::eyre!("Addon page is not valid"))?;
    }

    // Remove unused Data
    let ids = page
        .get_object_ids()
        .into_iter()
        .map(|v| v.guid)
        .collect::<Vec<_>>();

    page.set_data(
        page.data()
            .iter()
            .filter_map(|(key, v)| {
                if key.is_website() || ids.contains(key) {
                    Some((*key, v.clone()))
                } else {
                    None
                }
            })
            .collect(),
    );

    // Add Object Ids to Website Page
    addon_page.object_ids =
        sqlx::types::Json(page.get_object_ids().into_iter().map(|v| v.id).collect());

    // TODO: Update only changed
    addon_page.update(&mut acq).await?;

    AddonTemplatePageContentModel::new(addon_page.id, page)
        .update(&mut acq)
        .await?;

    Ok(Json(WrappingResponse::okay("ok")))
}

// ADDON

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonPageWithDataItem {
    pub public_id: Uuid,

    pub path: String,
    pub display_name: String,

    pub settings: api::WebsitePageSettings,

    pub content: DisplayStore,
    pub version: i32,
}

async fn get_all_template_data(
    Path(addon_id): Path<Uuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonListResponse<AddonPageWithDataItem>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon_id, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let list = AddonTemplatePageModel::find_by_addon_id(addon.id, &mut acq).await?;

    let mut items = Vec::new();

    for model in list {
        let Some(content) =
            AddonTemplatePageContentModel::find_one_by_page_id(model.id, &mut acq).await?
        else {
            panic!("Unable to find Page Content");
        };

        items.push(AddonPageWithDataItem {
            public_id: model.public_id,
            path: model.path,
            display_name: model.display_name,
            settings: model.settings.0,
            content: content.content.0,
            version: content.version,
        });
    }

    Ok(Json(WrappingResponse::okay(ListResponse::all(items))))
}

// TODO: From Main Program request addon schemas - remember if the schema is already in main program db then use main one.

async fn get_addon_schemas(
    Path(addon): Path<Uuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonListResponse<BasicCmsInfo>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let schemas = SchemaModel::find_by_addon_id(addon.id, &mut acq).await?;

    Ok(Json(WrappingResponse::okay(ListResponse::all(
        schemas
            .into_iter()
            .map(|schema| BasicCmsInfo {
                id: schema.name,
                name: schema.display_name,
                namespace: Some(format!("@{}", addon.name_id)),
                is_single: false,
            })
            .collect(),
    ))))
}

pub async fn new_cms_collection(
    Path(addon_id): Path<Uuid>,
    State(db): State<SqlitePool>,

    Json(CmsCreate {
        id: coll,
        name,
        // TODO
        is_single,
        update,
        columns,
        data,
    }): Json<CmsCreate>,
) -> Result<JsonResponse<CmsCreateResponse>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    // TODO: Id replace invalids
    // .replace(/[^a-zA-Z0-9_\s]/g, "")
    // .replace(/(?:^\w|[A-Z]|\b\w)/g, function (word, index) {
    //     return index === 0 ? word.toLowerCase() : word.toUpperCase();
    // })
    // .replace(/\s+/g, "")
    // .slice(0, 32);

    // TODO: What about Namespace?

    if coll.id.trim().len() < 2
        || name.trim().len() < 2
        || coll.id.contains('-')
        || coll.id.contains('/')
        || name.contains('/')
    {
        return Err(eyre::eyre!("Invalid Characters present"))?;
    }

    if SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .is_some()
    {
        return Err(eyre::eyre!("Schema ID already Exists"))?;
    }

    let (schema, data_ids) = acq
        .transaction(|trx| {
            Box::pin(async move {
                let fields = HashMap::from_iter([
                    (
                        SchematicFieldKey::Id,
                        SchematicField {
                            display_name: String::from("ID"),
                            sortable: true,
                            is_deleted: false,
                            system_field: true,
                            field_type: SchematicFieldType::Text,
                            index: 0,
                            referenced_schema: None,
                        },
                    ),
                    (
                        SchematicFieldKey::Owner,
                        SchematicField {
                            display_name: String::from("Owner"),
                            sortable: true,
                            is_deleted: false,
                            system_field: true,
                            field_type: SchematicFieldType::Text,
                            index: 1,
                            referenced_schema: None,
                        },
                    ),
                    (
                        SchematicFieldKey::CreatedAt,
                        SchematicField {
                            display_name: String::from("Created Date"),
                            sortable: true,
                            is_deleted: false,
                            system_field: true,
                            field_type: SchematicFieldType::DateTime,
                            index: 2,
                            referenced_schema: None,
                        },
                    ),
                    (
                        SchematicFieldKey::UpdatedAt,
                        SchematicField {
                            display_name: String::from("Updated Date"),
                            sortable: true,
                            is_deleted: false,
                            system_field: true,
                            field_type: SchematicFieldType::DateTime,
                            index: 3,
                            referenced_schema: None,
                        },
                    ),
                ]);

                let mut schema = NewSchemaModel {
                    addon_id: addon.id,
                    primary_field: String::from(SchematicFieldKey::CreatedAt.as_str()),
                    display_name: name.trim().to_string(),
                    permissions: Default::default(),
                    version: 1.0,
                    allowed_operations: Vec::new(),
                    ttl: None,
                    default_sort: None,
                    name: coll.id,
                    store: String::from("cms"),
                    fields,
                    views: update.views.unwrap_or_else(|| vec![Default::default()]),
                }
                .insert(trx)
                .await?;

                if let Some(columns) = columns {
                    for column in columns {
                        insert_columns(column, &mut schema)?;
                    }

                    schema.update(trx).await?;
                }

                let data_ids = if let Some(data) = data {
                    Some(insert_rows(data, addon.id, &schema, trx).await?)
                } else {
                    None
                };

                Result::<_, crate::Error>::Ok((schema, data_ids))
            })
        })
        .await?;

    Ok(Json(WrappingResponse::okay(CmsCreateResponse {
        id: schema.name,
        name: schema.display_name,
        namespace: Some(addon.tag_line),
        data_ids,
    })))
}

pub async fn get_cms_info(
    Path((addon_id, coll)): Path<(Uuid, CollectionName)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<CmsResponse>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    let tags = SchemaDataTagModel::get_all(schema.id, &mut acq).await?;

    Ok(Json(WrappingResponse::okay(CmsResponse {
        form_id: None,

        collection: PublicSchema {
            schema_id: schema.name,
            namespace: Some(format!("@{}", addon.name_id)),
            primary_field: schema.primary_field,
            display_name: schema.display_name,
            permissions: schema.permissions.0,
            version: schema.version as f32,
            allowed_operations: schema.allowed_operations.0,
            is_single: false,
            fields: schema.fields.0,
            ttl: schema.ttl,
            default_sort: schema.default_sort,
            views: schema.views.0,
            created_at: schema.created_at,
            updated_at: schema.updated_at,
            deleted_at: schema.deleted_at,
        },
        tags: tags
            .into_iter()
            .map(|t| SchemaTag {
                id: *t.id,
                row_id: t.row_id,
                name: t.name,
                color: t.color,
            })
            .collect(),
    })))
}

pub async fn update_cms(
    Path((addon_id, coll)): Path<(Uuid, CollectionName)>,
    State(db): State<SqlitePool>,

    Json(CmsUpdate { views }): Json<CmsUpdate>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let mut schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    if let Some(views) = views {
        schema.views.0 = views;
    }

    schema.update(&mut acq).await?;

    Ok(Json(WrappingResponse::okay("ok")))
}

// TODO: Instead of addon id use instance id ??
// We need to not only return an instances' cms but also default values
pub async fn get_cms_query(
    Path((addon_id, coll)): Path<(Uuid, CollectionName)>,
    QsQuery(CmsQuery {
        filters,
        sort,
        columns,
        limit,
        offset,
        include_files,
    }): QsQuery<CmsQuery>,
    State(db): State<SqlitePool>,
) -> Result<JsonListResponse<CmsRowResponse>> {
    let mut acq = db.acquire().await?;

    let addon = if addon_id.is_nil() && coll.ns.is_some() {
        match AddonModel::find_one_by_name_id(coll.ns.as_deref().unwrap(), &mut acq).await? {
            Some(v) => v,
            None => {
                return Err(eyre::eyre!("Addon not found"))?;
            }
        }
    } else {
        match AddonModel::find_one_by_guid(addon_id, &mut acq).await? {
            Some(v) => v,
            None => {
                return Err(eyre::eyre!("Addon not found"))?;
            }
        }
    };

    // addon.no_access_error(member.id(), &mut *acq).await?;

    let schema = match SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq).await? {
        Some(v) => v,
        None => {
            // TODO: If coll.ns exists and SchemaModel isn't found search Query Addons Program
            return Err(eyre::eyre!("Schema not found"))?;
        }
    };

    let offset = offset.unwrap_or(0) as i64;
    let limit = limit.unwrap_or(50).max(20) as i64;

    if schema.store == "addon" {
        let Some(url) = addon.action_url else {
            return Err(eyre::eyre!("Addon Action URL not found"))?;
        };

        let resp = CLIENT
            .get(format!("{url}/cms/{}/query", Uuid::nil()))
            .send()
            .await?;

        if resp.status().is_success() {
            let resp: WrappingResponse<ListResponse<HashMap<SchematicFieldKey, SimpleValue>>> =
                resp.json().await?;

            match resp {
                WrappingResponse::Resp(resp) => {
                    // TODO: Maybe we shouldn't pass it directly to the client.

                    // for item in &resp.items {
                    //     validate_item(&schema.fields, item)?;
                    // }

                    Ok(Json(WrappingResponse::okay(ListResponse {
                        offset: resp.offset,
                        limit: resp.limit,
                        total: resp.total,
                        items: resp
                            .items
                            .into_iter()
                            .map(|fields| CmsRowResponse {
                                files: Vec::new(),
                                fields,
                            })
                            .collect(),
                    })))
                }

                WrappingResponse::Error(e) => Ok(Json(WrappingResponse::Error(e))),
            }
        } else {
            Ok(Json(resp.json().await?))
        }
    } else {
        let total =
            SchemaDataModel::count_by(addon.id, &schema, filters.as_deref(), &mut acq).await?;

        let data = SchemaDataModel::find_by(
            addon.id,
            &schema,
            filters.as_deref(),
            sort,
            offset,
            limit,
            &mut acq,
        )
        .await?;

        let columns =
            columns.map(|columns| HashSet::from_iter(columns.split(',').map(|v| v.to_string())));

        let mut items = Vec::new();

        {
            for model in data {
                let mut uuids = Vec::new();

                if let Some(value) = model.field_audio.as_ref() {
                    uuids.append(&mut value.values().copied().collect());
                }

                if let Some(value) = model.field_document.as_ref() {
                    uuids.append(&mut value.values().copied().collect());
                }

                if let Some(value) = model.field_image.as_ref() {
                    uuids.append(&mut value.values().copied().collect());
                }

                if let Some(value) = model.field_video.as_ref() {
                    uuids.append(&mut value.values().copied().collect());
                }

                if let Some(value) = model.field_multi_document.as_ref() {
                    uuids.append(&mut value.values().flatten().copied().collect());
                }

                uuids.sort_unstable();
                uuids.dedup();

                let fields = map_to_field_value(&schema, model, columns.as_ref())?;

                // let mut files = Vec::new();
                //
                // if include_files {
                //     for uuid in uuids {
                //         if let Some(upload_id) =
                //             WebsiteUploadLink::find_one_by_public_id(&uuid.to_string(), &mut *acq)
                //                 .await?
                //                 .and_then(|v| v.upload_id)
                //         {
                //             if let Some(item) =
                //                 MemberUploadModel::find_one_by_id(upload_id, &mut *acq).await?
                //             {
                //                 // Replace public id w/ Field ID as to not expose things.
                //                 files.push(WebsiteUpload {
                //                     id: Some(item.id),
                //                     public_id: uuid.to_string(),
                //                     upload_type: String::from("media"),
                //                     display_name: item.file_name,
                //                     created_at: item.created_at,
                //                     deleted_at: None,
                //                     media: Some(WebsiteUploadFile {
                //                         file_size: item.file_size,
                //                         file_type: item.file_type,
                //                         media_width: item.media_width,
                //                         media_height: item.media_height,
                //                         media_duration: item.media_duration,
                //                         is_editable: item.is_editable,
                //                         has_thumbnail: item.has_thumbnail,
                //                         is_global: item.is_global,
                //                     }),
                //                     using_variant: None,
                //                 });
                //             }
                //         }
                //     }
                // }

                items.push(CmsRowResponse {
                    files: Vec::new(),
                    fields,
                });
            }
        }

        Ok(Json(WrappingResponse::okay(ListResponse {
            offset,
            limit,
            total,
            items,
        })))
    }
}

// Column

pub async fn create_new_data_column(
    Path((addon_id, coll)): Path<(Uuid, CollectionName)>,
    State(db): State<SqlitePool>,

    Json(create_data): Json<CmsCreateDataColumn>,
) -> Result<JsonResponse<SchematicField>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let mut schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    // TODO: Id replace invalids
    // .replace(/[^a-zA-Z0-9_\s]/g, "")
    // .replace(/^_+/g, "")
    // .replace(/(?:^\w|[A-Z]|\b\w)/g, function (word, index) {
    //     return index === 0 ? word.toLowerCase() : word.toUpperCase();
    // })
    // .replace(/\s+/g, "")
    // .slice(0, 32);

    let field = insert_columns(create_data, &mut schema)?;

    schema.update(&mut acq).await?;

    Ok(Json(WrappingResponse::okay(field)))
}

fn insert_columns(
    CmsCreateDataColumn {
        id,
        name,
        type_of,
        referenced_schema,
    }: CmsCreateDataColumn,
    schema: &mut SchemaModel,
) -> Result<SchematicField> {
    let key = SchematicFieldKey::Other(id.trim().to_string());

    if schema.fields.iter().filter(|(_, v)| !v.is_deleted).count() >= 20 {
        Err(eyre::eyre!("Too many columns"))?;
    }

    if schema.fields.len() >= 100 {
        Err(eyre::eyre!("Too many columns created and deleted"))?;
    }

    if let Some(field) = schema.fields.get(&key) {
        if field.is_deleted {
            Err(eyre::eyre!(
                "Cannot create a new schema from a previously used ID"
            ))?;
        } else {
            Err(eyre::eyre!("Column ID Already Exists"))?;
        }
    }

    // Check for missing field_type values
    if (type_of == SchematicFieldType::Reference || type_of == SchematicFieldType::MultiReference)
        && referenced_schema.is_none()
    {
        Err(eyre::eyre!("Reference is missing the schema"))?;
    }

    let len = schema.fields.len();

    let field = SchematicField {
        display_name: name,
        sortable: true,
        is_deleted: false,
        system_field: false,
        field_type: type_of,
        index: len as u16,
        referenced_schema,
    };

    schema.fields.insert(key, field.clone());

    Ok(field)
}

pub async fn add_data_column_tag(
    Path((addon_id, coll, column_id)): Path<(Uuid, CollectionName, String)>,
    State(db): State<SqlitePool>,

    Json(CmsCreateDataColumnTag { tag }): Json<CmsCreateDataColumnTag>,
) -> Result<JsonResponse<api::SchemaTag>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let mut schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    let key = SchematicFieldKey::Other(column_id);

    if let Some(field) = schema.fields.get(&key) {
        if field.field_type == SchematicFieldType::Tags {
            let tag = SchemaDataTagModel::insert(
                schema.id,
                key.to_string(),
                tag.trim().to_string(),
                String::from("#AFA"),
                &mut acq,
            )
            .await?;

            schema.update(&mut acq).await?;

            Ok(Json(WrappingResponse::okay(api::SchemaTag {
                id: *tag.id as i64,
                row_id: tag.row_id,
                name: tag.name,
                color: tag.color,
            })))
        } else {
            Err(eyre::eyre!("Schema field incorrect"))?
        }
    } else {
        Err(eyre::eyre!("Schema field not found"))?
    }
}

pub async fn delete_data_column(
    Path((addon_id, coll, column_id)): Path<(Uuid, CollectionName, String)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let mut schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    let key = SchematicFieldKey::Other(column_id);

    if let Some(field) = schema.fields.get_mut(&key) {
        field.is_deleted = true;
    } else {
        return Err(eyre::eyre!("Schema field not found"))?;
    }

    schema.update(&mut acq).await?;

    Ok(Json(WrappingResponse::okay("ok")))
}

// ROW

pub async fn get_cms_row(
    Path((addon_id, coll, row_id)): Path<(Uuid, CollectionName, Uuid)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<api::CmsRowResponse>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let schema: SchemaModel = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    // TODO: add schema.id to find
    let Some(schema_data) = SchemaDataModel::find_by_public_id(row_id, &mut acq).await? else {
        return Err(eyre::eyre!("Schema Data not found"))?;
    };

    let mut uuids: Vec<Uuid> = Vec::new();

    if let Some(value) = schema_data.field_audio.as_ref() {
        uuids.append(&mut value.values().copied().collect());
    }

    if let Some(value) = schema_data.field_document.as_ref() {
        uuids.append(&mut value.values().copied().collect());
    }

    if let Some(value) = schema_data.field_image.as_ref() {
        uuids.append(&mut value.values().copied().collect());
    }

    if let Some(value) = schema_data.field_video.as_ref() {
        uuids.append(&mut value.values().copied().collect());
    }

    if let Some(value) = schema_data.field_multi_document.as_ref() {
        uuids.append(&mut value.values().flatten().copied().collect());
    }

    uuids.sort_unstable();
    uuids.dedup();

    let fields = map_to_field_value(&schema, schema_data, None)?;

    let files = Vec::new();

    // TODO: Send request to main program to return a list of uploads for the given UUIDs

    // if let Some(upload_id) =
    //     WebsiteUploadLink::find_one_by_public_id(&uuid.to_string(), &mut *acq)
    //         .await?
    //         .and_then(|v| v.upload_id)
    // {
    //     if let Some(item) = MemberUploadModel::find_one_by_id(upload_id, &mut *acq).await? {
    //         // Replace public id w/ Field ID as to not expose things.
    //         files.push(WebsiteUpload {
    //             namespace: None,
    //             public_id: uuid.to_string(),
    //             upload_type: String::from("media"),
    //             display_name: item.file_name,
    //             created_at: item.created_at,
    //             deleted_at: None,
    //             media: Some(WebsiteUploadFile {
    //                 file_size: item.file_size,
    //                 file_type: item.file_type,
    //                 media_width: item.media_width,
    //                 media_height: item.media_height,
    //                 media_duration: item.media_duration,
    //                 is_editable: item.is_editable,
    //                 has_thumbnail: item.has_thumbnail,
    //                 is_global: item.is_global,
    //             }),
    //             using_variant: None,
    //         });
    //     }
    // }

    Ok(Json(WrappingResponse::okay(api::CmsRowResponse {
        files,
        fields,
    })))
}

pub async fn update_cms_row_cell(
    Path((addon_id, coll, row_id)): Path<(Uuid, CollectionName, Uuid)>,
    State(db): State<SqlitePool>,

    Json(CmsUpdateDataCell { field_name, value }): Json<CmsUpdateDataCell>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    let Some(schema_field) = schema
        .fields
        .get(&SchematicFieldKey::Other(field_name.clone()))
    else {
        return Err(eyre::eyre!("Schema Field not found"))?;
    };

    let Some(schema_data) =
        SchemaDataFieldUpdate::find_data_field_by_uuid(row_id, schema_field.field_type, &mut acq)
            .await?
    else {
        return Err(eyre::eyre!("Schema Data not found"))?;
    };

    schema_data
        .update(
            field_name,
            value
                .map(|v| schema_field.field_type.parse_value(v))
                .transpose()?,
            &mut acq,
        )
        .await?;

    Ok(Json(WrappingResponse::okay("ok")))
}

pub async fn create_new_data_row(
    Path((addon_id, coll)): Path<(Uuid, CollectionName)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<api::CmsRowResponse>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    let data_row = NewSchemaDataModel::new(addon.id, schema.id)
        .insert(&mut acq)
        .await?;

    Ok(Json(WrappingResponse::okay(api::CmsRowResponse {
        files: Vec::new(),
        fields: map_to_field_value(&schema, data_row, None)?,
    })))
}

pub async fn import_data_rows(
    Path((addon_id, coll)): Path<(Uuid, CollectionName)>,
    State(db): State<SqlitePool>,

    Json(map): Json<HashMap<String, Vec<SimpleValue>>>,
) -> Result<JsonResponse<&'static str>> {
    // TODO: Receive full file to somehow hash each line to have a unique import_id to prevent duplication?

    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    acq.transaction(|trx| {
        Box::pin(async move {
            insert_rows(map, addon.id, &schema, trx).await?;

            Result::<_, crate::Error>::Ok(())
        })
    })
    .await?;

    Ok(Json(WrappingResponse::okay("ok")))
}

async fn insert_rows(
    data: HashMap<String, Vec<SimpleValue>>,
    addon_id: AddonId,
    schema: &SchemaModel,
    db: &mut SqliteConnection,
) -> Result<Vec<SchemaDataPublicId>> {
    let mut inserting_rows = data
        .values()
        .next()
        .unwrap()
        .iter()
        .map(|_| NewSchemaDataModel::new(addon_id, schema.id))
        .collect::<Vec<_>>();

    for (key, mut value) in data {
        let key = SchematicFieldKey::Other(key);

        if let Some(field) = schema.fields.get(&key) {
            // TODO: Incorporate into insert_field/parse_value
            if field.field_type == SchematicFieldType::Tags {
                let mut adding_tags: HashMap<String, i64> = HashMap::new();

                for val in value.iter_mut() {
                    if let SimpleValue::Text(text_list) = val.clone() {
                        // Nothing in Text?
                        if text_list.trim().is_empty() {
                            *val = SimpleValue::ListNumber(Vec::new());
                            continue;
                        }

                        let mut items = Vec::new();

                        for text in text_list.split(",") {
                            let trimmed = text.trim();

                            // We don't want to call the DB for no reason
                            if let Some(found) = adding_tags.get(&trimmed.to_lowercase()).copied() {
                                items.push(found.into());
                            } else {
                                let model = SchemaDataTagModel::insert(
                                    schema.id,
                                    key.to_string(),
                                    trimmed.to_string(),
                                    String::from("#FAF"),
                                    db,
                                )
                                .await?;

                                adding_tags.insert(model.name.to_lowercase(), *model.id);

                                items.push((*model.id as i32).into());
                            }
                        }

                        *val = SimpleValue::ListNumber(items);
                    } else {
                        warn!("Import: Expected Text, Found {val:?}");
                    }
                }
            }

            for (i, value) in value.into_iter().enumerate() {
                inserting_rows[i].insert_field(
                    key.to_string(),
                    false,
                    field.field_type,
                    field.field_type.parse_value(value).wrap_err_with(|| {
                        format!("Parse Value into Type: {:?}", field.field_type)
                    })?,
                )?;
            }
        }
    }

    let mut inserted = Vec::new();

    for row in inserting_rows {
        let model = row.insert(db).await?;
        inserted.push(SchemaDataPublicId::from(model.public_id));
    }

    Ok(inserted)
}

pub async fn duplicate_cms_row_cell(
    Path((addon_id, coll, row_id)): Path<(Uuid, CollectionName, Uuid)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<api::CmsRowResponse>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let schema = SchemaModel::find_one_by_public_id(addon.id, &coll.id, &mut acq)
        .await?
        .context("Schema not found")?;

    let schema_data = SchemaDataModel::find_by_public_id(row_id, &mut acq)
        .await?
        .context("Schema Data not found")?
        .into_new()
        .insert(&mut acq)
        .await?;

    Ok(Json(WrappingResponse::okay(api::CmsRowResponse {
        files: Vec::new(),
        fields: map_to_field_value(&schema, schema_data, None)?,
    })))
}

fn map_to_field_value(
    schema: &SchemaModel,
    mut model: SchemaDataModel,
    columns: Option<&HashSet<String>>,
) -> Result<HashMap<SchematicFieldKey, SimpleValue>> {
    let mut map = HashMap::new();

    let mut unable_to_find = schema.fields.0.keys().cloned().collect::<Vec<_>>();

    for (key, field) in &schema.fields.0 {
        // System Field Names
        match key {
            SchematicFieldKey::Id => {
                map.insert(key.clone(), SimpleValue::Text(model.public_id.to_string()));
                continue;
            }
            SchematicFieldKey::Owner => {
                map.insert(key.clone(), SimpleValue::Text(Uuid::nil().to_string()));
                continue;
            }
            SchematicFieldKey::CreatedAt => {
                map.insert(key.clone(), SimpleValue::DateTime(model.created_at));
                continue;
            }
            SchematicFieldKey::UpdatedAt => {
                map.insert(key.clone(), SimpleValue::DateTime(model.updated_at));
                continue;
            }
            _ => (),
        }

        // If columns is set and its' not in there, continue
        if let Some(columns) = columns {
            if !columns.contains(key.as_str()) {
                continue;
            }
        }

        // Custom field names
        map.insert(
            key.clone(),
            match field.field_type {
                SchematicFieldType::Text => {
                    let Some(field) = model.field_text.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value)
                }
                SchematicFieldType::Number => {
                    let Some(field) = model.field_number.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Number(value)
                }
                SchematicFieldType::URL => {
                    let Some(field) = model.field_url.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value)
                }
                SchematicFieldType::Email => {
                    let Some(field) = model.field_email.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value)
                }
                SchematicFieldType::Address => {
                    let Some(field) = model.field_address.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value)
                }
                SchematicFieldType::Phone => {
                    let Some(field) = model.field_phone.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value)
                }
                SchematicFieldType::Boolean => {
                    let Some(field) = model.field_bool.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Boolean(value)
                }
                SchematicFieldType::DateTime => {
                    let Some(field) = model.field_datetime.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::DateTime(value)
                }
                SchematicFieldType::Date => {
                    let Some(field) = model.field_date.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Date(value)
                }
                SchematicFieldType::Time => {
                    let Some(field) = model.field_time.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Time(value)
                }
                SchematicFieldType::RichContent => {
                    let Some(field) = model.field_rich_content.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value)
                }
                SchematicFieldType::RichText => {
                    let Some(field) = model.field_rich_text.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value)
                }
                SchematicFieldType::Reference => {
                    let Some(field) = model.field_reference.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value.to_string())
                }
                SchematicFieldType::MultiReference => {
                    let Some(field) = model.field_multi_reference.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::ListString(value.into_iter().map(|v| v.to_string()).collect())
                }
                SchematicFieldType::MediaGallery => {
                    let Some(field) = model.field_gallery.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::ListString(value.into_iter().map(|v| v.to_string()).collect())
                }
                SchematicFieldType::Document => {
                    let Some(field) = model.field_document.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value.to_string())
                }
                SchematicFieldType::MultiDocument => {
                    let Some(field) = model.field_multi_document.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::ListString(value.into_iter().map(|v| v.to_string()).collect())
                }
                SchematicFieldType::Image => {
                    let Some(field) = model.field_image.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value.to_string())
                }
                SchematicFieldType::Video => {
                    let Some(field) = model.field_video.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value.to_string())
                }
                SchematicFieldType::Audio => {
                    let Some(field) = model.field_audio.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::Text(value.to_string())
                }
                SchematicFieldType::Tags => {
                    let Some(field) = model.field_tags.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::ListNumber(value.into_iter().map(|v| (*v).into()).collect())
                }
                SchematicFieldType::Array => {
                    let Some(field) = model.field_array.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::ArrayUnknown(value)
                }
                SchematicFieldType::Object => {
                    let Some(field) = model.field_object.as_mut() else {
                        continue;
                    };

                    let Some(value) = field.remove(key.as_str()) else {
                        continue;
                    };

                    SimpleValue::ObjectUnknown(value)
                }
            },
        );
    }

    for key in map.keys() {
        if let Some(index) = unable_to_find.iter().position(|v| v == key) {
            unable_to_find.swap_remove(index);
        }
    }

    // Not the non-found keys, we'll go through the model field array to see if they're in it.
    for not_found in unable_to_find {
        let Some(field) = model.field_array.as_mut() else {
            continue;
        };

        let Some(value) = field.remove(not_found.as_str()) else {
            continue;
        };

        // TODO: Remove the `[]` - used to prevent frontend from erroring
        map.insert(
            SchematicFieldKey::Other(format!("{not_found}[]")),
            SimpleValue::ArrayUnknown(value),
        );
    }

    Ok(map)
}
