use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};

/// Read an env var, trying the primary name first then a fallback.
pub fn env_or(primary: &str, fallback: &str) -> Option<String> {
    std::env::var(primary)
        .ok()
        .or_else(|| std::env::var(fallback).ok())
}

/// Resolve the S3 endpoint.
/// Fly/Tigris sets `AWS_ENDPOINT_URL_S3`, local dev uses `S3_ENDPOINT`.
fn endpoint() -> String {
    env_or("AWS_ENDPOINT_URL_S3", "S3_ENDPOINT")
        .expect("AWS_ENDPOINT_URL_S3 or S3_ENDPOINT must be set")
}

/// Resolve the bucket name.
/// Fly/Tigris sets `BUCKET_NAME`, local dev uses `S3_BUCKET`.
fn bucket_name() -> String {
    env_or("BUCKET_NAME", "S3_BUCKET").unwrap_or_else(|| "avatars".to_string())
}

/// Build an S3-compatible client from environment variables.
///
/// Supports both Fly/Tigris (`AWS_*`) and local MinIO (`S3_*`) naming:
///   - `AWS_ENDPOINT_URL_S3` / `S3_ENDPOINT`
///   - `AWS_ACCESS_KEY_ID`   / `S3_ACCESS_KEY`
///   - `AWS_SECRET_ACCESS_KEY` / `S3_SECRET_KEY`
///   - `AWS_REGION`          / `S3_REGION`
pub fn s3_client() -> Client {
    let endpoint = endpoint();
    let access_key = env_or("AWS_ACCESS_KEY_ID", "S3_ACCESS_KEY")
        .expect("AWS_ACCESS_KEY_ID or S3_ACCESS_KEY must be set");
    let secret_key = env_or("AWS_SECRET_ACCESS_KEY", "S3_SECRET_KEY")
        .expect("AWS_SECRET_ACCESS_KEY or S3_SECRET_KEY must be set");
    let region = env_or("AWS_REGION", "S3_REGION").unwrap_or_else(|| "us-east-1".to_string());

    let creds = Credentials::new(&access_key, &secret_key, None, None, "env");

    let config = aws_sdk_s3::Config::builder()
        .endpoint_url(&endpoint)
        .region(Region::new(region))
        .credentials_provider(creds)
        .force_path_style(true)
        .behavior_version_latest()
        .build();

    Client::from_conf(config)
}

/// Create the avatars bucket if it doesn't already exist, and set a public-read policy.
pub async fn ensure_bucket() {
    let bucket = bucket_name();
    let client = s3_client();

    let bucket_exists = client.head_bucket().bucket(&bucket).send().await.is_ok();

    if !bucket_exists {
        tracing::info!("Creating S3 bucket '{}'...", bucket);
        match client.create_bucket().bucket(&bucket).send().await {
            Ok(_) => tracing::info!("S3 bucket '{}' created", bucket),
            Err(e) => {
                tracing::warn!("Failed to create S3 bucket '{}': {}", bucket, e);
                return;
            }
        }
    } else {
        tracing::info!("S3 bucket '{}' already exists", bucket);
    }

    // Set public-read policy so avatar URLs are accessible from the browser.
    // Tigris manages public access via `fly storage update --public` instead of S3 bucket policies,
    // so we only apply the policy on non-Tigris providers (e.g. MinIO).
    let ep = endpoint();
    if !ep.contains("tigris") {
        let policy = format!(
            r#"{{"Version":"2012-10-17","Statement":[{{"Effect":"Allow","Principal":"*","Action":["s3:GetObject"],"Resource":["arn:aws:s3:::{}/*"]}}]}}"#,
            bucket
        );
        match client
            .put_bucket_policy()
            .bucket(&bucket)
            .policy(&policy)
            .send()
            .await
        {
            Ok(_) => tracing::info!("Public-read policy applied to '{}'", bucket),
            Err(e) => tracing::warn!("Failed to set bucket policy on '{}': {}", bucket, e),
        }
    }
}

/// Build the public URL for an object.
///
/// Tigris uses virtual-hosted style: `https://{bucket}.fly.storage.tigris.dev/{key}`
/// MinIO uses path style: `http://localhost:9000/{bucket}/{key}`
///
/// We detect Tigris by checking if the endpoint contains `tigris`.
fn public_url(bucket: &str, key: &str) -> String {
    let endpoint = endpoint();
    if endpoint.contains("tigris") {
        // Virtual-hosted style for Tigris
        let host = endpoint
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        format!("https://{}.{}/{}", bucket, host, key)
    } else {
        // Path style for MinIO / generic S3
        format!("{}/{}/{}", endpoint, bucket, key)
    }
}

/// Upload avatar bytes to S3 and return the public URL.
///
/// Objects are stored at `{user_id}/{uuid}.{ext}`.
pub async fn upload_avatar(
    user_id: i64,
    content_type: &str,
    bytes: &[u8],
) -> Result<String, String> {
    let bucket = bucket_name();

    let ext = match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => return Err(format!("Unsupported content type: {}", content_type)),
    };

    let file_id = uuid::Uuid::new_v4();
    let key = format!("{}/{}.{}", user_id, file_id, ext);

    let client = s3_client();
    client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .content_type(content_type)
        .body(ByteStream::from(bytes.to_vec()))
        .send()
        .await
        .map_err(|e| format!("S3 upload failed: {}", e))?;

    Ok(public_url(&bucket, &key))
}
