use axum::{extract::State, http::StatusCode, Json};
use sqlx::{Pool, Postgres};

use shared_types::{AppError, Court, InitTenantRequest, TenantStats};
use crate::error_convert::SqlxErrorExt;
use crate::tenant::CourtId;

/// POST /api/admin/tenants/init
#[utoipa::path(
    post,
    path = "/api/admin/tenants/init",
    request_body = InitTenantRequest,
    responses(
        (status = 200, description = "Tenant initialized", body = Court),
        (status = 400, description = "Invalid request", body = AppError)
    ),
    tag = "admin"
)]
pub async fn init_tenant(
    State(pool): State<Pool<Postgres>>,
    Json(body): Json<InitTenantRequest>,
) -> Result<(StatusCode, Json<Court>), AppError> {
    let court = sqlx::query_as!(
        Court,
        r#"
        INSERT INTO courts (id, name, court_type)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name
        RETURNING id, name, court_type, tier, created_at
        "#,
        body.id,
        body.name,
        body.court_type,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((StatusCode::OK, Json(court)))
}

/// GET /api/admin/tenants/stats
#[utoipa::path(
    get,
    path = "/api/admin/tenants/stats",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Tenant statistics", body = TenantStats)
    ),
    tag = "admin"
)]
pub async fn tenant_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<TenantStats>, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM attorneys WHERE court_id = $1"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(Json(TenantStats {
        court_id: court.0,
        attorney_count: count,
    }))
}

/// GET /api/courts — list all court districts (no auth required)
#[utoipa::path(
    get,
    path = "/api/courts",
    responses(
        (status = 200, description = "All court districts", body = Vec<Court>)
    ),
    tag = "admin"
)]
pub async fn list_courts(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<Court>>, AppError> {
    let courts = sqlx::query_as!(
        Court,
        r#"SELECT id, name, court_type, tier, created_at FROM courts ORDER BY name"#,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(Json(courts))
}
