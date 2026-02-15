use shared_types::{
    AppError, CreatePartyRequest, Party, Representation, UpdatePartyRequest,
    VALID_ENTITY_TYPES, VALID_PARTY_ROLES, VALID_PARTY_STATUSES, VALID_PARTY_TYPES,
    ServiceMethod,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error_convert::SqlxErrorExt;

// ---------------------------------------------------------------------------
// Lightweight query structs (not domain DTOs — repo-only)
// ---------------------------------------------------------------------------

/// Lightweight party info for dropdowns.
#[derive(Debug)]
pub struct PartyOption {
    pub id: Uuid,
    pub name: String,
    pub party_type: String,
}

/// Party info needed for auto-seeding service records and building NEF recipients.
#[derive(Debug, Clone)]
pub struct PartyServiceInfo {
    pub id: Uuid,
    pub name: String,
    pub party_type: String,
    pub service_method: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nef_sms_opt_in: bool,
}

/// Attorney contact info for NEF delivery via representations.
#[derive(Debug, Clone)]
pub struct AttorneyContact {
    pub party_id: Uuid,
    pub attorney_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nef_sms_opt_in: bool,
}

// ---------------------------------------------------------------------------
// Party CRUD
// ---------------------------------------------------------------------------

/// Insert a new party. Validates enums against the unified constant arrays.
pub async fn create(
    pool: &Pool<Postgres>,
    court_id: &str,
    req: &CreatePartyRequest,
) -> Result<Party, AppError> {
    let case_id = Uuid::parse_str(&req.case_id)
        .map_err(|_| AppError::bad_request("Invalid case_id UUID"))?;

    if !VALID_PARTY_TYPES.contains(&req.party_type.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid party_type '{}'. Valid: {:?}",
            req.party_type, VALID_PARTY_TYPES
        )));
    }
    if !VALID_ENTITY_TYPES.contains(&req.entity_type.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid entity_type '{}'. Valid: {:?}",
            req.entity_type, VALID_ENTITY_TYPES
        )));
    }

    let role = req.party_role.as_deref().unwrap_or("Lead");
    if !VALID_PARTY_ROLES.contains(&role) {
        return Err(AppError::bad_request(format!(
            "Invalid party_role '{}'. Valid: {:?}",
            role, VALID_PARTY_ROLES
        )));
    }

    let service_method = req.service_method.as_deref().unwrap_or("Electronic");
    // Validate service_method
    ServiceMethod::try_from(service_method).map_err(AppError::bad_request)?;

    let dob = req
        .date_of_birth
        .as_deref()
        .map(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::bad_request("Invalid date_of_birth format (expected YYYY-MM-DD)"))
        })
        .transpose()?;

    let pro_se = req.pro_se.unwrap_or(false);

    let row = sqlx::query_as!(
        Party,
        r#"
        INSERT INTO parties (
            court_id, case_id, party_type, party_role, name, entity_type,
            first_name, last_name, middle_name, organization_name,
            email, phone, date_of_birth, ssn_last_four, ein,
            address_street1, address_city, address_state, address_zip, address_country,
            service_method, pro_se
        )
        VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10,
            $11, $12, $13, $14, $15,
            $16, $17, $18, $19, $20,
            $21, $22
        )
        RETURNING
            id, court_id, case_id, party_type, party_role, name, entity_type,
            first_name, middle_name, last_name, date_of_birth,
            organization_name, address_street1, address_city, address_state,
            address_zip, address_country, phone, email,
            represented, pro_se, service_method, status,
            joined_date, terminated_date,
            ssn_last_four, ein, nef_sms_opt_in,
            created_at, updated_at
        "#,
        court_id,
        case_id,
        req.party_type,
        role,
        req.name,
        req.entity_type,
        req.first_name.as_deref(),
        req.last_name.as_deref(),
        req.middle_name.as_deref(),
        req.organization_name.as_deref(),
        req.email.as_deref(),
        req.phone.as_deref(),
        dob,
        req.ssn_last_four.as_deref(),
        req.ein.as_deref(),
        req.address.as_ref().map(|a| a.street1.as_str()),
        req.address.as_ref().map(|a| a.city.as_str()),
        req.address.as_ref().map(|a| a.state.as_str()),
        req.address.as_ref().map(|a| a.zip_code.as_str()),
        req.address.as_ref().map(|a| a.country.as_str()),
        service_method,
        pro_se,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Find a party by ID within a court.
pub async fn find_by_id(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Party>, AppError> {
    let row = sqlx::query_as!(
        Party,
        r#"
        SELECT
            id, court_id, case_id, party_type, party_role, name, entity_type,
            first_name, middle_name, last_name, date_of_birth,
            organization_name, address_street1, address_city, address_state,
            address_zip, address_country, phone, email,
            represented, pro_se, service_method, status,
            joined_date, terminated_date,
            ssn_last_four, ein, nef_sms_opt_in,
            created_at, updated_at
        FROM parties
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

/// Update a party using read-modify-write pattern.
pub async fn update(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    req: &UpdatePartyRequest,
) -> Result<Option<Party>, AppError> {
    // Validate optional enum fields
    if let Some(ref pt) = req.party_type {
        if !VALID_PARTY_TYPES.contains(&pt.as_str()) {
            return Err(AppError::bad_request(format!(
                "Invalid party_type '{}'. Valid: {:?}",
                pt, VALID_PARTY_TYPES
            )));
        }
    }
    if let Some(ref pr) = req.party_role {
        if !VALID_PARTY_ROLES.contains(&pr.as_str()) {
            return Err(AppError::bad_request(format!(
                "Invalid party_role '{}'. Valid: {:?}",
                pr, VALID_PARTY_ROLES
            )));
        }
    }
    if let Some(ref et) = req.entity_type {
        if !VALID_ENTITY_TYPES.contains(&et.as_str()) {
            return Err(AppError::bad_request(format!(
                "Invalid entity_type '{}'. Valid: {:?}",
                et, VALID_ENTITY_TYPES
            )));
        }
    }
    if let Some(ref s) = req.status {
        if !VALID_PARTY_STATUSES.contains(&s.as_str()) {
            return Err(AppError::bad_request(format!(
                "Invalid status '{}'. Valid: {:?}",
                s, VALID_PARTY_STATUSES
            )));
        }
    }
    if let Some(ref sm) = req.service_method {
        ServiceMethod::try_from(sm.as_str()).map_err(AppError::bad_request)?;
    }

    let dob = req
        .date_of_birth
        .as_deref()
        .map(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::bad_request("Invalid date_of_birth format (expected YYYY-MM-DD)"))
        })
        .transpose()?;

    let row = sqlx::query_as!(
        Party,
        r#"
        UPDATE parties SET
            party_type = COALESCE($3, party_type),
            party_role = COALESCE($4, party_role),
            name = COALESCE($5, name),
            entity_type = COALESCE($6, entity_type),
            first_name = COALESCE($7, first_name),
            last_name = COALESCE($8, last_name),
            middle_name = COALESCE($9, middle_name),
            organization_name = COALESCE($10, organization_name),
            email = COALESCE($11, email),
            phone = COALESCE($12, phone),
            date_of_birth = COALESCE($13, date_of_birth),
            ssn_last_four = COALESCE($14, ssn_last_four),
            ein = COALESCE($15, ein),
            address_street1 = COALESCE($16, address_street1),
            address_city = COALESCE($17, address_city),
            address_state = COALESCE($18, address_state),
            address_zip = COALESCE($19, address_zip),
            address_country = COALESCE($20, address_country),
            service_method = COALESCE($21, service_method),
            status = COALESCE($22, status),
            pro_se = COALESCE($23, pro_se),
            nef_sms_opt_in = COALESCE($24, nef_sms_opt_in),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING
            id, court_id, case_id, party_type, party_role, name, entity_type,
            first_name, middle_name, last_name, date_of_birth,
            organization_name, address_street1, address_city, address_state,
            address_zip, address_country, phone, email,
            represented, pro_se, service_method, status,
            joined_date, terminated_date,
            ssn_last_four, ein, nef_sms_opt_in,
            created_at, updated_at
        "#,
        id,
        court_id,
        req.party_type.as_deref(),
        req.party_role.as_deref(),
        req.name.as_deref(),
        req.entity_type.as_deref(),
        req.first_name.as_deref(),
        req.last_name.as_deref(),
        req.middle_name.as_deref(),
        req.organization_name.as_deref(),
        req.email.as_deref(),
        req.phone.as_deref(),
        dob,
        req.ssn_last_four.as_deref(),
        req.ein.as_deref(),
        req.address.as_ref().map(|a| a.street1.as_str()),
        req.address.as_ref().map(|a| a.city.as_str()),
        req.address.as_ref().map(|a| a.state.as_str()),
        req.address.as_ref().map(|a| a.zip_code.as_str()),
        req.address.as_ref().map(|a| a.country.as_str()),
        req.service_method.as_deref(),
        req.status.as_deref(),
        req.pro_se,
        req.nef_sms_opt_in,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Delete a party by ID within a court. Returns true if a row was deleted.
pub async fn delete(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM parties WHERE id = $1 AND court_id = $2",
        id,
        court_id,
    )
    .execute(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(result.rows_affected() > 0)
}

/// List full party objects for a case (for case detail views).
pub async fn list_full_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<Party>, AppError> {
    let rows = sqlx::query_as!(
        Party,
        r#"
        SELECT
            id, court_id, case_id, party_type, party_role, name, entity_type,
            first_name, middle_name, last_name, date_of_birth,
            organization_name, address_street1, address_city, address_state,
            address_zip, address_country, phone, email,
            represented, pro_se, service_method, status,
            joined_date, terminated_date,
            ssn_last_four, ein, nef_sms_opt_in,
            created_at, updated_at
        FROM parties
        WHERE court_id = $1 AND case_id = $2
        ORDER BY name ASC
        "#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List parties represented by a specific attorney (via JOIN on representations).
pub async fn list_by_attorney(
    pool: &Pool<Postgres>,
    court_id: &str,
    attorney_id: Uuid,
) -> Result<Vec<Party>, AppError> {
    let rows = sqlx::query_as!(
        Party,
        r#"
        SELECT DISTINCT
            p.id, p.court_id, p.case_id, p.party_type, p.party_role, p.name, p.entity_type,
            p.first_name, p.middle_name, p.last_name, p.date_of_birth,
            p.organization_name, p.address_street1, p.address_city, p.address_state,
            p.address_zip, p.address_country, p.phone, p.email,
            p.represented, p.pro_se, p.service_method, p.status,
            p.joined_date, p.terminated_date,
            p.ssn_last_four, p.ein, p.nef_sms_opt_in,
            p.created_at, p.updated_at
        FROM parties p
        JOIN representations r ON r.party_id = p.id AND r.court_id = p.court_id
        WHERE p.court_id = $1 AND r.attorney_id = $2 AND r.status = 'Active'
        ORDER BY p.name ASC
        "#,
        court_id,
        attorney_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// List unrepresented parties in a court.
pub async fn list_unrepresented(
    pool: &Pool<Postgres>,
    court_id: &str,
) -> Result<Vec<Party>, AppError> {
    let rows = sqlx::query_as!(
        Party,
        r#"
        SELECT
            id, court_id, case_id, party_type, party_role, name, entity_type,
            first_name, middle_name, last_name, date_of_birth,
            organization_name, address_street1, address_city, address_state,
            address_zip, address_country, phone, email,
            represented, pro_se, service_method, status,
            joined_date, terminated_date,
            ssn_last_four, ein, nef_sms_opt_in,
            created_at, updated_at
        FROM parties
        WHERE court_id = $1 AND represented = false AND status = 'Active'
        ORDER BY name ASC
        "#,
        court_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(rows)
}

/// Update party status only.
pub async fn update_status(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
    status: &str,
) -> Result<Option<Party>, AppError> {
    if !VALID_PARTY_STATUSES.contains(&status) {
        return Err(AppError::bad_request(format!(
            "Invalid status '{}'. Valid: {:?}",
            status, VALID_PARTY_STATUSES
        )));
    }

    let terminated_date = if status == "Terminated" || status == "Dismissed" || status == "Deceased" {
        Some(chrono::Utc::now())
    } else {
        None
    };

    let row = sqlx::query_as!(
        Party,
        r#"
        UPDATE parties
        SET status = $3,
            terminated_date = COALESCE($4, terminated_date),
            updated_at = NOW()
        WHERE id = $1 AND court_id = $2
        RETURNING
            id, court_id, case_id, party_type, party_role, name, entity_type,
            first_name, middle_name, last_name, date_of_birth,
            organization_name, address_street1, address_city, address_state,
            address_zip, address_country, phone, email,
            represented, pro_se, service_method, status,
            joined_date, terminated_date,
            ssn_last_four, ein, nef_sms_opt_in,
            created_at, updated_at
        "#,
        id,
        court_id,
        status,
        terminated_date,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

/// Check if a party is represented (has active representations).
pub async fn is_represented(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM representations
        WHERE party_id = $1 AND court_id = $2 AND status = 'Active'"#,
        id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(count > 0)
}

/// Check if a party has pending (unserved) service records.
pub async fn needs_service(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<bool, AppError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM service_records
        WHERE party_id = $1 AND court_id = $2 AND successful = false"#,
        id,
        court_id,
    )
    .fetch_one(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(count > 0)
}

/// Get the lead counsel representation for a party.
pub async fn get_lead_counsel(
    pool: &Pool<Postgres>,
    court_id: &str,
    id: Uuid,
) -> Result<Option<Representation>, AppError> {
    let row = sqlx::query_as!(
        Representation,
        r#"
        SELECT
            id, court_id, attorney_id, party_id, case_id,
            representation_type, status, start_date, end_date,
            lead_counsel, local_counsel, court_appointed,
            limited_appearance, cja_appointment_id, scope_of_representation,
            withdrawal_reason, notes
        FROM representations
        WHERE party_id = $1 AND court_id = $2 AND lead_counsel = true AND status = 'Active'
        LIMIT 1
        "#,
        id,
        court_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)?;

    Ok(row)
}

// ---------------------------------------------------------------------------
// Existing query functions (kept as-is for backward compatibility)
// ---------------------------------------------------------------------------

/// List parties belonging to a case within a court (lightweight, for dropdowns).
pub async fn list_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<PartyOption>, AppError> {
    sqlx::query_as!(
        PartyOption,
        r#"
        SELECT id, name, party_type
        FROM parties
        WHERE court_id = $1 AND case_id = $2
        ORDER BY name ASC
        "#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// List parties with service info for a case (for auto-seeding service records + NEF).
pub async fn list_service_info_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<PartyServiceInfo>, AppError> {
    sqlx::query_as!(
        PartyServiceInfo,
        r#"
        SELECT id, name, party_type, service_method, email as "email?", phone as "phone?",
               nef_sms_opt_in
        FROM parties
        WHERE court_id = $1 AND case_id = $2 AND status = 'Active'
        ORDER BY name ASC
        "#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}

/// List attorneys representing parties on a case (for NEF delivery to counsel).
pub async fn list_attorney_contacts_by_case(
    pool: &Pool<Postgres>,
    court_id: &str,
    case_id: Uuid,
) -> Result<Vec<AttorneyContact>, AppError> {
    sqlx::query_as!(
        AttorneyContact,
        r#"
        SELECT r.party_id,
               (a.first_name || ' ' || a.last_name) as "attorney_name!",
               a.email as "email?",
               a.phone as "phone?",
               a.nef_sms_opt_in
        FROM representations r
        JOIN attorneys a ON a.id = r.attorney_id AND a.court_id = r.court_id
        WHERE r.court_id = $1 AND r.case_id = $2 AND r.status = 'Active'
        ORDER BY a.last_name ASC
        "#,
        court_id,
        case_id,
    )
    .fetch_all(pool)
    .await
    .map_err(SqlxErrorExt::into_app_error)
}
