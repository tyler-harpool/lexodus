use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use shared_types::AppError;
use crate::tenant::CourtId;
use crate::typst::{build_document_source, compile_typst, DocumentParams};

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

/// A single result in a batch PDF generation response.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct BatchPdfResponseItem {
    pub case_id: String,
    pub document_type: String,
    pub filename: String,
    pub pdf_base64: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `DocumentParams` from the common request fields.
fn build_params(
    court_id: &str,
    doc_type: &str,
    req_case_id: &Uuid,
    title: Option<&str>,
    body_text: Option<&str>,
    signed: bool,
    judge_id: Option<&Uuid>,
) -> DocumentParams {
    let today = chrono::Utc::now().format("%B %d, %Y").to_string();
    DocumentParams {
        court_name: court_id.to_string(),
        doc_type: doc_type.to_string(),
        case_id: req_case_id.to_string(),
        title: title.unwrap_or(doc_type).to_string(),
        content_body: body_text.unwrap_or("Content pending.").to_string(),
        show_signature: signed,
        signer_id: judge_id
            .map(|u| u.to_string())
            .unwrap_or_else(|| "N/A".to_string()),
        document_date: today,
    }
}

/// Generate a PDF and return it as an `application/pdf` HTTP response.
async fn generate_document_pdf(
    court_id: &str,
    doc_type: &str,
    req: &PdfGenerateRequest,
    signed: bool,
) -> Result<impl IntoResponse, AppError> {
    let params = build_params(
        court_id,
        doc_type,
        &req.case_id,
        req.title.as_deref(),
        req.body_text.as_deref(),
        signed,
        req.judge_id.as_ref(),
    );

    let source = build_document_source(&params);
    let pdf_bytes = compile_typst(&source).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "inline; filename=\"document.pdf\"",
            ),
        ],
        pdf_bytes,
    ))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/rule16b
// ---------------------------------------------------------------------------

/// Generate a Rule 16(b) Scheduling Order PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/rule16b",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_rule16b(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    generate_document_pdf(&court.0, "Rule 16(b) Scheduling Order", &body, false).await
}

// ---------------------------------------------------------------------------
// POST /api/pdf/signed/rule16b
// ---------------------------------------------------------------------------

/// Generate a signed Rule 16(b) Scheduling Order PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/signed/rule16b",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated signed PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_signed_rule16b(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    generate_document_pdf(&court.0, "Rule 16(b) Scheduling Order", &body, true).await
}

// ---------------------------------------------------------------------------
// POST /api/pdf/court-order
// ---------------------------------------------------------------------------

/// Generate a Court Order PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/court-order",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_court_order(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    generate_document_pdf(&court.0, "Court Order", &body, true).await
}

// ---------------------------------------------------------------------------
// POST /api/pdf/minute-entry
// ---------------------------------------------------------------------------

/// Generate a Minute Entry PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/minute-entry",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_minute_entry(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    generate_document_pdf(&court.0, "Minute Entry", &body, false).await
}

// ---------------------------------------------------------------------------
// POST /api/pdf/waiver-indictment
// ---------------------------------------------------------------------------

/// Generate a Waiver of Indictment PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/waiver-indictment",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_waiver(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    generate_document_pdf(&court.0, "Waiver of Indictment", &body, false).await
}

// ---------------------------------------------------------------------------
// POST /api/pdf/conditions-release
// ---------------------------------------------------------------------------

/// Generate a Conditions of Release PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/conditions-release",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_conditions(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    generate_document_pdf(&court.0, "Conditions of Release", &body, true).await
}

// ---------------------------------------------------------------------------
// POST /api/pdf/criminal-judgment
// ---------------------------------------------------------------------------

/// Generate a Criminal Judgment PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/criminal-judgment",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_judgment(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    generate_document_pdf(&court.0, "Criminal Judgment", &body, true).await
}

// ---------------------------------------------------------------------------
// Civil case request types
// ---------------------------------------------------------------------------

/// Request body for JS-44 Cover Sheet generation.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct Js44CoverSheetRequest {
    pub case_id: Uuid,
    pub plaintiff_name: String,
    pub defendant_name: String,
    #[serde(default)]
    pub county: Option<String>,
    #[serde(default)]
    pub attorney_info: Option<String>,
    pub jurisdiction_basis: String,
    pub nature_of_suit: String,
    #[serde(default)]
    pub nos_description: Option<String>,
    #[serde(default)]
    pub cause_of_action: Option<String>,
    #[serde(default)]
    pub class_action: Option<bool>,
    #[serde(default)]
    pub jury_demand: Option<String>,
    #[serde(default)]
    pub amount_in_controversy: Option<f64>,
}

/// Request body for Civil Summons generation.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CivilSummonsRequest {
    pub case_id: Uuid,
    pub plaintiff_name: String,
    pub defendant_name: String,
    #[serde(default)]
    pub attorney_info: Option<String>,
}

// ---------------------------------------------------------------------------
// POST /api/pdf/js44-cover-sheet
// ---------------------------------------------------------------------------

/// Generate a JS-44 Civil Cover Sheet PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/js44-cover-sheet",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = Js44CoverSheetRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_js44_cover_sheet(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<Js44CoverSheetRequest>,
) -> Result<impl IntoResponse, AppError> {
    use crate::typst::{build_civil_cover_sheet_source, CivilCoverSheetParams};

    let today = chrono::Utc::now().format("%B %d, %Y").to_string();
    let params = CivilCoverSheetParams {
        court_name: court.0.clone(),
        case_number: body.case_id.to_string(),
        plaintiff_name: body.plaintiff_name,
        defendant_name: body.defendant_name,
        county: body.county.unwrap_or_default(),
        attorney_info: body.attorney_info.unwrap_or_default(),
        jurisdiction_basis: body.jurisdiction_basis,
        nature_of_suit: body.nature_of_suit,
        nos_description: body.nos_description.unwrap_or_default(),
        cause_of_action: body.cause_of_action.unwrap_or_default(),
        class_action: body.class_action.unwrap_or(false),
        jury_demand: body.jury_demand.unwrap_or_else(|| "none".to_string()),
        amount_in_controversy: body
            .amount_in_controversy
            .map(|a| format!("{:.2}", a))
            .unwrap_or_default(),
        document_date: today,
    };

    let source = build_civil_cover_sheet_source(&params);
    let pdf_bytes = compile_typst(&source).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "inline; filename=\"js44-cover-sheet.pdf\"",
            ),
        ],
        pdf_bytes,
    ))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/civil-summons
// ---------------------------------------------------------------------------

/// Generate a Civil Summons PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/civil-summons",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = CivilSummonsRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_civil_summons(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<CivilSummonsRequest>,
) -> Result<impl IntoResponse, AppError> {
    use crate::typst::{build_civil_summons_source, CivilSummonsParams};

    let today = chrono::Utc::now().format("%B %d, %Y").to_string();
    let params = CivilSummonsParams {
        court_name: court.0.clone(),
        case_number: body.case_id.to_string(),
        plaintiff_name: body.plaintiff_name,
        defendant_name: body.defendant_name,
        attorney_info: body.attorney_info.unwrap_or_default(),
        document_date: today,
    };

    let source = build_civil_summons_source(&params);
    let pdf_bytes = compile_typst(&source).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "inline; filename=\"civil-summons.pdf\"",
            ),
        ],
        pdf_bytes,
    ))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/civil-scheduling-order
// ---------------------------------------------------------------------------

/// Generate a Civil Scheduling Order (FRCP Rule 16(b)) PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/civil-scheduling-order",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_civil_scheduling_order(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    use crate::typst::build_civil_scheduling_order_source;

    let params = build_params(
        &court.0,
        "Scheduling Order (FRCP 16(b))",
        &body.case_id,
        body.title.as_deref(),
        body.body_text.as_deref(),
        true,
        body.judge_id.as_ref(),
    );

    let source = build_civil_scheduling_order_source(&params);
    let pdf_bytes = compile_typst(&source).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "inline; filename=\"civil-scheduling-order.pdf\"",
            ),
        ],
        pdf_bytes,
    ))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/civil-judgment
// ---------------------------------------------------------------------------

/// Generate a Civil Judgment (FRCP Rule 58) PDF.
#[utoipa::path(
    post,
    path = "/api/pdf/civil-judgment",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = PdfGenerateRequest,
    responses((status = 200, description = "Generated PDF", content_type = "application/pdf")),
    tag = "pdf"
)]
pub async fn generate_civil_judgment(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<PdfGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    use crate::typst::build_civil_judgment_source;

    let params = build_params(
        &court.0,
        "Judgment in a Civil Case",
        &body.case_id,
        body.title.as_deref(),
        body.body_text.as_deref(),
        true,
        body.judge_id.as_ref(),
    );

    let source = build_civil_judgment_source(&params);
    let pdf_bytes = compile_typst(&source).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "inline; filename=\"civil-judgment.pdf\"",
            ),
        ],
        pdf_bytes,
    ))
}

// ---------------------------------------------------------------------------
// POST /api/pdf/batch
// ---------------------------------------------------------------------------

/// Batch-generate multiple document PDFs at once.
#[utoipa::path(
    post,
    path = "/api/pdf/batch",
    params(("X-Court-District" = String, Header, description = "Court district ID")),
    request_body = BatchPdfRequest,
    responses((status = 200, description = "Batch generation result", body = Vec<BatchPdfResponseItem>)),
    tag = "pdf"
)]
pub async fn batch_generate(
    State(_pool): State<Pool<Postgres>>,
    court: CourtId,
    Json(body): Json<BatchPdfRequest>,
) -> Result<Json<Vec<BatchPdfResponseItem>>, AppError> {
    let mut results = Vec::with_capacity(body.documents.len());

    for item in &body.documents {
        let signed = matches!(
            item.document_type.as_str(),
            "court-order"
                | "conditions-release"
                | "criminal-judgment"
                | "civil-scheduling-order"
                | "civil-judgment"
        );
        let doc_type_label = match item.document_type.as_str() {
            "rule16b" => "Rule 16(b) Scheduling Order",
            "court-order" => "Court Order",
            "minute-entry" => "Minute Entry",
            "waiver-indictment" => "Waiver of Indictment",
            "conditions-release" => "Conditions of Release",
            "criminal-judgment" => "Criminal Judgment",
            "js44-cover-sheet" => "JS-44 Cover Sheet",
            "civil-summons" => "Civil Summons",
            "civil-scheduling-order" => "Scheduling Order (FRCP 16(b))",
            "civil-judgment" => "Judgment in a Civil Case",
            other => other,
        };

        let params = build_params(
            &court.0,
            doc_type_label,
            &item.case_id,
            item.title.as_deref(),
            item.body_text.as_deref(),
            signed,
            item.judge_id.as_ref(),
        );

        let source = build_document_source(&params);
        let pdf_bytes = compile_typst(&source).await?;
        let pdf_base64 =
            base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

        let filename = format!(
            "{}_{}.pdf",
            item.document_type,
            item.case_id.to_string().split('-').next().unwrap_or("doc")
        );

        results.push(BatchPdfResponseItem {
            case_id: item.case_id.to_string(),
            document_type: item.document_type.clone(),
            filename,
            pdf_base64,
        });
    }

    Ok(Json(results))
}
