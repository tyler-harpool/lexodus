use shared_types::{AppError, CreateJudicialOpinionRequest, JudicialOpinion, UpdateJudicialOpinionRequest};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

/// Insert a new judicial opinion.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: CreateJudicialOpinionRequest,
) -> Result<JudicialOpinion, AppError> {
    let status = req.status.as_deref().unwrap_or("Draft");

    let row = sqlx::query_as!(
        JudicialOpinion,
        r#"
        INSERT INTO judicial_opinions
            (court_id, case_id, case_name, docket_number, author_judge_id,
             author_judge_name, opinion_type, disposition, title, syllabus,
             content, status, keywords)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, court_id, case_id, case_name, docket_number,
                  author_judge_id, author_judge_name, opinion_type,
                  COALESCE(disposition, '') as "disposition!",
                  title,
                  COALESCE(syllabus, '') as "syllabus!",
                  content, status, is_published, is_precedential,
                  citation_volume, citation_reporter, citation_page,
                  filed_at, published_at, keywords, created_at, updated_at
        "#,
        court_id,
        req.case_id,
        req.case_name,
        req.docket_number,
        req.author_judge_id,
        req.author_judge_name,
        req.opinion_type,
        req.disposition.as_deref(),
        req.title,
        req.syllabus.as_deref(),
        req.content,
        status,
        &req.keywords as &[String],
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a judicial opinion by ID within a specific court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<JudicialOpinion>, AppError> {
    let row = sqlx::query_as!(
        JudicialOpinion,
        r#"
        SELECT id, court_id, case_id, case_name, docket_number,
               author_judge_id, author_judge_name, opinion_type,
               COALESCE(disposition, '') as "disposition!",
               title,
               COALESCE(syllabus, '') as "syllabus!",
               content, status, is_published, is_precedential,
               citation_volume, citation_reporter, citation_page,
               filed_at, published_at, keywords, created_at, updated_at
        FROM judicial_opinions
        WHERE id = $1 AND court_id = $2
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all opinions for a given case within a court.
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<JudicialOpinion>, AppError> {
    let rows = sqlx::query_as!(
        JudicialOpinion,
        r#"
        SELECT id, court_id, case_id, case_name, docket_number,
               author_judge_id, author_judge_name, opinion_type,
               COALESCE(disposition, '') as "disposition!",
               title,
               COALESCE(syllabus, '') as "syllabus!",
               content, status, is_published, is_precedential,
               citation_volume, citation_reporter, citation_page,
               filed_at, published_at, keywords, created_at, updated_at
        FROM judicial_opinions
        WHERE case_id = $1 AND court_id = $2
        ORDER BY created_at DESC
        "#,
        case_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List all opinions authored by a given judge within a court.
pub async fn list_by_judge(
    pool: &Pool<Postgres>,
    court_id: &str,
    judge_id: Uuid,
) -> Result<Vec<JudicialOpinion>, AppError> {
    let rows = sqlx::query_as!(
        JudicialOpinion,
        r#"
        SELECT id, court_id, case_id, case_name, docket_number,
               author_judge_id, author_judge_name, opinion_type,
               COALESCE(disposition, '') as "disposition!",
               title,
               COALESCE(syllabus, '') as "syllabus!",
               content, status, is_published, is_precedential,
               citation_volume, citation_reporter, citation_page,
               filed_at, published_at, keywords, created_at, updated_at
        FROM judicial_opinions
        WHERE author_judge_id = $1 AND court_id = $2
        ORDER BY created_at DESC
        "#,
        judge_id,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Search opinions by title, case name, or keywords text (ILIKE).
pub async fn search(
    pool: &Pool<Postgres>,
    court_id: &str,
    query: &str,
) -> Result<Vec<JudicialOpinion>, AppError> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query_as!(
        JudicialOpinion,
        r#"
        SELECT id, court_id, case_id, case_name, docket_number,
               author_judge_id, author_judge_name, opinion_type,
               COALESCE(disposition, '') as "disposition!",
               title,
               COALESCE(syllabus, '') as "syllabus!",
               content, status, is_published, is_precedential,
               citation_volume, citation_reporter, citation_page,
               filed_at, published_at, keywords, created_at, updated_at
        FROM judicial_opinions
        WHERE court_id = $1
          AND (title ILIKE $2 OR case_name ILIKE $2 OR array_to_string(keywords, ',') ILIKE $2)
        ORDER BY created_at DESC
        "#,
        court_id,
        pattern,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update an opinion with only the provided fields.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: UpdateJudicialOpinionRequest,
) -> Result<Option<JudicialOpinion>, AppError> {
    let row = sqlx::query_as!(
        JudicialOpinion,
        r#"
        UPDATE judicial_opinions SET
            title       = COALESCE($3, title),
            content     = COALESCE($4, content),
            status      = COALESCE($5, status),
            disposition = COALESCE($6, disposition),
            syllabus    = COALESCE($7, syllabus),
            keywords    = COALESCE($8, keywords),
            updated_at  = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING id, court_id, case_id, case_name, docket_number,
                  author_judge_id, author_judge_name, opinion_type,
                  COALESCE(disposition, '') as "disposition!",
                  title,
                  COALESCE(syllabus, '') as "syllabus!",
                  content, status, is_published, is_precedential,
                  citation_volume, citation_reporter, citation_page,
                  filed_at, published_at, keywords, created_at, updated_at
        "#,
        id,
        court_id,
        req.title.as_deref(),
        req.content.as_deref(),
        req.status.as_deref(),
        req.disposition.as_deref(),
        req.syllabus.as_deref(),
        req.keywords.as_deref().map(|k| k as &[String]),
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// List all opinions for a court with optional search, pagination.
pub async fn list_all(
    pool: &Pool<Postgres>,
    court_id: &str,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<JudicialOpinion>, i64), AppError> {
    let search = q.map(|s| format!("%{}%", s.to_lowercase()));

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM judicial_opinions
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(title) LIKE $2 OR LOWER(case_name) LIKE $2 OR LOWER(author_judge_name) LIKE $2)
        "#,
        court_id,
        search.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    let rows = sqlx::query_as!(
        JudicialOpinion,
        r#"
        SELECT id, court_id, case_id, case_name, docket_number,
               author_judge_id, author_judge_name, opinion_type,
               COALESCE(disposition, '') as "disposition!",
               title,
               COALESCE(syllabus, '') as "syllabus!",
               content, status, is_published, is_precedential,
               citation_volume, citation_reporter, citation_page,
               filed_at, published_at, keywords, created_at, updated_at
        FROM judicial_opinions
        WHERE court_id = $1
          AND ($2::TEXT IS NULL OR LOWER(title) LIKE $2 OR LOWER(case_name) LIKE $2 OR LOWER(author_judge_name) LIKE $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        court_id,
        search.as_deref(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok((rows, total))
}

/// Delete an opinion. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM judicial_opinions WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}
