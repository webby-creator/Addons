use std::net::SocketAddr;

use axum::{
    extract::{self, multipart::Field},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use common::{
    api::AddonPublic,
    generate::generate_file_name,
    upload::{
        get_full_file_path, get_next_uploading_file_path, get_thumb_file_path,
        read_and_upload_data, register_b2, StorageService,
    },
    ListResponse, MemberId, WrappingResponse,
};
use database::{
    AddonModel, MediaUploadModel, NewAddonMediaModel, NewAddonModel, NewMediaUploadModel,
};
use futures::TryStreamExt;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite, SqlitePool};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, net::TcpListener};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::Result;

pub type JsonResponse<T> = Json<WrappingResponse<T>>;
pub type JsonListResponse<T> = Json<WrappingResponse<ListResponse<T>>>;

pub async fn serve(pool: Pool<Sqlite>) -> Result<()> {
    let port = 5950;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    debug!("addons listening on {addr}");

    let uploader = register_b2().await;

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(
        listener,
        Router::new()
            .route("/addons", get(get_addon_list))
            .route("/addon", post(new_addon))
            .route("/addon/:guid", get(get_addon))
            .route("/addon/:guid/dashboard-info", get(get_addon_dashboard_info))
            .route("/addon/:guid/dashboard/*O", get(get_addon_dashboard_page))
            .route("/addon/:guid/icon", post(upload_icon))
            .route("/addon/:guid/gallery", post(upload_gallery_item))
            .layer(TraceLayer::new_for_http())
            .layer(Extension(uploader.clone()))
            .with_state(pool),
    )
    .await?;

    Ok(())
}

#[derive(Deserialize)]
struct Query {
    pub view: Option<String>,
}

async fn get_addon_list(
    extract::State(db): extract::State<SqlitePool>,
    extract::Query(Query { view }): extract::Query<Query>,
) -> Result<Response> {
    match view.as_deref() {
        None | Some("simple") => {
            let addons = AddonModel::find_all(&mut *db.acquire().await?).await?;

            Ok(Json(WrappingResponse::okay(ListResponse {
                offset: 0,
                limit: addons.len(),
                total: addons.len(),
                items: addons
                    .into_iter()
                    .map(|a| a.into_public(Uuid::nil(), None, None))
                    .collect(),
            }))
            .into_response())
        }

        Some("extended") => {
            // TODO: Extended Variant
            let addons = AddonModel::find_all(&mut *db.acquire().await?).await?;

            Ok(Json(WrappingResponse::okay(ListResponse {
                offset: 0,
                limit: addons.len(),
                total: addons.len(),
                items: addons
                    .into_iter()
                    .map(|a| a.into_public(Uuid::nil(), None, None))
                    .collect(),
            }))
            .into_response())
        }

        _ => Ok(Json(WrappingResponse::okay(ListResponse::<()>::empty())).into_response()),
    }
}

async fn get_addon(
    extract::Path(guid): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
) -> Result<JsonResponse<AddonPublic>> {
    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut *db.acquire().await?).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    Ok(Json(WrappingResponse::okay(addon.into_public(
        Uuid::nil(),
        None,
        None,
    ))))
}

async fn get_addon_dashboard_info(
    extract::Path(guid): extract::Path<Uuid>,
    extract::State(db): extract::State<SqlitePool>,
) -> Result<JsonResponse<serde_json::Value>> {
    let Some(_addon) = AddonModel::find_one_by_guid(guid, &mut *db.acquire().await?).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    Ok(Json(WrappingResponse::okay(serde_json::json!({
        "routes": [
            { "name": "Overview", "path": "/" },
            // { "name": "Analytics", "path": "/analytics" },
        ]
    }))))
}

async fn get_addon_dashboard_page(
    extract::Path((guid, path)): extract::Path<(Uuid, String)>,
    extract::State(db): extract::State<SqlitePool>,
) -> Result<impl IntoResponse> {
    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut *db.acquire().await?).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

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

#[derive(serde::Deserialize)]
struct Asdf {
    title: String,
    description: String,
    tagline: String,
    tags: Vec<String>,
}

async fn new_addon(
    extract::State(db): extract::State<SqlitePool>,
    extract::Json(Asdf {
        title,
        description,
        tagline,
        tags: _,
    }): extract::Json<Asdf>,
) -> Result<JsonResponse<AddonPublic>> {
    let addon = NewAddonModel {
        member_id: MemberId::from(1),
        name: title,
        tag_line: tagline,
        description,
        icon: None,
        version: String::new(),
    }
    .insert(&mut *db.acquire().await?)
    .await?;

    //

    Ok(Json(WrappingResponse::okay(addon.into_public(
        Uuid::nil(),
        None,
        None,
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
