use shared_types::{AppError, ConflictCheck};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new conflict check record.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    case_id: Option<Uuid>,
    party_names: &[String],
    adverse_parties: &[String],
    notes: Option<&str>,
) -> Result<ConflictCheck, AppError> {
    let row = sqlx::query_as!(
        ConflictCheck,
        r#"
        INSERT INTO conflict_checks (
            court_id, attorney_id, case_id, party_names,
            adverse_parties, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id, court_id, attorney_id, check_date, case_id,
            party_names, adverse_parties, cleared, waiver_obtained, notes
        "#,
        court_id,
        attorney_id,
        case_id,
        party_names,
        adverse_parties,
        notes,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all conflict checks for a given attorney within a court.
pub async fn list_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<ConflictCheck>, AppError> {
    let rows = sqlx::query_as!(
        ConflictCheck,
        r#"
        SELECT
            id, court_id, attorney_id, check_date, case_id,
            party_names, adverse_parties, cleared, waiver_obtained, notes
        FROM conflict_checks
        WHERE court_id = $1 AND attorney_id = $2
        ORDER BY check_date DESC
        "#,
        court_id,
        attorney_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Clear (mark as resolved) a conflict check. Returns the updated record or None.
pub async fn clear(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<ConflictCheck>, AppError> {
    let row = sqlx::query_as!(
        ConflictCheck,
        r#"
        UPDATE conflict_checks
        SET cleared = true
        WHERE id = $1 AND court_id = $2
        RETURNING
            id, court_id, attorney_id, check_date, case_id,
            party_names, adverse_parties, cleared, waiver_obtained, notes
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Run a live conflict check by searching existing representations.
/// Returns a list of conflicting case descriptions.
pub async fn run_check(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    party_names: &[String],
) -> Result<Vec<String>, AppError> {
    // Search for cases where the attorney currently represents a party
    // whose name matches any of the provided party names (potential adverse parties).
    let rows = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT
            'Attorney already represents ' || p.name || ' in case ' || c.case_number as "conflict!"
        FROM representations r
        JOIN parties p ON p.id = r.party_id AND p.court_id = r.court_id
        JOIN criminal_cases c ON c.id = r.case_id AND c.court_id = r.court_id
        WHERE r.court_id = $1
          AND r.attorney_id = $2
          AND r.status = 'Active'
          AND p.name = ANY($3)
        "#,
        court_id,
        attorney_id,
        party_names,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}
