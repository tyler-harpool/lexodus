use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Order validation constants ──────────────────────────────────────

/// Valid order type values matching the DB CHECK constraint.
pub const ORDER_TYPES: &[&str] = &[
    "Scheduling", "Protective", "Restraining", "Dismissal", "Sentencing",
    "Detention", "Release", "Discovery", "Sealing", "Contempt",
    "Procedural", "Standing", "Other",
];

/// Valid order status values matching the DB CHECK constraint.
pub const ORDER_STATUSES: &[&str] = &[
    "Draft", "Pending Signature", "Signed", "Filed", "Vacated", "Amended", "Superseded",
];

/// Check whether an order type string is valid.
pub fn is_valid_order_type(s: &str) -> bool {
    ORDER_TYPES.contains(&s)
}

/// Check whether an order status string is valid.
pub fn is_valid_order_status(s: &str) -> bool {
    ORDER_STATUSES.contains(&s)
}

// ── Order list query params ──────────────────────────────────────────

/// Query parameters for listing orders with optional filters.
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::IntoParams))]
pub struct OrderListParams {
    pub case_id: Option<String>,
    pub judge_id: Option<String>,
    pub status: Option<String>,
    pub is_sealed: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ── JudicialOrder DB struct ─────────────────────────────────────────

/// A judicial order issued in a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct JudicialOrder {
    pub id: Uuid,
    pub court_id: String,
    pub case_id: Uuid,
    pub judge_id: Uuid,
    /// Resolved judge name from LEFT JOIN judges.
    pub judge_name: Option<String>,
    /// OrderType enum stored as text (e.g. "Scheduling", "Protective", "Warrant").
    pub order_type: String,
    pub title: String,
    pub content: String,
    /// OrderStatus enum stored as text (e.g. "Draft", "Signed", "Filed", "Vacated").
    pub status: String,
    pub is_sealed: bool,
    pub signer_name: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
    pub signature_hash: Option<String>,
    pub issued_at: Option<DateTime<Utc>>,
    pub effective_date: Option<DateTime<Utc>>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub related_motions: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── JudicialOrder API response ──────────────────────────────────────

/// API response shape for a judicial order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JudicialOrderResponse {
    pub id: String,
    pub case_id: String,
    pub judge_id: String,
    /// Resolved judge name from the judges table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judge_name: Option<String>,
    pub order_type: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub is_sealed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    pub related_motions: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<JudicialOrder> for JudicialOrderResponse {
    fn from(o: JudicialOrder) -> Self {
        Self {
            id: o.id.to_string(),
            case_id: o.case_id.to_string(),
            judge_id: o.judge_id.to_string(),
            judge_name: o.judge_name,
            order_type: o.order_type,
            title: o.title,
            content: o.content,
            status: o.status,
            is_sealed: o.is_sealed,
            signer_name: o.signer_name,
            signed_at: o.signed_at.map(|dt| dt.to_rfc3339()),
            signature_hash: o.signature_hash,
            issued_at: o.issued_at.map(|dt| dt.to_rfc3339()),
            effective_date: o.effective_date.map(|dt| dt.to_rfc3339()),
            expiration_date: o.expiration_date.map(|dt| dt.to_rfc3339()),
            related_motions: o.related_motions.into_iter().map(|id| id.to_string()).collect(),
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.updated_at.to_rfc3339(),
        }
    }
}

// ── JudicialOrder request types ─────────────────────────────────────

/// Request to create a new judicial order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateJudicialOrderRequest {
    pub case_id: Uuid,
    pub judge_id: Uuid,
    pub order_type: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub is_sealed: Option<bool>,
    #[serde(default)]
    pub effective_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub expiration_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub related_motions: Vec<Uuid>,
}

/// Request to update a judicial order (all fields optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateJudicialOrderRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_sealed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_motions: Option<Vec<Uuid>>,
}

// ── OrderTemplate DB struct ─────────────────────────────────────────

/// A reusable template for judicial orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct OrderTemplate {
    pub id: Uuid,
    pub court_id: String,
    pub order_type: String,
    pub name: String,
    pub description: Option<String>,
    pub content_template: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── OrderTemplate API response ──────────────────────────────────────

/// API response shape for an order template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OrderTemplateResponse {
    pub id: String,
    pub order_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content_template: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<OrderTemplate> for OrderTemplateResponse {
    fn from(t: OrderTemplate) -> Self {
        Self {
            id: t.id.to_string(),
            order_type: t.order_type,
            name: t.name,
            description: t.description,
            content_template: t.content_template,
            is_active: t.is_active,
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}

// ── OrderTemplate request types ─────────────────────────────────────

/// Request to create a new order template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateOrderTemplateRequest {
    pub order_type: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub content_template: String,
}

/// Request to update an order template (all fields optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateOrderTemplateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

// ── Order workflow request types ────────────────────────────────────

/// Request to sign an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SignOrderRequest {
    pub signed_by: String,
}

/// Request to issue an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IssueOrderRequest {
    pub issued_by: String,
}

/// Request to serve an order on parties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ServeOrderRequest {
    pub served_to: Vec<String>,
    pub service_method: String,
}

/// Statistics about orders in a court.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OrderStatistics {
    pub total: i64,
    pub by_type: serde_json::Value,
    pub by_status: serde_json::Value,
    pub avg_days_to_sign: Option<f64>,
}

/// Request to create an order from a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateFromTemplateRequest {
    pub template_id: Uuid,
    pub case_id: Uuid,
    pub judge_id: Option<Uuid>,
    pub variables: serde_json::Value,
}

/// Request to generate order content from variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GenerateContentRequest {
    pub variables: serde_json::Value,
}
