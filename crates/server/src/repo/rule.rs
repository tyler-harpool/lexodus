use chrono::{DateTime, Utc};
use shared_types::{AppError, Rule};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new rule. Returns the created rule.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    name: &str,
    description: &str,
    source: &str,
    category: &str,
    priority: i32,
    status: &str,
    jurisdiction: Option<&str>,
    citation: Option<&str>,
    effective_date: Option<DateTime<Utc>>,
    conditions: &serde_json::Value,
    actions: &serde_json::Value,
    triggers: Option<&serde_json::Value>,
) -> Result<Rule, AppError> {
    let default_triggers = serde_json::Value::Array(vec![]);
    let final_triggers = triggers.unwrap_or(&default_triggers);

    let row = sqlx::query_as!(
        Rule,
        r#"
        INSERT INTO rules (
            court_id, name, description, source, category,
            priority, status, jurisdiction, citation,
            effective_date, conditions, actions, triggers
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING
            id, court_id, name, description, source, category,
            priority, status, jurisdiction, citation,
            effective_date, expiration_date, supersedes_rule_id,
            conditions, actions, triggers, created_at, updated_at
        "#,
        court_id,
        name,
        description,
        source,
        category,
        priority,
        status,
        jurisdiction,
        citation,
        effective_date,
        conditions,
        actions,
        final_triggers,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a rule by ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Rule>, AppError> {
    let row = sqlx::query_as!(
        Rule,
        r#"
        SELECT
            id, court_id, name, description, source, category,
            priority, status, jurisdiction, citation,
            effective_date, expiration_date, supersedes_rule_id,
            conditions, actions, triggers, created_at, updated_at
        FROM rules
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

/// List all rules for a court, ordered by priority descending.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<Rule>, AppError> {
    let rows = sqlx::query_as!(
        Rule,
        r#"
        SELECT
            id, court_id, name, description, source, category,
            priority, status, jurisdiction, citation,
            effective_date, expiration_date, supersedes_rule_id,
            conditions, actions, triggers, created_at, updated_at
        FROM rules
        WHERE court_id = $1
        ORDER BY priority DESC, name
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List rules filtered by category.
pub async fn list_by_category(
    pool: &Pool<Postgres>,
    court_id: &str,
    category: &str,
) -> Result<Vec<Rule>, AppError> {
    let rows = sqlx::query_as!(
        Rule,
        r#"
        SELECT
            id, court_id, name, description, source, category,
            priority, status, jurisdiction, citation,
            effective_date, expiration_date, supersedes_rule_id,
            conditions, actions, triggers, created_at, updated_at
        FROM rules
        WHERE court_id = $1 AND category = $2
        ORDER BY priority DESC, name
        "#,
        court_id,
        category,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List rules filtered by source (trigger type).
pub async fn list_by_trigger(
    pool: &Pool<Postgres>,
    court_id: &str,
    trigger: &str,
) -> Result<Vec<Rule>, AppError> {
    let rows = sqlx::query_as!(
        Rule,
        r#"
        SELECT
            id, court_id, name, description, source, category,
            priority, status, jurisdiction, citation,
            effective_date, expiration_date, supersedes_rule_id,
            conditions, actions, triggers, created_at, updated_at
        FROM rules
        WHERE court_id = $1 AND source = $2
        ORDER BY priority DESC, name
        "#,
        court_id,
        trigger,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List rules filtered by jurisdiction.
pub async fn list_by_jurisdiction(
    pool: &Pool<Postgres>,
    court_id: &str,
    jurisdiction: &str,
) -> Result<Vec<Rule>, AppError> {
    let rows = sqlx::query_as!(
        Rule,
        r#"
        SELECT
            id, court_id, name, description, source, category,
            priority, status, jurisdiction, citation,
            effective_date, expiration_date, supersedes_rule_id,
            conditions, actions, triggers, created_at, updated_at
        FROM rules
        WHERE court_id = $1 AND jurisdiction = $2
        ORDER BY priority DESC, name
        "#,
        court_id,
        jurisdiction,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update a rule using a read-modify-write pattern. Returns None if not found.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    source: Option<&str>,
    category: Option<&str>,
    priority: Option<i32>,
    status: Option<&str>,
    jurisdiction: Option<&str>,
    citation: Option<&str>,
    effective_date: Option<DateTime<Utc>>,
    conditions: Option<&serde_json::Value>,
    actions: Option<&serde_json::Value>,
    triggers: Option<&serde_json::Value>,
) -> Result<Option<Rule>, AppError> {
    let existing = match find_by_id(pool, court_id, id).await? {
        Some(r) => r,
        None => return Ok(None),
    };

    let final_name = name.unwrap_or(&existing.name);
    let final_description = description.or(existing.description.as_deref());
    let final_source = source.unwrap_or(&existing.source);
    let final_category = category.unwrap_or(&existing.category);
    let final_priority = priority.unwrap_or(existing.priority);
    let final_status = status.unwrap_or(&existing.status);
    let final_jurisdiction = jurisdiction.or(existing.jurisdiction.as_deref());
    let final_citation = citation.or(existing.citation.as_deref());
    let final_effective = effective_date.or(existing.effective_date);
    let final_conditions = conditions.unwrap_or(&existing.conditions);
    let final_actions = actions.unwrap_or(&existing.actions);
    let final_triggers = triggers.unwrap_or(&existing.triggers);

    let row = sqlx::query_as!(
        Rule,
        r#"
        UPDATE rules SET
            name = $3, description = $4, source = $5, category = $6,
            priority = $7, status = $8, jurisdiction = $9, citation = $10,
            effective_date = $11, conditions = $12, actions = $13,
            triggers = $14, updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING
            id, court_id, name, description, source, category,
            priority, status, jurisdiction, citation,
            effective_date, expiration_date, supersedes_rule_id,
            conditions, actions, triggers, created_at, updated_at
        "#,
        id,
        court_id,
        final_name,
        final_description,
        final_source,
        final_category,
        final_priority,
        final_status,
        final_jurisdiction,
        final_citation,
        final_effective,
        final_conditions,
        final_actions,
        final_triggers,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a rule. Returns true if a row was actually deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM rules WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// List active rules for a court, optionally filtered by category.
/// Used by the rule evaluation engine.
pub async fn list_active(
    pool: &Pool<Postgres>,
    court_id: &str,
    category: Option<&str>,
) -> Result<Vec<Rule>, AppError> {
    let rows = if let Some(cat) = category {
        sqlx::query_as!(
            Rule,
            r#"
            SELECT
                id, court_id, name, description, source, category,
                priority, status, jurisdiction, citation,
                effective_date, expiration_date, supersedes_rule_id,
                conditions, actions, created_at, updated_at
            FROM rules
            WHERE court_id = $1 AND status = 'Active' AND category = $2
            ORDER BY priority DESC, name
            "#,
            court_id,
            cat,
        )
        .fetch_all(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?
    } else {
        sqlx::query_as!(
            Rule,
            r#"
            SELECT
                id, court_id, name, description, source, category,
                priority, status, jurisdiction, citation,
                effective_date, expiration_date, supersedes_rule_id,
                conditions, actions, created_at, updated_at
            FROM rules
            WHERE court_id = $1 AND status = 'Active'
            ORDER BY priority DESC, name
            "#,
            court_id,
        )
        .fetch_all(pool)
        .await
        .map_err(SqlxErrorExt::into_app_error)?
    };

    Ok(rows)
}
