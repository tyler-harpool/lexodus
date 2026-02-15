/// S3 integration tests — conditional on RustFS/MinIO availability.
///
/// These tests require a running S3-compatible server at the configured
/// S3_ENDPOINT. They are skipped if the ATTACHMENTS_BUCKET env var is not
/// set or if the S3 endpoint is unreachable.

/// Check S3 availability by actually attempting to list buckets.
/// Returns true only if the S3 server is reachable AND credentials are valid.
async fn s3_available() -> bool {
    let _ = dotenvy::dotenv();

    let endpoint = std::env::var("S3_ENDPOINT")
        .or_else(|_| std::env::var("AWS_ENDPOINT_URL_S3"));
    if endpoint.is_err() {
        return false;
    }

    // Try an authenticated operation to verify credentials
    let store = server::storage::S3ObjectStore::from_env();
    store.ensure_bucket().await;

    // If ensure_bucket didn't panic, try a HEAD to verify access
    use server::storage::ObjectStore;
    // A HEAD on a non-existent key should return Ok(false) if creds work,
    // or Err if creds are invalid.
    match store.head("__s3_availability_probe__").await {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tokio::test]
async fn presign_upload_head_finalize_list() {
    if !s3_available().await {
        eprintln!("Skipping S3 integration test: S3 endpoint not configured or unreachable");
        return;
    }

    use axum::http::StatusCode;
    use crate::common::{
        create_test_case, create_test_docket_entry, get_with_court, post_json, test_app,
    };

    let (app, pool, _guard) = test_app().await;

    // Ensure the attachments bucket exists
    let store = server::storage::S3ObjectStore::from_env();
    store.ensure_bucket().await;

    // Create a docket entry
    let case_id = create_test_case(&pool, "district9", "ATT-S3-001").await;
    let entry = create_test_docket_entry(&app, "district9", &case_id, "exhibit").await;
    let entry_id = entry["id"].as_str().unwrap();

    // POST to get presigned URL
    let body = serde_json::json!({
        "file_name": "exhibit_b.pdf",
        "content_type": "application/pdf",
        "file_size": 11,
    });
    let uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (status, resp) = post_json(&app, &uri, &body.to_string(), "district9").await;
    assert_eq!(status, StatusCode::CREATED);

    let presign_url = resp["presign_url"].as_str().unwrap();
    let attachment_id = resp["attachment_id"].as_str().unwrap();
    let required_headers = resp["required_headers"].as_object().unwrap();

    // Upload file bytes via reqwest PUT to the presigned URL
    let client = reqwest::Client::new();
    let mut req = client
        .put(presign_url)
        .body(b"hello world".to_vec());

    // Add all required headers
    for (k, v) in required_headers {
        req = req.header(k.as_str(), v.as_str().unwrap());
    }

    let upload_resp = req.send().await.expect("upload should succeed");
    if !upload_resp.status().is_success() {
        let status = upload_resp.status();
        let body = upload_resp.text().await.unwrap_or_default();
        panic!(
            "S3 PUT should succeed, got {}\nPresign URL: {}\nRequired headers: {:?}\nResponse body: {}",
            status, presign_url, required_headers, body
        );
    }

    // HEAD confirms object exists
    use server::storage::ObjectStore;
    let object_key = resp["object_key"].as_str().unwrap();
    let exists = store.head(object_key).await.expect("HEAD should succeed");
    assert!(exists, "object should exist after upload");

    // Finalize the attachment
    let finalize_uri = format!("/api/docket/attachments/{}/finalize", attachment_id);
    let (fin_status, fin_resp) = post_json(&app, &finalize_uri, "{}", "district9").await;
    assert_eq!(fin_status, StatusCode::OK, "finalize should return 200: {:?}", fin_resp);
    assert!(fin_resp["uploaded_at"].is_string(), "should have uploaded_at after finalize");

    // GET list should now include the attachment
    let list_uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (list_status, list_resp) = get_with_court(&app, &list_uri, "district9").await;
    assert_eq!(list_status, StatusCode::OK);

    let arr = list_resp.as_array().expect("should be array");
    assert_eq!(arr.len(), 1, "should have exactly one attachment");
    assert_eq!(arr[0]["filename"].as_str(), Some("exhibit_b.pdf"));
    assert_eq!(arr[0]["file_size"].as_i64(), Some(11));
}
