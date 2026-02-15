use axum::Json;

use shared_types::{AppError, FederalRule};

/// GET /api/deadlines/federal-rules
///
/// Returns a list of common federal rules with their deadline calculations.
/// These are hardcoded reference data based on the Federal Rules of Criminal Procedure.
#[utoipa::path(
    get,
    path = "/api/deadlines/federal-rules",
    responses(
        (status = 200, description = "List of federal rules", body = Vec<FederalRule>)
    ),
    tag = "federal_rules"
)]
pub async fn list_federal_rules() -> Result<Json<Vec<FederalRule>>, AppError> {
    let rules = vec![
        FederalRule {
            rule_code: "FRCP-5".to_string(),
            title: "Initial Appearance".to_string(),
            description: "Defendant must be taken before a magistrate judge without unnecessary delay".to_string(),
            days: 1,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "FRCP-5.1".to_string(),
            title: "Preliminary Hearing".to_string(),
            description: "Preliminary hearing within 14 days if detained, 21 days if released".to_string(),
            days: 14,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "FRCP-10".to_string(),
            title: "Arraignment".to_string(),
            description: "Arraignment must occur without unnecessary delay after indictment".to_string(),
            days: 14,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "FRCP-12".to_string(),
            title: "Pretrial Motions".to_string(),
            description: "Pretrial motions deadline set by the court, typically 14 days before trial".to_string(),
            days: 14,
            business_days_only: true,
        },
        FederalRule {
            rule_code: "FRCP-16".to_string(),
            title: "Discovery".to_string(),
            description: "Government must disclose certain information within 14 days of arraignment".to_string(),
            days: 14,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "FRCP-29".to_string(),
            title: "Motion for Judgment of Acquittal".to_string(),
            description: "Motion must be filed within 14 days after guilty verdict or plea".to_string(),
            days: 14,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "FRCP-33".to_string(),
            title: "Motion for New Trial".to_string(),
            description: "Motion must be filed within 14 days after verdict".to_string(),
            days: 14,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "FRCP-35".to_string(),
            title: "Correcting or Reducing a Sentence".to_string(),
            description: "Motion to correct sentence within 14 days of sentencing".to_string(),
            days: 14,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "STA-70".to_string(),
            title: "Speedy Trial Act - 70 Day Rule".to_string(),
            description: "Trial must begin within 70 days of indictment or initial appearance".to_string(),
            days: 70,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "STA-30".to_string(),
            title: "Speedy Trial Act - 30 Day Indictment".to_string(),
            description: "Indictment must be filed within 30 days of arrest".to_string(),
            days: 30,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "FRAP-4".to_string(),
            title: "Notice of Appeal".to_string(),
            description: "Notice of appeal must be filed within 14 days after entry of judgment".to_string(),
            days: 14,
            business_days_only: false,
        },
        FederalRule {
            rule_code: "18USC3161".to_string(),
            title: "Speedy Trial Act Compliance".to_string(),
            description: "Statutory speedy trial requirement per 18 U.S.C. 3161".to_string(),
            days: 70,
            business_days_only: false,
        },
    ];

    Ok(Json(rules))
}
