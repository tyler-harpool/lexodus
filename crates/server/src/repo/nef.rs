use shared_types::{AppError, DocketEntry, Document, Filing, Nef};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Create a persisted NEF record for a filing.
///
/// Fetches active parties from the case to build the recipients list,
/// generates an HTML snapshot of the notice, then inserts the NEF row.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    filing: &Filing,
    document: &Document,
    docket_entry: &DocketEntry,
    case_number: &str,
) -> Result<Nef, AppError> {
    // Fetch parties for recipient list
    let parties =
        crate::repo::party::list_service_info_by_case(pool, court_id, filing.case_id).await?;

    // Build recipients JSON (includes contact info for audit trail)
    let recipients: Vec<serde_json::Value> = parties
        .iter()
        .map(|p| {
            let method = p.service_method.as_deref().unwrap_or("Electronic");
            let electronic = method == "Electronic";
            serde_json::json!({
                "party_id": p.id.to_string(),
                "name": p.name,
                "service_method": method,
                "electronic": electronic,
                "email": p.email,
                "phone": p.phone,
            })
        })
        .collect();
    let recipients_json = serde_json::Value::Array(recipients.clone());

    // Build HTML snapshot
    let html_snapshot = build_html_snapshot(
        case_number,
        &document.title,
        &filing.filed_by,
        docket_entry.entry_number,
        &recipients,
    );

    let nef = sqlx::query_as!(
        Nef,
        r#"
        INSERT INTO nefs
            (court_id, filing_id, document_id, case_id, docket_entry_id, recipients, html_snapshot)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, court_id, filing_id, document_id, case_id,
                  docket_entry_id, recipients, html_snapshot, created_at
        "#,
        court_id,
        filing.id,
        document.id,
        filing.case_id,
        docket_entry.id,
        recipients_json,
        html_snapshot,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(nef)
}

/// Find a NEF by its primary ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    nef_id: Uuid,
) -> Result<Option<Nef>, AppError> {
    sqlx::query_as!(
        Nef,
        r#"
        SELECT id, court_id, filing_id, document_id, case_id,
               docket_entry_id, recipients, html_snapshot, created_at
        FROM nefs
        WHERE court_id = $1 AND id = $2
        "#,
        court_id,
        nef_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Find a NEF by its associated filing ID within a court.
pub async fn find_by_filing(
    pool: &Pool<Postgres>,
    court_id: &str,
    filing_id: Uuid,
) -> Result<Option<Nef>, AppError> {
    sqlx::query_as!(
        Nef,
        r#"
        SELECT id, court_id, filing_id, document_id, case_id,
               docket_entry_id, recipients, html_snapshot, created_at
        FROM nefs
        WHERE court_id = $1 AND filing_id = $2
        "#,
        court_id,
        filing_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Find a NEF by its associated docket entry ID within a court.
pub async fn find_by_docket_entry(
    pool: &Pool<Postgres>,
    court_id: &str,
    docket_entry_id: Uuid,
) -> Result<Option<Nef>, AppError> {
    sqlx::query_as!(
        Nef,
        r#"
        SELECT id, court_id, filing_id, document_id, case_id,
               docket_entry_id, recipients, html_snapshot, created_at
        FROM nefs
        WHERE court_id = $1 AND docket_entry_id = $2
        "#,
        court_id,
        docket_entry_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// List all NEFs for a case, newest first.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<Nef>, AppError> {
    sqlx::query_as::<_, Nef>(
        r#"
        SELECT id, court_id, filing_id, document_id, case_id,
               docket_entry_id, recipients, html_snapshot, created_at
        FROM nefs
        WHERE court_id = $1 AND case_id = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(court_id)
    .bind(case_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

// ---------------------------------------------------------------------------
// HTML snapshot generation
// ---------------------------------------------------------------------------

/// Build a CM/ECF-style HTML snapshot for a Notice of Electronic Filing.
fn build_html_snapshot(
    case_number: &str,
    document_title: &str,
    filed_by: &str,
    docket_number: i32,
    recipients: &[serde_json::Value],
) -> String {
    let now = chrono::Utc::now().format("%B %d, %Y at %I:%M %p UTC");

    let recipient_items: String = recipients
        .iter()
        .map(|r| {
            let name = r["name"].as_str().unwrap_or("Unknown");
            let method = r["service_method"].as_str().unwrap_or("Electronic");
            format!("  <li>{name} &mdash; {method}</li>")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<div class="nef">
  <h2>NOTICE OF ELECTRONIC FILING</h2>
  <p><strong>Case:</strong> {case_number}</p>
  <p><strong>Document:</strong> {document_title}</p>
  <p><strong>Filed by:</strong> {filed_by}</p>
  <p><strong>Date:</strong> {now}</p>
  <p><strong>Docket #:</strong> {docket_number}</p>
  <h3>Recipients</h3>
  <ul>
{recipient_items}
  </ul>
</div>"#
    )
}
