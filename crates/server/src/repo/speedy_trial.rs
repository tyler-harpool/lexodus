use shared_types::{
    AppError, SpeedyTrialClock, StartSpeedyTrialRequest, UpdateSpeedyTrialClockRequest,
    ExcludableDelay, CreateExcludableDelayRequest,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

// ── Speedy Trial Clock ────────────────────────────────────────────

/// Start (insert) a speedy trial clock for a case.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: StartSpeedyTrialRequest,
) -> Result<SpeedyTrialClock, AppError> {
    let row = sqlx::query_as!(
        SpeedyTrialClock,
        r#"
        INSERT INTO speedy_trial
            (case_id, court_id, arrest_date, indictment_date,
             arraignment_date, trial_start_deadline)
        VALUES ($1, $2, $3, $4, $5,
                COALESCE($6, COALESCE($3, $4, $5, NOW()) + INTERVAL '70 days'))
        RETURNING case_id, court_id, arrest_date, indictment_date,
                  arraignment_date,
                  COALESCE(trial_start_deadline, NOW() + INTERVAL '70 days') as "trial_start_deadline!",
                  days_elapsed, days_remaining, is_tolled, waived
        "#,
        req.case_id,
        court_id,
        req.arrest_date,
        req.indictment_date,
        req.arraignment_date,
        req.trial_start_deadline,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Get a speedy trial clock by case ID.
pub async fn find_by_case_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Option<SpeedyTrialClock>, AppError> {
    let row = sqlx::query_as!(
        SpeedyTrialClock,
        r#"
        SELECT case_id, court_id, arrest_date, indictment_date,
               arraignment_date,
               COALESCE(trial_start_deadline, NOW() + INTERVAL '70 days') as "trial_start_deadline!",
               days_elapsed, days_remaining, is_tolled, waived
        FROM speedy_trial
        WHERE case_id = $1 AND court_id = $2
        "#,
        case_id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Update a speedy trial clock.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
    req: UpdateSpeedyTrialClockRequest,
) -> Result<Option<SpeedyTrialClock>, AppError> {
    let row = sqlx::query_as!(
        SpeedyTrialClock,
        r#"
        UPDATE speedy_trial SET
            arrest_date          = COALESCE($3, arrest_date),
            indictment_date      = COALESCE($4, indictment_date),
            arraignment_date     = COALESCE($5, arraignment_date),
            trial_start_deadline = COALESCE($6, trial_start_deadline),
            days_elapsed         = COALESCE($7, days_elapsed),
            days_remaining       = COALESCE($8, days_remaining),
            is_tolled            = COALESCE($9, is_tolled),
            waived               = COALESCE($10, waived)
        WHERE case_id = $1 AND court_id = $2
        RETURNING case_id, court_id, arrest_date, indictment_date,
                  arraignment_date,
                  COALESCE(trial_start_deadline, NOW() + INTERVAL '70 days') as "trial_start_deadline!",
                  days_elapsed, days_remaining, is_tolled, waived
        "#,
        case_id,
        court_id,
        req.arrest_date,
        req.indictment_date,
        req.arraignment_date,
        req.trial_start_deadline,
        req.days_elapsed,
        req.days_remaining,
        req.is_tolled,
        req.waived,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List cases approaching their speedy trial deadline (within given days).
pub async fn list_approaching(
    pool: &Pool<Postgres>,
    court_id: &str,
    within_days: i64,
) -> Result<Vec<SpeedyTrialClock>, AppError> {
    let rows = sqlx::query_as!(
        SpeedyTrialClock,
        r#"
        SELECT case_id, court_id, arrest_date, indictment_date,
               arraignment_date,
               COALESCE(trial_start_deadline, NOW() + INTERVAL '70 days') as "trial_start_deadline!",
               days_elapsed, days_remaining, is_tolled, waived
        FROM speedy_trial
        WHERE court_id = $1
          AND waived = FALSE
          AND is_tolled = FALSE
          AND days_remaining <= $2
          AND days_remaining > 0
        ORDER BY days_remaining ASC
        "#,
        court_id,
        within_days,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List cases that have violated the speedy trial deadline.
pub async fn list_violations(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<SpeedyTrialClock>, AppError> {
    let rows = sqlx::query_as!(
        SpeedyTrialClock,
        r#"
        SELECT case_id, court_id, arrest_date, indictment_date,
               arraignment_date,
               COALESCE(trial_start_deadline, NOW() + INTERVAL '70 days') as "trial_start_deadline!",
               days_elapsed, days_remaining, is_tolled, waived
        FROM speedy_trial
        WHERE court_id = $1
          AND waived = FALSE
          AND days_remaining <= 0
        ORDER BY days_remaining ASC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

// ── Excludable Delays ─────────────────────────────────────────────

/// Add an excludable delay for a case.
pub async fn create_delay(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
    req: CreateExcludableDelayRequest,
) -> Result<ExcludableDelay, AppError> {
    let days = req.days_excluded.unwrap_or(0);

    let row = sqlx::query_as!(
        ExcludableDelay,
        r#"
        INSERT INTO excludable_delays
            (court_id, case_id, start_date, end_date, reason,
             statutory_reference, days_excluded, order_reference)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, court_id, case_id, start_date, end_date, reason,
                  COALESCE(statutory_reference, '') as "statutory_reference!",
                  days_excluded, order_reference
        "#,
        court_id,
        case_id,
        req.start_date,
        req.end_date,
        req.reason,
        req.statutory_reference.as_deref(),
        days,
        req.order_reference.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all excludable delays for a case.
pub async fn list_delays_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<ExcludableDelay>, AppError> {
    let rows = sqlx::query_as!(
        ExcludableDelay,
        r#"
        SELECT id, court_id, case_id, start_date, end_date, reason,
               COALESCE(statutory_reference, '') as "statutory_reference!",
               days_excluded, order_reference
        FROM excludable_delays
        WHERE case_id = $1 AND court_id = $2
        ORDER BY start_date ASC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Delete an excludable delay. Returns true if a row was deleted.
pub async fn delete_delay(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM excludable_delays WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
