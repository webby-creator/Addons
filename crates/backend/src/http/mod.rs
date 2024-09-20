mod dev;

use std::{borrow::Cow, net::SocketAddr};

use addon_common::{
    InstallResponse, JsonListResponse, JsonResponse, ListResponse, WrappingResponse,
};
use axum::{
    body::Body,
    extract::{self, multipart::Field},
    http::HeaderValue,
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Extension, Json, Router,
};
use database::{
    AddonDashboardPage, AddonInstanceModel, AddonModel, AddonPermissionModel, MediaUploadModel,
    NewAddonInstanceModel, NewAddonMediaModel, NewAddonModel, NewMediaUploadModel, WidgetModel,
};
use futures::TryStreamExt;
use hyper::header::CONTENT_TYPE;
use lazy_static::lazy_static;
use local_common::{
    api::AddonPublic,
    generate::generate_file_name,
    upload::{
        get_full_file_path, get_next_uploading_file_path, get_thumb_file_path,
        read_and_upload_data, register_b2, StorageService,
    },
    DashboardPageInfo, MemberId, MemberModel, WebsiteModel, WidgetId,
};
use mime_guess::mime::APPLICATION_JSON;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite, SqlitePool};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, net::TcpListener};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::Result;

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
            .route("/addon", post(new_addon))
            .route("/addon/:guid", get(get_addon_public))
            .route("/addon/:guid/instance/:website", get(get_addon_instance))
            .route("/addon/:guid/dashboard/*O", get(get_addon_dashboard_page))
            .route("/addon/:guid/install", post(post_addon_install_user))
            .route("/addon/:guid/icon", post(upload_icon))
            .route("/addon/:guid/gallery", post(upload_gallery_item))
            // Private
            .route("/addon/:guid/item", post(add_addon_item))
            .route("/addon/:guid/access/:user", get(get_addon_member_access))
            .nest("/dev", dev::routes())
            .layer(TraceLayer::new_for_http())
            .layer(Extension(uploader.clone()))
            .with_state(pool),
    )
    .await?;

    Ok(())
}

async fn handle_api(
    extract::Path((addon_id, rest)): extract::Path<(Uuid, String)>,
    extract::State(db): extract::State<SqlitePool>,
    req: extract::Request<Body>,
) -> Result<impl IntoResponse> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon_id, &mut *acq).await? else {
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

    Ok((
        [(
            CONTENT_TYPE,
            content_type.unwrap_or_else(|| HeaderValue::from_static(APPLICATION_JSON.as_ref())),
        )],
        Body::from_stream(resp.bytes_stream()),
    ))
}

async fn get_dashboard_pages(
    extract::Path(website): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
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

async fn get_active_addon_list(
    extract::Path(website): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
) -> Result<JsonListResponse<AddonPublic>> {
    let active =
        AddonInstanceModel::find_by_website_uuid(website, &mut *db.acquire().await?).await?;

    let mut items = Vec::new();

    for instance in active {
        let addon = AddonModel::find_one_by_id(instance.addon_id, &mut *db.acquire().await?)
            .await?
            .unwrap();

        items.push(addon.into_public(None, None, Vec::new()));
    }

    Ok(Json(WrappingResponse::okay(ListResponse::all(items))))
}

// TODO: Route: (User) Uninstall
// TODO: Route: (User) Resume Install
// TODO: Route: (Addon) Instance Install Complete

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddonInstall {
    website_id: uuid::Uuid,
    member_id: uuid::Uuid,

    // TODO: Both of these are said Models'
    member: MemberModel,
    website: WebsiteModel,
}

async fn post_addon_install_user(
    extract::Path(guid): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
    extract::Json(value): extract::Json<AddonInstall>,
) -> Result<JsonResponse<Cow<'static, str>>> {
    let mut acq = db.acquire().await?;

    // let from_server = headers.get("x-server-ip").expect("Expected Server Origin IP")
    //     .to_str().unwrap().to_string();
    // debug!("{from_server}");

    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut *acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    // TODO: Check if website already has addon installed
    // TODO: Ensure member_id is owner of website or has admin

    if let Some(url) = addon.action_url {
        // TODO: Utilize perms
        let _perms =
            AddonPermissionModel::find_by_scope_addon_id(addon.id, "member", &mut *acq).await?;

        // 1. Insert Website Addon
        let mut inst = NewAddonInstanceModel {
            addon_id: addon.id,
            website_id: value.website.id,
            website_uuid: value.website_id,
        }
        .insert(&mut *acq)
        .await?;

        // 2. Send install request
        let resp = CLIENT
            .post(format!("{url}/registration"))
            .json(&serde_json::json!({
                "instanceId": inst.public_id,

                "ownerId": value.member_id,
                "websiteId": value.website_id,

                // TODO: Use Permissions
                "member": value.member,
                "website": value.website,
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            // 3. Get Response - Can have multiple resolutions.
            //  - Could want to redirect the user to finish on another site.
            //  - Could be finished now
            //  - Could be step 1 and require multiple setup requests & permission steps.
            let resp: WrappingResponse<InstallResponse> = resp.json().await?;

            match resp {
                WrappingResponse::Resp(InstallResponse::Complete) => {
                    inst.is_setup = true;
                    inst.update(&mut *acq).await?;
                }

                WrappingResponse::Resp(InstallResponse::Redirect(url)) => {
                    // TODO
                }

                WrappingResponse::Error(e) => return Ok(Json(WrappingResponse::Error(e))),
            }

            Ok(Json(WrappingResponse::okay(Cow::Borrowed("ok"))))
        } else {
            Ok(Json(resp.json().await?))
        }
    } else {
        Ok(Json(WrappingResponse::error("Addon is missing Action URL")))
    }
}

#[derive(Deserialize)]
struct Query {
    pub view: Option<String>,
    pub member: Option<Uuid>,
}

async fn get_addon_list(
    extract::State(db): extract::State<SqlitePool>,
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
    extract::Path((addon_id, website_id)): extract::Path<(Uuid, Uuid)>,
    extract::State(db): extract::State<SqlitePool>,
) -> Result<JsonResponse<serde_json::Value>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon_id, &mut *acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let Some(inst) =
        AddonInstanceModel::find_by_addon_website_id(addon.id, website_id, &mut *acq).await?
    else {
        return Err(eyre::eyre!("Addon Instance not found"))?;
    };

    Ok(Json(WrappingResponse::okay(serde_json::json!({
        "uuid": inst.public_id,
        "isSetup": inst.is_setup,
    }))))
}

async fn get_addon_public(
    extract::Path(guid): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
) -> Result<JsonResponse<AddonPublic>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut *acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let perms = AddonPermissionModel::find_by_addon_id(addon.id, &mut *acq).await?;

    Ok(Json(WrappingResponse::okay(addon.into_public(
        None,
        None,
        perms.into_iter().map(|p| p.perm.to_string()).collect(),
    ))))
}

async fn get_addon_member_access(
    extract::Path((addon, member)): extract::Path<(Uuid, Uuid)>,
    extract::State(db): extract::State<SqlitePool>,
) -> Result<JsonResponse<bool>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon, &mut *acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    Ok(Json(WrappingResponse::okay(addon.member_uuid == member)))
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AddItemJson {
    Widget { uuid: Uuid, id: WidgetId },
}

async fn add_addon_item(
    extract::Path(addon): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
    extract::Json(value): extract::Json<AddItemJson>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon, &mut *acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    match value {
        AddItemJson::Widget { id, uuid } => {
            WidgetModel {
                addon_id: addon.id,
                widget_id: id,
                public_id: uuid,
            }
            .insert(&mut *acq)
            .await?;
        }
    }

    Ok(Json(WrappingResponse::okay("ok")))
}

async fn get_addon_dashboard_page(
    extract::Path((guid, _path)): extract::Path<(Uuid, String)>,
    extract::State(db): extract::State<SqlitePool>,
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

        if meta.is_file() {
            if entry.file_name().to_string_lossy().ends_with(".js") {
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
    }

    Ok(resp_builder.body(String::new()).unwrap().into_response())
}

#[derive(Deserialize)]
pub struct NewAddonJson {
    title: String,
    description: String,
    tagline: String,
    tags: Vec<String>,
}

async fn new_addon(
    extract::State(db): extract::State<SqlitePool>,
    extract::Json(NewAddonJson {
        title,
        description,
        tagline,
        tags: _,
    }): extract::Json<NewAddonJson>,
) -> Result<JsonResponse<AddonPublic>> {
    let addon = NewAddonModel {
        member_id: MemberId::from(1),
        member_uuid: Uuid::nil(),
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
    extract::Path(guid): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
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
    extract::Path(guid): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
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
