use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::AppError;
use crate::tenant::CourtId;

// ---------------------------------------------------------------------------
// Shared types for PDF generation
// ---------------------------------------------------------------------------

/// Generic request body for PDF generation endpoints.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct PdfGenerateRequest {
    pub case_id: Uuid,
    #[serde(default)]
    pub judge_id: Option<Uuid>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body_text: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Response containing generated HTML (to be rendered as PDF by a downstream service).
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PdfGenerateResponse {
    pub case_id: String,
    pub document_type: String,
    pub format: String,
    pub html: String,
    pub metadata: serde_json::Value,
}

/// Request body for batch PDF generation.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct BatchPdfRequest {
    pub documents: Vec<BatchPdfItem>,
}

/// A single item in a batch PDF generation request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct BatchPdfItem {
    pub document_type: String,
    pub case_id: Uuid,
    #[serde(default)]
    pub judge_id: Option<Uuid>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body_text: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Helper: build HTML for a document type
// ---------------------------------------------------------------------------

fn build_html(
    court_id: &str,
    doc_type: &str,
    case_id: &Uuid,
    title: Option<&str>,
    body_text: Option<&str>,
    signed: bool,
    judge_id: Option<&Uuid>,
) -> String {
    let doc_title = title.unwrap_or(doc_type);
    let body = body_text.unwrap_or("Content pending.");
    let signature_block = if signed {
        let jid = judge_id.map(|u| u.to_string()).unwrap_or_else(|| "N/A".to_string());
        format!(
            r#"<div class="signature-block">
                <hr/>
                <p>Electronically signed by Judge (ID: {})</p>
                <p>Court: {}</p>
            </div>"#,
            jid, court_id
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8"/>
    <title>{doc_title}</title>
    <style>
        body {{ font-family: 'Times New Roman', serif; margin: 1in; }}
        .header {{ text-align: center; margin-bottom: 2em; }}
        .court-name {{ font-size: 14pt; font-weight: bold; text-transform: uppercase; }}
        .doc-type {{ font-size: 12pt; margin-top: 1em; }}
        .case-info {{ margin: 1em 0; }}
        .body-text {{ margin: 2em 0; line-height: 1.6; }}
        .signature-block {{ margin-top: 3em; }}
    </style>
</head>
<body>
    <div class="header">
        <div class="court-name">United States District Court - {court_id}</div>
        <div class="doc-type">{doc_type}</div>
    </div>
    <div class="case-info">
        <p><strong>Case ID:</strong> {case_id}</p>
    </div>
    <div class="body-text">
        {body}
    </div>
    {signature_block}
</body>
</html>"#,
    )
}

fn make_response(
    court_id: &str,
    doc_type: &str,
    format: &str,
    req: &PdfGenerateRequest,
    signed: bool,
) -> PdfGenerateResponse {
    let html = build_html(
        court_id,
        doc_type,
        &req.case_id,
        req.title.as_deref(),
        req.body_text.as_deref(),
        signed,
        req.judge_id.as_ref(),
    );

    PdfGenerateResponse {
        case_id: req.case_id.to_string(),
        document_type: doc_type.to_string(),
        format: format.to_string(),
        html,
        metadata: req.metadata.clone().unwrap_or(serde_json::json!({})),
    }
}

// ---------------------------------------------------------------------------
// POST /api/pdf/rule16b
// ---------------------------------------------------------------------------

/// Generate a Rule 16(b) Scheduling Order document.
#[utoipa::path(
    post,
    path = "/api/pdf/rule16b",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_rule16b(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Rule 16(b) Scheduling Order", "html", &body, false)))
}

/// Generate a Rule 16(b) Scheduling Order in a specified format.
#[utoipa::path(
    post,
    path = "/api/pdf/rule16b/{format}",
    params(
        ("format" = String, Path, description = "Output format"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_rule16b_fmt(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(format): Path<String>,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Rule 16(b) Scheduling Order", &format, &body, false)))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/signed/rule16b
// ---------------------------------------------------------------------------

/// Generate a signed Rule 16(b) Scheduling Order.
#[utoipa::path(
    post,
    path = "/api/pdf/signed/rule16b",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated signed document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_signed_rule16b(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Rule 16(b) Scheduling Order", "html", &body, true)))
}

/// Generate a signed Rule 16(b) Scheduling Order in a specified format.
#[utoipa::path(
    post,
    path = "/api/pdf/signed/rule16b/{format}",
    params(
        ("format" = String, Path, description = "Output format"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated signed document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_signed_rule16b_fmt(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(format): Path<String>,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Rule 16(b) Scheduling Order", &format, &body, true)))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/court-order
// ---------------------------------------------------------------------------

/// Generate a Court Order document.
#[utoipa::path(
    post,
    path = "/api/pdf/court-order",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_court_order(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Court Order", "html", &body, true)))
}

/// Generate a Court Order in a specified format.
#[utoipa::path(
    post,
    path = "/api/pdf/court-order/{format}",
    params(
        ("format" = String, Path, description = "Output format"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_court_order_fmt(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(format): Path<String>,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Court Order", &format, &body, true)))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/minute-entry
// ---------------------------------------------------------------------------

/// Generate a Minute Entry document.
#[utoipa::path(
    post,
    path = "/api/pdf/minute-entry",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_minute_entry(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Minute Entry", "html", &body, false)))
}

/// Generate a Minute Entry in a specified format.
#[utoipa::path(
    post,
    path = "/api/pdf/minute-entry/{format}",
    params(
        ("format" = String, Path, description = "Output format"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_minute_entry_fmt(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(format): Path<String>,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Minute Entry", &format, &body, false)))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/waiver-indictment
// ---------------------------------------------------------------------------

/// Generate a Waiver of Indictment document.
#[utoipa::path(
    post,
    path = "/api/pdf/waiver-indictment",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_waiver(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Waiver of Indictment", "html", &body, false)))
}

/// Generate a Waiver of Indictment in a specified format.
#[utoipa::path(
    post,
    path = "/api/pdf/waiver-indictment/{format}",
    params(
        ("format" = String, Path, description = "Output format"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_waiver_fmt(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(format): Path<String>,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Waiver of Indictment", &format, &body, false)))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/conditions-release
// ---------------------------------------------------------------------------

/// Generate a Conditions of Release document.
#[utoipa::path(
    post,
    path = "/api/pdf/conditions-release",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_conditions(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Conditions of Release", "html", &body, true)))
}

/// Generate a Conditions of Release document in a specified format.
#[utoipa::path(
    post,
    path = "/api/pdf/conditions-release/{format}",
    params(
        ("format" = String, Path, description = "Output format"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_conditions_fmt(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(format): Path<String>,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Conditions of Release", &format, &body, true)))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/criminal-judgment
// ---------------------------------------------------------------------------

/// Generate a Criminal Judgment document.
#[utoipa::path(
    post,
    path = "/api/pdf/criminal-judgment",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_judgment(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Criminal Judgment", "html", &body, true)))
}

/// Generate a Criminal Judgment document in a specified format.
#[utoipa::path(
    post,
    path = "/api/pdf/criminal-judgment/{format}",
    params(
        ("format" = String, Path, description = "Output format"),
        ("X-Court-District" = String, Header, description = "Court district ID")
    ),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated document", body = PdfGenerateResponse)),
    tag = "pdf"
)]
pub async fn generate_judgment_fmt(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Path(format): Path<String>,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<Json<PdfGenerateResponse>, AppError> {
    Ok(Json(make_response(&court.0, "Criminal Judgment", &format, &body, true)))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/batch
// ---------------------------------------------------------------------------

/// Batch-generate multiple documents at once.
#[utoipa::path(
    post,
    path = "/api/pdf/batch",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = BatchPdfRequest,
    responses((status = 200, description = "Batch generation result", body = Vec<PdfGenerateResponse>)),
    tag = "pdf"
)]
pub async fn batch_generate(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<BatchPdfRequest>,
) -> Result<Json<Vec<PdfGenerateResponse>>, AppError> {
    let mut results = Vec::with_capacity(body.documents.len());

    for item in &body.documents {
        let signed = matches!(
            item.document_type.as_str(),
            "court-order" | "conditions-release" | "criminal-judgment"
        );
        let doc_type_label = match item.document_type.as_str() {
            "rule16b" => "Rule 16(b) Scheduling Order",
            "court-order" => "Court Order",
            "minute-entry" => "Minute Entry",
            "waiver-indictment" => "Waiver of Indictment",
            "conditions-release" => "Conditions of Release",
            "criminal-judgment" => "Criminal Judgment",
            other => other,
        };

        let html = build_html(
            &court.0,
            doc_type_label,
            &item.case_id,
            item.title.as_deref(),
            item.body_text.as_deref(),
            signed,
            item.judge_id.as_ref(),
        );

        results.push(PdfGenerateResponse {
            case_id: item.case_id.to_string(),
            document_type: item.document_type.clone(),
            format: "html".to_string(),
            html,
            metadata: item.metadata.clone().unwrap_or(serde_json::json!({})),
        });
    }

    Ok(Json(results))
}
