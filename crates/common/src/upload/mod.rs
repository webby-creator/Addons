// TODO: Move into Shared common between editor and addons

use std::{
    io::SeekFrom,
    mem::MaybeUninit,
    num::NonZeroUsize,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use btwo::{
    endpoint::{self, AccountAuthorization, UploadFileResponse, UploadUrlResponse},
    BucketId, Credentials, FileId,
};
use bytes::{Bytes, BytesMut};
use concread::EbrCell;
use eyre::Result;
use lazy_static::lazy_static;
use mime::Mime;
use reqwest::Client;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
    process::Command,
    runtime::Runtime,
};

pub mod image;

pub use self::image::process_image;

pub const PARTIAL_UPLOAD_FILES_DIR: &str = "app/.partial_upload_files";
pub const MAX_SINGLE_UPLOAD_SIZE: i64 = 10_000_000;

static NEXT_FILE_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn get_full_file_path(store_path: &str) -> PathBuf {
    let mut path = PathBuf::from("/addon_member_upload");

    path.push(&store_path);

    path
}

pub fn get_thumb_file_path(store_path: &str) -> PathBuf {
    let mut path = PathBuf::from("/addon_member_upload_thumb");

    path.push(&store_path);

    path
}

pub fn get_next_uploading_file_path() -> String {
    format!(
        "{PARTIAL_UPLOAD_FILES_DIR}/uploading{}.uploading",
        NEXT_FILE_INDEX.fetch_add(1, Ordering::SeqCst)
    )
}

// exiftool -j PATH
pub async fn exif_file(path: &str) -> Result<serde_json::Value> {
    let output = Command::new("exiftool")
        .args(&["-j", path])
        .output()
        .await?;

    let stdout = String::from_utf8(output.stdout)?;

    Ok(serde_json::from_str(&stdout)?)
}

#[derive(Clone)]
struct AuthWrapper {
    credentials: Credentials,
    auth: AccountAuthorization,
    last_authed: Instant,
}

impl AuthWrapper {
    pub async fn re_auth(&mut self) -> btwo::Result<()> {
        self.auth = self.credentials.authorize(&CLIENT).await?;
        self.last_authed = Instant::now();

        Ok(())
    }
}

lazy_static! {
    static ref AUTH: EbrCell<Option<AuthWrapper>> = EbrCell::new(None);
    static ref CLIENT: Client = Client::new();
}

// TODO: Use check_and_update_auth for 401 error.

fn get_auth() -> Result<AccountAuthorization> {
    #[allow(clippy::unwrap_used)]
    Ok(AUTH.read().as_ref().unwrap().auth.clone())
}

async fn check_and_update_auth() -> Result<()> {
    #[allow(clippy::unwrap_used)]
    if AUTH.read().as_ref().unwrap().last_authed.elapsed() >= Duration::from_secs(60 * 60 * 16) {
        let mut wrapper = AUTH.write();

        let mutation = wrapper.get_mut();

        if let Err(e) = mutation.as_mut().unwrap().re_auth().await {
            error!("{e}");
        }

        wrapper.commit();
    }

    Ok(())
}

pub async fn register_b2() -> StorageService {
    let bucket_id = BucketId::new_static("d0efdf517160da2b81840017");
    let credentials = Credentials::new(
        "0000ff110ab14070000000009",
        "K000ffVgoIv/DFCL+cj6ZVwv1GPtICw",
    );

    let auth = credentials.authorize(&CLIENT).await.unwrap();

    std::thread::spawn(|| {
        #[allow(clippy::expect_used)]
        let rt = Runtime::new().expect("Thread Auth RT");

        loop {
            std::thread::sleep(Duration::from_secs(30));

            rt.block_on(async {
                if let Err(e) = check_and_update_auth().await {
                    error!("Auth Thread Error: {}", e);
                }
            });
        }
    });

    {
        let mut write = AUTH.write();

        *write.get_mut() = Some(AuthWrapper {
            credentials,
            auth,
            last_authed: Instant::now(),
        });

        write.commit();
    }

    StorageService { bucket_id }
}

#[derive(Clone)]
pub struct StorageService {
    bucket_id: BucketId,
}

impl StorageService {
    pub async fn upload(
        &self,
        full_file_path: PathBuf,
        ext: Mime,
        contents: Vec<u8>,
    ) -> Result<UploadFileResponse> {
        let auth = get_auth()?;

        let upload = endpoint::get_upload_url(&self.bucket_id, &auth, &CLIENT).await?;

        Ok(endpoint::upload_file(
            full_file_path.to_str().unwrap(),
            ext.essence_str(),
            contents,
            &upload,
            &CLIENT,
        )
        .await?)
    }

    pub async fn hide_file(&self, full_file_path: PathBuf) -> Result<()> {
        let auth = get_auth()?;

        endpoint::hide_file(
            &self.bucket_id,
            full_file_path.to_str().unwrap(),
            &auth,
            &CLIENT,
        )
        .await?;

        Ok(())
    }

    /// Uploads a file in chunks if it is larger than 10MB.
    pub async fn upload_large(
        &self,
        full_file_path: PathBuf,
        ext: Mime,
        assumed_length: i64,
        mut file: File,
    ) -> Result<LargeFileResponse> {
        // TODO: Stream uploads

        file.seek(SeekFrom::Start(0)).await?;

        let auth = get_auth()?;

        let min_part_size = auth.absolute_minimum_part_size;
        let rec_part_size = auth.recommended_part_size;

        // This length may be inaccurate. For example it could be Multipart bounds not just stream length.
        if assumed_length <= (rec_part_size + min_part_size) as i64 {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await?;

            let sha256 = format!("{:X}", Sha256::digest(&contents));

            let b2 = self.upload(full_file_path, ext, contents).await?;

            return Ok(LargeFileResponse { sha256, b2 });
        }

        let file_name = full_file_path
            .to_str()
            .ok_or_else(|| eyre::eyre!("Converting PathBuf to String"))?;

        let mut large_upload_file_id = MaybeUninit::uninit();

        // We add the min size just incase the last part is under.
        //  - If WE FILL the buffer we'll only send the recommend size.
        //  - If WE DON'T FILL the buffer then we'll send the whole buffer.
        let mut all_bytes = BytesMut::new();
        all_bytes.reserve(rec_part_size + min_part_size);

        let mut sha256 = Sha256::new();

        let mut ordered_sha1_parts = Vec::new();

        // TODO: Thread uploading for 2 & 3.
        let mut part = NonZeroUsize::new(1).unwrap();

        loop {
            // TODO: If part == 1 AND we don't fill the array. We'll upload without parting out.

            // 1st. Start Large File
            if part.get() == 1 {
                debug!("[LargeFileUpload]: start");

                let start_resp = endpoint::start_large_file(
                    &self.bucket_id,
                    file_name,
                    ext.essence_str(),
                    &auth,
                    &CLIENT,
                )
                .await?;

                large_upload_file_id.write(start_resp.file_id);
            }

            let is_finished;

            // TODO: Handle errors. Retry/Cancel if errored.
            {
                let mut bytes = BytesMut::with_capacity(1024 * 1024);

                loop {
                    let count = file.read_buf(&mut bytes).await.unwrap();

                    if count == 0 {
                        is_finished = true;
                        break;
                    }

                    all_bytes.extend_from_slice(&bytes[..count]);
                    sha256.update(&bytes[..count]);

                    if all_bytes.len() > rec_part_size + min_part_size {
                        is_finished = false;
                        break;
                    }
                }
            }

            if is_finished {
                assert!(part.get() != 1, "Shouldn't be finished on part 1");
            }

            // Take only recommended size.
            let data = if is_finished {
                std::mem::take(&mut all_bytes)
            } else {
                all_bytes.split_to(rec_part_size)
            };

            ordered_sha1_parts.push(format!("{:X}", Sha1::digest(&data)));

            /// Uploads a part of a large file.
            async fn upload(
                auth: &AccountAuthorization,
                file_id: &FileId,
                part: NonZeroUsize,
                data: Bytes,
            ) -> Result<()> {
                // 2nd. Get Upload Part URL.
                let upload_resp = endpoint::get_upload_part_url(file_id, auth, &CLIENT).await?;

                // 3rd. Upload Part
                endpoint::upload_part(part, data, &upload_resp, &CLIENT).await?;

                Ok(())
            }

            debug!("[LargeFileUpload]: uploading part: {}", part.get());

            unsafe {
                if let Err(e) = upload(
                    &auth,
                    large_upload_file_id.assume_init_ref(),
                    part,
                    data.freeze(),
                )
                .await
                {
                    error!("[LargeFileUpload]: {e:?}");

                    endpoint::cancel_large_file(
                        large_upload_file_id.assume_init_ref(),
                        &auth,
                        &CLIENT,
                    )
                    .await?;

                    return Err(e);
                }
            }

            debug!("[LargeFileUpload]: uploaded part: {}", part.get());

            if is_finished {
                break;
            }

            part = part.saturating_add(1);
        }

        debug!("[LargeFileUpload]: end: {}", part.get());

        // 4th. Finish Large File.
        unsafe {
            let b2 = endpoint::finish_large_file(
                large_upload_file_id.assume_init_ref(),
                &ordered_sha1_parts,
                &auth,
                &CLIENT,
            )
            .await?;

            Ok(LargeFileResponse {
                sha256: format!("{:X}", sha256.finalize()),
                b2,
            })
        }
    }

    pub async fn get_upload_url(&self, auth: &AccountAuthorization) -> Result<UploadUrlResponse> {
        Ok(endpoint::get_upload_url(&self.bucket_id, auth, &CLIENT).await?)
    }
}

mod auth {
    use async_trait::async_trait;
    use axum::{extract::FromRequestParts, http::request::Parts, Extension};

    #[async_trait]
    impl<S> FromRequestParts<S> for super::StorageService
    where
        S: Send + Sync,
    {
        type Rejection = ();

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let Extension(auth_cx): Extension<super::StorageService> =
                Extension::from_request_parts(parts, state)
                    .await
                    .expect("Auth extension missing. Is the auth layer installed?");

            Ok(auth_cx)
        }
    }
}

pub struct UploadResponse {
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub media_width: Option<i32>,
    pub media_height: Option<i32>,
    pub hash: String,
    pub has_thumbnail: bool,
}

pub struct LargeFileResponse {
    pub sha256: String,
    pub b2: UploadFileResponse,
}

pub async fn read_and_upload_data(
    store_path: &str,
    file_name: String,
    upload_path: String,
    set_dimensions: Option<(u32, u32)>,
    mut uploading_file: tokio::fs::File,
    storage: &StorageService,
) -> Result<UploadResponse> {
    // TODO: Figure out the file type
    if !file_name.contains('.') {
        return Err(eyre::eyre!("No file extension provided"))?;
    }

    // TODO: Better file type detection. Ex: .tar.gz
    let (_, file_type_s) = file_name.rsplit_once('.').unwrap();

    let meme = mime_guess::from_ext(file_type_s).first_or_text_plain();

    uploading_file.seek(SeekFrom::Start(0)).await?;

    let upload = move |_uploading_file_path: String| async move {
        match meme.type_() {
            mime::IMAGE => {
                process_image(
                    store_path,
                    &file_name,
                    // TODO: Stream the file
                    {
                        let mut bytes = Vec::new();
                        uploading_file.read_to_end(&mut bytes).await?;
                        bytes
                    },
                    true,
                    set_dimensions,
                    storage,
                )
                .await
            }

            _ => Err(eyre::eyre!("Unknown File Type")),
        }
    };

    let uploaded = upload(upload_path.clone()).await;

    tokio::fs::remove_file(upload_path).await?;

    match uploaded {
        Ok(v) => Ok(v),
        Err(e) => {
            error!("Failed to upload file: {e}");

            Err(eyre::eyre!("Failed to upload file: {e}").into())
        }
    }
}
