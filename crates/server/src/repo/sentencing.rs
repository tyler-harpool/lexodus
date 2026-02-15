use shared_types::{AppError, CreateSentencingRequest, SentencingRecord, UpdateSentencingRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new sentencing record.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateSentencingRequest,
) -> Result<SentencingRecord, AppError> {
    let appeal_waiver = req.appeal_waiver.unwrap_or(false);

    let row = sqlx::query_as!(
        SentencingRecord,
        r#"
        INSERT INTO sentencing
            (court_id, case_id, defendant_id, judge_id,
             base_offense_level, specific_offense_level,
             adjusted_offense_level, total_offense_level,
             criminal_history_category, criminal_history_points,
             guidelines_range_low_months, guidelines_range_high_months,
             custody_months, probation_months,
             fine_amount, restitution_amount, forfeiture_amount, special_assessment,
             departure_type, departure_reason,
             variance_type, variance_justification,
             supervised_release_months, appeal_waiver,
             sentencing_date, judgment_date)
        VALUES ($1, $2, $3, $4,
                $5, $6, $7, $8,
                $9, $10, $11, $12,
                $13, $14,
                $15::FLOAT8, $16::FLOAT8, $17::FLOAT8, $18::FLOAT8,
                $19, $20, $21, $22,
                $23, $24, $25, $26)
        RETURNING id, court_id, case_id, defendant_id, judge_id,
                  base_offense_level, specific_offense_level,
                  adjusted_offense_level, total_offense_level,
                  criminal_history_category, criminal_history_points,
                  guidelines_range_low_months, guidelines_range_high_months,
                  custody_months, probation_months,
                  fine_amount::FLOAT8 as "fine_amount: f64",
                  restitution_amount::FLOAT8 as "restitution_amount: f64",
                  forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
                  special_assessment::FLOAT8 as "special_assessment: f64",
                  departure_type, departure_reason,
                  variance_type, variance_justification,
                  supervised_release_months, appeal_waiver,
                  sentencing_date, judgment_date,
                  created_at, updated_at
        "#,
        court_id,
        req.case_id,
        req.defendant_id,
        req.judge_id,
        req.base_offense_level,
        req.specific_offense_level,
        req.adjusted_offense_level,
        req.total_offense_level,
        req.criminal_history_category.as_deref(),
        req.criminal_history_points,
        req.guidelines_range_low_months,
        req.guidelines_range_high_months,
        req.custody_months,
        req.probation_months,
        req.fine_amount,
        req.restitution_amount,
        req.forfeiture_amount,
        req.special_assessment,
        req.departure_type.as_deref(),
        req.departure_reason.as_deref(),
        req.variance_type.as_deref(),
        req.variance_justification.as_deref(),
        req.supervised_release_months,
        appeal_waiver,
        req.sentencing_date,
        req.judgment_date,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a sentencing record by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<SentencingRecord>, AppError> {
    let row = sqlx::query_as!(
        SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
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

/// List all sentencing records for a given case within a court.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<SentencingRecord>, AppError> {
    let rows = sqlx::query_as!(
        SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE case_id = $1 AND court_id = $2
        ORDER BY sentencing_date DESC NULLS LAST
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all sentencing records for a given defendant within a court.
pub async fn list_by_defendant(
    pool: &Pool<Postgres>,
    court_id: &str,
    defendant_id: Uuid,
) -> Result<Vec<SentencingRecord>, AppError> {
    let rows = sqlx::query_as!(
        SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE defendant_id = $1 AND court_id = $2
        ORDER BY sentencing_date DESC NULLS LAST
        "#,
        defendant_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all sentencing records for a given judge within a court.
pub async fn list_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<SentencingRecord>, AppError> {
    let rows = sqlx::query_as!(
        SentencingRecord,
        r#"
        SELECT id, court_id, case_id, defendant_id, judge_id,
               base_offense_level, specific_offense_level,
               adjusted_offense_level, total_offense_level,
               criminal_history_category, criminal_history_points,
               guidelines_range_low_months, guidelines_range_high_months,
               custody_months, probation_months,
               fine_amount::FLOAT8 as "fine_amount: f64",
               restitution_amount::FLOAT8 as "restitution_amount: f64",
               forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
               special_assessment::FLOAT8 as "special_assessment: f64",
               departure_type, departure_reason,
               variance_type, variance_justification,
               supervised_release_months, appeal_waiver,
               sentencing_date, judgment_date,
               created_at, updated_at
        FROM sentencing
        WHERE judge_id = $1 AND court_id = $2
        ORDER BY sentencing_date DESC NULLS LAST
        "#,
        judge_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update a sentencing record with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateSentencingRequest,
) -> Result<Option<SentencingRecord>, AppError> {
    let row = sqlx::query_as!(
        SentencingRecord,
        r#"
        UPDATE sentencing SET
            base_offense_level          = COALESCE($3, base_offense_level),
            specific_offense_level      = COALESCE($4, specific_offense_level),
            adjusted_offense_level      = COALESCE($5, adjusted_offense_level),
            total_offense_level         = COALESCE($6, total_offense_level),
            criminal_history_category   = COALESCE($7, criminal_history_category),
            criminal_history_points     = COALESCE($8, criminal_history_points),
            guidelines_range_low_months = COALESCE($9, guidelines_range_low_months),
            guidelines_range_high_months = COALESCE($10, guidelines_range_high_months),
            custody_months              = COALESCE($11, custody_months),
            probation_months            = COALESCE($12, probation_months),
            fine_amount                 = COALESCE($13::FLOAT8, fine_amount),
            restitution_amount          = COALESCE($14::FLOAT8, restitution_amount),
            forfeiture_amount           = COALESCE($15::FLOAT8, forfeiture_amount),
            special_assessment          = COALESCE($16::FLOAT8, special_assessment),
            departure_type              = COALESCE($17, departure_type),
            departure_reason            = COALESCE($18, departure_reason),
            variance_type               = COALESCE($19, variance_type),
            variance_justification      = COALESCE($20, variance_justification),
            supervised_release_months   = COALESCE($21, supervised_release_months),
            appeal_waiver               = COALESCE($22, appeal_waiver),
            sentencing_date             = COALESCE($23, sentencing_date),
            judgment_date               = COALESCE($24, judgment_date),
            updated_at                  = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, defendant_id, judge_id,
                  base_offense_level, specific_offense_level,
                  adjusted_offense_level, total_offense_level,
                  criminal_history_category, criminal_history_points,
                  guidelines_range_low_months, guidelines_range_high_months,
                  custody_months, probation_months,
                  fine_amount::FLOAT8 as "fine_amount: f64",
                  restitution_amount::FLOAT8 as "restitution_amount: f64",
                  forfeiture_amount::FLOAT8 as "forfeiture_amount: f64",
                  special_assessment::FLOAT8 as "special_assessment: f64",
                  departure_type, departure_reason,
                  variance_type, variance_justification,
                  supervised_release_months, appeal_waiver,
                  sentencing_date, judgment_date,
                  created_at, updated_at
        "#,
        id,
        court_id,
        req.base_offense_level,
        req.specific_offense_level,
        req.adjusted_offense_level,
        req.total_offense_level,
        req.criminal_history_category.as_deref(),
        req.criminal_history_points,
        req.guidelines_range_low_months,
        req.guidelines_range_high_months,
        req.custody_months,
        req.probation_months,
        req.fine_amount,
        req.restitution_amount,
        req.forfeiture_amount,
        req.special_assessment,
        req.departure_type.as_deref(),
        req.departure_reason.as_deref(),
        req.variance_type.as_deref(),
        req.variance_justification.as_deref(),
        req.supervised_release_months,
        req.appeal_waiver,
        req.sentencing_date,
        req.judgment_date,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a sentencing record. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM sentencing WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
