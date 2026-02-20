use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::db::get_db;

// ── Queue Server Functions ─────────────────────────────

#[server]
pub async fn search_queue(
    court_id: String,
    status: Option<String>,
    queue_type: Option<String>,
    priority: Option<i32>,
    assigned_to: Option<i64>,
    case_id: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
) -> Result<shared_types::QueueSearchResponse, ServerFnError> {
    let pool = get_db().await;
    let case_uuid = case_id.as_deref()
        .map(|s| uuid::Uuid::parse_str(s))
        .transpose()
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (items, total) = crate::repo::queue::search(
        pool,
        &court_id,
        status.as_deref(),
        queue_type.as_deref(),
        priority,
        assigned_to,
        case_uuid,
        offset.unwrap_or(0),
        limit.unwrap_or(20).clamp(1, 100),
    )
    .await
    .map_err(|e| ServerFnError::new(e.message))?;

    Ok(shared_types::QueueSearchResponse {
        items: items.into_iter().map(shared_types::QueueItemResponse::from).collect(),
        total,
    })
}

#[server]
pub async fn get_queue_stats(
    court_id: String,
    user_id: Option<i64>,
) -> Result<shared_types::QueueStats, ServerFnError> {
    let pool = get_db().await;
    let stats = crate::repo::queue::stats(pool, &court_id, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.message))?;
    Ok(stats)
}

#[server]
pub async fn claim_queue_item_fn(
    court_id: String,
    id: String,
    user_id: i64,
) -> Result<shared_types::QueueItemResponse, ServerFnError> {
    let pool = get_db().await;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let item = crate::repo::queue::claim(pool, &court_id, uuid, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Item not available for claiming".to_string()))?;
    Ok(shared_types::QueueItemResponse::from(item))
}

#[server]
pub async fn release_queue_item_fn(
    court_id: String,
    id: String,
    user_id: i64,
) -> Result<shared_types::QueueItemResponse, ServerFnError> {
    let pool = get_db().await;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let item = crate::repo::queue::release(pool, &court_id, uuid, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Item not found or not assigned to user".to_string()))?;
    Ok(shared_types::QueueItemResponse::from(item))
}

#[server]
pub async fn advance_queue_item_fn(
    court_id: String,
    id: String,
) -> Result<shared_types::QueueItemResponse, ServerFnError> {
    let pool = get_db().await;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let current = crate::repo::queue::find_by_id(pool, &court_id, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Queue item not found".to_string()))?;

    let (next, status, completed_at) = match shared_types::next_step(&current.queue_type, &current.current_step) {
        Some(next) => {
            if shared_types::next_step(&current.queue_type, next).is_none() {
                (next, "completed", Some(chrono::Utc::now()))
            } else {
                (next, "processing", None)
            }
        }
        None => ("completed", "completed", Some(chrono::Utc::now())),
    };

    let item = crate::repo::queue::advance(pool, &court_id, uuid, next, status, completed_at)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Queue item not found".to_string()))?;
    Ok(shared_types::QueueItemResponse::from(item))
}

#[server]
pub async fn reject_queue_item_fn(
    court_id: String,
    id: String,
    reason: String,
) -> Result<shared_types::QueueItemResponse, ServerFnError> {
    let pool = get_db().await;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let item = crate::repo::queue::reject(pool, &court_id, uuid, &reason)
        .await
        .map_err(|e| ServerFnError::new(e.message))?
        .ok_or_else(|| ServerFnError::new("Queue item not found".to_string()))?;
    Ok(shared_types::QueueItemResponse::from(item))
}
