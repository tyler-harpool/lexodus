use shared_types::AppError;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// A single row from the `fee_schedule` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct FeeScheduleEntry {
    pub id: Uuid,
    pub court_id: String,
    pub fee_id: String,
    pub category: String,
    pub description: String,
    pub amount_cents: i32,
    pub statute: Option<String>,
    pub waivable: bool,
    pub waiver_form: Option<String>,
    pub cap_cents: Option<i32>,
    pub cap_description: Option<String>,
    pub effective_date: chrono::NaiveDate,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request body for creating a new fee schedule entry.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateFeeRequest {
    pub fee_id: String,
    pub category: String,
    pub description: String,
    pub amount_cents: i32,
    pub statute: Option<String>,
    pub waivable: Option<bool>,
    pub waiver_form: Option<String>,
    pub cap_cents: Option<i32>,
    pub cap_description: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
}

/// Request body for partially updating an existing fee schedule entry.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateFeeRequest {
    pub description: Option<String>,
    pub amount_cents: Option<i32>,
    pub statute: Option<String>,
    pub waivable: Option<bool>,
    pub waiver_form: Option<String>,
    pub cap_cents: Option<i32>,
    pub cap_description: Option<String>,
}

/// List all active fee schedule entries for the given court.
pub async fn list_active(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<FeeScheduleEntry>, AppError> {
    sqlx::query_as!(
        FeeScheduleEntry,
        r#"
        SELECT id, court_id, fee_id, category, description, amount_cents,
               statute, waivable, waiver_form, cap_cents, cap_description,
               effective_date, active, created_at, updated_at
        FROM fee_schedule
        WHERE court_id = $1 AND active = true
        ORDER BY category ASC, fee_id ASC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Fetch a single fee schedule entry by ID, scoped to a court.
pub async fn get_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<FeeScheduleEntry>, AppError> {
    sqlx::query_as!(
        FeeScheduleEntry,
        r#"
        SELECT id, court_id, fee_id, category, description, amount_cents,
               statute, waivable, waiver_form, cap_cents, cap_description,
               effective_date, active, created_at, updated_at
        FROM fee_schedule
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Insert a new fee schedule entry. Returns the created row.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateFeeRequest,
) -> Result<FeeScheduleEntry, AppError> {
    let waivable = req.waivable.unwrap_or(false);

    sqlx::query_as!(
        FeeScheduleEntry,
        r#"
        INSERT INTO fee_schedule
            (court_id, fee_id, category, description, amount_cents,
             statute, waivable, waiver_form, cap_cents, cap_description,
             effective_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                COALESCE($11, CURRENT_DATE))
        RETURNING id, court_id, fee_id, category, description, amount_cents,
                  statute, waivable, waiver_form, cap_cents, cap_description,
                  effective_date, active, created_at, updated_at
        "#,
        court_id,
        req.fee_id,
        req.category,
        req.description,
        req.amount_cents,
        req.statute,
        waivable,
        req.waiver_form,
        req.cap_cents,
        req.cap_description,
        req.effective_date,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Partially update a fee schedule entry using the COALESCE pattern.
/// Returns the updated row, or None if the entry was not found.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateFeeRequest,
) -> Result<Option<FeeScheduleEntry>, AppError> {
    sqlx::query_as!(
        FeeScheduleEntry,
        r#"
        UPDATE fee_schedule SET
            description     = COALESCE($3, description),
            amount_cents    = COALESCE($4, amount_cents),
            statute         = COALESCE($5, statute),
            waivable        = COALESCE($6, waivable),
            waiver_form     = COALESCE($7, waiver_form),
            cap_cents       = COALESCE($8, cap_cents),
            cap_description = COALESCE($9, cap_description),
            updated_at      = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, fee_id, category, description, amount_cents,
                  statute, waivable, waiver_form, cap_cents, cap_description,
                  effective_date, active, created_at, updated_at
        "#,
        id,
        court_id,
        req.description,
        req.amount_cents,
        req.statute,
        req.waivable,
        req.waiver_form,
        req.cap_cents,
        req.cap_description,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// Soft-delete a fee schedule entry by setting active=false.
/// Returns true if a row was updated, false if not found.
pub async fn soft_delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE fee_schedule
        SET active = false, updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
