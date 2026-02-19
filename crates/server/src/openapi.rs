use axum::Router;
use shared_types::{
    // Template types
    AppError, AppErrorKind, ApproveDeviceRequest, AuthResponse, AuthUser, BillingEvent,
    CheckoutRequest, CheckoutResponse, CreateProductRequest, CreateUserRequest, DashboardStats,
    DeviceAuthStatus, DeviceFlowInitResponse, DeviceFlowPollResponse, ForgotPasswordRequest,
    InitiateDeviceRequest, LoginRequest, MessageResponse, PollDeviceRequest, Product,
    RegisterRequest, ResetPasswordRequest, SendPhoneVerificationRequest, SubscriptionStatus,
    UpdateProductRequest, UpdateProfileRequest, UpdateTierRequest, UpdateUserRequest, User,
    UserTier, VerifyPhoneRequest,
    // Common types
    Address, Court, TenantStats, CourtRoleRequestResponse, ReviewCourtRoleRequest,
    SetCourtRoleRequest,
    // Attorney types
    AttorneyResponse, CreateAttorneyRequest, UpdateAttorneyRequest, BulkUpdateStatusRequest,
    BarAdmissionResponse, CreateBarAdmissionRequest, FederalAdmissionResponse,
    CreateFederalAdmissionRequest, DisciplineRecordResponse, CreateDisciplineRecordRequest,
    ProHacViceResponse, CreateProHacViceRequest, UpdatePhvStatusRequest,
    CjaAppointmentResponse, CreateCjaAppointmentRequest, EcfRegistrationResponse,
    UpsertEcfRegistrationRequest, AttorneyMetrics, AttorneyCaseLoad, GoodStandingResult,
    CanPracticeResult, WinRateResult, ConflictCheckRequest, ConflictCheckResult,
    AttorneyAddToCaseRequest,
    PracticeAreaResponse, AddPracticeAreaRequest,
    // Calendar types
    CalendarEntryResponse, CalendarSearchResponse, CalendarEventType, EventStatus,
    ScheduleEventRequest, UpdateEventStatusRequest, CourtUtilization, AvailableSlot,
    // Case types
    CaseResponse, CaseSearchResponse, CreateCaseRequest, UpdateCaseRequest,
    UpdateCaseStatusRequest, CaseStatistics, PleaRequest, UpdatePriorityRequest,
    SealCaseRequest, AddCaseEventRequest,
    // Defendant types
    DefendantResponse, CreateDefendantRequest, UpdateDefendantRequest,
    // Charge types
    ChargeResponse, CreateChargeRequest, UpdateChargeRequest,
    // Motion types
    MotionResponse, CreateMotionRequest, UpdateMotionRequest,
    // Evidence types
    EvidenceResponse, CreateEvidenceRequest, UpdateEvidenceRequest,
    // Custody transfer types
    CustodyTransferResponse, CreateCustodyTransferRequest,
    // Case note types
    CaseNoteResponse, CreateCaseNoteRequest, UpdateCaseNoteRequest,
    // Speedy trial types
    SpeedyTrialResponse, StartSpeedyTrialRequest, UpdateSpeedyTrialClockRequest,
    ExcludableDelayResponse, CreateExcludableDelayRequest, DeadlineCheckResponse,
    // Docket types
    DocketEntryResponse, DocketSearchResponse, CreateDocketEntryRequest, LinkDocumentRequest,
    DocketAttachmentResponse, CreateAttachmentRequest, CreateAttachmentResponse,
    DocketSheet, DocketStatistics, FilingStatsResponse, ServiceCheckResponse,
    // Deadline types
    DeadlineResponse, DeadlineSearchResponse, CreateDeadlineRequest, UpdateDeadlineRequest,
    UpdateDeadlineStatusRequest, CalculateDeadlineRequest, CalculateDeadlineResponse,
    // Compliance types
    ComplianceStats, ComplianceReport,
    // Extension types
    ExtensionResponse, CreateExtensionRequest, UpdateExtensionRulingRequest,
    // Reminder types
    ReminderResponse, SendReminderRequest,
    // Federal rule types
    FederalRule,
    // Service record types
    ServiceRecord, ServiceRecordResponse, CreateServiceRecordRequest,
    BulkCreateServiceRecordRequest, ServiceMethod,
    // Document types
    Document, DocumentResponse, DocumentEventResponse, PromoteAttachmentRequest,
    SealDocumentRequest, ReplaceDocumentRequest, SealingLevel, UserRole,
    // Filing types
    Filing, ValidateFilingRequest, ValidateFilingResponse, FilingValidationError,
    FilingResponse, Nef, NefResponse, NefSummary, JurisdictionInfo,
    FilingUpload, InitFilingUploadRequest, InitFilingUploadResponse,
    FinalizeFilingUploadResponse,
    // Event types
    SubmitEventRequest, SubmitEventResponse, TimelineEntry, TimelineResponse, EventKind,
    // Party & Representation types
    Party, Representation,
    CreatePartyRequest, UpdatePartyRequest, UpdatePartyStatusRequest, PartyResponse,
    CreateRepresentationRequest, EndRepresentationRequest, SubstituteAttorneyRequest,
    RepresentationResponse, MigrateRepresentationRequest,
    // Conflict check types
    ConflictCheck, ConflictCheckResponse, CreateConflictCheckRequest,
    RunConflictCheckRequest, RunConflictCheckResult,
    // Judge types
    JudgeResponse, CreateJudgeRequest, UpdateJudgeRequest, UpdateJudgeStatusRequest,
    JudgeConflictResponse, CreateJudgeConflictRequest,
    CaseAssignmentResponse, CreateCaseAssignmentRequest,
    RecusalMotionResponse, CreateRecusalMotionRequest, UpdateRecusalRulingRequest,
    JudgeWorkload, AssignmentHistory,
    // Order types
    JudicialOrderResponse, CreateJudicialOrderRequest, UpdateJudicialOrderRequest,
    OrderTemplateResponse, CreateOrderTemplateRequest, UpdateOrderTemplateRequest,
    SignOrderRequest, IssueOrderRequest, ServeOrderRequest, OrderStatistics,
    CreateFromTemplateRequest, GenerateContentRequest,
    // Opinion types
    JudicialOpinionResponse, CreateJudicialOpinionRequest, UpdateJudicialOpinionRequest,
    OpinionVoteResponse, CreateOpinionVoteRequest,
    OpinionCitationResponse, CreateOpinionCitationRequest,
    HeadnoteResponse, CreateHeadnoteRequest,
    OpinionDraftResponse, CreateOpinionDraftRequest,
    DraftCommentResponse, CreateDraftCommentRequest,
    OpinionStatistics, CitationStatistics,
    // Sentencing types
    SentencingResponse, CreateSentencingRequest, UpdateSentencingRequest,
    SpecialConditionResponse, CreateSpecialConditionRequest,
    BopDesignationResponse, CreateBopDesignationRequest,
    PriorSentenceResponse, CreatePriorSentenceRequest,
    DepartureRequest, VarianceRequest, SupervisedReleaseRequest,
    GuidelinesRequest, GuidelinesResult, OffenseLevelRequest, SentencingStatistics,
    // Config types
    ConfigOverrideResponse, SetConfigOverrideRequest,
    JudgeSignatureResponse, CreateSignatureRequest,
    FeatureFlagResponse, UpdateFeatureFlagRequest, SetFeatureOverrideRequest,
    FeatureStatusResponse, ConfigPreviewRequest,
    // Rule types
    RuleResponse, CreateRuleRequest, UpdateRuleRequest,
    EvaluateRulesRequest, EvaluateRulesResponse,
    // Todo types
    TodoResponse, CreateTodoRequest,
    // Victim types
    VictimResponse, CreateVictimRequest,
    VictimNotificationResponse, SendVictimNotificationRequest,
};
use sqlx::{Pool, Postgres};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::db::AppState;
use crate::health;
use crate::rest;
use crate::rest::pdf::{PdfGenerateRequest, BatchPdfRequest, BatchPdfItem, BatchPdfResponseItem};

/// OpenAPI documentation for the API.
#[derive(OpenApi)]
#[openapi(
    paths(
        // Template SaaS endpoints
        rest::list_users,
        rest::get_user,
        rest::create_user,
        rest::update_user,
        rest::delete_user,
        rest::update_user_tier,
        rest::list_products,
        rest::create_product,
        rest::update_product,
        rest::delete_product,
        rest::get_dashboard_stats,
        rest::register,
        rest::login,
        rest::logout,
        rest::upload_avatar,
        rest::create_checkout,
        rest::create_portal,
        rest::get_subscription,
        rest::cancel_subscription,
        rest::stripe_webhook,
        rest::mailgun_webhook,
        rest::verify_email,
        rest::forgot_password,
        rest::reset_password,
        rest::send_phone_verification,
        rest::verify_phone,
        rest::initiate_device,
        rest::poll_device,
        rest::approve_device,
        // Attorneys
        rest::attorney::create_attorney,
        rest::attorney::get_attorney,
        rest::attorney::get_attorney_by_bar_number,
        rest::attorney::list_attorneys,
        rest::attorney::search_attorneys,
        rest::attorney::update_attorney,
        rest::attorney::delete_attorney,
        rest::attorney::bulk_update_status,
        rest::attorney::list_attorneys_by_status,
        rest::attorney::list_attorneys_by_firm,
        rest::attorney::add_bar_admission,
        rest::attorney::remove_bar_admission,
        rest::attorney::list_attorneys_by_bar_state,
        rest::attorney::add_federal_admission,
        rest::attorney::remove_federal_admission,
        rest::attorney::list_attorneys_by_federal_court,
        rest::attorney::list_discipline,
        rest::attorney::add_discipline,
        rest::attorney::list_attorneys_with_discipline,
        rest::attorney::add_pro_hac_vice,
        rest::attorney::update_phv_status,
        rest::attorney::list_active_phv,
        rest::attorney::list_phv_by_case,
        rest::attorney::add_cja_panel,
        rest::attorney::remove_cja_panel,
        rest::attorney::list_cja_panel,
        rest::attorney::list_cja_appts,
        rest::attorney::add_cja_appt,
        rest::attorney::list_pending_vouchers,
        rest::attorney::upsert_ecf_registration,
        rest::attorney::check_good_standing,
        rest::attorney::check_can_practice,
        rest::attorney::check_ecf_privileges,
        rest::attorney::calculate_win_rate,
        rest::attorney::list_ecf_access,
        rest::attorney::revoke_ecf_access,
        rest::attorney::list_attorney_cases,
        rest::attorney::add_to_case,
        rest::attorney::remove_from_case,
        rest::attorney::get_case_load,
        rest::attorney::get_rep_history,
        rest::attorney::run_conflict_check,
        rest::attorney::get_metrics,
        rest::attorney::get_win_rate,
        rest::attorney::get_case_count,
        rest::attorney::list_top_performers,
        rest::attorney::add_practice_area,
        rest::attorney::list_practice_areas,
        rest::attorney::remove_practice_area,
        // Calendar
        rest::calendar::schedule_event,
        rest::calendar::update_event_status,
        rest::calendar::delete_event,
        rest::calendar::search_calendar,
        rest::calendar::get_case_calendar,
        rest::calendar::list_calendar_by_case,
        rest::calendar::list_by_judge,
        rest::calendar::find_available_slot,
        rest::calendar::get_utilization,
        rest::calendar::list_by_courtroom,
        // Cases
        rest::case::create_case,
        rest::case::get_case,
        rest::case::delete_case,
        rest::case::update_case,
        rest::case::update_case_status,
        rest::case::search_cases,
        rest::case::case_statistics,
        rest::case::get_by_case_number,
        rest::case::list_by_judge,
        rest::case::count_by_status,
        rest::case::enter_plea,
        rest::case::add_case_event,
        rest::case::update_priority,
        rest::case::seal_case,
        rest::case::unseal_case,
        rest::case::get_filing_stats,
        // Defendants
        rest::defendant::create_defendant,
        rest::defendant::get_defendant,
        rest::defendant::list_defendants_by_case,
        rest::defendant::update_defendant,
        rest::defendant::delete_defendant,
        // Charges
        rest::charge::create_charge,
        rest::charge::get_charge,
        rest::charge::list_charges_by_defendant,
        rest::charge::update_charge,
        rest::charge::delete_charge,
        // Motions
        rest::motion::create_motion,
        rest::motion::get_motion,
        rest::motion::list_motions_by_case,
        rest::motion::update_motion,
        rest::motion::delete_motion,
        // Evidence
        rest::evidence::create_evidence,
        rest::evidence::get_evidence,
        rest::evidence::list_evidence_by_case,
        rest::evidence::update_evidence,
        rest::evidence::delete_evidence,
        // Custody Transfers
        rest::evidence::create_custody_transfer,
        rest::evidence::get_custody_transfer,
        rest::evidence::list_custody_transfers_by_evidence,
        rest::evidence::delete_custody_transfer,
        // Case Notes
        rest::case_note::create_case_note,
        rest::case_note::get_case_note,
        rest::case_note::list_case_notes_by_case,
        rest::case_note::update_case_note,
        rest::case_note::delete_case_note,
        // Speedy Trial
        rest::speedy_trial::start_speedy_trial,
        rest::speedy_trial::get_speedy_trial,
        rest::speedy_trial::update_speedy_trial_clock,
        rest::speedy_trial::list_approaching,
        rest::speedy_trial::list_violations,
        rest::speedy_trial::create_delay,
        rest::speedy_trial::list_delays,
        rest::speedy_trial::delete_delay,
        rest::speedy_trial::deadline_check,
        // Docket
        rest::docket::create_docket_entry,
        rest::docket::get_docket_entry,
        rest::docket::delete_docket_entry,
        rest::docket::get_case_docket,
        rest::docket::search_docket_entries,
        rest::docket::link_document,
        rest::docket::list_docket_by_case,
        rest::docket::get_docket_sheet,
        rest::docket::list_by_type,
        rest::docket::list_sealed,
        rest::docket::search_in_case,
        rest::docket::docket_statistics,
        rest::docket::service_check,
        // Attachments
        rest::attachment::list_entry_attachments,
        rest::attachment::create_entry_attachment,
        rest::attachment::finalize_attachment,
        rest::attachment::download_attachment,
        rest::attachment::serve_attachment_file,
        // Deadlines
        rest::deadline::create_deadline,
        rest::deadline::get_deadline,
        rest::deadline::update_deadline,
        rest::deadline::delete_deadline,
        rest::deadline::update_deadline_status,
        rest::deadline::search_deadlines,
        rest::deadline::list_by_case,
        rest::deadline::complete_deadline,
        rest::deadline::list_by_case_and_type,
        rest::deadline::list_upcoming,
        rest::deadline::list_urgent,
        rest::deadline::calculate_deadline,
        // Compliance
        rest::compliance::compliance_stats,
        rest::compliance::compliance_report,
        rest::compliance::compliance_performance,
        rest::compliance::missed_jurisdictional,
        // Reminders
        rest::reminder::list_pending_reminders,
        rest::reminder::send_reminder,
        rest::reminder::list_reminders_by_deadline,
        rest::reminder::list_by_recipient,
        rest::reminder::acknowledge_reminder,
        // Extensions
        rest::extension::request_extension,
        rest::extension::rule_on_extension,
        rest::extension::get_extension,
        rest::extension::list_extensions,
        rest::extension::list_pending_extensions,
        // Federal Rules
        rest::federal_rule::list_federal_rules,
        // Filings
        rest::filing::validate_filing,
        rest::filing::submit_filing,
        rest::filing::list_jurisdictions,
        rest::filing::init_filing_upload,
        rest::filing::finalize_filing_upload,
        // NEFs
        rest::nef::get_nef,
        rest::nef::get_nef_by_id,
        rest::nef::get_nef_by_docket_entry,
        // Documents
        rest::document::promote_attachment,
        rest::document::seal_document,
        rest::document::unseal_document,
        rest::document::replace_document,
        rest::document::strike_document,
        rest::document::list_document_events,
        // Events
        rest::event::submit_event,
        rest::event::get_case_timeline,
        // Court Role Membership
        rest::membership::list_pending_requests,
        rest::membership::approve_request,
        rest::membership::deny_request,
        rest::membership::get_user_court_roles,
        rest::membership::remove_court_role,
        rest::membership::set_court_role,
        // Parties
        rest::party::create_party,
        rest::party::get_party,
        rest::party::update_party,
        rest::party::delete_party,
        rest::party::list_parties_by_case,
        rest::party::list_parties_by_attorney,
        rest::party::update_party_status,
        rest::party::check_needs_service,
        rest::party::get_lead_counsel,
        rest::party::check_is_represented,
        rest::party::list_unrepresented,
        // Representations
        rest::representation::add_representation,
        rest::representation::get_representation,
        rest::representation::end_representation,
        rest::representation::list_active_by_attorney,
        rest::representation::list_by_case,
        rest::representation::substitute_attorney,
        rest::representation_ext::migrate_representation,
        // Service Records
        rest::service_record::list_service_records,
        rest::service_record::create_service_record,
        rest::service_record::list_by_document,
        rest::service_record::list_by_party,
        rest::service_record::bulk_create,
        rest::service_record::complete_service_record,
        // Judges
        rest::judge::create_judge,
        rest::judge::list_judges,
        rest::judge::search_judges,
        rest::judge::list_judges_by_status,
        rest::judge::get_judge,
        rest::judge::update_judge,
        rest::judge::delete_judge,
        rest::judge::update_judge_status,
        rest::judge::list_available,
        rest::judge::get_workload,
        rest::judge::list_by_district,
        rest::judge::list_on_vacation,
        rest::judge::check_conflicts_for_party,
        rest::judge::get_assignment_history,
        rest::judge::process_recusal,
        // Judge Conflicts
        rest::judge::create_conflict,
        rest::judge::list_conflicts,
        rest::judge::get_conflict,
        rest::judge::delete_conflict,
        // Case Assignments
        rest::judge::create_assignment,
        rest::judge::list_assignments_by_case,
        rest::judge::delete_assignment,
        // Recusal Motions
        rest::judge::create_recusal,
        rest::judge::update_recusal_ruling,
        rest::judge::list_pending_recusals,
        rest::judge::list_recusals_by_case,
        rest::judge::list_recusals_by_judge,
        // Orders
        rest::order::create_order,
        rest::order::list_orders,
        rest::order::get_order,
        rest::order::update_order,
        rest::order::delete_order,
        rest::order::list_orders_by_case,
        rest::order::list_orders_by_judge,
        rest::order::sign_order,
        rest::order::issue_order,
        rest::order::serve_order,
        rest::order::check_expired,
        rest::order::check_requires_attention,
        rest::order::list_pending_signatures,
        rest::order::list_expiring,
        rest::order::order_statistics,
        rest::order::create_from_template,
        rest::order::generate_content,
        // Order Templates
        rest::order::create_template,
        rest::order::list_templates,
        rest::order::list_active_templates,
        rest::order::get_template,
        rest::order::update_template,
        rest::order::delete_template,
        // Opinions
        rest::opinion::create_opinion,
        rest::opinion::list_opinions,
        rest::opinion::search_opinions,
        rest::opinion::get_opinion,
        rest::opinion::update_opinion,
        rest::opinion::delete_opinion,
        rest::opinion::list_opinions_by_case,
        rest::opinion::list_opinions_by_judge,
        rest::opinion::add_vote,
        rest::opinion::add_citation,
        rest::opinion::add_headnote,
        rest::opinion::create_draft,
        rest::opinion::list_drafts,
        rest::opinion::get_current_draft,
        rest::opinion::add_draft_comment,
        rest::opinion::resolve_comment,
        rest::opinion::file_opinion,
        rest::opinion::publish_opinion,
        rest::opinion::check_is_majority,
        rest::opinion::check_is_binding,
        rest::opinion::calculate_statistics,
        rest::opinion::list_precedential,
        rest::opinion::citation_statistics,
        // Sentencing
        rest::sentencing::create_sentencing,
        rest::sentencing::get_sentencing,
        rest::sentencing::update_sentencing,
        rest::sentencing::delete_sentencing,
        rest::sentencing::list_by_case,
        rest::sentencing::list_by_defendant,
        rest::sentencing::list_by_judge,
        rest::sentencing::list_pending,
        rest::sentencing::calculate_guidelines,
        rest::sentencing::departure_stats,
        rest::sentencing::variance_stats,
        rest::sentencing::judge_stats,
        rest::sentencing::district_stats,
        rest::sentencing::trial_penalty_stats,
        rest::sentencing::offense_stats,
        rest::sentencing::record_departure,
        rest::sentencing::record_variance,
        rest::sentencing::list_substantial_assistance,
        rest::sentencing::add_special_condition,
        rest::sentencing::update_supervised_release,
        rest::sentencing::list_active_supervision,
        rest::sentencing::add_bop_designation,
        rest::sentencing::list_rdap_eligible,
        rest::sentencing::add_prior_sentence,
        rest::sentencing::list_upcoming,
        rest::sentencing::list_appeal_deadlines,
        rest::sentencing::list_by_date_range,
        rest::sentencing::calc_history_points,
        rest::sentencing::calc_offense_level,
        rest::sentencing::lookup_guidelines,
        rest::sentencing::check_safety_valve,
        // Conflict Checks
        rest::conflict_check::create_conflict_check,
        rest::conflict_check::list_by_attorney,
        rest::conflict_check::run_conflict_check,
        rest::conflict_check::clear_conflict,
        // Config
        rest::config::get_config,
        rest::config::get_district_overrides,
        rest::config::set_district_override,
        rest::config::delete_district_override,
        rest::config::get_judge_overrides,
        rest::config::set_judge_override,
        rest::config::delete_judge_override,
        rest::config::preview_config,
        // Rules
        rest::rule::list_rules,
        rest::rule::create_rule,
        rest::rule::get_rule,
        rest::rule::update_rule,
        rest::rule::delete_rule,
        rest::rule::list_by_category,
        rest::rule::list_by_trigger,
        rest::rule::list_by_jurisdiction,
        rest::rule::evaluate_rules,
        // Features
        rest::feature::list_features,
        rest::feature::update_features,
        rest::feature::get_impl,
        rest::feature::update_impl,
        rest::feature::list_blocked,
        rest::feature::list_ready,
        rest::feature::manage_feature,
        rest::feature::check_enabled,
        rest::feature::set_override,
        rest::feature::clear_overrides,
        // TODOs
        rest::todo::list_todos,
        rest::todo::create_todo,
        rest::todo::get_todo,
        rest::todo::delete_todo,
        rest::todo::toggle_todo,
        // Victims
        rest::victim::list_victims,
        rest::victim::add_victim,
        rest::victim::send_notification,
        // Signatures
        rest::signature::upload_signature,
        rest::signature::get_signature,
        // PDF Generation
        rest::pdf::generate_rule16b,
        rest::pdf::generate_signed_rule16b,
        rest::pdf::generate_court_order,
        rest::pdf::generate_minute_entry,
        rest::pdf::generate_waiver,
        rest::pdf::generate_conditions,
        rest::pdf::generate_judgment,
        rest::pdf::batch_generate,
        // Admin
        rest::admin::init_tenant,
        rest::admin::tenant_stats,
        health::health_check,
    ),
    components(schemas(
        // Template SaaS schemas
        User, Product, DashboardStats, AppError, AppErrorKind,
        CreateUserRequest, UpdateUserRequest, CreateProductRequest, UpdateProductRequest,
        AuthUser, UserTier, LoginRequest, RegisterRequest, AuthResponse,
        UpdateProfileRequest, UpdateTierRequest,
        CheckoutRequest, CheckoutResponse, SubscriptionStatus,
        ForgotPasswordRequest, ResetPasswordRequest, MessageResponse,
        SendPhoneVerificationRequest, VerifyPhoneRequest, BillingEvent,
        InitiateDeviceRequest, PollDeviceRequest, ApproveDeviceRequest,
        DeviceFlowInitResponse, DeviceFlowPollResponse, DeviceAuthStatus,
        // Common schemas
        Address, Court, TenantStats, CourtRoleRequestResponse, ReviewCourtRoleRequest,
        SetCourtRoleRequest,
        // Attorney schemas
        AttorneyResponse, CreateAttorneyRequest, UpdateAttorneyRequest, BulkUpdateStatusRequest,
        BarAdmissionResponse, CreateBarAdmissionRequest,
        FederalAdmissionResponse, CreateFederalAdmissionRequest,
        DisciplineRecordResponse, CreateDisciplineRecordRequest,
        ProHacViceResponse, CreateProHacViceRequest, UpdatePhvStatusRequest,
        CjaAppointmentResponse, CreateCjaAppointmentRequest,
        EcfRegistrationResponse, UpsertEcfRegistrationRequest,
        AttorneyMetrics, AttorneyCaseLoad, GoodStandingResult, CanPracticeResult, WinRateResult,
        ConflictCheckRequest, ConflictCheckResult, AttorneyAddToCaseRequest,
        PracticeAreaResponse, AddPracticeAreaRequest,
        // Calendar schemas
        CalendarEntryResponse, CalendarSearchResponse, CalendarEventType, EventStatus,
        ScheduleEventRequest, UpdateEventStatusRequest, CourtUtilization, AvailableSlot,
        // Case schemas
        CaseResponse, CaseSearchResponse, CreateCaseRequest, UpdateCaseRequest,
        UpdateCaseStatusRequest, CaseStatistics, PleaRequest, UpdatePriorityRequest,
        SealCaseRequest, AddCaseEventRequest,
        // Defendant schemas
        DefendantResponse, CreateDefendantRequest, UpdateDefendantRequest,
        // Charge schemas
        ChargeResponse, CreateChargeRequest, UpdateChargeRequest,
        // Motion schemas
        MotionResponse, CreateMotionRequest, UpdateMotionRequest,
        // Evidence schemas
        EvidenceResponse, CreateEvidenceRequest, UpdateEvidenceRequest,
        CustodyTransferResponse, CreateCustodyTransferRequest,
        // Case note schemas
        CaseNoteResponse, CreateCaseNoteRequest, UpdateCaseNoteRequest,
        // Speedy trial schemas
        SpeedyTrialResponse, StartSpeedyTrialRequest, UpdateSpeedyTrialClockRequest,
        ExcludableDelayResponse, CreateExcludableDelayRequest, DeadlineCheckResponse,
        // Docket schemas
        DocketEntryResponse, DocketSearchResponse, CreateDocketEntryRequest, LinkDocumentRequest,
        DocketAttachmentResponse, CreateAttachmentRequest, CreateAttachmentResponse,
        DocketSheet, DocketStatistics, FilingStatsResponse, ServiceCheckResponse,
        // Deadline schemas
        DeadlineResponse, DeadlineSearchResponse, CreateDeadlineRequest, UpdateDeadlineRequest,
        UpdateDeadlineStatusRequest, CalculateDeadlineRequest, CalculateDeadlineResponse,
        // Compliance schemas
        ComplianceStats, ComplianceReport,
        // Extension schemas
        ExtensionResponse, CreateExtensionRequest, UpdateExtensionRulingRequest,
        // Reminder schemas
        ReminderResponse, SendReminderRequest,
        // Federal rule schemas
        FederalRule,
        // Service record schemas
        ServiceRecord, ServiceRecordResponse, CreateServiceRecordRequest,
        BulkCreateServiceRecordRequest, ServiceMethod,
        // Document schemas
        Document, DocumentResponse, DocumentEventResponse, PromoteAttachmentRequest,
        SealDocumentRequest, ReplaceDocumentRequest, SealingLevel, UserRole,
        // Filing schemas
        Filing, ValidateFilingRequest, ValidateFilingResponse, FilingValidationError,
        FilingResponse, Nef, NefResponse, NefSummary, JurisdictionInfo,
        FilingUpload, InitFilingUploadRequest, InitFilingUploadResponse,
        FinalizeFilingUploadResponse,
        // Event schemas
        SubmitEventRequest, SubmitEventResponse, TimelineEntry, TimelineResponse, EventKind,
        // Party & Representation schemas
        Party, Representation, CreatePartyRequest, UpdatePartyRequest,
        UpdatePartyStatusRequest, PartyResponse,
        CreateRepresentationRequest, EndRepresentationRequest, SubstituteAttorneyRequest,
        RepresentationResponse, MigrateRepresentationRequest,
        // Conflict check schemas
        ConflictCheck, ConflictCheckResponse, CreateConflictCheckRequest,
        RunConflictCheckRequest, RunConflictCheckResult,
        // Judge schemas
        JudgeResponse, CreateJudgeRequest, UpdateJudgeRequest, UpdateJudgeStatusRequest,
        JudgeConflictResponse, CreateJudgeConflictRequest,
        CaseAssignmentResponse, CreateCaseAssignmentRequest,
        RecusalMotionResponse, CreateRecusalMotionRequest, UpdateRecusalRulingRequest,
        JudgeWorkload, AssignmentHistory,
        // Order schemas
        JudicialOrderResponse, CreateJudicialOrderRequest, UpdateJudicialOrderRequest,
        OrderTemplateResponse, CreateOrderTemplateRequest, UpdateOrderTemplateRequest,
        SignOrderRequest, IssueOrderRequest, ServeOrderRequest, OrderStatistics,
        CreateFromTemplateRequest, GenerateContentRequest,
        // Opinion schemas
        JudicialOpinionResponse, CreateJudicialOpinionRequest, UpdateJudicialOpinionRequest,
        OpinionVoteResponse, CreateOpinionVoteRequest,
        OpinionCitationResponse, CreateOpinionCitationRequest,
        HeadnoteResponse, CreateHeadnoteRequest,
        OpinionDraftResponse, CreateOpinionDraftRequest,
        DraftCommentResponse, CreateDraftCommentRequest,
        OpinionStatistics, CitationStatistics,
        // Sentencing schemas
        SentencingResponse, CreateSentencingRequest, UpdateSentencingRequest,
        SpecialConditionResponse, CreateSpecialConditionRequest,
        BopDesignationResponse, CreateBopDesignationRequest,
        PriorSentenceResponse, CreatePriorSentenceRequest,
        DepartureRequest, VarianceRequest, SupervisedReleaseRequest,
        GuidelinesRequest, GuidelinesResult, OffenseLevelRequest, SentencingStatistics,
        // Config schemas
        ConfigOverrideResponse, SetConfigOverrideRequest,
        JudgeSignatureResponse, CreateSignatureRequest,
        FeatureFlagResponse, UpdateFeatureFlagRequest, SetFeatureOverrideRequest,
        FeatureStatusResponse, ConfigPreviewRequest,
        // Rule schemas
        RuleResponse, CreateRuleRequest, UpdateRuleRequest,
        EvaluateRulesRequest, EvaluateRulesResponse,
        // Todo schemas
        TodoResponse, CreateTodoRequest,
        // Victim schemas
        VictimResponse, CreateVictimRequest,
        VictimNotificationResponse, SendVictimNotificationRequest,
        // PDF schemas
        PdfGenerateRequest, BatchPdfRequest, BatchPdfItem, BatchPdfResponseItem,
        health::HealthResponse,
    )),
    tags(
        // Template SaaS tags
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "products", description = "Product management endpoints"),
        (name = "dashboard", description = "Dashboard statistics"),
        (name = "billing", description = "Billing and subscription endpoints"),
        (name = "account", description = "Account management endpoints"),
        (name = "webhooks", description = "Webhook receivers"),
        // Lexodus court domain tags
        (name = "attorneys", description = "Attorney management endpoints"),
        (name = "calendar", description = "Calendar event management endpoints"),
        (name = "cases", description = "Case management endpoints"),
        (name = "defendants", description = "Defendant management endpoints"),
        (name = "charges", description = "Criminal charge management endpoints"),
        (name = "motions", description = "Motion management endpoints"),
        (name = "evidence", description = "Evidence and custody chain management endpoints"),
        (name = "case-notes", description = "Case note management endpoints"),
        (name = "speedy-trial", description = "Speedy Trial Act clock and delay management"),
        (name = "docket", description = "Docket entry management endpoints"),
        (name = "deadlines", description = "Deadline management endpoints"),
        (name = "documents", description = "Document management endpoints"),
        (name = "filings", description = "Electronic filing submission endpoints"),
        (name = "events", description = "Unified event composition and timeline endpoints"),
        (name = "nefs", description = "Notice of Electronic Filing endpoints"),
        (name = "parties", description = "Party management endpoints"),
        (name = "representations", description = "Representation lifecycle endpoints"),
        (name = "service-records", description = "Service record management endpoints"),
        (name = "judges", description = "Judge management endpoints"),
        (name = "judge-conflicts", description = "Judge conflict of interest management"),
        (name = "case-assignments", description = "Case-to-judge assignment management"),
        (name = "recusals", description = "Recusal motion management"),
        (name = "orders", description = "Judicial order management endpoints"),
        (name = "order-templates", description = "Order template management endpoints"),
        (name = "opinions", description = "Judicial opinion management endpoints"),
        (name = "sentencing", description = "Sentencing record management endpoints"),
        (name = "conflict-checks", description = "Conflict of interest check endpoints"),
        (name = "config", description = "Configuration override management"),
        (name = "rules", description = "Legal rule management endpoints"),
        (name = "features", description = "Feature flag management endpoints"),
        (name = "compliance", description = "Deadline compliance reporting"),
        (name = "reminders", description = "Deadline reminder management"),
        (name = "extensions", description = "Deadline extension management"),
        (name = "federal-rules", description = "Federal rule reference data"),
        (name = "todos", description = "TODO task management"),
        (name = "victims", description = "Victim notification management (CVRA)"),
        (name = "signatures", description = "Electronic signature management"),
        (name = "pdf", description = "PDF document generation"),
        (name = "courtrooms", description = "Courtroom management endpoints"),
        (name = "admin", description = "Tenant administration endpoints"),
        (name = "health", description = "Health check endpoint")
    ),
    info(
        title = "Lexodus API",
        description = "Federal Court Case Management System API",
        version = "1.0.0"
    )
)]
pub struct ApiDoc;

/// Build an Axum router that serves the API docs at `/docs`
/// and the REST API at `/api/*`.
pub fn api_router(pool: Pool<Postgres>) -> Router {
    let search = std::sync::Arc::new(crate::search::SearchIndex::new());
    let state = AppState { pool, search };
    let flags = crate::config::feature_flags();

    let mut router = Router::new()
        .merge(rest::api_router())
        .route("/health", axum::routing::get(health::health_check));

    if flags.oauth {
        router = router.route(
            "/auth/callback/{provider}",
            axum::routing::get(crate::auth::oauth_callback::oauth_callback),
        );
    }

    router
        .with_state(state)
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
}
