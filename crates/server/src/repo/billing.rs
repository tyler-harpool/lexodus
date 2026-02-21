use shared_types::AppError;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// Row from billing_accounts.
pub struct BillingAccountRow {
    pub id: Uuid,
    pub user_id: i64,
    pub balance_cents: i64,
    pub account_type: String,
    pub stripe_customer_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Get or create a billing account for a user.
pub async fn get_or_create_account(
    pool: &Pool<Postgres>,
    user_id: i64,
) -> Result<BillingAccountRow, AppError> {
    let existing = sqlx::query_as!(
        BillingAccountRow,
        r#"SELECT id, user_id, balance_cents, account_type,
                  stripe_customer_id, created_at
           FROM billing_accounts WHERE user_id = $1"#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(format!("billing account query: {}", e)))?;

    if let Some(row) = existing {
        return Ok(row);
    }

    let row = sqlx::query_as!(
        BillingAccountRow,
        r#"INSERT INTO billing_accounts (user_id)
           VALUES ($1)
           RETURNING id, user_id, balance_cents, account_type,
                     stripe_customer_id, created_at"#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::internal(format!("billing account create: {}", e)))?;

    Ok(row)
}

/// Deduct a fee from a billing account. Returns the new balance.
pub async fn deduct_fee(
    pool: &Pool<Postgres>,
    user_id: i64,
    fee_cents: i32,
) -> Result<i64, AppError> {
    let row = sqlx::query_scalar!(
        r#"UPDATE billing_accounts
           SET balance_cents = balance_cents - $2, updated_at = NOW()
           WHERE user_id = $1
           RETURNING balance_cents"#,
        user_id,
        fee_cents as i64
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::internal(format!("billing deduct: {}", e)))?;

    Ok(row)
}

/// Credit a billing account (e.g., after Stripe payment).
pub async fn credit_account(
    pool: &Pool<Postgres>,
    user_id: i64,
    amount_cents: i64,
) -> Result<i64, AppError> {
    let row = sqlx::query_scalar!(
        r#"UPDATE billing_accounts
           SET balance_cents = balance_cents + $2, updated_at = NOW()
           WHERE user_id = $1
           RETURNING balance_cents"#,
        user_id,
        amount_cents
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::internal(format!("billing credit: {}", e)))?;

    Ok(row)
}

/// Record a search transaction and deduct the fee.
pub async fn record_search_transaction(
    pool: &Pool<Postgres>,
    user_id: i64,
    query: &str,
    court_ids: &[String],
    result_count: i32,
    action_type: &str,
) -> Result<(Uuid, i32), AppError> {
    // Look up fee from schedule
    let fee_cents: i32 = sqlx::query_scalar!(
        "SELECT fee_cents FROM search_fee_schedule WHERE action_type = $1",
        action_type
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(format!("fee lookup: {}", e)))?
    .unwrap_or(0);

    let id = sqlx::query_scalar!(
        r#"INSERT INTO search_transactions (user_id, query, court_ids, result_count, fee_cents, action_type)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id"#,
        user_id,
        query,
        court_ids,
        result_count,
        fee_cents,
        action_type
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::internal(format!("transaction insert: {}", e)))?;

    // Deduct fee if user has a billing account (exempt users skip)
    let account = get_or_create_account(pool, user_id).await?;
    if account.account_type != "exempt" && fee_cents > 0 {
        deduct_fee(pool, user_id, fee_cents).await?;
    }

    Ok((id, fee_cents))
}

/// Row from search_transactions.
pub struct TransactionRow {
    pub id: Uuid,
    pub user_id: i64,
    pub query: String,
    pub court_ids: Vec<String>,
    pub result_count: i32,
    pub fee_cents: i32,
    pub action_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List transactions for a user with pagination.
pub async fn list_transactions(
    pool: &Pool<Postgres>,
    user_id: i64,
    page: i64,
    per_page: i64,
) -> Result<(Vec<TransactionRow>, i64), AppError> {
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM search_transactions WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::internal(format!("transaction count: {}", e)))?
    .unwrap_or(0);

    let offset = (page.max(1) - 1) * per_page;
    let rows = sqlx::query_as!(
        TransactionRow,
        r#"SELECT id, user_id, query, court_ids, result_count, fee_cents, action_type, created_at
           FROM search_transactions
           WHERE user_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
        user_id,
        per_page,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::internal(format!("transaction list: {}", e)))?;

    Ok((rows, total))
}

/// Admin: summary row from aggregate query.
pub struct SummaryRow {
    pub total_revenue_cents: Option<i64>,
    pub total_searches: Option<i64>,
}

/// Admin: get billing summary stats.
pub async fn billing_summary(pool: &Pool<Postgres>) -> Result<SummaryRow, AppError> {
    let row = sqlx::query_as!(
        SummaryRow,
        r#"SELECT COALESCE(SUM(fee_cents::BIGINT), 0) as total_revenue_cents,
                  COUNT(*) as total_searches
           FROM search_transactions"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::internal(format!("billing summary: {}", e)))?;

    Ok(row)
}
