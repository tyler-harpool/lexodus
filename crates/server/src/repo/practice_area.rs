use shared_types::{AppError, PracticeArea};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new practice area for an attorney.
pub async fn add(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    area: &str,
) -> Result<PracticeArea, AppError> {
    let row = sqlx::query_as::<_, PracticeArea>(
        r#"
        INSERT INTO attorney_practice_areas
            (court_id, attorney_id, area)
        VALUES ($1, $2, $3)
        RETURNING id, court_id, attorney_id, area, created_at
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(area)
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all practice areas for a specific attorney within a court.
pub async fn list_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<PracticeArea>, AppError> {
    let rows = sqlx::query_as::<_, PracticeArea>(
        r#"
        SELECT id, court_id, attorney_id, area, created_at
        FROM attorney_practice_areas
        WHERE court_id = $1 AND attorney_id = $2
        ORDER BY area ASC
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Remove a practice area by attorney and area name within a court.
pub async fn remove(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
    area: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM attorney_practice_areas
        WHERE court_id = $1 AND attorney_id = $2 AND area = $3
        "#,
    )
    .bind(court_id)
    .bind(attorney_id)
    .bind(area)
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
