use shared_types::{AppError, DocumentEvent};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Create a document event audit record.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    document_id: Uuid,
    event_type: &str,
    actor: &str,
    detail: serde_json::Value,
) -> Result<DocumentEvent, AppError> {
    sqlx::query_as!(
        DocumentEvent,
        r#"
        INSERT INTO document_events (court_id, document_id, event_type, actor, detail)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, court_id, document_id, event_type, actor, detail, created_at
        "#,
        court_id,
        document_id,
        event_type,
        actor,
        detail,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// List all document events for a case (via documents table join), newest first.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<DocumentEvent>, AppError> {
    sqlx::query_as::<_, DocumentEvent>(
        r#"
        SELECT de.id, de.court_id, de.document_id, de.event_type, de.actor,
               de.detail, de.created_at
        FROM document_events de
        INNER JOIN documents d ON d.id = de.document_id AND d.court_id = de.court_id
        WHERE de.court_id = $1 AND d.case_id = $2
        ORDER BY de.created_at DESC
        "#,
    )
    .bind(court_id)
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// List all events for a document, newest first.
pub async fn list_by_document(
    pool: &Pool<Postgres>,
    court_id: &str,
    document_id: Uuid,
) -> Result<Vec<DocumentEvent>, AppError> {
    sqlx::query_as!(
        DocumentEvent,
        r#"
        SELECT id, court_id, document_id, event_type, actor, detail, created_at
        FROM document_events
        WHERE court_id = $1 AND document_id = $2
        ORDER BY created_at DESC
        "#,
        court_id,
        document_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}
