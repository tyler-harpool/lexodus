use dioxus::prelude::*;
use shared_types::{
    CaseAssignmentResponse, CreateCaseAssignmentRequest, CreateJudgeConflictRequest,
    CreateJudgeRequest, CreateRecusalMotionRequest, JudgeConflictResponse, JudgeResponse,
    JudicialOrderResponse, MotionResponse, RecusalMotionResponse, UpdateJudgeRequest,
    UpdateRecusalRulingRequest,
};

// ── Judge Server Functions ─────────────────────────────

#[server]
pub async fn list_judges(court_id: String) -> Result<Vec<JudgeResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;

    let pool = get_db().await;
    let rows = judge::list_by_court(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(JudgeResponse::from).collect())
}

#[server]
pub async fn search_judges(
    court_id: String,
    query: String,
) -> Result<Vec<JudgeResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;

    let pool = get_db().await;
    let rows = judge::search(pool, &court_id, &query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(JudgeResponse::from).collect())
}

#[server]
pub async fn get_judge(court_id: String, id: String) -> Result<JudgeResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = judge::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(JudgeResponse::from(row))
}

#[server]
pub async fn create_judge(
    court_id: String,
    body: CreateJudgeRequest,
) -> Result<JudgeResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;

    let pool = get_db().await;
    let row = judge::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(JudgeResponse::from(row))
}

#[server]
pub async fn update_judge(
    court_id: String,
    id: String,
    body: UpdateJudgeRequest,
) -> Result<JudgeResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = judge::update(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(JudgeResponse::from(row))
}

#[server]
pub async fn delete_judge(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    judge::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Judge Conflict Server Functions ────────────────────

#[server]
pub async fn list_judge_conflicts(
    court_id: String,
    judge_id: String,
) -> Result<Vec<JudgeConflictResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge_conflict;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = judge_conflict::list_by_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(JudgeConflictResponse::from).collect())
}

#[server]
pub async fn create_judge_conflict(
    court_id: String,
    judge_id: String,
    body: CreateJudgeConflictRequest,
) -> Result<JudgeConflictResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge_conflict;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let row = judge_conflict::create(pool, &court_id, j_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(JudgeConflictResponse::from(row))
}

#[server]
pub async fn delete_judge_conflict(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::judge_conflict;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    judge_conflict::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Case Assignment Server Functions ───────────────────

#[server]
pub async fn list_case_assignments(
    court_id: String,
    case_id: String,
) -> Result<Vec<CaseAssignmentResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;
    use uuid::Uuid;

    let pool = get_db().await;
    let case_uuid =
        Uuid::parse_str(&case_id).map_err(|_| ServerFnError::new("Invalid case_id UUID"))?;
    let rows = case_assignment::list_by_case(pool, &court_id, case_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(CaseAssignmentResponse::from).collect())
}

#[server]
pub async fn create_case_assignment(
    court_id: String,
    body: CreateCaseAssignmentRequest,
) -> Result<CaseAssignmentResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;

    let pool = get_db().await;
    let row = case_assignment::create(pool, &court_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(CaseAssignmentResponse::from(row))
}

#[server]
pub async fn delete_case_assignment(court_id: String, id: String) -> Result<(), ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    case_assignment::delete(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Recusal Motion Server Functions ────────────────────

#[server]
pub async fn create_recusal(
    court_id: String,
    judge_id: String,
    body: CreateRecusalMotionRequest,
) -> Result<RecusalMotionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid =
        Uuid::parse_str(&judge_id).map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let row = recusal_motion::create(pool, &court_id, j_uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(RecusalMotionResponse::from(row))
}

#[server]
pub async fn list_pending_recusals(
    court_id: String,
) -> Result<Vec<RecusalMotionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;

    let pool = get_db().await;
    let rows = recusal_motion::list_pending(pool, &court_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(RecusalMotionResponse::from).collect())
}

#[server]
pub async fn rule_on_recusal(
    court_id: String,
    id: String,
    body: UpdateRecusalRulingRequest,
) -> Result<RecusalMotionResponse, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let uuid = Uuid::parse_str(&id).map_err(|_| ServerFnError::new("Invalid UUID"))?;
    let row = recusal_motion::update_ruling(pool, &court_id, uuid, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;
    Ok(RecusalMotionResponse::from(row))
}

// ── Judge Sub-Domain ────────────────────────────────────────

#[server]
pub async fn list_assignments_by_judge(
    court_id: String,
    judge_id: String,
) -> Result<Vec<CaseAssignmentResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::case_assignment;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = case_assignment::list_by_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(CaseAssignmentResponse::from).collect())
}

#[server]
pub async fn list_recusals_by_judge(
    court_id: String,
    judge_id: String,
) -> Result<Vec<RecusalMotionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::recusal_motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = recusal_motion::list_by_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows.into_iter().map(RecusalMotionResponse::from).collect())
}

// ── Pending Motions for Judge ───────────────────────────

/// List all pending motions on cases assigned to a specific judge.
///
/// This powers the judge dashboard "Pending Motions" work list by joining
/// the motions table with case_assignments and resolving the case number.
#[server]
pub async fn list_pending_motions_for_judge(
    court_id: String,
    judge_id: String,
) -> Result<Vec<MotionResponse>, ServerFnError> {
    use crate::db::get_db;
    use crate::repo::motion;
    use uuid::Uuid;

    let pool = get_db().await;
    let j_uuid = Uuid::parse_str(&judge_id)
        .map_err(|_| ServerFnError::new("Invalid judge_id UUID"))?;
    let rows = motion::list_pending_for_judge(pool, &court_id, j_uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows)
}

// ── Rule on Motion ─────────────────────────────────────

/// Judge rules on a motion — updates status, creates order.
#[server]
pub async fn rule_on_motion(
    court_id: String,
    motion_id: String,
    disposition: String,
    ruling_text: Option<String>,
    judge_id: String,
    judge_name: String,
) -> Result<JudicialOrderResponse, ServerFnError> {
    use crate::db::get_db;
    use shared_types::RuleMotionRequest;

    let pool = get_db().await;
    let body = RuleMotionRequest {
        disposition,
        ruling_text,
        judge_id,
        judge_name,
    };

    crate::rest::motion::rule_motion_inner(pool, &court_id, &motion_id, body)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
