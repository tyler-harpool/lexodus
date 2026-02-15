use axum::{
    extract::State,
    Json,
};
use sqlx::{Pool, Postgres};

use shared_types::{AppError, ComplianceReport, ComplianceStats};
use crate::error_convert::SqlxErrorExt;
use crate::tenant::CourtId;

/// GET /api/deadlines/compliance-stats
#[utoipa::path(
    get,
    path = "/api/deadlines/compliance-stats",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Compliance statistics", body = ComplianceStats)
    ),
    tag = "compliance"
)]
pub async fn compliance_stats(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<ComplianceStats>, AppError> {
    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM deadlines WHERE court_id = $1"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let met = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM deadlines WHERE court_id = $1 AND status = 'met'"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let missed = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM deadlines WHERE court_id = $1 AND status = 'expired'"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let extended = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM deadlines WHERE court_id = $1 AND status = 'extended'"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let compliance_rate = if total > 0 {
        (met as f64) / (total as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(ComplianceStats {
        total_deadlines: total,
        met,
        missed,
        extended,
        compliance_rate,
    }))
}

/// GET /api/deadlines/compliance-report
#[utoipa::path(
    get,
    path = "/api/deadlines/compliance-report",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Compliance report", body = ComplianceReport)
    ),
    tag = "compliance"
)]
pub async fn compliance_report(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<ComplianceReport>, AppError> {
    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM deadlines WHERE court_id = $1"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let met = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM deadlines WHERE court_id = $1 AND status = 'met'"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let missed = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM deadlines WHERE court_id = $1 AND status = 'expired'"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let extended = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM deadlines WHERE court_id = $1 AND status = 'extended'"#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let compliance_rate = if total > 0 {
        (met as f64) / (total as f64) * 100.0
    } else {
        0.0
    };

    // By-type breakdown using JSON aggregation
    let by_type_row = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(
            json_object_agg(status, cnt),
            '{}'::json
        )::TEXT as "json!"
        FROM (
            SELECT status, COUNT(*) as cnt
            FROM deadlines
            WHERE court_id = $1
            GROUP BY status
        ) sub
        "#,
        court.0,
    )
    .fetch_one(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let by_type: serde_json::Value = serde_json::from_str(&by_type_row)
        .unwrap_or(serde_json::json!({}));

    Ok(Json(ComplianceReport {
        stats: ComplianceStats {
            total_deadlines: total,
            met,
            missed,
            extended,
            compliance_rate,
        },
        by_type,
    }))
}

/// GET /api/deadlines/performance-metrics
#[utoipa::path(
    get,
    path = "/api/deadlines/performance-metrics",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Compliance performance", body = ComplianceStats)
    ),
    tag = "compliance"
)]
pub async fn compliance_performance(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<ComplianceStats>, AppError> {
    // Performance is the same stats view, focused on actionable metrics
    compliance_stats(State(pool), court).await
}

/// GET /api/deadlines/missed-jurisdictional
#[utoipa::path(
    get,
    path = "/api/deadlines/missed-jurisdictional",
    params(
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    responses(
        (status = 200, description = "Missed jurisdictional deadlines", body = Vec<shared_types::DeadlineResponse>)
    ),
    tag = "compliance"
)]
pub async fn missed_jurisdictional(
    State(pool): State<Pool<Postgres>>,
    court: CourtId,
) -> Result<Json<Vec<shared_types::DeadlineResponse>>, AppError> {
    let rows = sqlx::query_as!(
        shared_types::Deadline,
        r#"
        SELECT id, court_id, case_id, title, rule_code, due_at, status, notes, created_at, updated_at
        FROM deadlines
        WHERE court_id = $1
          AND status = 'expired'
          AND rule_code IS NOT NULL
        ORDER BY due_at ASC
        "#,
        court.0,
    )
    .fetch_all(&pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let response: Vec<shared_types::DeadlineResponse> =
        rows.into_iter().map(shared_types::DeadlineResponse::from).collect();

    Ok(Json(response))
}
