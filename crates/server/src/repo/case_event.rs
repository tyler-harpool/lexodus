use sqlx::PgPool;
use uuid::Uuid;

/// Insert a new case event record, returning the generated UUID.
pub async fn insert(
    pool: &PgPool,
    court_id: &str,
    case_id: Uuid,
    case_type: &str,
    trigger_event: &str,
    actor_id: Option<Uuid>,
    payload: &serde_json::Value,
    compliance_report: Option<&serde_json::Value>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO case_events (court_id, case_id, case_type, trigger_event, actor_id, payload, compliance_report)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id"#,
    )
    .bind(court_id)
    .bind(case_id)
    .bind(case_type)
    .bind(trigger_event)
    .bind(actor_id)
    .bind(payload)
    .bind(compliance_report)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// List recent case events for a given case, ordered by creation time descending.
/// Returns up to 100 events as raw JSON values.
pub async fn list_by_case(
    pool: &PgPool,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<(serde_json::Value,)> = sqlx::query_as(
        r#"SELECT row_to_json(e)
           FROM case_events e
           WHERE court_id = $1 AND case_id = $2
           ORDER BY created_at DESC
           LIMIT 100"#,
    )
    .bind(court_id)
    .bind(case_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(v,)| v).collect())
}
