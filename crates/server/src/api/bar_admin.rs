use dioxus::prelude::*;

// ── Representation Server Functions ────────────────────

#[server]
pub async fn list_representations_by_case(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::RepresentationResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = representation::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::RepresentationResponse::from).collect())
}

#[server]
pub async fn list_active_representations(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::RepresentationResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = representation::list_active_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::RepresentationResponse::from).collect())
}

#[server]
pub async fn get_representation(
    court_id: String,
    id: String,
) -> Result<shared_types::RepresentationResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = representation::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::RepresentationResponse::from(row))
}

#[server]
pub async fn create_representation(
    court_id: String,
    body: shared_types::CreateRepresentationRequest,
) -> Result<shared_types::RepresentationResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;

    let pool = get_db().await;
    let row = representation::create(pool, &court_id, &body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::RepresentationResponse::from(row))
}

#[server]
pub async fn end_representation(
    court_id: String,
    id: String,
    reason: Option<String>,
) -> Result<shared_types::RepresentationResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::representation;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = representation::end_representation(pool, &court_id, uuid, reason.as_deref())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::RepresentationResponse::from(row))
}

// ── Charge Server Functions ────────────────────────────

#[server]
pub async fn list_charges_by_defendant(
    court_id: String,
    defendant_id: String,
) -> Result<Vec<shared_types::ChargeResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;
    use uuid::Uuid;

    let pool = get_db().await;
    let def_uuid = Uuid::parse_str(&defendant_id)
        .map_err(|_| ServerFnError::new("Invalid defendant_id UUID"))?;
    let rows = charge::list_by_defendant(pool, &court_id, def_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::ChargeResponse::from).collect())
}

#[server]
pub async fn get_charge(
    court_id: String,
    id: String,
) -> Result<shared_types::ChargeResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = charge::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::ChargeResponse::from(row))
}

#[server]
pub async fn create_charge(
    court_id: String,
    body: shared_types::CreateChargeRequest,
) -> Result<shared_types::ChargeResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;

    let pool = get_db().await;
    let row = charge::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::ChargeResponse::from(row))
}

#[server]
pub async fn update_charge(
    court_id: String,
    id: String,
    body: shared_types::UpdateChargeRequest,
) -> Result<shared_types::ChargeResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = charge::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::ChargeResponse::from(row))
}

#[server]
pub async fn delete_charge(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::charge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    charge::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Motion Server Functions ────────────────────────────

#[server]
pub async fn list_motions_by_case(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::MotionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = motion::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::MotionResponse::from).collect())
}

#[server]
pub async fn get_motion(
    court_id: String,
    id: String,
) -> Result<shared_types::MotionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = motion::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::MotionResponse::from(row))
}

#[server]
pub async fn create_motion(
    court_id: String,
    body: shared_types::CreateMotionRequest,
) -> Result<shared_types::MotionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;

    let pool = get_db().await;
    let row = motion::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::MotionResponse::from(row))
}

#[server]
pub async fn update_motion(
    court_id: String,
    id: String,
    body: shared_types::UpdateMotionRequest,
) -> Result<shared_types::MotionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = motion::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::MotionResponse::from(row))
}

#[server]
pub async fn delete_motion(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    motion::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Case Note Server Functions ─────────────────────────

#[server]
pub async fn list_case_notes(
    court_id: String,
    case_id: String,
) -> Result<Vec<shared_types::CaseNoteResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = case_note::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::CaseNoteResponse::from).collect())
}

#[server]
pub async fn get_case_note(
    court_id: String,
    id: String,
) -> Result<shared_types::CaseNoteResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = case_note::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::CaseNoteResponse::from(row))
}

#[server]
pub async fn create_case_note(
    court_id: String,
    body: shared_types::CreateCaseNoteRequest,
) -> Result<shared_types::CaseNoteResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;

    let pool = get_db().await;
    let row = case_note::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::CaseNoteResponse::from(row))
}

#[server]
pub async fn update_case_note(
    court_id: String,
    id: String,
    body: shared_types::UpdateCaseNoteRequest,
) -> Result<shared_types::CaseNoteResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = case_note::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::CaseNoteResponse::from(row))
}

#[server]
pub async fn delete_case_note(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_note;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    case_note::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Rule Server Functions ──────────────────────────────

#[server]
pub async fn list_rules(court_id: String) -> Result<Vec<shared_types::RuleResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::rule;

    let pool = get_db().await;
    let rows = rule::list_all(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::RuleResponse::from).collect())
}

#[server]
pub async fn get_rule(
    court_id: String,
    id: String,
) -> Result<shared_types::RuleResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::rule;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = rule::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(shared_types::RuleResponse::from(row))
}

#[server]
pub async fn create_rule(
    court_id: String,
    body: shared_types::CreateRuleRequest,
) -> Result<shared_types::RuleResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::rule;

    let pool = get_db().await;
    let row = rule::create(
        pool,
        &court_id,
        &body.name,
        &body.description,
        &body.source,
        &body.category,
        body.priority,
        body.status.as_deref().unwrap_or("active"),
        body.jurisdiction.as_deref(),
        body.citation.as_deref(),
        None,
        body.conditions.as_ref().unwrap_or(&serde_json::json!({})),
        body.actions.as_ref().unwrap_or(&serde_json::json!({})),
        body.triggers.as_ref(),
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::RuleResponse::from(row))
}

#[server]
pub async fn delete_rule(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::rule;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    rule::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Conflict Check Server Functions ────────────────────

#[server]
pub async fn run_conflict_check(
    court_id: String,
    attorney_id: String,
    party_names: Vec<String>,
) -> Result<Vec<String>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::conflict_check;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = conflict_check::run_check(pool, &court_id, att_uuid, &party_names)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows)
}

#[server]
pub async fn list_conflicts_by_attorney(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::ConflictCheckResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::conflict_check;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = conflict_check::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::ConflictCheckResponse::from).collect())
}

// ── Bar Admissions ──────────────────────────────────────────

#[server]
pub async fn list_bar_admissions(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::BarAdmissionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = bar_admission::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::BarAdmissionResponse::from).collect())
}

#[server]
pub async fn create_bar_admission(
    court_id: String,
    attorney_id: String,
    body: shared_types::CreateBarAdmissionRequest,
) -> Result<shared_types::BarAdmissionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = bar_admission::create(pool, &court_id, att_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::BarAdmissionResponse::from(row))
}

#[server]
pub async fn delete_bar_admission(
    court_id: String,
    attorney_id: String,
    state: String,
) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::bar_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    bar_admission::delete_by_state(pool, &court_id, att_uuid, &state)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Federal Admissions ──────────────────────────────────────

#[server]
pub async fn list_federal_admissions(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::FederalAdmissionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = federal_admission::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::FederalAdmissionResponse::from).collect())
}

#[server]
pub async fn create_federal_admission(
    court_id: String,
    attorney_id: String,
    body: shared_types::CreateFederalAdmissionRequest,
) -> Result<shared_types::FederalAdmissionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = federal_admission::create(pool, &court_id, att_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::FederalAdmissionResponse::from(row))
}

#[server]
pub async fn delete_federal_admission(
    court_id: String,
    attorney_id: String,
    court_name: String,
) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::federal_admission;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    federal_admission::delete_by_court_name(pool, &court_id, att_uuid, &court_name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── CJA Appointments ────────────────────────────────────────

#[server]
pub async fn list_cja_appointments(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::CjaAppointmentResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = cja_appointment::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::CjaAppointmentResponse::from).collect())
}

#[server]
pub async fn create_cja_appointment(
    court_id: String,
    attorney_id: String,
    body: shared_types::CreateCjaAppointmentRequest,
) -> Result<shared_types::CjaAppointmentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = cja_appointment::create(pool, &court_id, att_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::CjaAppointmentResponse::from(row))
}

#[server]
pub async fn list_pending_cja_vouchers(
    court_id: String,
) -> Result<Vec<shared_types::CjaAppointmentResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::cja_appointment;

    let pool = get_db().await;
    let rows = cja_appointment::list_pending_vouchers(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::CjaAppointmentResponse::from).collect())
}

// ── Pro Hac Vice ────────────────────────────────────────────

#[server]
pub async fn list_pro_hac_vice(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::ProHacViceResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = pro_hac_vice::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::ProHacViceResponse::from).collect())
}

#[server]
pub async fn create_pro_hac_vice(
    court_id: String,
    attorney_id: String,
    body: shared_types::CreateProHacViceRequest,
) -> Result<shared_types::ProHacViceResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = pro_hac_vice::create(pool, &court_id, att_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::ProHacViceResponse::from(row))
}

#[server]
pub async fn update_pro_hac_vice_status(
    court_id: String,
    attorney_id: String,
    case_id: String,
    new_status: String,
) -> Result<shared_types::ProHacViceResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::pro_hac_vice;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let case_uuid = Uuid::parse_str(&case_id)
        .map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let row = pro_hac_vice::update_status(pool, &court_id, att_uuid, case_uuid, &new_status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("PHV record not found"))?;
    Ok(shared_types::ProHacViceResponse::from(row))
}

// ── Discipline Records ──────────────────────────────────────

#[server]
pub async fn list_discipline_records(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::DisciplineRecordResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::discipline;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = discipline::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::DisciplineRecordResponse::from).collect())
}

#[server]
pub async fn create_discipline_record(
    court_id: String,
    attorney_id: String,
    body: shared_types::CreateDisciplineRecordRequest,
) -> Result<shared_types::DisciplineRecordResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::discipline;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = discipline::create(pool, &court_id, att_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::DisciplineRecordResponse::from(row))
}

// ── Practice Areas ──────────────────────────────────────────

#[server]
pub async fn list_practice_areas(
    court_id: String,
    attorney_id: String,
) -> Result<Vec<shared_types::PracticeAreaResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let rows = practice_area::list_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(shared_types::PracticeAreaResponse::from).collect())
}

#[server]
pub async fn add_practice_area(
    court_id: String,
    attorney_id: String,
    area: String,
) -> Result<shared_types::PracticeAreaResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = practice_area::add(pool, &court_id, att_uuid, &area)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::PracticeAreaResponse::from(row))
}

#[server]
pub async fn remove_practice_area(
    court_id: String,
    attorney_id: String,
    area: String,
) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::practice_area;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    practice_area::remove(pool, &court_id, att_uuid, &area)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── ECF Registration ────────────────────────────────────────

#[server]
pub async fn get_ecf_registration(
    court_id: String,
    attorney_id: String,
) -> Result<Option<shared_types::EcfRegistrationResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = ecf_registration::find_by_attorney(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(row.map(shared_types::EcfRegistrationResponse::from))
}

#[server]
pub async fn upsert_ecf_registration(
    court_id: String,
    attorney_id: String,
    status: String,
) -> Result<shared_types::EcfRegistrationResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    let row = ecf_registration::upsert(pool, &court_id, att_uuid, &status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(shared_types::EcfRegistrationResponse::from(row))
}

#[server]
pub async fn revoke_ecf_registration(
    court_id: String,
    attorney_id: String,
) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::ecf_registration;
    use uuid::Uuid;

    let pool = get_db().await;
    let att_uuid = Uuid::parse_str(&attorney_id)
        .map_err(|_| ServerFnError::new("Invalid attorney_id UUID"))?;
    ecf_registration::revoke(pool, &court_id, att_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
