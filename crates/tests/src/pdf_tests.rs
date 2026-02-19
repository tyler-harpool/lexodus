use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::common::{post_json, post_json_raw, test_app};

const COURT: &str = "district9";

fn sample_body() -> String {
    json!({
        "case_id": Uuid::nil(),
        "title": "Test Document",
        "body_text": "This is test content for the document."
    })
    .to_string()
}

fn sample_body_with_judge() -> String {
    json!({
        "case_id": Uuid::nil(),
        "judge_id": Uuid::nil(),
        "title": "Test Signed Document",
        "body_text": "This is test content for a signed document."
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Individual endpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pdf_rule16b_returns_pdf() {
    let (app, _pool, _guard) = test_app().await;

    let (status, headers, bytes) =
        post_json_raw(&app, "/api/pdf/rule16b", &sample_body(), COURT).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/pdf"
    );
    assert!(
        bytes.starts_with(b"%PDF-"),
        "Response should start with PDF magic bytes"
    );
}

#[tokio::test]
async fn pdf_signed_rule16b_returns_pdf() {
    let (app, _pool, _guard) = test_app().await;

    let (status, headers, bytes) = post_json_raw(
        &app,
        "/api/pdf/signed/rule16b",
        &sample_body_with_judge(),
        COURT,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/pdf"
    );
    assert!(bytes.starts_with(b"%PDF-"));
}

#[tokio::test]
async fn pdf_court_order_returns_pdf() {
    let (app, _pool, _guard) = test_app().await;

    let (status, headers, bytes) =
        post_json_raw(&app, "/api/pdf/court-order", &sample_body_with_judge(), COURT).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/pdf"
    );
    assert!(bytes.starts_with(b"%PDF-"));
}

#[tokio::test]
async fn pdf_minute_entry_returns_pdf() {
    let (app, _pool, _guard) = test_app().await;

    let (status, headers, bytes) =
        post_json_raw(&app, "/api/pdf/minute-entry", &sample_body(), COURT).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/pdf"
    );
    assert!(bytes.starts_with(b"%PDF-"));
}

#[tokio::test]
async fn pdf_waiver_indictment_returns_pdf() {
    let (app, _pool, _guard) = test_app().await;

    let (status, headers, bytes) =
        post_json_raw(&app, "/api/pdf/waiver-indictment", &sample_body(), COURT).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/pdf"
    );
    assert!(bytes.starts_with(b"%PDF-"));
}

#[tokio::test]
async fn pdf_conditions_release_returns_pdf() {
    let (app, _pool, _guard) = test_app().await;

    let (status, headers, bytes) = post_json_raw(
        &app,
        "/api/pdf/conditions-release",
        &sample_body_with_judge(),
        COURT,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/pdf"
    );
    assert!(bytes.starts_with(b"%PDF-"));
}

#[tokio::test]
async fn pdf_criminal_judgment_returns_pdf() {
    let (app, _pool, _guard) = test_app().await;

    let (status, headers, bytes) = post_json_raw(
        &app,
        "/api/pdf/criminal-judgment",
        &sample_body_with_judge(),
        COURT,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/pdf"
    );
    assert!(bytes.starts_with(b"%PDF-"));
}

// ---------------------------------------------------------------------------
// Batch endpoint test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pdf_batch_returns_base64_pdfs() {
    let (app, _pool, _guard) = test_app().await;

    let batch_body = json!({
        "documents": [
            {
                "document_type": "rule16b",
                "case_id": Uuid::nil(),
                "title": "Batch Test 1",
                "body_text": "Content for batch item 1."
            },
            {
                "document_type": "court-order",
                "case_id": Uuid::nil(),
                "judge_id": Uuid::nil(),
                "title": "Batch Test 2",
                "body_text": "Content for batch item 2."
            }
        ]
    })
    .to_string();

    let (status, body) = post_json(&app, "/api/pdf/batch", &batch_body, COURT).await;

    assert_eq!(status, StatusCode::OK);

    let arr = body.as_array().expect("batch response should be an array");
    assert_eq!(arr.len(), 2);

    for item in arr {
        assert!(item.get("pdf_base64").is_some(), "should have pdf_base64 field");
        assert!(item.get("filename").is_some(), "should have filename field");
        assert!(item.get("case_id").is_some(), "should have case_id field");
        assert!(item.get("document_type").is_some(), "should have document_type field");

        let b64 = item["pdf_base64"].as_str().unwrap();
        assert!(!b64.is_empty(), "pdf_base64 should not be empty");

        let filename = item["filename"].as_str().unwrap();
        assert!(filename.ends_with(".pdf"), "filename should end with .pdf");
    }
}

// ---------------------------------------------------------------------------
// Removed routes should 404
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pdf_format_routes_removed() {
    let (app, _pool, _guard) = test_app().await;

    let endpoints = [
        "/api/pdf/rule16b/html",
        "/api/pdf/signed/rule16b/pdf",
        "/api/pdf/court-order/html",
        "/api/pdf/minute-entry/html",
        "/api/pdf/waiver-indictment/html",
        "/api/pdf/conditions-release/html",
        "/api/pdf/criminal-judgment/html",
    ];

    for endpoint in endpoints {
        let (status, _) = post_json(&app, endpoint, &sample_body(), COURT).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "/{{format}} route {} should be removed",
            endpoint
        );
    }
}
