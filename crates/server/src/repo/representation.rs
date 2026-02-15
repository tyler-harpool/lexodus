use shared_types::{
    AppError, CreateRepresentationRequest, Representation,
    VALID_REPRESENTATION_TYPES, VALID_WITHDRAWAL_REASONS,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new representation. Also sets party.represented = true.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: &CreateRepresentationRequest,
) -> Result<Representation, AppError> {
    let attorney_id = Uuid::parse_str(&req.attorney_id)
        .map_err(|_| AppError::bad_request("Invalid attorney_id UUID"))?;
    let party_id = Uuid::parse_str(&req.party_id)
        .map_err(|_| AppError::bad_request("Invalid party_id UUID"))?;
    let case_id = Uuid::parse_str(&req.case_id)
        .map_err(|_| AppError::bad_request("Invalid case_id UUID"))?;
    let cja_id = req
        .cja_appointment_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::bad_request("Invalid cja_appointment_id UUID"))?;

    let rep_type = req
        .representation_type
        .as_deref()
        .unwrap_or("Private");
    if !VALID_REPRESENTATION_TYPES.contains(&rep_type) {
        return Err(AppError::bad_request(format!(
            "Invalid representation_type '{}'. Valid: {:?}",
            rep_type, VALID_REPRESENTATION_TYPES
        )));
    }

    // Validate attorney exists in this court
    let atty_exists = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM attorneys WHERE id = $1 AND court_id = $2"#,
        attorney_id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;
    if atty_exists == 0 {
        return Err(AppError::not_found("Attorney not found in this court"));
    }

    // Validate party exists in this court
    let party_exists = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM parties WHERE id = $1 AND court_id = $2"#,
        party_id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;
    if party_exists == 0 {
        return Err(AppError::not_found("Party not found in this court"));
    }

    let lead = req.lead_counsel.unwrap_or(false);
    let local = req.local_counsel.unwrap_or(false);
    let limited = req.limited_appearance.unwrap_or(false);
    let appointed = req.court_appointed.unwrap_or(false);

    let row = sqlx::query_as!(
        Representation,
        r#"
        INSERT INTO representations (
            court_id, attorney_id, party_id, case_id,
            representation_type, lead_counsel, local_counsel,
            limited_appearance, court_appointed, cja_appointment_id,
            scope_of_representation, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING
            id, court_id, attorney_id, party_id, case_id,
            representation_type, status, start_date, end_date,
            lead_counsel, local_counsel, court_appointed,
            limited_appearance, cja_appointment_id, scope_of_representation,
            withdrawal_reason, notes
        "#,
        court_id,
        attorney_id,
        party_id,
        case_id,
        rep_type,
        lead,
        local,
        limited,
        appointed,
        cja_id,
        req.scope_of_representation.as_deref(),
        req.notes.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // Mark the party as represented
    sqlx::query!(
        "UPDATE parties SET represented = true WHERE id = $1 AND court_id = $2",
        party_id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a representation by ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Representation>, AppError> {
    let row = sqlx::query_as!(
        Representation,
        r#"
        SELECT
            id, court_id, attorney_id, party_id, case_id,
            representation_type, status, start_date, end_date,
            lead_counsel, local_counsel, court_appointed,
            limited_appearance, cja_appointment_id, scope_of_representation,
            withdrawal_reason, notes
        FROM representations
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// End a representation (set status to Withdrawn, set end_date to now).
pub async fn end_representation(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    reason: Option<&str>,
) -> Result<Option<Representation>, AppError> {
    if let Some(r) = reason {
        if !VALID_WITHDRAWAL_REASONS.contains(&r) {
            return Err(AppError::bad_request(format!(
                "Invalid withdrawal reason '{}'. Valid: {:?}",
                r, VALID_WITHDRAWAL_REASONS
            )));
        }
    }

    let row = sqlx::query_as!(
        Representation,
        r#"
        UPDATE representations
        SET status = 'Withdrawn',
            end_date = NOW(),
            withdrawal_reason = COALESCE($3, withdrawal_reason)
        WHERE id = $1 AND court_id = $2
        RETURNING
            id, court_id, attorney_id, party_id, case_id,
            representation_type, status, start_date, end_date,
            lead_counsel, local_counsel, court_appointed,
            limited_appearance, cja_appointment_id, scope_of_representation,
            withdrawal_reason, notes
        "#,
        id,
        court_id,
        reason,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    // If the party has no more active representations, mark as unrepresented
    if let Some(ref rep) = row {
        let active_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM representations
            WHERE party_id = $1 AND court_id = $2 AND status = 'Active'"#,
            rep.party_id,
            court_id,
        )
        .fetch_one(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;

        if active_count == 0 {
            sqlx::query!(
                "UPDATE parties SET represented = false WHERE id = $1 AND court_id = $2",
                rep.party_id,
                court_id,
            )
            .execute(pool)
            .await
            .map_err(SqlxErrorExt::into_app_error)?;
        }
    }

    Ok(row)
}

/// List all representations for a case.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<Representation>, AppError> {
    let rows = sqlx::query_as!(
        Representation,
        r#"
        SELECT
            id, court_id, attorney_id, party_id, case_id,
            representation_type, status, start_date, end_date,
            lead_counsel, local_counsel, court_appointed,
            limited_appearance, cja_appointment_id, scope_of_representation,
            withdrawal_reason, notes
        FROM representations
        WHERE court_id = $1 AND case_id = $2
        ORDER BY start_date DESC
        "#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List active representations for an attorney.
pub async fn list_active_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<Representation>, AppError> {
    let rows = sqlx::query_as!(
        Representation,
        r#"
        SELECT
            id, court_id, attorney_id, party_id, case_id,
            representation_type, status, start_date, end_date,
            lead_counsel, local_counsel, court_appointed,
            limited_appearance, cja_appointment_id, scope_of_representation,
            withdrawal_reason, notes
        FROM representations
        WHERE court_id = $1 AND attorney_id = $2 AND status = 'Active'
        ORDER BY start_date DESC
        "#,
        court_id,
        attorney_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all representations for a party (for nesting in PartyResponse).
pub async fn list_by_party(
    pool: &Pool<Postgres>,
    court_id: &str,
    party_id: Uuid,
) -> Result<Vec<Representation>, AppError> {
    let rows = sqlx::query_as!(
        Representation,
        r#"
        SELECT
            id, court_id, attorney_id, party_id, case_id,
            representation_type, status, start_date, end_date,
            lead_counsel, local_counsel, court_appointed,
            limited_appearance, cja_appointment_id, scope_of_representation,
            withdrawal_reason, notes
        FROM representations
        WHERE court_id = $1 AND party_id = $2
        ORDER BY start_date DESC
        "#,
        court_id,
        party_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Substitute one attorney for another on a case. Ends the old representation
/// and creates a new one for the new attorney, inheriting the same parties.
pub async fn substitute(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
    old_attorney_id: Uuid,
    new_attorney_id: Uuid,
) -> Result<Vec<Representation>, AppError> {
    // Find all active representations for old attorney on this case
    let old_reps = sqlx::query_as!(
        Representation,
        r#"
        SELECT
            id, court_id, attorney_id, party_id, case_id,
            representation_type, status, start_date, end_date,
            lead_counsel, local_counsel, court_appointed,
            limited_appearance, cja_appointment_id, scope_of_representation,
            withdrawal_reason, notes
        FROM representations
        WHERE court_id = $1 AND case_id = $2 AND attorney_id = $3 AND status = 'Active'
        "#,
        court_id,
        case_id,
        old_attorney_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    if old_reps.is_empty() {
        return Err(AppError::not_found(
            "No active representations found for old attorney on this case",
        ));
    }

    // Validate new attorney exists
    let new_exists = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM attorneys WHERE id = $1 AND court_id = $2"#,
        new_attorney_id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;
    if new_exists == 0 {
        return Err(AppError::not_found("New attorney not found in this court"));
    }

    let mut new_reps = Vec::new();

    for old_rep in &old_reps {
        // End old representation
        sqlx::query!(
            r#"
            UPDATE representations
            SET status = 'Substituted', end_date = NOW(), withdrawal_reason = 'Client Request'
            WHERE id = $1 AND court_id = $2
            "#,
            old_rep.id,
            court_id,
        )
        .execute(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;

        // Create new representation for new attorney
        let new_rep = sqlx::query_as!(
            Representation,
            r#"
            INSERT INTO representations (
                court_id, attorney_id, party_id, case_id,
                representation_type, lead_counsel, local_counsel,
                limited_appearance, court_appointed, cja_appointment_id,
                scope_of_representation, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id, court_id, attorney_id, party_id, case_id,
                representation_type, status, start_date, end_date,
                lead_counsel, local_counsel, court_appointed,
                limited_appearance, cja_appointment_id, scope_of_representation,
                withdrawal_reason, notes
            "#,
            court_id,
            new_attorney_id,
            old_rep.party_id,
            case_id,
            old_rep.representation_type,
            old_rep.lead_counsel,
            old_rep.local_counsel,
            old_rep.limited_appearance,
            old_rep.court_appointed,
            old_rep.cja_appointment_id,
            old_rep.scope_of_representation.as_deref(),
            old_rep.notes.as_deref(),
        )
        .fetch_one(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?;

        new_reps.push(new_rep);
    }

    Ok(new_reps)
}
