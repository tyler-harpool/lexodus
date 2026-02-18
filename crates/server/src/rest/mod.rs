pub mod attorney;
pub mod attachment;
pub mod calendar;
pub mod case;
pub mod charge;
pub mod deadline;
pub mod defendant;
pub mod document;
pub mod docket;
pub mod motion;
pub mod evidence;
pub mod case_note;
pub mod speedy_trial;
pub mod event;
pub mod filing;
pub mod admin;
pub mod membership;
pub mod nef;
pub mod party;
pub mod representation;
pub mod service_record;
pub mod judge;
pub mod order;
pub mod opinion;
pub mod sentencing;
pub mod template_crud;
pub mod config;
pub mod conflict_check;
pub mod feature;
pub mod pdf;
pub mod representation_ext;
pub mod rule;
pub mod signature;
pub mod todo;
pub mod extension;
pub mod compliance;
pub mod reminder;
pub mod federal_rule;
pub mod victim;
pub mod queue;

use axum::{routing::{get, post, put, delete, patch}, Router};
use crate::db::AppState;

// Re-export template functions so openapi.rs can reference them at rest::*
pub use template_crud::*;

/// Build the combined REST API router (court domain + template SaaS).
pub fn api_router() -> Router<AppState> {
    Router::new()
        // Attorney CRUD
        .route("/api/attorneys", get(attorney::list_attorneys))
        .route("/api/attorneys", post(attorney::create_attorney))
        .route("/api/attorneys/search", get(attorney::search_attorneys))
        .route("/api/attorneys/bulk/update-status", post(attorney::bulk_update_status))
        .route("/api/attorneys/bar-number/{bar_number}", get(attorney::get_attorney_by_bar_number))
        .route("/api/attorneys/{id}", get(attorney::get_attorney))
        .route("/api/attorneys/{id}", put(attorney::update_attorney))
        .route("/api/attorneys/{id}", delete(attorney::delete_attorney))
        // Calendar
        .route("/api/calendar/events", post(calendar::schedule_event))
        .route("/api/calendar/events/{event_id}/status", patch(calendar::update_event_status))
        .route("/api/calendar/events/{id}", delete(calendar::delete_event))
        .route("/api/calendar/search", get(calendar::search_calendar))
        .route("/api/cases/{case_id}/calendar", get(calendar::get_case_calendar))
        // Calendar extras (moved under judges/courtrooms per spec)
        .route("/api/judges/{judge_id}/schedule", get(calendar::list_by_judge))
        .route("/api/judges/{judge_id}/available-slot", get(calendar::find_available_slot))
        .route("/api/courtrooms/utilization", get(calendar::get_utilization))
        .route("/api/courtrooms/{courtroom}/events", get(calendar::list_by_courtroom))
        // Cases
        .route("/api/cases/statistics", get(case::case_statistics))
        .route("/api/cases/by-number/{case_number}", get(case::get_by_case_number))
        .route("/api/cases/by-judge/{judge_id}", get(case::list_by_judge))
        .route("/api/cases/count-by-status/{status}", get(case::count_by_status))
        .route("/api/cases", get(case::search_cases).post(case::create_case))
        .route("/api/cases/{id}", get(case::get_case).delete(case::delete_case).patch(case::update_case))
        .route("/api/cases/{id}/status", patch(case::update_case_status))
        .route("/api/cases/{id}/plea", post(case::enter_plea))
        .route("/api/cases/{id}/events", post(case::add_case_event))
        .route("/api/cases/{id}/priority", patch(case::update_priority))
        .route("/api/cases/{id}/seal", post(case::seal_case))
        .route("/api/cases/{id}/unseal", post(case::unseal_case))
        .route("/api/cases/{case_id}/filing-stats", get(case::get_filing_stats))
        // Victims
        .route("/api/cases/{id}/victims", get(victim::list_victims).post(victim::add_victim))
        .route("/api/cases/{id}/victims/{victim_id}/notifications", post(victim::send_notification))
        // Defendants
        .route("/api/defendants", post(defendant::create_defendant))
        .route("/api/cases/{case_id}/defendants", get(defendant::list_defendants_by_case))
        .route("/api/defendants/{id}", get(defendant::get_defendant).put(defendant::update_defendant).delete(defendant::delete_defendant))
        // Charges
        .route("/api/charges", post(charge::create_charge))
        .route("/api/charges/defendant/{defendant_id}", get(charge::list_charges_by_defendant))
        .route("/api/charges/{id}", get(charge::get_charge).put(charge::update_charge).delete(charge::delete_charge))
        // Motions
        .route("/api/motions", post(motion::create_motion))
        .route("/api/cases/{case_id}/motions", get(motion::list_motions_by_case))
        .route("/api/motions/{id}", get(motion::get_motion).put(motion::update_motion).delete(motion::delete_motion))
        // Evidence
        .route("/api/evidence", post(evidence::create_evidence))
        .route("/api/cases/{case_id}/evidence", get(evidence::list_evidence_by_case))
        .route("/api/evidence/{id}", get(evidence::get_evidence).put(evidence::update_evidence).delete(evidence::delete_evidence))
        // Custody Transfers
        .route("/api/custody-transfers", post(evidence::create_custody_transfer))
        .route("/api/custody-transfers/evidence/{evidence_id}", get(evidence::list_custody_transfers_by_evidence))
        .route("/api/custody-transfers/{id}", get(evidence::get_custody_transfer).delete(evidence::delete_custody_transfer))
        // Case Notes
        .route("/api/case-notes", post(case_note::create_case_note))
        .route("/api/cases/{case_id}/case-notes", get(case_note::list_case_notes_by_case))
        .route("/api/case-notes/{id}", get(case_note::get_case_note).put(case_note::update_case_note).delete(case_note::delete_case_note))
        // Speedy Trial
        .route("/api/cases/{id}/speedy-trial/start", post(speedy_trial::start_speedy_trial))
        .route("/api/speedy-trial/deadlines/approaching", get(speedy_trial::list_approaching))
        .route("/api/speedy-trial/violations", get(speedy_trial::list_violations))
        .route("/api/speedy-trial/delays/{id}", delete(speedy_trial::delete_delay))
        .route("/api/cases/{case_id}/speedy-trial", get(speedy_trial::get_speedy_trial).put(speedy_trial::update_speedy_trial_clock))
        .route("/api/cases/{case_id}/speedy-trial/delays", get(speedy_trial::list_delays))
        .route("/api/cases/{case_id}/speedy-trial/deadline-check", get(speedy_trial::deadline_check))
        .route("/api/cases/{id}/speedy-trial/exclude", post(speedy_trial::create_delay))
        // Docket
        .route("/api/docket/entries", post(docket::create_docket_entry))
        .route("/api/docket/entries/{id}", get(docket::get_docket_entry).delete(docket::delete_docket_entry))
        .route("/api/docket/entries/{entry_id}/link-document", post(docket::link_document))
        .route("/api/docket/search", get(docket::search_docket_entries))
        .route("/api/cases/{case_id}/docket", get(docket::get_case_docket))
        // Docket extras (case sub-resource nesting per spec)
        .route("/api/cases/{case_id}/docket-sheet", get(docket::get_docket_sheet))
        .route("/api/cases/{case_id}/docket/type/{entry_type}", get(docket::list_by_type))
        .route("/api/cases/{case_id}/docket/sealed", get(docket::list_sealed))
        .route("/api/cases/{case_id}/docket/search/{text}", get(docket::search_in_case))
        .route("/api/cases/{case_id}/docket/statistics", get(docket::docket_statistics))
        .route("/api/docket/service-check/{entry_type}", get(docket::service_check))
        // Docket Attachments
        .route("/api/docket/entries/{entry_id}/attachments", get(attachment::list_entry_attachments).post(attachment::create_entry_attachment))
        .route("/api/docket/attachments/{attachment_id}/finalize", post(attachment::finalize_attachment))
        .route("/api/docket/attachments/{attachment_id}/download", get(attachment::download_attachment))
        .route("/api/docket/attachments/{attachment_id}/file", get(attachment::serve_attachment_file))
        // Deadlines
        .route("/api/deadlines", post(deadline::create_deadline))
        .route("/api/deadlines/search", get(deadline::search_deadlines))
        .route("/api/cases/{case_id}/deadlines", get(deadline::list_by_case))
        .route("/api/deadlines/{id}/complete", patch(deadline::complete_deadline))
        .route("/api/cases/{case_id}/deadlines/type/{deadline_type}", get(deadline::list_by_case_and_type))
        .route("/api/deadlines/upcoming", get(deadline::list_upcoming))
        .route("/api/deadlines/urgent", get(deadline::list_urgent))
        .route("/api/deadlines/calculate", post(deadline::calculate_deadline))
        .route("/api/deadlines/{id}", get(deadline::get_deadline))
        .route("/api/deadlines/{id}", put(deadline::update_deadline))
        .route("/api/deadlines/{id}", delete(deadline::delete_deadline))
        .route("/api/deadlines/{id}/status", patch(deadline::update_deadline_status))
        // Extensions
        .route("/api/deadlines/{deadline_id}/extensions", post(extension::request_extension).get(extension::list_extensions))
        .route("/api/extensions/{extension_id}/ruling", patch(extension::rule_on_extension))
        .route("/api/extensions/{id}", get(extension::get_extension))
        .route("/api/extensions/pending", get(extension::list_pending_extensions))
        // Compliance (under /deadlines per spec)
        .route("/api/deadlines/compliance-stats", get(compliance::compliance_stats))
        .route("/api/deadlines/compliance-report", get(compliance::compliance_report))
        .route("/api/deadlines/performance-metrics", get(compliance::compliance_performance))
        .route("/api/deadlines/missed-jurisdictional", get(compliance::missed_jurisdictional))
        // Reminders (under /deadlines per spec)
        .route("/api/deadlines/reminders/pending", get(reminder::list_pending_reminders))
        .route("/api/deadlines/reminders/send", post(reminder::send_reminder))
        .route("/api/deadlines/{deadline_id}/reminders", get(reminder::list_reminders_by_deadline))
        .route("/api/deadlines/reminders/recipient/{recipient}", get(reminder::list_by_recipient))
        .route("/api/deadlines/reminders/{reminder_id}/acknowledge", patch(reminder::acknowledge_reminder))
        // Federal Rules (under /deadlines per spec)
        .route("/api/deadlines/federal-rules", get(federal_rule::list_federal_rules))
        // Filings
        .route("/api/filings/validate", post(filing::validate_filing))
        .route("/api/filings", post(filing::submit_filing))
        .route("/api/filings/jurisdictions", get(filing::list_jurisdictions))
        .route("/api/filings/upload/init", post(filing::init_filing_upload))
        .route("/api/filings/upload/{id}/finalize", post(filing::finalize_filing_upload))
        .route("/api/filings/{filing_id}/nef", get(nef::get_nef))
        .route("/api/nef/{id}", get(nef::get_nef_by_id))
        .route("/api/nef/docket-entry/{docket_entry_id}", get(nef::get_nef_by_docket_entry))
        // Documents
        .route("/api/documents/from-attachment", post(document::promote_attachment))
        .route("/api/documents/{id}/seal", post(document::seal_document))
        .route("/api/documents/{id}/unseal", post(document::unseal_document))
        .route("/api/documents/{id}/replace", post(document::replace_document))
        .route("/api/documents/{id}/strike", post(document::strike_document))
        .route("/api/documents/{id}/events", get(document::list_document_events))
        // Unified Events
        .route("/api/events", post(event::submit_event))
        .route("/api/cases/{case_id}/timeline", get(event::get_case_timeline))
        // Parties
        .route("/api/parties", post(party::create_party))
        .route("/api/parties/unrepresented", get(party::list_unrepresented))
        .route("/api/parties/case/{case_id}", get(party::list_parties_by_case))
        .route("/api/parties/attorney/{attorney_id}", get(party::list_parties_by_attorney))
        .route("/api/parties/{id}", get(party::get_party).put(party::update_party).delete(party::delete_party))
        .route("/api/parties/{id}/status", patch(party::update_party_status))
        .route("/api/parties/{id}/needs-service", get(party::check_needs_service))
        .route("/api/parties/{id}/lead-counsel", get(party::get_lead_counsel))
        .route("/api/parties/{id}/is-represented", get(party::check_is_represented))
        // Representations
        .route("/api/representations", post(representation::add_representation))
        .route("/api/representations/substitute", post(representation::substitute_attorney))
        .route("/api/representations/migrate", post(representation_ext::migrate_representation))
        .route("/api/representations/attorney/{attorney_id}/active", get(representation::list_active_by_attorney))
        .route("/api/representations/case/{case_id}", get(representation::list_by_case))
        .route("/api/representations/{id}", get(representation::get_representation))
        .route("/api/representations/{id}/end", post(representation::end_representation))
        // Service Records
        .route("/api/service-records", get(service_record::list_service_records).post(service_record::create_service_record))
        .route("/api/service-records/document/{document_id}", get(service_record::list_by_document))
        .route("/api/service-records/party/{party_id}", get(service_record::list_by_party))
        .route("/api/service-records/bulk/{document_id}", post(service_record::bulk_create))
        .route("/api/service-records/{id}/complete", post(service_record::complete_service_record))
        // Courts (public)
        .route("/api/courts", get(admin::list_courts))
        // Admin
        .route("/api/admin/tenants/init", post(admin::init_tenant))
        .route("/api/admin/tenants/stats", get(admin::tenant_stats))
        // Court Role Memberships (Admin or Clerk — scoped inside handlers)
        .route("/api/admin/court-role-requests", get(membership::list_pending_requests))
        .route("/api/admin/court-role-requests/{id}/approve", post(membership::approve_request))
        .route("/api/admin/court-role-requests/{id}/deny", post(membership::deny_request))
        .route("/api/admin/court-memberships/user/{user_id}", get(membership::get_user_court_roles))
        .route("/api/admin/court-memberships/{user_id}/{court_id}", delete(membership::remove_court_role))
        .route("/api/admin/court-memberships", put(membership::set_court_role))
        // Judges
        .route("/api/judges", post(judge::create_judge).get(judge::list_judges))
        .route("/api/judges/search", get(judge::search_judges))
        .route("/api/judges/available", get(judge::list_available))
        .route("/api/judges/vacation", get(judge::list_on_vacation))
        .route("/api/judges/conflicts/check/{party_name}", get(judge::check_conflicts_for_party))
        .route("/api/judges/status/{status}", get(judge::list_judges_by_status))
        .route("/api/judges/district/{district}", get(judge::list_by_district))
        .route("/api/judges/{id}", get(judge::get_judge).put(judge::update_judge).delete(judge::delete_judge))
        .route("/api/judges/{id}/status", patch(judge::update_judge_status))
        .route("/api/judges/{id}/workload", get(judge::get_workload))
        // Judge Conflicts
        .route("/api/judges/{judge_id}/conflicts", post(judge::create_conflict).get(judge::list_conflicts))
        .route("/api/judges/{judge_id}/conflicts/{conflict_id}", get(judge::get_conflict).delete(judge::delete_conflict))
        // Case Assignments
        .route("/api/judges/assignments", post(judge::create_assignment))
        .route("/api/cases/{case_id}/assignment", get(judge::list_assignments_by_case))
        .route("/api/cases/{case_id}/assignment-history", get(judge::get_assignment_history))
        .route("/api/assignments/{id}", delete(judge::delete_assignment))
        // Recusal Motions
        .route("/api/judges/{judge_id}/recusals", post(judge::create_recusal))
        .route("/api/recusals/{recusal_id}/process", post(judge::process_recusal))
        .route("/api/recusals/{recusal_id}/ruling", patch(judge::update_recusal_ruling))
        .route("/api/recusals/pending", get(judge::list_pending_recusals))
        .route("/api/cases/{case_id}/recusals", get(judge::list_recusals_by_case))
        .route("/api/recusals/judge/{judge_id}", get(judge::list_recusals_by_judge))
        // Orders
        .route("/api/orders", get(order::list_orders).post(order::create_order))
        .route("/api/orders/expired", get(order::check_expired))
        .route("/api/orders/requires-attention", get(order::check_requires_attention))
        .route("/api/orders/pending-signatures", get(order::list_pending_signatures))
        .route("/api/orders/expiring", get(order::list_expiring))
        .route("/api/orders/statistics", get(order::order_statistics))
        .route("/api/orders/from-template", post(order::create_from_template))
        .route("/api/orders/{order_id}", get(order::get_order).patch(order::update_order).delete(order::delete_order))
        .route("/api/orders/{order_id}/sign", post(order::sign_order))
        .route("/api/orders/{order_id}/issue", post(order::issue_order))
        .route("/api/orders/{order_id}/service", post(order::serve_order))
        .route("/api/cases/{case_id}/orders", get(order::list_orders_by_case))
        .route("/api/judges/{judge_id}/orders", get(order::list_orders_by_judge))
        // Order Templates
        .route("/api/templates/orders", post(order::create_template).get(order::list_templates))
        .route("/api/templates/orders/active", get(order::list_active_templates))
        .route("/api/templates/orders/{template_id}", get(order::get_template).put(order::update_template).delete(order::delete_template))
        .route("/api/templates/orders/{template_id}/generate", post(order::generate_content))
        // Opinions
        .route("/api/opinions", get(opinion::list_opinions).post(opinion::create_opinion))
        .route("/api/opinions/search", get(opinion::search_opinions))
        .route("/api/opinions/statistics", get(opinion::calculate_statistics))
        .route("/api/opinions/precedential", get(opinion::list_precedential))
        .route("/api/opinions/citations/statistics", get(opinion::citation_statistics))
        .route("/api/opinions/{opinion_id}", get(opinion::get_opinion).patch(opinion::update_opinion).delete(opinion::delete_opinion))
        .route("/api/opinions/{opinion_id}/file", post(opinion::file_opinion))
        .route("/api/opinions/{opinion_id}/publish", post(opinion::publish_opinion))
        .route("/api/opinions/{opinion_id}/is-majority", get(opinion::check_is_majority))
        .route("/api/opinions/{opinion_id}/is-binding", get(opinion::check_is_binding))
        .route("/api/cases/{case_id}/opinions", get(opinion::list_opinions_by_case))
        .route("/api/judges/{judge_id}/opinions", get(opinion::list_opinions_by_judge))
        .route("/api/opinions/{opinion_id}/votes", post(opinion::add_vote))
        .route("/api/opinions/{opinion_id}/citations", post(opinion::add_citation))
        .route("/api/opinions/{opinion_id}/headnotes", post(opinion::add_headnote))
        .route("/api/opinions/{opinion_id}/drafts", post(opinion::create_draft).get(opinion::list_drafts))
        .route("/api/opinions/{opinion_id}/drafts/current", get(opinion::get_current_draft))
        .route("/api/opinions/{opinion_id}/drafts/{draft_id}/comments", post(opinion::add_draft_comment))
        .route("/api/opinions/{opinion_id}/drafts/{draft_id}/comments/{comment_id}/resolve", patch(opinion::resolve_comment))
        // Sentencing
        .route("/api/sentencing", post(sentencing::create_sentencing))
        .route("/api/sentencing/pending", get(sentencing::list_pending))
        .route("/api/sentencing/upcoming", get(sentencing::list_upcoming))
        .route("/api/sentencing/appeal-deadlines", get(sentencing::list_appeal_deadlines))
        .route("/api/sentencing/date-range", get(sentencing::list_by_date_range))
        .route("/api/sentencing/substantial-assistance", get(sentencing::list_substantial_assistance))
        .route("/api/sentencing/active-supervision", get(sentencing::list_active_supervision))
        .route("/api/sentencing/rdap-eligible", get(sentencing::list_rdap_eligible))
        .route("/api/sentencing/calculate-guidelines", post(sentencing::calculate_guidelines))
        .route("/api/sentencing/statistics/departures", get(sentencing::departure_stats))
        .route("/api/sentencing/statistics/variances", get(sentencing::variance_stats))
        .route("/api/sentencing/statistics/district", get(sentencing::district_stats))
        .route("/api/sentencing/statistics/trial-penalty", get(sentencing::trial_penalty_stats))
        .route("/api/sentencing/statistics/offense/{offense_type}", get(sentencing::offense_stats))
        .route("/api/sentencing/statistics/judge/{judge_id}", get(sentencing::judge_stats))
        .route("/api/cases/{case_id}/sentencing", get(sentencing::list_by_case))
        .route("/api/sentencing/defendant/{defendant_id}", get(sentencing::list_by_defendant))
        .route("/api/sentencing/judge/{judge_id}", get(sentencing::list_by_judge))
        .route("/api/sentencing/{id}", get(sentencing::get_sentencing).put(sentencing::update_sentencing).delete(sentencing::delete_sentencing))
        .route("/api/sentencing/{id}/departure", post(sentencing::record_departure))
        .route("/api/sentencing/{id}/variance", post(sentencing::record_variance))
        .route("/api/sentencing/{id}/supervised-release", put(sentencing::update_supervised_release))
        .route("/api/sentencing/{id}/special-conditions", post(sentencing::add_special_condition))
        .route("/api/sentencing/{id}/bop-designation", post(sentencing::add_bop_designation))
        .route("/api/sentencing/{id}/prior-sentences", post(sentencing::add_prior_sentence))
        .route("/api/sentencing/{id}/criminal-history-points", get(sentencing::calc_history_points))
        .route("/api/sentencing/{id}/safety-valve-eligible", get(sentencing::check_safety_valve))
        .route("/api/sentencing/{id}/calculate-offense-level", post(sentencing::calc_offense_level))
        .route("/api/sentencing/{id}/lookup-guidelines-range", post(sentencing::lookup_guidelines))
        // TODOs
        .route("/api/todos", get(todo::list_todos).post(todo::create_todo))
        .route("/api/todos/{id}", get(todo::get_todo).delete(todo::delete_todo))
        .route("/api/todos/{id}/toggle", post(todo::toggle_todo))
        // Configuration
        .route("/api/config", get(config::get_config))
        .route("/api/config/overrides/district", get(config::get_district_overrides).put(config::set_district_override).delete(config::delete_district_override))
        .route("/api/config/overrides/judge", get(config::get_judge_overrides).put(config::set_judge_override).delete(config::delete_judge_override))
        .route("/api/config/preview", post(config::preview_config))
        // Rules
        .route("/api/rules", get(rule::list_rules).post(rule::create_rule))
        .route("/api/rules/category/{category}", get(rule::list_by_category))
        .route("/api/rules/trigger/{trigger}", get(rule::list_by_trigger))
        .route("/api/rules/jurisdiction/{jurisdiction}", get(rule::list_by_jurisdiction))
        .route("/api/rules/evaluate", post(rule::evaluate_rules))
        .route("/api/rules/{id}", get(rule::get_rule).put(rule::update_rule).delete(rule::delete_rule))
        // Conflict Checks
        .route("/api/conflict-checks", post(conflict_check::create_conflict_check))
        .route("/api/conflict-checks/attorney/{attorney_id}", get(conflict_check::list_by_attorney))
        .route("/api/conflict-checks/check", post(conflict_check::run_conflict_check))
        .route("/api/conflict-checks/{id}/clear", post(conflict_check::clear_conflict))
        // PDF Generation
        .route("/api/pdf/rule16b", post(pdf::generate_rule16b))
        .route("/api/pdf/signed/rule16b", post(pdf::generate_signed_rule16b))
        .route("/api/pdf/court-order", post(pdf::generate_court_order))
        .route("/api/pdf/minute-entry", post(pdf::generate_minute_entry))
        .route("/api/pdf/waiver-indictment", post(pdf::generate_waiver))
        .route("/api/pdf/conditions-release", post(pdf::generate_conditions))
        .route("/api/pdf/criminal-judgment", post(pdf::generate_judgment))
        .route("/api/pdf/batch", post(pdf::batch_generate))
        // Signatures
        .route("/api/signatures", post(signature::upload_signature))
        .route("/api/signatures/{judge_id}", get(signature::get_signature))
        // Features
        .route("/api/features", get(feature::list_features).patch(feature::update_features))
        .route("/api/features/implementation", get(feature::get_impl).patch(feature::update_impl))
        .route("/api/features/blocked", get(feature::list_blocked))
        .route("/api/features/ready", get(feature::list_ready))
        .route("/api/features/manager", post(feature::manage_feature))
        .route("/api/features/{feature_path}/enabled", get(feature::check_enabled))
        .route("/api/features/override", post(feature::set_override).delete(feature::clear_overrides))
        // Attorney sub-resources (PR 13-14)
        .route("/api/attorneys/status/{status}", get(attorney::list_attorneys_by_status))
        .route("/api/attorneys/firm/{firm_name}", get(attorney::list_attorneys_by_firm))
        .route("/api/attorneys/{id}/bar-admissions", post(attorney::add_bar_admission))
        .route("/api/attorneys/{id}/bar-admissions/{state}", delete(attorney::remove_bar_admission))
        .route("/api/attorneys/bar-state/{state}", get(attorney::list_attorneys_by_bar_state))
        .route("/api/attorneys/{id}/federal-admissions", post(attorney::add_federal_admission))
        .route("/api/attorneys/{id}/federal-admissions/{court}", delete(attorney::remove_federal_admission))
        .route("/api/attorneys/federal-court/{court}", get(attorney::list_attorneys_by_federal_court))
        .route("/api/attorneys/{id}/practice-areas", get(attorney::list_practice_areas).post(attorney::add_practice_area))
        .route("/api/attorneys/{id}/practice-areas/{area}", delete(attorney::remove_practice_area))
        .route("/api/attorneys/{id}/pro-hac-vice", post(attorney::add_pro_hac_vice))
        .route("/api/attorneys/{id}/pro-hac-vice/{case_id}/status", patch(attorney::update_phv_status))
        .route("/api/attorneys/pro-hac-vice/active", get(attorney::list_active_phv))
        .route("/api/attorneys/pro-hac-vice/case/{case_id}", get(attorney::list_phv_by_case))
        .route("/api/attorneys/{id}/cja-panel/{cja_district}", post(attorney::add_cja_panel).delete(attorney::remove_cja_panel))
        .route("/api/attorneys/cja-panel/{cja_district}", get(attorney::list_cja_panel))
        .route("/api/attorneys/{id}/cja-appointments", get(attorney::list_cja_appts).post(attorney::add_cja_appt))
        .route("/api/attorneys/cja/pending-vouchers", get(attorney::list_pending_vouchers))
        .route("/api/attorneys/{id}/ecf-registration", put(attorney::upsert_ecf_registration))
        .route("/api/attorneys/{id}/good-standing", get(attorney::check_good_standing))
        .route("/api/attorneys/{id}/can-practice/{court}", get(attorney::check_can_practice))
        .route("/api/attorneys/{id}/has-ecf-privileges", get(attorney::check_ecf_privileges))
        .route("/api/attorneys/{id}/calculate-win-rate", post(attorney::calculate_win_rate))
        .route("/api/attorneys/ecf-access", get(attorney::list_ecf_access))
        .route("/api/attorneys/{id}/ecf-access", delete(attorney::revoke_ecf_access))
        .route("/api/attorneys/{id}/disciplinary-actions", get(attorney::list_discipline).post(attorney::add_discipline))
        .route("/api/attorneys/with-discipline", get(attorney::list_attorneys_with_discipline))
        .route("/api/attorneys/{attorney_id}/cases", get(attorney::list_attorney_cases).post(attorney::add_to_case))
        .route("/api/attorneys/{attorney_id}/cases/{case_id}", delete(attorney::remove_from_case))
        .route("/api/attorneys/{attorney_id}/case-load", get(attorney::get_case_load))
        .route("/api/attorneys/{attorney_id}/representation-history", get(attorney::get_rep_history))
        .route("/api/attorneys/{attorney_id}/conflict-check", post(attorney::run_conflict_check))
        .route("/api/attorneys/{id}/metrics", get(attorney::get_metrics))
        .route("/api/attorneys/{id}/win-rate", get(attorney::get_win_rate))
        .route("/api/attorneys/{id}/case-count", get(attorney::get_case_count))
        .route("/api/attorneys/top-performers", get(attorney::list_top_performers))
        // Queue
        .route("/api/queue", get(queue::list_queue).post(queue::create_queue_item))
        .route("/api/queue/stats", get(queue::queue_stats))
        .route("/api/queue/{id}", get(queue::get_queue_item))
        .route("/api/queue/{id}/claim", post(queue::claim_queue_item))
        .route("/api/queue/{id}/release", post(queue::release_queue_item))
        .route("/api/queue/{id}/advance", post(queue::advance_queue_item))
        .route("/api/queue/{id}/reject", post(queue::reject_queue_item))
        // Template SaaS routes (users, products, auth, billing)
        .merge(template_crud::rest_router())
}

/// Build the REST API router with rate limiting applied.
pub fn api_router_with_rate_limit(
    rate_limit: crate::rate_limit::RateLimitState,
) -> Router<AppState> {
    api_router().layer(axum::middleware::from_fn_with_state(
        rate_limit,
        crate::rate_limit::rate_limit_middleware,
    ))
}
