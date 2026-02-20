use dioxus::prelude::*;

// ── PDF Generation ────────────────────────────────────────

/// Generate court order PDF via Typst compilation. Returns base64-encoded PDF bytes.
#[server]
pub async fn generate_order_pdf(
    court_id: String,
    order_id: String,
    signed: bool,
) -> Result<String, ServerFnError> {
    use crate::db::get_db;
    use base64::Engine;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&order_id).map_err(|_| ServerFnError::new("Invalid UUID"))?;

    let order = sqlx::query_as!(
        shared_types::JudicialOrder,
        r#"
        SELECT o.id, o.court_id, o.case_id, o.judge_id,
               j.name as judge_name,
               COALESCE(cc.case_number, cv.case_number) as "case_number?",
               o.order_type, o.title, o.content,
               o.status, o.is_sealed, o.signer_name, o.signed_at, o.signature_hash,
               o.issued_at, o.effective_date, o.expiration_date, o.related_motions,
               o.created_at, o.updated_at
        FROM judicial_orders o
        LEFT JOIN judges j ON o.judge_id = j.id AND j.court_id = o.court_id
        LEFT JOIN criminal_cases cc ON o.case_id = cc.id
        LEFT JOIN civil_cases cv ON o.case_id = cv.id
        WHERE o.id = $1 AND o.court_id = $2
        "#,
        uuid,
        &court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Order not found"))?;

    let signer_name = order
        .signer_name
        .as_deref()
        .unwrap_or("N/A")
        .to_string();
    let signed_date = order
        .signed_at
        .map(|d| d.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "Not yet signed".to_string());
    let order_date = order.created_at.format("%B %d, %Y").to_string();

    use crate::typst::escape_typst;

    // Build the data bindings that precede the court-order template
    let data_bindings = format!(
        r##"#let court_name = "{court_name}"
#let case_id = "{case_id}"
#let order_type = "{order_type}"
#let title = "{title}"
#let status = "{status}"
#let content_body = "{content}"
#let show_signature = {show_sig}
#let signer_name = "{signer_name}"
#let signed_date = "{signed_date}"
#let order_date = "{order_date}"

"##,
        court_name = escape_typst(&court_id),
        case_id = escape_typst(&order.case_id.to_string()),
        order_type = escape_typst(&order.order_type),
        title = escape_typst(&order.title),
        status = escape_typst(&order.status),
        content = escape_typst(&order.content),
        show_sig = signed,
        signer_name = escape_typst(&signer_name),
        signed_date = escape_typst(&signed_date),
        order_date = escape_typst(&order_date),
    );

    // Read the template file and prepend data bindings
    let template = include_str!("../../../../templates/court-order.typ");
    let full_source = format!("{data_bindings}{template}");

    // Compile via shared typst module
    let pdf_bytes = crate::typst::compile_typst(&full_source)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Return base64-encoded PDF bytes
    let b64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);
    Ok(b64)
}
