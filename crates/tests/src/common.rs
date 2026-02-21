use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    Router,
};
use serde_json::Value;
use sqlx::{Pool, Postgres};
use tokio::sync::Mutex;
use tower::ServiceExt;

/// Global mutex ensuring tests run sequentially against the shared database.
/// Each test acquires this lock before truncating and seeding, preventing
/// concurrent tests from interfering with each other's data.
static TEST_MUTEX: std::sync::LazyLock<Mutex<()>> = std::sync::LazyLock::new(|| Mutex::new(()));

/// Build a test router backed by a real Postgres pool.
/// Acquires a global lock, truncates all tables, and re-seeds the courts table.
/// The returned `MutexGuard` must be held for the duration of the test to
/// prevent concurrent tests from truncating data.
pub async fn test_app() -> (Router, Pool<Postgres>, tokio::sync::MutexGuard<'static, ()>) {
    // Acquire the global test lock — held until the test completes
    let guard = TEST_MUTEX.lock().await;

    let _ = dotenvy::dotenv();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for tests");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Truncate all data and re-seed
    sqlx::query("TRUNCATE attorneys, attorney_bar_admissions, attorney_federal_admissions, attorney_practice_areas, attorney_discipline_history, calendar_events, deadlines, docket_attachments, docket_entries, criminal_cases, civil_cases, judges, service_records, documents, document_events, parties, filings, filing_uploads, nefs, court_role_requests, clerk_queue, case_events, fee_schedule, rules, motions, judicial_orders, billing_accounts, search_transactions, search_fee_schedule CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to truncate");

    sqlx::query(
        "INSERT INTO courts (id, name, court_type) VALUES ('district9', 'District 9 (Test)', 'test'), ('district12', 'District 12 (Test)', 'test') ON CONFLICT (id) DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to seed courts");

    // Seed a test user (id=1) for token-based auth tests
    sqlx::query(
        "INSERT INTO users (id, username, display_name, email, role, court_roles) VALUES (1, 'testuser', 'Test User', 'test@test.com', 'user', '{}') ON CONFLICT (id) DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to seed test user");

    // Re-seed search fee schedule (truncated above)
    sqlx::query(
        "INSERT INTO search_fee_schedule (action_type, fee_cents, cap_cents, description) VALUES
            ('search', 10, NULL, 'Per-search fee'),
            ('document_view', 10, 300, 'Per-page fee for document access, $3.00 cap per document'),
            ('report', 10, NULL, 'Per-page fee for report generation'),
            ('export', 10, NULL, 'Per-page fee for CSV/PDF export')
        ON CONFLICT (action_type) DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to seed search fee schedule");

    let search = std::sync::Arc::new(server::search::SearchIndex::new());
    let state = server::db::AppState { pool: pool.clone(), search };
    // Include the permissive auth middleware so AuthRequired extractors work
    // when a JWT Bearer token is present; unauthenticated requests still pass through.
    let router = server::rest::api_router()
        .layer(middleware::from_fn_with_state(
            state.clone(),
            server::auth::middleware::auth_middleware,
        ))
        .with_state(state);

    (router, pool, guard)
}

/// POST JSON to a route with a court header.
pub async fn post_json(
    app: &Router,
    uri: &str,
    body: &str,
    court: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-court-district", court)
        .body(Body::from(body.to_string()))
        .unwrap();

    send(app, req).await
}

/// GET a route with a court header.
pub async fn get_with_court(
    app: &Router,
    uri: &str,
    court: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("x-court-district", court)
        .body(Body::empty())
        .unwrap();

    send(app, req).await
}

/// PUT JSON to a route with a court header.
pub async fn put_json(
    app: &Router,
    uri: &str,
    body: &str,
    court: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-court-district", court)
        .body(Body::from(body.to_string()))
        .unwrap();

    send(app, req).await
}

/// DELETE a route with a court header.
pub async fn delete_with_court(
    app: &Router,
    uri: &str,
    court: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("DELETE")
        .uri(uri)
        .header("x-court-district", court)
        .body(Body::empty())
        .unwrap();

    send(app, req).await
}

/// POST JSON WITHOUT a court header (for testing missing header).
pub async fn post_no_court(
    app: &Router,
    uri: &str,
    body: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    send(app, req).await
}

/// GET WITHOUT a court header.
pub async fn get_no_court(
    app: &Router,
    uri: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    send(app, req).await
}

/// Send a request through the router and parse the response.
async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to send request");

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");

    let body: Value = if body_bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(Value::String(
            String::from_utf8_lossy(&body_bytes).to_string(),
        ))
    };

    (status, body)
}

/// Send a request and return raw bytes + status + headers (for non-JSON responses like PDFs).
pub async fn send_raw(
    app: &Router,
    req: Request<Body>,
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to send request");

    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");

    (status, headers, body_bytes.to_vec())
}

/// POST JSON to a route with a court header, returning raw bytes (for PDF endpoints).
pub async fn post_json_raw(
    app: &Router,
    uri: &str,
    body: &str,
    court: &str,
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-court-district", court)
        .body(Body::from(body.to_string()))
        .unwrap();

    send_raw(app, req).await
}

/// Build a test router with a very tight rate limit for testing 429 responses.
pub async fn test_app_rate_limited(
    max_requests: u32,
) -> (Router, Pool<Postgres>, tokio::sync::MutexGuard<'static, ()>) {
    let guard = TEST_MUTEX.lock().await;
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for tests");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    sqlx::query("TRUNCATE attorneys, attorney_bar_admissions, attorney_federal_admissions, attorney_practice_areas, attorney_discipline_history, calendar_events, deadlines, docket_attachments, docket_entries, criminal_cases, civil_cases, judges, service_records, documents, document_events, parties, filings, filing_uploads, nefs, court_role_requests, clerk_queue, case_events, fee_schedule, rules, motions, judicial_orders, billing_accounts, search_transactions, search_fee_schedule CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to truncate");

    sqlx::query(
        "INSERT INTO courts (id, name, court_type) VALUES ('district9', 'District 9 (Test)', 'test'), ('district12', 'District 12 (Test)', 'test') ON CONFLICT (id) DO NOTHING"
    )
    .execute(&pool)
    .await
    .expect("Failed to seed courts");

    let rate_limit = server::rate_limit::RateLimitState::new(
        max_requests,
        std::time::Duration::from_secs(60),
    );
    let search = std::sync::Arc::new(server::search::SearchIndex::new());
    let state = server::db::AppState { pool: pool.clone(), search };
    let router = server::rest::api_router_with_rate_limit(rate_limit)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            server::auth::middleware::auth_middleware,
        ))
        .with_state(state);

    (router, pool, guard)
}

/// Create a JWT access token for testing with a given role (e.g. "clerk", "judge", "attorney").
/// Requires JWT_SECRET to be set in the environment.
/// For per-court role checks, use `create_test_token_with_courts` instead.
pub fn create_test_token(role: &str) -> String {
    server::auth::jwt::create_access_token(1, "test@test.com", role, "enterprise", &std::collections::HashMap::new())
        .expect("Failed to create test JWT")
}

/// Create a JWT access token with per-court role memberships.
/// The global `role` param is typically "user"; actual permissions come from `court_roles`.
pub fn create_test_token_with_courts(role: &str, court_roles: &std::collections::HashMap<String, String>) -> String {
    server::auth::jwt::create_access_token(1, "test@test.com", role, "enterprise", court_roles)
        .expect("Failed to create test JWT")
}

/// POST JSON to a route with a court header and a JWT Bearer token (for role-gated endpoints).
pub async fn post_json_authed(
    app: &Router,
    uri: &str,
    body: &str,
    court: &str,
    token: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-court-district", court)
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(body.to_string()))
        .unwrap();

    send(app, req).await
}

/// PUT JSON to a route with a court header and a JWT Bearer token.
pub async fn put_json_authed(
    app: &Router,
    uri: &str,
    body: &str,
    court: &str,
    token: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-court-district", court)
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(body.to_string()))
        .unwrap();

    send(app, req).await
}

/// DELETE a route with a court header and a JWT Bearer token.
pub async fn delete_authed(
    app: &Router,
    uri: &str,
    court: &str,
    token: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("DELETE")
        .uri(uri)
        .header("x-court-district", court)
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    send(app, req).await
}

/// GET a route with a court header and a JWT Bearer token.
pub async fn get_authed(
    app: &Router,
    uri: &str,
    court: &str,
    token: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("x-court-district", court)
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    send(app, req).await
}

/// Create a test attorney and return its ID.
pub async fn create_test_attorney(
    app: &Router,
    court: &str,
    bar_number: &str,
) -> String {
    let body = serde_json::json!({
        "bar_number": bar_number,
        "first_name": "Test",
        "last_name": "Attorney",
        "email": format!("{}@test.com", bar_number.to_lowercase()),
        "phone": "555-0100",
        "address": {
            "street1": "123 Test St",
            "city": "Test City",
            "state": "TC",
            "zip_code": "12345",
            "country": "USA"
        }
    });

    let (status, response) = post_json(app, "/api/attorneys", &body.to_string(), court).await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "Failed to create test attorney: {} {:?}",
        status,
        response
    );
    response["id"].as_str().unwrap().to_string()
}

/// Create a test judge directly in the DB and return its UUID string.
pub async fn create_test_judge(pool: &Pool<Postgres>, court: &str, name: &str) -> String {
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO judges (court_id, name, title, district)
        VALUES ($1, $2, 'Judge', $1)
        RETURNING id
        "#,
        court,
        name,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test judge");

    row.to_string()
}

/// Create a test criminal case directly in the DB and return its UUID string.
pub async fn create_test_case(pool: &Pool<Postgres>, court: &str, case_number: &str) -> String {
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO criminal_cases (court_id, case_number, title, crime_type, district_code)
        VALUES ($1, $2, 'Test Case', 'fraud', $1)
        RETURNING id
        "#,
        court,
        case_number,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test case");

    row.to_string()
}

/// Create a test criminal case via the API and return the response JSON.
pub async fn create_test_case_via_api(
    app: &Router,
    court: &str,
    title: &str,
) -> Value {
    let body = serde_json::json!({
        "title": title,
        "crime_type": "fraud",
        "district_code": court,
    });

    let (status, response) = post_json(app, "/api/cases", &body.to_string(), court).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create test case: {} {:?}",
        status,
        response
    );
    response
}

/// Create a test civil case via the API and return the response JSON.
pub async fn create_test_civil_case_via_api(
    app: &Router,
    court: &str,
    title: &str,
) -> Value {
    let body = serde_json::json!({
        "title": title,
        "nature_of_suit": "110",
        "jurisdiction_basis": "federal_question",
    });

    let (status, response) = post_json(app, "/api/civil-cases", &body.to_string(), court).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create test civil case: {} {:?}",
        status,
        response
    );
    response
}

/// PATCH JSON to a route with a court header.
pub async fn patch_json(
    app: &Router,
    uri: &str,
    body: &str,
    court: &str,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("PATCH")
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-court-district", court)
        .body(Body::from(body.to_string()))
        .unwrap();

    send(app, req).await
}

/// Create a test deadline via the API and return the response JSON.
pub async fn create_test_deadline(
    app: &Router,
    court: &str,
    title: &str,
) -> Value {
    let body = serde_json::json!({
        "title": title,
        "due_at": "2026-07-01T17:00:00Z",
    });

    let (status, response) = post_json(app, "/api/deadlines", &body.to_string(), court).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create test deadline: {} {:?}",
        status,
        response
    );
    response
}

/// Create a test deadline with a case_id linked.
pub async fn create_test_deadline_with_case(
    app: &Router,
    court: &str,
    title: &str,
    case_id: &str,
) -> Value {
    let body = serde_json::json!({
        "title": title,
        "case_id": case_id,
        "due_at": "2026-07-01T17:00:00Z",
    });

    let (status, response) = post_json(app, "/api/deadlines", &body.to_string(), court).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create test deadline: {} {:?}",
        status,
        response
    );
    response
}

/// Create a test docket entry via the API and return the response JSON.
/// Uses a clerk JWT token with per-court membership since docket entry creation is role-gated.
pub async fn create_test_docket_entry(
    app: &Router,
    court: &str,
    case_id: &str,
    entry_type: &str,
) -> Value {
    let body = serde_json::json!({
        "case_id": case_id,
        "entry_type": entry_type,
        "description": format!("Test {} entry", entry_type),
    });

    let court_roles = std::collections::HashMap::from([(court.to_string(), "clerk".to_string())]);
    let token = create_test_token_with_courts("user", &court_roles);
    let (status, response) = post_json_authed(app, "/api/docket/entries", &body.to_string(), court, &token).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create test docket entry: {} {:?}",
        status,
        response
    );
    response
}

/// Create a test docket attachment via the API and return the response JSON.
pub async fn create_test_docket_attachment(
    app: &Router,
    court: &str,
    entry_id: &str,
    filename: &str,
) -> Value {
    let body = serde_json::json!({
        "file_name": filename,
        "content_type": "application/pdf",
        "file_size": 12345,
    });

    let uri = format!("/api/docket/entries/{}/attachments", entry_id);
    let (status, response) = post_json(app, &uri, &body.to_string(), court).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create test docket attachment: {} {:?}",
        status,
        response
    );
    response
}

/// Create a test calendar event via the API and return the response JSON.
pub async fn create_test_calendar_event(
    app: &Router,
    court: &str,
    case_id: &str,
    judge_id: &str,
    event_type: &str,
) -> Value {
    let body = serde_json::json!({
        "case_id": case_id,
        "judge_id": judge_id,
        "event_type": event_type,
        "scheduled_date": "2026-06-15T09:00:00Z",
        "duration_minutes": 60,
        "courtroom": "Courtroom 4A",
        "description": "Test calendar event",
        "participants": ["Prosecutor", "Defense Counsel"],
        "is_public": true
    });

    let (status, response) = post_json(app, "/api/calendar/events", &body.to_string(), court).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Failed to create test calendar event: {} {:?}",
        status,
        response
    );
    response
}

/// Create a test document directly in the DB and return its UUID string.
pub async fn create_test_document(
    pool: &Pool<Postgres>,
    court: &str,
    case_id: &str,
) -> String {
    let case_uuid =
        uuid::Uuid::parse_str(case_id).expect("Invalid case_id UUID");
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO documents (court_id, case_id, title, document_type, storage_key, checksum, file_size, content_type, uploaded_by)
        VALUES ($1, $2, 'Test Document', 'Motion', 'test-key', 'abc123', 1024, 'application/pdf', 'test-user')
        RETURNING id
        "#,
        court,
        case_uuid,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test document");

    row.to_string()
}

/// Create a test docket attachment directly in the DB AND mark it uploaded.
/// Returns the attachment UUID string.
pub async fn create_uploaded_attachment(
    pool: &Pool<Postgres>,
    court: &str,
    entry_id: &str,
) -> String {
    let entry_uuid = uuid::Uuid::parse_str(entry_id).expect("Invalid entry_id UUID");
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO docket_attachments
            (court_id, docket_entry_id, filename, file_size, content_type, storage_key, uploaded_at)
        VALUES ($1, $2, 'test-file.pdf', 12345, 'application/pdf', 'test/storage/key.pdf', NOW())
        RETURNING id
        "#,
        court,
        entry_uuid,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create uploaded attachment");

    row.to_string()
}

/// Create a test docket attachment directly in the DB WITHOUT marking it uploaded.
/// Returns the attachment UUID string.
pub async fn create_pending_attachment(
    pool: &Pool<Postgres>,
    court: &str,
    entry_id: &str,
) -> String {
    let entry_uuid = uuid::Uuid::parse_str(entry_id).expect("Invalid entry_id UUID");
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO docket_attachments
            (court_id, docket_entry_id, filename, file_size, content_type, storage_key)
        VALUES ($1, $2, 'pending-file.pdf', 5000, 'application/pdf', 'test/pending/key.pdf')
        RETURNING id
        "#,
        court,
        entry_uuid,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create pending attachment");

    row.to_string()
}

/// Create a test party directly in the DB and return its UUID string.
pub async fn create_test_party(
    pool: &Pool<Postgres>,
    court: &str,
    case_id: &str,
) -> String {
    let case_uuid =
        uuid::Uuid::parse_str(case_id).expect("Invalid case_id UUID");
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO parties (court_id, case_id, party_type, party_role, name, entity_type, represented, pro_se, service_method, status, joined_date)
        VALUES ($1, $2, 'Defendant', 'Lead', 'Test Party', 'Individual', false, true, 'Electronic', 'Active', NOW())
        RETURNING id
        "#,
        court,
        case_uuid,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test party");

    row.to_string()
}

/// Create a test queue item via the API and return the response JSON.
pub async fn create_test_queue_item(
    app: &Router,
    court: &str,
    title: &str,
    source_id: &str,
) -> serde_json::Value {
    let body = serde_json::json!({
        "queue_type": "filing",
        "priority": 3,
        "title": title,
        "source_type": "filing",
        "source_id": source_id,
    });
    let (status, resp) = post_json(app, "/api/queue", &body.to_string(), court).await;
    assert_eq!(status, StatusCode::CREATED, "Failed to create queue item: {resp}");
    resp
}

/// Create a test party with a specific service method and name.
pub async fn create_test_party_with_method(
    pool: &Pool<Postgres>,
    court: &str,
    case_id: &str,
    name: &str,
    service_method: &str,
) -> String {
    let case_uuid =
        uuid::Uuid::parse_str(case_id).expect("Invalid case_id UUID");
    let row = sqlx::query_scalar!(
        r#"
        INSERT INTO parties (court_id, case_id, party_type, party_role, name, entity_type, represented, pro_se, service_method, status, joined_date)
        VALUES ($1, $2, 'Defendant', 'Lead', $3, 'Individual', false, true, $4, 'Active', NOW())
        RETURNING id
        "#,
        court,
        case_uuid,
        name,
        service_method,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test party");

    row.to_string()
}
