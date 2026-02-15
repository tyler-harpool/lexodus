use std::collections::HashMap;
use std::time::Duration;

use aws_sdk_s3::{
    config::{Credentials, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
    types::ServerSideEncryption,
    Client,
};

use crate::s3::env_or;

/// Bucket name for docket attachments (from env or default).
fn attachments_bucket() -> String {
    std::env::var("ATTACHMENTS_BUCKET").unwrap_or_else(|_| "attachments".to_string())
}

/// Default presign expiry (15 minutes).
const PRESIGN_EXPIRY_SECS: u64 = 900;

// ── Trait ────────────────────────────────────────────────────────────

/// Object storage operations for docket attachments.
#[allow(async_fn_in_trait)]
pub trait ObjectStore: Send + Sync {
    /// Generate a presigned PUT URL. Returns (url, required_headers).
    async fn presign_put(
        &self,
        key: &str,
        content_type: &str,
    ) -> Result<(String, HashMap<String, String>), String>;

    /// Generate a presigned GET URL for downloading.
    async fn presign_get(&self, key: &str) -> Result<String, String>;

    /// Check if an object exists.
    async fn head(&self, key: &str) -> Result<bool, String>;

    /// Delete an object.
    async fn delete(&self, key: &str) -> Result<(), String>;

    /// Download object bytes from S3.
    async fn get(&self, key: &str) -> Result<Vec<u8>, String>;

    /// Upload bytes directly to S3 with SSE-S3 encryption.
    async fn put(
        &self,
        key: &str,
        content_type: &str,
        body: Vec<u8>,
    ) -> Result<(), String>;
}

// ── S3 implementation ───────────────────────────────────────────────

/// S3-compatible object store backed by RustFS/MinIO.
/// All uploads are encrypted with SSE-S3 (AES256).
pub struct S3ObjectStore {
    client: Client,
    bucket: String,
}

impl S3ObjectStore {
    /// Build a new S3ObjectStore from environment variables.
    pub fn from_env() -> Self {
        let endpoint = env_or("AWS_ENDPOINT_URL_S3", "S3_ENDPOINT")
            .expect("AWS_ENDPOINT_URL_S3 or S3_ENDPOINT must be set");
        let access_key = env_or("AWS_ACCESS_KEY_ID", "S3_ACCESS_KEY")
            .expect("AWS_ACCESS_KEY_ID or S3_ACCESS_KEY must be set");
        let secret_key = env_or("AWS_SECRET_ACCESS_KEY", "S3_SECRET_KEY")
            .expect("AWS_SECRET_ACCESS_KEY or S3_SECRET_KEY must be set");
        let region =
            env_or("AWS_REGION", "S3_REGION").unwrap_or_else(|| "us-east-1".to_string());

        let creds = Credentials::new(&access_key, &secret_key, None, None, "env");

        let config = aws_sdk_s3::Config::builder()
            .endpoint_url(&endpoint)
            .region(Region::new(region))
            .credentials_provider(creds)
            .force_path_style(true)
            .behavior_version_latest()
            .build();

        Self {
            client: Client::from_conf(config),
            bucket: attachments_bucket(),
        }
    }

    /// Ensure the attachments bucket exists (no public-read policy).
    pub async fn ensure_bucket(&self) {
        let exists = self
            .client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .is_ok();

        if !exists {
            tracing::info!("Creating attachments bucket '{}'...", self.bucket);
            match self.client.create_bucket().bucket(&self.bucket).send().await {
                Ok(_) => tracing::info!("Attachments bucket '{}' created", self.bucket),
                Err(e) => tracing::warn!(
                    "Failed to create attachments bucket '{}': {}",
                    self.bucket,
                    e
                ),
            }
        }
    }
}

impl ObjectStore for S3ObjectStore {
    async fn presign_put(
        &self,
        key: &str,
        content_type: &str,
    ) -> Result<(String, HashMap<String, String>), String> {
        let presign_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(PRESIGN_EXPIRY_SECS))
            .build()
            .map_err(|e| format!("Presign config error: {}", e))?;

        // SSE is NOT signed into the presigned URL — signing it causes 403
        // on some S3-compatible backends.  Instead it's included as a
        // required (unsigned) header for the client to attach.
        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(presign_config)
            .await
            .map_err(|e| format!("Presign PUT failed: {}", e))?;

        let mut required_headers = HashMap::new();
        required_headers.insert("Content-Type".to_string(), content_type.to_string());
        required_headers.insert(
            "x-amz-server-side-encryption".to_string(),
            "AES256".to_string(),
        );

        Ok((presigned.uri().to_string(), required_headers))
    }

    async fn presign_get(&self, key: &str) -> Result<String, String> {
        let presign_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(PRESIGN_EXPIRY_SECS))
            .build()
            .map_err(|e| format!("Presign config error: {}", e))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presign_config)
            .await
            .map_err(|e| format!("Presign GET failed: {}", e))?;

        Ok(presigned.uri().to_string())
    }

    async fn head(&self, key: &str) -> Result<bool, String> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let svc_err = e.into_service_error();
                if svc_err.is_not_found() {
                    Ok(false)
                } else {
                    Err(format!("HEAD failed: {}", svc_err))
                }
            }
        }
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, String> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let svc = e.into_service_error();
                tracing::error!("S3 GetObject failed for key '{}': {:?}", key, svc);
                format!("S3 download failed: {}", svc)
            })?;

        resp.body
            .collect()
            .await
            .map(|data| data.into_bytes().to_vec())
            .map_err(|e| format!("Failed to read S3 response body: {}", e))
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| format!("DELETE failed: {}", e))?;
        Ok(())
    }

    async fn put(
        &self,
        key: &str,
        content_type: &str,
        body: Vec<u8>,
    ) -> Result<(), String> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .server_side_encryption(ServerSideEncryption::Aes256)
            .body(ByteStream::from(body))
            .send()
            .await
            .map_err(|e| {
                let svc = e.into_service_error();
                tracing::error!("S3 PutObject failed for key '{}': {:?}", key, svc);
                format!("S3 upload failed: {}", svc)
            })?;

        Ok(())
    }
}
