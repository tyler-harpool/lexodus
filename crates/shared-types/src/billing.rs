use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user's prepaid billing account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingAccount {
    pub id: Uuid,
    pub user_id: i64,
    pub balance_cents: i64,
    pub account_type: String,
    pub stripe_customer_id: Option<String>,
    pub created_at: String,
}

/// A recorded billable action (search, document view, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchTransaction {
    pub id: Uuid,
    pub user_id: i64,
    pub query: String,
    pub court_ids: Vec<String>,
    pub result_count: i32,
    pub fee_cents: i32,
    pub action_type: String,
    pub created_at: String,
}

/// Paginated list of search transactions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionListResponse {
    pub transactions: Vec<SearchTransaction>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// A fee schedule entry defining the cost per action type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchFeeScheduleEntry {
    pub id: Uuid,
    pub action_type: String,
    pub fee_cents: i32,
    pub cap_cents: Option<i32>,
    pub description: String,
    pub effective_date: String,
}

/// Summary statistics for the admin billing dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingSummary {
    pub total_revenue_cents: i64,
    pub total_searches: i64,
    pub active_accounts: i64,
    pub top_users: Vec<UserBillingStats>,
}

/// Per-user billing stats for admin dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserBillingStats {
    pub user_id: i64,
    pub username: String,
    pub total_searches: i64,
    pub total_fee_cents: i64,
}

/// Request to create a Stripe Checkout session for account top-up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopUpRequest {
    pub amount_cents: i64,
}
