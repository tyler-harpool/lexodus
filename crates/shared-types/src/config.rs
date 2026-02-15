use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain Structs
// ---------------------------------------------------------------------------

/// A feature flag for toggling functionality.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct FeatureFlag {
    pub id: Uuid,
    pub feature_path: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A stored electronic signature for a judge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct JudgeSignature {
    pub id: Uuid,
    pub court_id: String,
    pub judge_id: Uuid,
    pub signature_data: String,
    pub created_at: DateTime<Utc>,
}

/// A configuration override scoped to a district or judge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct ConfigOverride {
    pub id: Uuid,
    pub court_id: String,
    /// Scope of the override: "district" or "judge".
    pub scope: String,
    pub scope_id: String,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// ConfigOverride Request/Response DTOs
// ---------------------------------------------------------------------------

/// API response for a configuration override.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConfigOverrideResponse {
    pub id: String,
    pub scope: String,
    pub scope_id: String,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ConfigOverride> for ConfigOverrideResponse {
    fn from(c: ConfigOverride) -> Self {
        Self {
            id: c.id.to_string(),
            scope: c.scope,
            scope_id: c.scope_id,
            config_key: c.config_key,
            config_value: c.config_value,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

/// Request body for setting a configuration override.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetConfigOverrideRequest {
    pub config_key: String,
    pub config_value: serde_json::Value,
}

// ---------------------------------------------------------------------------
// JudgeSignature Request/Response DTOs
// ---------------------------------------------------------------------------

/// API response for a judge signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JudgeSignatureResponse {
    pub id: String,
    pub judge_id: String,
    pub signature_data: String,
    pub created_at: String,
}

impl From<JudgeSignature> for JudgeSignatureResponse {
    fn from(s: JudgeSignature) -> Self {
        Self {
            id: s.id.to_string(),
            judge_id: s.judge_id.to_string(),
            signature_data: s.signature_data,
            created_at: s.created_at.to_rfc3339(),
        }
    }
}

/// Request body for creating or updating a judge signature.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateSignatureRequest {
    pub judge_id: Uuid,
    pub signature_data: String,
}

// ---------------------------------------------------------------------------
// FeatureFlag Request/Response DTOs
// ---------------------------------------------------------------------------

/// API response for a feature flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FeatureFlagResponse {
    pub id: String,
    pub feature_path: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<FeatureFlag> for FeatureFlagResponse {
    fn from(f: FeatureFlag) -> Self {
        Self {
            id: f.id.to_string(),
            feature_path: f.feature_path,
            enabled: f.enabled,
            description: f.description,
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
}

/// Request body for updating a feature flag.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateFeatureFlagRequest {
    pub feature_path: String,
    pub enabled: bool,
}

/// Request body for setting a feature override scoped to a court or judge.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetFeatureOverrideRequest {
    pub feature_path: String,
    pub enabled: bool,
    #[serde(default)]
    pub scope: Option<String>,
}

/// Response for a feature-enabled status check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FeatureStatusResponse {
    pub feature_path: String,
    pub enabled: bool,
}

/// Request body for previewing a configuration with overrides applied.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConfigPreviewRequest {
    pub overrides: serde_json::Value,
}
