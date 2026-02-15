use axum::http::StatusCode;

use crate::common;

#[tokio::test]
async fn test_create_attorney_success() {
    let (app, _pool, _guard) = common::test_app().await;

    let body = serde_json::json!({
        "bar_number": "CA123456",
        "first_name": "John",
        "last_name": "Doe",
        "email": "john.doe@law.com",
        "phone": "415-555-0100",
        "address": {
            "street1": "123 Market St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status, response) = common::post_json(&app, "/api/attorneys", &body.to_string(), "district9").await;

    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "Create attorney should return 200 or 201, got {}",
        status
    );
    assert!(response.get("id").is_some(), "Response should contain attorney ID");
    assert_eq!(response["bar_number"], "CA123456");
    assert_eq!(response["first_name"], "John");
    assert_eq!(response["last_name"], "Doe");
}

#[tokio::test]
async fn test_create_attorney_with_optional_fields() {
    let (app, _pool, _guard) = common::test_app().await;

    let body = serde_json::json!({
        "bar_number": "CA789012",
        "first_name": "Jane",
        "last_name": "Smith",
        "middle_name": "Marie",
        "firm_name": "Smith & Associates",
        "email": "jane.smith@smithlaw.com",
        "phone": "415-555-0200",
        "fax": "415-555-0201",
        "address": {
            "street1": "456 Mission St",
            "street2": "Suite 500",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status, response) = common::post_json(&app, "/api/attorneys", &body.to_string(), "district9").await;

    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "Create attorney with optional fields should succeed, got {}",
        status
    );
    assert_eq!(response["middle_name"], "Marie");
    assert_eq!(response["firm_name"], "Smith & Associates");
    assert_eq!(response["fax"], "415-555-0201");
}

#[tokio::test]
async fn test_create_attorney_missing_required_field() {
    let (app, _pool, _guard) = common::test_app().await;

    let body = serde_json::json!({
        "first_name": "Invalid",
        "last_name": "Attorney",
        "email": "invalid@law.com",
        "phone": "415-555-9999",
        "address": {
            "street1": "999 Error St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status, _response) = common::post_json(&app, "/api/attorneys", &body.to_string(), "district9").await;

    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Should return 400 or 422 for missing required field, got {}",
        status
    );
}

#[tokio::test]
async fn test_create_attorney_invalid_email() {
    let (app, _pool, _guard) = common::test_app().await;

    let body = serde_json::json!({
        "bar_number": "CA345678",
        "first_name": "Test",
        "last_name": "Invalid",
        "email": "not-an-email",
        "phone": "415-555-0300",
        "address": {
            "street1": "789 Test St",
            "city": "San Francisco",
            "state": "CA",
            "zip_code": "94105",
            "country": "USA"
        }
    });

    let (status, _response) = common::post_json(&app, "/api/attorneys", &body.to_string(), "district9").await;

    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Should return 400 or 422 for invalid email format, got {}",
        status
    );
}

#[tokio::test]
async fn test_create_attorney_duplicate_bar_number() {
    let (app, _pool, _guard) = common::test_app().await;

    let body1 = serde_json::json!({
        "bar_number": "CA999999",
        "first_name": "First",
        "last_name": "Attorney",
        "email": "first@law.com",
        "phone": "415-555-1111",
        "address": { "street1": "111 First St", "city": "SF", "state": "CA", "zip_code": "94105", "country": "USA" }
    });

    let (status1, _) = common::post_json(&app, "/api/attorneys", &body1.to_string(), "district9").await;
    assert!(status1 == StatusCode::OK || status1 == StatusCode::CREATED);

    let body2 = serde_json::json!({
        "bar_number": "CA999999",
        "first_name": "Second",
        "last_name": "Attorney",
        "email": "second@law.com",
        "phone": "415-555-2222",
        "address": { "street1": "222 Second St", "city": "SF", "state": "CA", "zip_code": "94105", "country": "USA" }
    });

    let (status2, _response2) = common::post_json(&app, "/api/attorneys", &body2.to_string(), "district9").await;

    assert!(
        status2 == StatusCode::CONFLICT || status2 == StatusCode::BAD_REQUEST,
        "Should return 409 Conflict or 400 for duplicate bar number, got {}",
        status2
    );
}

#[tokio::test]
async fn test_create_attorney_requires_district_header() {
    let (app, _pool, _guard) = common::test_app().await;

    let body = serde_json::json!({
        "bar_number": "CA777777",
        "first_name": "No",
        "last_name": "District",
        "email": "nodistrict@law.com",
        "phone": "415-555-7777",
        "address": { "street1": "777 No District St", "city": "SF", "state": "CA", "zip_code": "94105", "country": "USA" }
    });

    let (status, _) = common::post_no_court(&app, "/api/attorneys", &body.to_string()).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "Should return 400 when district header is missing");
}
