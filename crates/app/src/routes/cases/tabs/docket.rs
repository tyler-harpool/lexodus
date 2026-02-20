use dioxus::prelude::*;
use shared_types::{
    CreateServiceRecordRequest, DocketAttachmentResponse, DocketEntryResponse,
    NefResponse, ServiceRecordResponse, SubmitEventRequest, SubmitEventResponse, UserRole,
    VALID_DOCUMENT_TYPES,
};
use shared_ui::components::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader, DataTable,
    DataTableBody, DataTableCell, DataTableColumn, DataTableHeader, DataTableRow, FormSelect,
    Input, Separator, Skeleton, Textarea, Tooltip, TooltipContent, TooltipTrigger,
};

use crate::auth::{can, use_auth, use_user_role, Action};
use crate::CourtContext;

#[component]
pub fn DocketTab(case_id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let cid = case_id.clone();

    let mut docket_data = use_resource(move || {
        let court = court_id.clone();
        let case = cid.clone();
        async move {
            server::api::get_case_docket(court, case, None, Some(100)).await.ok()
        }
    });

    let mut show_composer = use_signal(|| false);
    let role = use_user_role();
    let can_add_entry = can(&role, Action::CreateDocketEntry);

    rsx! {
        div { class: "docket-section",
            div { class: "docket-header",
                h3 { "Docket Entries" }
                div { class: "flex items-center gap-2",
                    if can_add_entry {
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                let current = *show_composer.read();
                                show_composer.set(!current);
                            },
                            if *show_composer.read() { "Cancel" } else { "New Docket Event" }
                        }
                    }
                }
            }

            if *show_composer.read() {
                EventComposer {
                    case_id: case_id.clone(),
                    can_add_text_entry: can_add_entry,
                    on_submitted: move || {
                        show_composer.set(false);
                        docket_data.restart();
                    },
                }
            }

            match &*docket_data.read() {
                Some(Some(resp)) if !resp.entries.is_empty() => rsx! {
                    DocketTable { entries: resp.entries.clone() }
                },
                Some(Some(_)) => rsx! {
                    Card {
                        CardContent {
                            p { "No docket entries yet." }
                        }
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            p { "Failed to load docket entries." }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading",
                        Skeleton {}
                    }
                },
            }
        }
    }
}

/// Unified event composer — replaces separate "Add Text Entry" and "File Document" forms.
/// Adapts the form fields shown based on the selected event kind.
#[component]
fn EventComposer(
    case_id: String,
    can_add_text_entry: bool,
    on_submitted: EventHandler<()>,
) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let auth = use_auth();
    let current_user_name = auth
        .current_user
        .read()
        .as_ref()
        .map(|u| u.display_name.clone())
        .unwrap_or_default();

    // Event kind selection
    let mut event_kind = use_signal(|| {
        if can_add_text_entry {
            "text_entry".to_string()
        } else {
            "filing".to_string()
        }
    });

    // Text entry fields
    let mut entry_type = use_signal(|| "motion".to_string());
    let mut description = use_signal(String::new);

    // Filing fields
    let mut document_type = use_signal(|| "Motion".to_string());
    let mut title = use_signal(String::new);
    let filed_by = use_signal(move || current_user_name.clone());
    let mut is_sealed = use_signal(|| false);
    let mut sealing_level = use_signal(|| "SealedCourtOnly".to_string());
    let mut reason_code = use_signal(String::new);

    // File attachment fields (filing only)
    let mut file_name: Signal<Option<String>> = use_signal(|| None);
    let mut file_content_type: Signal<Option<String>> = use_signal(|| None);
    let mut file_bytes: Signal<Option<Vec<u8>>> = use_signal(|| None);
    let mut file_size = use_signal(|| 0i64);
    let mut upload_id: Signal<Option<String>> = use_signal(|| None);

    let mut error_msg = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);
    // After successful submit, holds the parsed response for the receipt panel
    let mut submit_result = use_signal(|| None::<SubmitEventResponse>);
    let mut show_nef_modal = use_signal(|| false);
    let mut nef_html = use_signal(|| None::<String>);

    let handle_file = move |evt: FormEvent| async move {
        let files = evt.files();
        if let Some(f) = files.first() {
            let name = f.name();
            let ct = f
                .content_type()
                .unwrap_or_else(|| mime_from_filename(&name));
            match f.read_bytes().await {
                Ok(bytes) => {
                    file_size.set(bytes.len() as i64);
                    file_bytes.set(Some(bytes.to_vec()));
                    file_content_type.set(Some(ct));
                    file_name.set(Some(name));
                }
                Err(_) => {
                    error_msg.set(Some("Failed to read file.".to_string()));
                }
            }
        }
    };

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let court = court_id.clone();
        let cid = case_id.clone();
        let kind = event_kind.read().clone();

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            // Upload file first if present (filing only)
            if kind == "filing" {
                if let (Some(ref name), Some(ref ct), Some(ref bytes)) = (
                    &*file_name.read(),
                    &*file_content_type.read(),
                    &*file_bytes.read(),
                ) {
                    match server::api::upload_filing_document(
                        court.clone(),
                        name.clone(),
                        ct.clone(),
                        *file_size.read(),
                        bytes.clone(),
                    )
                    .await
                    {
                        Ok(uid) => upload_id.set(Some(uid)),
                        Err(e) => {
                            error_msg.set(Some(format!("Upload failed: {}", e)));
                            submitting.set(false);
                            return;
                        }
                    }
                }
            }

            // Build the unified event request
            let mut req = SubmitEventRequest {
                event_kind: kind.clone(),
                case_id: cid.clone(),
                entry_type: None,
                description: None,
                document_type: None,
                title: None,
                filed_by: None,
                upload_id: None,
                is_sealed: None,
                sealing_level: None,
                reason_code: None,
                attachment_id: None,
                promote_title: None,
                promote_document_type: None,
            };

            match kind.as_str() {
                "text_entry" => {
                    let desc = description.read().clone();
                    let et = entry_type.read().clone();
                    if desc.trim().is_empty() {
                        error_msg.set(Some("Description is required.".to_string()));
                        submitting.set(false);
                        return;
                    }
                    req.entry_type = Some(et);
                    req.description = Some(desc);
                    let fb = filed_by.read().clone();
                    if !fb.trim().is_empty() {
                        req.filed_by = Some(fb.trim().to_string());
                    }
                }
                "filing" => {
                    let t = title.read().clone();
                    let dt = document_type.read().clone();
                    let fb = filed_by.read().clone();
                    if t.trim().is_empty() {
                        error_msg.set(Some("Title is required.".to_string()));
                        submitting.set(false);
                        return;
                    }
                    if fb.trim().is_empty() {
                        error_msg.set(Some("Filed By is required.".to_string()));
                        submitting.set(false);
                        return;
                    }
                    req.document_type = Some(dt);
                    req.title = Some(t);
                    req.filed_by = Some(fb);
                    if let Some(ref uid) = *upload_id.read() {
                        req.upload_id = Some(uid.clone());
                    }
                    if *is_sealed.read() {
                        req.is_sealed = Some(true);
                        req.sealing_level = Some(sealing_level.read().clone());
                        let rc = reason_code.read().clone();
                        if !rc.is_empty() {
                            req.reason_code = Some(rc);
                        }
                    }
                }
                _ => {}
            }

            match server::api::submit_event(court.clone(), req).await {
                Ok(response) => {
                    submit_result.set(Some(response));
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed: {}", e)));
                }
            }
            submitting.set(false);
        });
    };

    // If we have a submit result, fetch the NEF HTML for the modal
    let fetch_nef_html = {
        let ctx2 = use_context::<CourtContext>();
        move |nef_id: String| {
            let court = ctx2.court_id.read().clone();
            spawn(async move {
                match server::api::get_nef_by_id(court, nef_id).await {
                    Ok(Some(nef)) => {
                        if let Some(html) = nef.html_snapshot {
                            nef_html.set(Some(html));
                            show_nef_modal.set(true);
                        }
                    }
                    _ => {}
                }
            });
        }
    };

    let current_kind = event_kind.read().clone();

    // If submit succeeded, show receipt instead of form
    if let Some(result) = &*submit_result.read() {
        let result = result.clone();
        let kind_label = match result.event_kind.as_str() {
            "text_entry" => "Text Entry",
            "filing" => "Filing",
            "promote_attachment" => "Promote Attachment",
            _ => "Event",
        };
        let entry_number = result.entry_number;
        let has_nef = result.nef_id.is_some();
        let nef_id_val = result.nef_id.clone().unwrap_or_default();

        return rsx! {
            Card {
                CardContent {
                    div { class: "space-y-3",
                        div { class: "flex items-center gap-2 text-primary font-medium text-lg",
                            span { "\u{2713}" }
                            span { "Notice of Electronic Filing" }
                        }

                        div { class: "grid grid-cols-2 gap-2 text-sm",
                            span { class: "text-muted", "Docket #:" }
                            span { class: "font-medium", "{entry_number}" }
                            span { class: "text-muted", "Event Type:" }
                            span { class: "font-medium", "{kind_label}" }
                            if let Some(ref doc_id) = result.document_id {
                                span { class: "text-muted", "Document ID:" }
                                span { class: "font-mono text-xs", "{doc_id}" }
                            }
                        }

                        div { class: "flex items-center gap-2 mt-4",
                            if has_nef {
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: {
                                        let nid = nef_id_val.clone();
                                        move |_| {
                                            fetch_nef_html(nid.clone());
                                        }
                                    },
                                    "View Full NEF"
                                }
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| {
                                    on_submitted.call(());
                                },
                                "Done"
                            }
                        }
                    }

                    if *show_nef_modal.read() {
                        if let Some(html) = &*nef_html.read() {
                            NefModal {
                                html: html.clone(),
                                on_close: move |_| show_nef_modal.set(false),
                            }
                        }
                    }
                }
            }
        };
    }

    rsx! {
        Card {
            CardHeader { "New Docket Event" }
            CardContent {
                if let Some(err) = &*error_msg.read() {
                    div { class: "error-message", "{err}" }
                }

                form { onsubmit: handle_submit,
                    // Event kind selector
                    div { class: "form-group mb-4",
                        FormSelect {
                            label: "Event Type",
                            value: "{event_kind}",
                            onchange: move |evt: Event<FormData>| event_kind.set(evt.value().to_string()),
                            if can_add_text_entry {
                                option { value: "text_entry", "Add Text Entry" }
                            }
                            option { value: "filing", "File Document" }
                        }
                    }

                    // ── Text Entry fields ───────────────────────────
                    if current_kind == "text_entry" {
                        div { class: "form-row",
                            div { class: "form-group",
                                FormSelect {
                                    label: "Entry Type *",
                                    value: "{entry_type}",
                                    onchange: move |evt: Event<FormData>| entry_type.set(evt.value().to_string()),
                                    for et in shared_types::DOCKET_ENTRY_TYPES.iter() {
                                        option { value: "{et}", "{et}" }
                                    }
                                }
                            }
                            div { class: "form-group",
                                div { class: "form-field",
                                    label { class: "form-label", "Filed By" }
                                    p { class: "form-static-value", "{filed_by}" }
                                }
                            }
                        }
                        div { class: "form-group",
                            Textarea {
                                label: "Description *",
                                value: description.read().clone(),
                                on_input: move |evt: FormEvent| description.set(evt.value().to_string()),
                                placeholder: "Docket entry text...",
                            }
                        }
                    }

                    // ── Filing fields ────────────────────────────────
                    if current_kind == "filing" {
                        div { class: "form-row",
                            div { class: "form-group",
                                FormSelect {
                                    label: "Document Type *",
                                    value: "{document_type}",
                                    onchange: move |evt: Event<FormData>| document_type.set(evt.value().to_string()),
                                    for dt in VALID_DOCUMENT_TYPES.iter() {
                                        option { value: "{dt}", "{dt}" }
                                    }
                                }
                            }
                            div { class: "form-group",
                                div { class: "form-field",
                                    label { class: "form-label", "Filed By" }
                                    p { class: "form-static-value", "{filed_by}" }
                                }
                            }
                        }

                        div { class: "form-group",
                            Input {
                                label: "Title *",
                                value: title.read().clone(),
                                on_input: move |evt: FormEvent| title.set(evt.value().to_string()),
                                placeholder: "e.g., Motion to Compel Discovery",
                            }
                        }

                        div { class: "form-row",
                            div { class: "form-group",
                                label { class: "form-label", "Attach File" }
                                form {
                                    onchange: handle_file,
                                    input {
                                        r#type: "file",
                                        disabled: *submitting.read(),
                                    }
                                }
                                if let Some(ref name) = &*file_name.read() {
                                    div { class: "flex items-center gap-2",
                                        span { class: "text-sm text-muted", "{name} ({format_file_size(*file_size.read())})" }
                                        Button {
                                            variant: ButtonVariant::Ghost,
                                            onclick: move |_| {
                                                file_name.set(None);
                                                file_content_type.set(None);
                                                file_bytes.set(None);
                                                file_size.set(0);
                                                upload_id.set(None);
                                            },
                                            "Remove"
                                        }
                                    }
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Options" }
                                label { class: "flex items-center gap-2",
                                    input {
                                        r#type: "checkbox",
                                        checked: *is_sealed.read(),
                                        onchange: move |evt: Event<FormData>| {
                                            is_sealed.set(evt.value() == "true");
                                        },
                                    }
                                    span { "Sealed filing" }
                                }
                                if *is_sealed.read() {
                                    div { class: "mt-2",
                                        FormSelect {
                                            label: "Sealing Level",
                                            value: "{sealing_level}",
                                            onchange: move |evt: Event<FormData>| sealing_level.set(evt.value().to_string()),
                                            option { value: "SealedCourtOnly", "Court Only" }
                                            option { value: "SealedCaseParticipants", "Case Participants" }
                                            option { value: "SealedAttorneysOnly", "Attorneys Only" }
                                        }
                                    }
                                    div { class: "mt-2",
                                        FormSelect {
                                            label: "Reason Code",
                                            value: "{reason_code}",
                                            onchange: move |evt: Event<FormData>| reason_code.set(evt.value().to_string()),
                                            option { value: "", "Select reason..." }
                                            option { value: "JuvenileRecord", "Juvenile Record" }
                                            option { value: "TradeSecret", "Trade Secret" }
                                            option { value: "InformantIdentity", "Informant Identity" }
                                            option { value: "NationalSecurity", "National Security" }
                                            option { value: "GrandJury", "Grand Jury" }
                                            option { value: "SealedIndictment", "Sealed Indictment" }
                                            option { value: "ProtectiveOrder", "Protective Order" }
                                            option { value: "Other", "Other" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "form-actions",
                        button {
                            class: "button",
                            "data-style": "primary",
                            r#type: "submit",
                            disabled: *submitting.read(),
                            if *submitting.read() {
                                "Submitting..."
                            } else {
                                match current_kind.as_str() {
                                    "text_entry" => "Add Entry",
                                    "filing" => "Submit Filing",
                                    _ => "Submit",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Modal displaying the full NEF HTML snapshot.
#[component]
fn NefModal(html: String, on_close: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4",
            onclick: move |evt| on_close.call(evt),
            div {
                class: "bg-surface rounded-lg shadow-lg max-w-2xl w-full max-h-[80vh] overflow-y-auto p-6",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "flex justify-between items-center mb-4",
                    h3 { class: "text-lg font-semibold", "Notice of Electronic Filing" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |evt| on_close.call(evt),
                        "\u{2715}"
                    }
                }
                div { dangerous_inner_html: "{html}" }
            }
        }
    }
}

#[component]
fn DocketEntryForm(case_id: String, on_created: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();

    let mut entry_type = use_signal(|| "motion".to_string());
    let mut description = use_signal(String::new);
    let filed_by = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let court = court_id.clone();
        let cid = case_id.clone();
        let et = entry_type.read().clone();
        let desc = description.read().clone();
        let fb = filed_by.read().clone();

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            if desc.trim().is_empty() {
                error_msg.set(Some("Description is required.".to_string()));
                submitting.set(false);
                return;
            }

            let case_uuid = match uuid::Uuid::parse_str(&cid) {
                Ok(u) => u,
                Err(_) => {
                    error_msg.set(Some("Invalid case ID.".to_string()));
                    submitting.set(false);
                    return;
                }
            };

            let req = shared_types::CreateDocketEntryRequest {
                case_id: case_uuid,
                entry_type: et,
                description: desc.trim().to_string(),
                filed_by: if fb.trim().is_empty() { None } else { Some(fb.trim().to_string()) },
                document_id: None,
                is_sealed: false,
                is_ex_parte: false,
                page_count: None,
                related_entries: Vec::new(),
                service_list: Vec::new(),
            };

            match server::api::create_docket_entry(court, req).await {
                Ok(_) => {
                    on_created.call(());
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to create entry: {}", e)));
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        Card {
            CardHeader { "New Docket Entry" }
            CardContent {
                if let Some(err) = &*error_msg.read() {
                    div { class: "error-message", "{err}" }
                }

                form { onsubmit: handle_submit,
                    div { class: "form-group",
                        FormSelect {
                            label: "Entry Type *",
                            value: "{entry_type}",
                            onchange: move |evt: Event<FormData>| entry_type.set(evt.value().to_string()),
                            option { value: "complaint", "Complaint" }
                            option { value: "indictment", "Indictment" }
                            option { value: "motion", "Motion" }
                            option { value: "response", "Response" }
                            option { value: "reply", "Reply" }
                            option { value: "notice", "Notice" }
                            option { value: "order", "Order" }
                            option { value: "minute_order", "Minute Order" }
                            option { value: "scheduling_order", "Scheduling Order" }
                            option { value: "hearing_notice", "Hearing Notice" }
                            option { value: "hearing_minutes", "Hearing Minutes" }
                            option { value: "transcript", "Transcript" }
                            option { value: "judgment", "Judgment" }
                            option { value: "verdict", "Verdict" }
                            option { value: "sentence", "Sentence" }
                            option { value: "summons", "Summons" }
                            option { value: "subpoena", "Subpoena" }
                            option { value: "appearance", "Appearance" }
                            option { value: "withdrawal", "Withdrawal" }
                            option { value: "exhibit", "Exhibit" }
                            option { value: "other", "Other" }
                        }
                    }

                    div { class: "form-group",
                        Textarea {
                            label: "Description *",
                            value: description.read().clone(),
                            on_input: move |evt: FormEvent| description.set(evt.value().to_string()),
                            placeholder: "Describe the docket entry...",
                        }
                    }

                    div { class: "form-group",
                        div { class: "form-field",
                            label { class: "form-label", "Filed By" }
                            p { class: "form-static-value", "{filed_by}" }
                        }
                    }

                    div { class: "form-actions",
                        button {
                            class: "button",
                            "data-style": "primary",
                            r#type: "submit",
                            disabled: *submitting.read(),
                            if *submitting.read() { "Adding..." } else { "Add Entry" }
                        }
                    }
                }
            }
        }
    }
}

/// Filing form for submitting an electronic filing (creates Document + DocketEntry + Filing).
#[component]
fn FilingForm(case_id: String, on_filed: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();

    let mut document_type = use_signal(|| "Motion".to_string());
    let mut title = use_signal(String::new);
    let filed_by = use_signal(String::new);
    let mut is_sealed = use_signal(|| false);
    let mut sealing_level = use_signal(|| "SealedCourtOnly".to_string());
    let mut reason_code = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    // File upload state
    let mut file_name = use_signal(|| None::<String>);
    let mut file_content_type = use_signal(|| None::<String>);
    let mut file_bytes = use_signal(|| None::<Vec<u8>>);
    let mut file_size = use_signal(|| 0i64);

    let handle_file = move |evt: FormEvent| async move {
        let files = evt.files();
        if let Some(f) = files.first() {
            let name = f.name();
            let ct = f
                .content_type()
                .unwrap_or_else(|| mime_from_filename(&name));
            match f.read_bytes().await {
                Ok(bytes) => {
                    file_size.set(bytes.len() as i64);
                    file_bytes.set(Some(bytes.to_vec()));
                    file_content_type.set(Some(ct));
                    file_name.set(Some(name));
                }
                Err(_) => {
                    error_msg.set(Some("Failed to read file".to_string()));
                }
            }
        }
    };

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let court = court_id.clone();
        let cid = case_id.clone();
        let dt = document_type.read().clone();
        let t = title.read().clone();
        let fb = filed_by.read().clone();
        let sealed = *is_sealed.read();
        let seal_level = sealing_level.read().clone();
        let reason = reason_code.read().clone();
        let fname = file_name.read().clone();
        let fct = file_content_type.read().clone();
        let fbytes = file_bytes.read().clone();
        let fsize = *file_size.read();

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            if t.trim().is_empty() {
                error_msg.set(Some("Title is required.".to_string()));
                submitting.set(false);
                return;
            }
            if fb.trim().is_empty() {
                error_msg.set(Some("Filed By is required.".to_string()));
                submitting.set(false);
                return;
            }

            // Step 1: Upload file if provided
            let upload_id = if let (Some(name), Some(ct), Some(bytes)) = (fname, fct, fbytes) {
                match server::api::upload_filing_document(
                    court.clone(),
                    name,
                    ct,
                    fsize,
                    bytes,
                )
                .await
                {
                    Ok(id) => Some(id),
                    Err(e) => {
                        error_msg.set(Some(format!("File upload failed: {}", e)));
                        submitting.set(false);
                        return;
                    }
                }
            } else {
                None
            };

            // Step 2: Submit filing
            let req = shared_types::ValidateFilingRequest {
                case_id: cid,
                document_type: dt,
                title: t.trim().to_string(),
                filed_by: fb.trim().to_string(),
                upload_id: upload_id.clone(),
                is_sealed: if sealed { Some(true) } else { None },
                sealing_level: if sealed { Some(seal_level) } else { None },
                reason_code: if sealed && !reason.trim().is_empty() {
                    Some(reason.trim().to_string())
                } else {
                    None
                },
            };

            match server::api::submit_filing(court, req).await {
                Ok(_) => {
                    on_filed.call(());
                }
                Err(e) => {
                    error_msg.set(Some(format!("Filing failed: {}", e)));
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        Card {
            CardHeader { "File Document" }
            CardContent {
                if let Some(err) = &*error_msg.read() {
                    div { class: "error-message", "{err}" }
                }
                form { onsubmit: handle_submit,
                    div { class: "form-row",
                        div { class: "form-group",
                            FormSelect {
                                label: "Document Type *",
                                value: "{document_type}",
                                onchange: move |evt: Event<FormData>| document_type.set(evt.value().to_string()),
                                for dt in VALID_DOCUMENT_TYPES.iter() {
                                    option { value: "{dt}", "{dt}" }
                                }
                            }
                        }
                        div { class: "form-group",
                            div { class: "form-field",
                                label { class: "form-label", "Filed By" }
                                p { class: "form-static-value", "{filed_by}" }
                            }
                        }
                    }

                    div { class: "form-group",
                        Input {
                            label: "Title *",
                            value: title.read().clone(),
                            on_input: move |evt: FormEvent| title.set(evt.value().to_string()),
                            placeholder: "e.g., Motion to Compel Discovery",
                        }
                    }

                    div { class: "form-row",
                        div { class: "form-group",
                            label { class: "form-label", "Attach File" }
                            form {
                                onchange: handle_file,
                                input {
                                    r#type: "file",
                                    disabled: *submitting.read(),
                                }
                            }
                            if let Some(ref name) = &*file_name.read() {
                                div { class: "flex items-center gap-2",
                                    span { class: "text-sm text-muted", "{name} ({format_file_size(*file_size.read())})" }
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        onclick: move |_| {
                                            file_name.set(None);
                                            file_content_type.set(None);
                                            file_bytes.set(None);
                                            file_size.set(0);
                                        },
                                        "Remove"
                                    }
                                }
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Options" }
                            label { class: "flex items-center gap-2",
                                input {
                                    r#type: "checkbox",
                                    checked: *is_sealed.read(),
                                    onchange: move |evt: Event<FormData>| {
                                        is_sealed.set(evt.value() == "true");
                                    },
                                }
                                span { "Sealed filing" }
                            }
                            if *is_sealed.read() {
                                div { class: "mt-2",
                                    FormSelect {
                                        label: "Sealing Level",
                                        value: "{sealing_level}",
                                        onchange: move |evt: Event<FormData>| sealing_level.set(evt.value().to_string()),
                                        option { value: "SealedCourtOnly", "Court Only" }
                                        option { value: "SealedCaseParticipants", "Case Participants" }
                                        option { value: "SealedAttorneysOnly", "Attorneys Only" }
                                    }
                                }
                                div { class: "mt-2",
                                    FormSelect {
                                        label: "Reason Code",
                                        value: "{reason_code}",
                                        onchange: move |evt: Event<FormData>| reason_code.set(evt.value().to_string()),
                                        option { value: "", "Select reason..." }
                                        option { value: "JuvenileRecord", "Juvenile Record" }
                                        option { value: "TradeSecret", "Trade Secret" }
                                        option { value: "InformantIdentity", "Informant Identity" }
                                        option { value: "NationalSecurity", "National Security" }
                                        option { value: "GrandJury", "Grand Jury" }
                                        option { value: "SealedIndictment", "Sealed Indictment" }
                                        option { value: "ProtectiveOrder", "Protective Order" }
                                        option { value: "Other", "Other" }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "form-actions",
                        button {
                            class: "button",
                            "data-style": "primary",
                            r#type: "submit",
                            disabled: *submitting.read(),
                            if *submitting.read() { "Submitting..." } else { "Submit Filing" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DocketTable(entries: Vec<DocketEntryResponse>) -> Element {
    let mut expanded_entry = use_signal(|| None::<String>);

    rsx! {
        DataTable {
            DataTableHeader {
                DataTableColumn { "#" }
                DataTableColumn { "Type" }
                DataTableColumn { "Description" }
                DataTableColumn { "Filed By" }
                DataTableColumn { "Date Filed" }
                DataTableColumn { "Sealed" }
                DataTableColumn { "Files" }
            }
            DataTableBody {
                for entry in entries {
                    {
                        let eid = entry.id.clone();
                        let is_expanded = expanded_entry.read().as_deref() == Some(&eid);
                        rsx! {
                            DocketRow {
                                entry: entry.clone(),
                                on_toggle: move |_| {
                                    let current = expanded_entry.read().clone();
                                    if current.as_deref() == Some(&eid) {
                                        expanded_entry.set(None);
                                    } else {
                                        expanded_entry.set(Some(eid.clone()));
                                    }
                                },
                            }
                            if is_expanded {
                                EntryDetailPanel {
                                    entry_id: entry.id.clone(),
                                    case_id: entry.case_id.clone(),
                                    document_id: entry.document_id.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DocketRow(entry: DocketEntryResponse, on_toggle: EventHandler<MouseEvent>) -> Element {
    let display_type = entry.entry_type.replace('_', " ");
    let display_date = format_date(&entry.date_filed);
    let filed_by = entry.filed_by.as_deref().unwrap_or("-");

    rsx! {
        DataTableRow {
            onclick: move |evt| on_toggle.call(evt),
            DataTableCell { "{entry.entry_number}" }
            DataTableCell { "{display_type}" }
            DataTableCell { "{entry.description}" }
            DataTableCell { "{filed_by}" }
            DataTableCell {
                span { style: "white-space: nowrap;", "{display_date}" }
            }
            DataTableCell {
                if entry.is_sealed {
                    Badge { variant: BadgeVariant::Destructive, "Sealed" }
                }
            }
            DataTableCell {
                Badge { variant: BadgeVariant::Secondary, "View" }
            }
        }
    }
}

/// Unified detail panel shown when a docket row is expanded.
/// Contains three sections: Documents & Files, Filing Notice, Service.
#[component]
fn EntryDetailPanel(entry_id: String, case_id: String, document_id: Option<String>) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let eid = entry_id.clone();

    // ── Attachments data ───────────────────────────────────────────
    let mut attachments_data = use_resource(move || {
        let court = court_id.clone();
        let entry = eid.clone();
        async move {
            server::api::list_entry_attachments(court, entry)
                .await
                .unwrap_or_default()
        }
    });

    let mut uploading = use_signal(|| false);
    let mut upload_error = use_signal(|| None::<String>);
    let mut promoted_doc_id = use_signal(|| document_id.clone());

    let upload_entry_id = use_signal(|| entry_id.clone());
    let handle_upload = move |evt: FormEvent| async move {
        let court = ctx.court_id.read().clone();
        let eid = upload_entry_id.read().clone();

        uploading.set(true);
        upload_error.set(None);

        let files = evt.files();
        let file = match files.first() {
            Some(f) => f,
            None => {
                upload_error.set(Some("No file selected".to_string()));
                uploading.set(false);
                return;
            }
        };

        let fname = file.name();
        let content_type = file
            .content_type()
            .unwrap_or_else(|| mime_from_filename(&fname));
        let file_bytes = match file.read_bytes().await {
            Ok(bytes) => bytes,
            Err(_) => {
                upload_error.set(Some("Failed to read file".to_string()));
                uploading.set(false);
                return;
            }
        };

        let file_size = file_bytes.len() as i64;

        match server::api::upload_docket_attachment(
            court,
            eid,
            fname,
            content_type,
            file_size,
            file_bytes.to_vec(),
        )
        .await
        {
            Ok(_) => {
                attachments_data.restart();
            }
            Err(e) => {
                upload_error.set(Some(format!("Upload failed: {}", e)));
            }
        }

        uploading.set(false);
    };

    // ── NEF data ───────────────────────────────────────────────────
    let nef_court = ctx.court_id.read().clone();
    let nef_eid = entry_id.clone();
    let nef_data = use_resource(move || {
        let court = nef_court.clone();
        let entry = nef_eid.clone();
        async move {
            match server::api::get_nef_by_docket_entry(court, entry).await {
                Ok(nef_opt) => nef_opt,
                _ => None,
            }
        }
    });

    // ── Service records data ───────────────────────────────────────
    let effective_doc_id = promoted_doc_id.read().clone();
    let sr_court = ctx.court_id.read().clone();
    let sr_doc = effective_doc_id.clone();
    let mut records_data = use_resource(move || {
        let court = sr_court.clone();
        let doc = sr_doc.clone();
        async move {
            match doc {
                Some(did) => {
                    server::api::list_document_service_records(court, did)
                        .await
                        .unwrap_or_default()
                }
                None => vec![],
            }
        }
    });

    let mut show_service_form = use_signal(|| false);

    rsx! {
        tr {
            td { colspan: "7",
                Card {
                    CardContent {
                        // ── Section 1: Documents & Files (always shown) ────
                        FilesSection {
                            attachments_data: attachments_data.clone(),
                            uploading: uploading,
                            upload_error: upload_error,
                            handle_upload: handle_upload,
                            on_promoted: move |doc_id: String| {
                                promoted_doc_id.set(Some(doc_id));
                            },
                        }

                        // ── Section 2: Document Actions (clerk/judge only) ──
                        {
                            let role = use_user_role();
                            let can_manage_docs = matches!(role, UserRole::Clerk | UserRole::Judge | UserRole::Admin);
                            if effective_doc_id.is_some() && can_manage_docs {
                                rsx! {
                                    DocumentActionsSection {
                                        document_id: effective_doc_id.clone().unwrap_or_default(),
                                        on_updated: move || {
                                            attachments_data.restart();
                                        },
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }

                        // ── Section 3: Filing Notice (only if NEF exists) ──
                        FilingNoticeSection { nef_data: nef_data.clone() }

                        // ── Section 4: Service (only if document linked) ───
                        if effective_doc_id.is_some() {
                            ServiceSection {
                                case_id: case_id.clone(),
                                document_id: effective_doc_id.clone().unwrap_or_default(),
                                records_data: records_data.clone(),
                                show_form: show_service_form,
                                on_form_toggle: move |_| {
                                    let current = *show_service_form.read();
                                    show_service_form.set(!current);
                                },
                                on_created: move || {
                                    show_service_form.set(false);
                                    records_data.restart();
                                },
                                on_completed: move || {
                                    records_data.restart();
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Section 1: Documents & Files — file list + upload.
#[component]
fn FilesSection(
    attachments_data: Resource<Vec<DocketAttachmentResponse>>,
    uploading: Signal<bool>,
    upload_error: Signal<Option<String>>,
    handle_upload: EventHandler<FormEvent>,
    on_promoted: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-between mb-3",
            h4 { class: "font-medium text-base", "Documents & Files" }
            form {
                onchange: move |evt| handle_upload.call(evt),
                label {
                    class: "inline-flex items-center gap-1 px-3 py-1 rounded text-sm font-medium bg-surface-alt text-secondary border border-border hover:bg-surface-hover transition-colors cursor-pointer",
                    input {
                        r#type: "file",
                        class: "hidden",
                        disabled: *uploading.read(),
                        multiple: false,
                    }
                    if *uploading.read() { "Uploading..." } else { "Attach File" }
                }
            }
        }

        if let Some(err) = &*upload_error.read() {
            div { class: "error-message mb-2", "{err}" }
        }

        match &*attachments_data.read() {
            Some(atts) if !atts.is_empty() => rsx! {
                DataTable {
                    DataTableHeader {
                        DataTableColumn { "Filename" }
                        DataTableColumn { "Size" }
                        DataTableColumn { "Type" }
                        DataTableColumn { "Uploaded" }
                        DataTableColumn { "Actions" }
                    }
                    DataTableBody {
                        for att in atts.iter() {
                            AttachmentRow {
                                attachment: att.clone(),
                                on_promoted: move |doc_id: String| {
                                    on_promoted.call(doc_id);
                                },
                            }
                        }
                    }
                }
            },
            Some(_) => rsx! {
                p { class: "text-muted", "No files attached." }
            },
            None => rsx! {
                Skeleton {}
            },
        }
    }
}

/// Section 2: Filing Notice — compact view of NEF recipients + View Full NEF button.
#[component]
fn FilingNoticeSection(nef_data: Resource<Option<NefResponse>>) -> Element {
    let mut show_nef_modal = use_signal(|| false);
    let nef_value = nef_data.read().clone();
    match nef_value.as_ref() {
        Some(Some(nef)) => {
            let nef = nef.clone();
            let recipients_list = nef.recipients.as_array().cloned().unwrap_or_default();
            let notified_date = format_date(&nef.created_at);
            let has_snapshot = nef.html_snapshot.is_some();
            let snapshot_html = nef.html_snapshot.clone().unwrap_or_default();
            rsx! {
                Separator {}
                div { class: "mt-4",
                    div { class: "flex items-center gap-2 mb-2",
                        h4 { class: "font-medium text-base", "Filing Notice" }
                        Badge { variant: BadgeVariant::Primary, "Sent" }
                        if has_snapshot {
                            Button {
                                variant: ButtonVariant::Ghost,
                                onclick: move |_| show_nef_modal.set(true),
                                "View Full NEF"
                            }
                        }
                    }
                    p { class: "text-sm text-muted mb-3", "Notified on {notified_date}" }

                    if !recipients_list.is_empty() {
                        div { class: "space-y-1",
                            for r in recipients_list.iter() {
                                {
                                    let name = r["name"].as_str().unwrap_or("-");
                                    let method = r["service_method"].as_str().unwrap_or("-");
                                    let is_electronic = r["electronic"].as_bool().unwrap_or(false);
                                    rsx! {
                                        div { class: "flex items-center gap-2 text-sm py-1",
                                            if is_electronic {
                                                span { class: "text-primary", "\u{2713}" }
                                            } else {
                                                span { class: "text-muted", "\u{25CB}" }
                                            }
                                            span { "{name}" }
                                            span { class: "text-muted", "\u{2014}" }
                                            Badge { variant: BadgeVariant::Secondary, "{method}" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if *show_nef_modal.read() && has_snapshot {
                        NefModal {
                            html: snapshot_html.clone(),
                            on_close: move |_| show_nef_modal.set(false),
                        }
                    }
                }
            }
        },
        // No NEF — don't render anything
        _ => rsx! {},
    }
}

/// Section 3: Service — progress bar, records table, inline form.
#[component]
fn ServiceSection(
    case_id: String,
    document_id: String,
    records_data: Resource<Vec<ServiceRecordResponse>>,
    show_form: Signal<bool>,
    on_form_toggle: EventHandler<MouseEvent>,
    on_created: EventHandler<()>,
    on_completed: EventHandler<()>,
) -> Element {
    // Reactively compute progress from the resource
    let progress_memo = use_memo(move || {
        match &*records_data.read() {
            Some(recs) if !recs.is_empty() => {
                let total = recs.len();
                let served = recs.iter().filter(|r| r.successful && r.proof_of_service_filed).count();
                (served, total)
            }
            _ => (0, 0),
        }
    });

    let (served_count, total_count) = *progress_memo.read();
    let summary_text = format!("{} of {} served", served_count, total_count);

    rsx! {
        Separator {}
        div { class: "mt-4",
            // Header row
            div { class: "flex items-center justify-between mb-2",
                div { class: "flex items-center gap-3",
                    h4 { class: "font-medium text-base", "Service" }
                    if total_count > 0 {
                        span { class: "text-sm text-muted", "{summary_text}" }
                    }
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |evt| on_form_toggle.call(evt),
                    if *show_form.read() { "Cancel" } else { "Record Service" }
                }
            }

            // Progress bar
            if total_count > 0 {
                div { class: "mb-3",
                    div {
                        class: "progress",
                        role: "progressbar",
                        "aria-valuemin": "0",
                        "aria-valuemax": "{total_count}",
                        "aria-valuenow": "{served_count}",
                        style: "--progress-value: {(served_count as f64 / total_count as f64) * 100.0}%",
                        div { class: "progress-indicator" }
                    }
                }
            }

            // Inline form
            if *show_form.read() {
                ServiceRecordForm {
                    case_id: case_id.clone(),
                    document_id: document_id.clone(),
                    on_created: move || on_created.call(()),
                }
            }

            // Records table
            match &*records_data.read() {
                Some(recs) if !recs.is_empty() => rsx! {
                    DataTable {
                        DataTableHeader {
                            DataTableColumn { "Party" }
                            DataTableColumn { "Role" }
                            DataTableColumn { "Method" }
                            DataTableColumn { "Date" }
                            DataTableColumn { "Status" }
                            DataTableColumn { "Actions" }
                        }
                        DataTableBody {
                            for rec in recs.iter() {
                                ServiceRecordTableRow {
                                    record: rec.clone(),
                                    on_completed: move || on_completed.call(()),
                                }
                            }
                        }
                    }
                },
                Some(_) => rsx! {
                    p { class: "text-sm text-muted", "No service records yet." }
                },
                None => rsx! {
                    Skeleton {}
                },
            }
        }
    }
}

/// A single service record row in the service table.
/// Must be a separate component so hooks (use_signal) work correctly.
#[component]
fn ServiceRecordTableRow(record: ServiceRecordResponse, on_completed: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let record_id = record.id.clone();
    let mut completing = use_signal(|| false);

    let is_served = record.successful && record.proof_of_service_filed;
    let display_date = format_date(&record.service_date);

    let handle_mark_served = move |_: MouseEvent| {
        let court = court_id.clone();
        let rid = record_id.clone();
        spawn(async move {
            completing.set(true);
            match server::api::complete_service_record(court, rid).await {
                Ok(_) => on_completed.call(()),
                Err(_) => {}
            }
            completing.set(false);
        });
    };

    rsx! {
        DataTableRow {
            DataTableCell { "{record.party_name}" }
            DataTableCell {
                Badge { variant: BadgeVariant::Outline, "{record.party_type}" }
            }
            DataTableCell { "{record.service_method}" }
            DataTableCell { "{display_date}" }
            DataTableCell {
                if is_served {
                    Badge { variant: BadgeVariant::Primary, "Served" }
                } else {
                    Badge { variant: BadgeVariant::Secondary, "Pending" }
                }
            }
            DataTableCell {
                if !is_served {
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: handle_mark_served,
                        disabled: *completing.read(),
                        if *completing.read() { "..." } else { "Mark Served" }
                    }
                }
            }
        }
    }
}

/// Inline form for recording service.
#[component]
fn ServiceRecordForm(case_id: String, document_id: String, on_created: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let cid = case_id.clone();

    // Fetch parties for the dropdown
    let parties_data = use_resource(move || {
        let court = court_id.clone();
        let case = cid.clone();
        async move {
            server::api::list_case_parties(court, case)
                .await
                .unwrap_or_default()
        }
    });

    let mut party_id_input = use_signal(String::new);
    let mut service_method = use_signal(|| "Electronic".to_string());
    let mut served_by = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let form_court = ctx.court_id.read().clone();
    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let court = form_court.clone();
        let doc_id = document_id.clone();
        let pid = party_id_input.read().clone();
        let method = service_method.read().clone();
        let sb = served_by.read().clone();
        let n = notes.read().clone();

        spawn(async move {
            submitting.set(true);
            error_msg.set(None);

            if pid.is_empty() {
                error_msg.set(Some("Please select a party.".to_string()));
                submitting.set(false);
                return;
            }
            if sb.trim().is_empty() {
                error_msg.set(Some("Served By is required.".to_string()));
                submitting.set(false);
                return;
            }

            let req = CreateServiceRecordRequest {
                document_id: doc_id,
                party_id: pid,
                service_method: method,
                served_by: sb.trim().to_string(),
                service_date: None,
                notes: if n.trim().is_empty() { None } else { Some(n.trim().to_string()) },
                certificate_of_service: None,
            };

            match server::api::create_service_record(court, req).await {
                Ok(_) => on_created.call(()),
                Err(e) => error_msg.set(Some(format!("Failed: {}", e))),
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "mb-4 p-4 border rounded",
            if let Some(err) = &*error_msg.read() {
                div { class: "error-message", "{err}" }
            }
            form { onsubmit: handle_submit,
                div { class: "form-row",
                    div { class: "form-group",
                        FormSelect {
                            label: "Party *",
                            value: "{party_id_input}",
                            onchange: move |evt: Event<FormData>| party_id_input.set(evt.value().to_string()),
                            option { value: "", "Select a party..." }
                            match &*parties_data.read() {
                                Some(parties) if !parties.is_empty() => rsx! {
                                    for p in parties.iter() {
                                        {
                                            let pid = p.id.clone();
                                            let label = format!("{} ({})", p.name, p.party_type);
                                            rsx! { option { value: "{pid}", "{label}" } }
                                        }
                                    }
                                },
                                Some(_) => rsx! {
                                    option { disabled: true, "No parties on this case" }
                                },
                                None => rsx! {
                                    option { disabled: true, "Loading parties..." }
                                },
                            }
                        }
                    }
                    div { class: "form-group",
                        FormSelect {
                            label: "Service Method *",
                            value: "{service_method}",
                            onchange: move |evt: Event<FormData>| service_method.set(evt.value().to_string()),
                            option { value: "Electronic", "Electronic" }
                            option { value: "Mail", "Mail" }
                            option { value: "Personal Service", "Personal Service" }
                            option { value: "Certified Mail", "Certified Mail" }
                            option { value: "Express Mail", "Express Mail" }
                            option { value: "Waiver", "Waiver" }
                            option { value: "Publication", "Publication" }
                            option { value: "Other", "Other" }
                        }
                    }
                }
                div { class: "form-group",
                    Input {
                        label: "Served By *",
                        value: served_by.read().clone(),
                        on_input: move |evt: FormEvent| served_by.set(evt.value().to_string()),
                        placeholder: "e.g., US Marshal, Process Server",
                    }
                }
                div { class: "form-group",
                    Textarea {
                        label: "Notes",
                        value: notes.read().clone(),
                        on_input: move |evt: FormEvent| notes.set(evt.value().to_string()),
                        placeholder: "Optional notes...",
                    }
                }
                div { class: "form-actions",
                    button {
                        class: "button",
                        "data-style": "primary",
                        r#type: "submit",
                        disabled: *submitting.read(),
                        if *submitting.read() { "Recording..." } else { "Record Service" }
                    }
                }
            }
        }
    }
}

/// A single service record row.
#[component]
fn ServiceRecordRow(record: ServiceRecordResponse, on_completed: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let record_id = record.id.clone();
    let mut completing = use_signal(|| false);

    let is_complete = record.successful && record.proof_of_service_filed;
    let display_date = format_date(&record.service_date);
    let short_party = if record.party_id.len() > 8 {
        format!("{}...", &record.party_id[..8])
    } else {
        record.party_id.clone()
    };

    let handle_complete = move |_: MouseEvent| {
        let court = court_id.clone();
        let rid = record_id.clone();
        spawn(async move {
            completing.set(true);
            match server::api::complete_service_record(court, rid).await {
                Ok(_) => on_completed.call(()),
                Err(_) => {}
            }
            completing.set(false);
        });
    };

    rsx! {
        DataTableRow {
            DataTableCell {
                Tooltip {
                    TooltipTrigger { "{short_party}" }
                    TooltipContent { "{record.party_id}" }
                }
            }
            DataTableCell { "{record.service_method}" }
            DataTableCell { "{record.served_by}" }
            DataTableCell { "{display_date}" }
            DataTableCell {
                if is_complete {
                    Badge { variant: BadgeVariant::Primary, "Complete" }
                } else {
                    Badge { variant: BadgeVariant::Secondary, "Pending" }
                }
            }
            DataTableCell {
                if !is_complete {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: handle_complete,
                        disabled: *completing.read(),
                        if *completing.read() { "..." } else { "Mark Complete" }
                    }
                }
            }
        }
    }
}

/// A single attachment row with download link.
/// Uses a server-proxied file endpoint — works on web, desktop, and mobile.
#[component]
fn AttachmentRow(attachment: DocketAttachmentResponse, on_promoted: EventHandler<String>) -> Element {
    let ctx = use_context::<CourtContext>();
    let court = ctx.court_id.read().clone();
    let download_url = format!(
        "/api/docket/attachments/{}/file?tenant={}",
        attachment.id, court
    );
    let display_size = format_file_size(attachment.file_size);
    let display_date = attachment
        .uploaded_at
        .as_deref()
        .map(|d| format_date(d))
        .unwrap_or_else(|| "-".to_string());

    let is_uploaded = attachment.uploaded_at.is_some();
    let att_id = attachment.id.clone();
    let promoting = use_signal(|| false);
    let promoted = use_signal(|| false);
    let promote_error = use_signal(|| None::<String>);
    let role = use_user_role();
    let can_promote = matches!(role, UserRole::Clerk | UserRole::Admin);

    rsx! {
        DataTableRow {
            DataTableCell { "{attachment.filename}" }
            DataTableCell { "{display_size}" }
            DataTableCell { "{attachment.content_type}" }
            DataTableCell { "{display_date}" }
            DataTableCell {
                div { class: "flex items-center gap-2",
                    a {
                        href: "{download_url}",
                        target: "_blank",
                        class: "inline-flex items-center gap-1 px-3 py-1 rounded text-sm font-medium bg-surface-alt text-secondary border border-border hover:bg-surface-hover transition-colors",
                        "Download"
                    }
                    if is_uploaded && !*promoted.read() && can_promote {
                        Button {
                            variant: ButtonVariant::Secondary,
                            disabled: *promoting.read(),
                            onclick: {
                                let court = court.clone();
                                let att_id = att_id.clone();
                                move |_| {
                                    let court = court.clone();
                                    let att_id = att_id.clone();
                                    let on_promoted = on_promoted.clone();
                                    let mut promoting = promoting.clone();
                                    let mut promoted = promoted.clone();
                                    let mut promote_error = promote_error.clone();
                                    spawn(async move {
                                        promoting.set(true);
                                        promote_error.set(None);
                                        match server::api::promote_attachment_to_document(
                                            court,
                                            att_id,
                                            None,
                                            None,
                                        )
                                        .await
                                        {
                                            Ok(doc) => {
                                                promoted.set(true);
                                                on_promoted.call(doc.id);
                                            }
                                            Err(e) => {
                                                promote_error.set(Some(format!("{}", e)));
                                            }
                                        }
                                        promoting.set(false);
                                    });
                                }
                            },
                            if *promoting.read() { "Registering..." } else { "Register as Filed Document" }
                        }
                    }
                    if *promoted.read() {
                        Badge { variant: BadgeVariant::Primary, "Registered" }
                    }
                }
                if let Some(err) = &*promote_error.read() {
                    div { class: "text-sm text-destructive mt-1", "{err}" }
                }
            }
        }
    }
}

/// Document actions section: seal/unseal, replace file, strike.
/// Shown only for clerk/judge/admin roles when a document is linked.
#[component]
fn DocumentActionsSection(document_id: String, on_updated: EventHandler<()>) -> Element {
    let ctx = use_context::<CourtContext>();

    let mut show_seal_form = use_signal(|| false);
    let mut show_replace_form = use_signal(|| false);
    let mut show_strike_form = use_signal(|| false);
    let mut action_error = use_signal(|| None::<String>);
    let mut action_success = use_signal(|| None::<String>);
    let mut acting = use_signal(|| false);

    // Seal form state
    let mut seal_level = use_signal(|| "SealedCourtOnly".to_string());
    let mut seal_reason = use_signal(String::new);

    // Replace form state
    let mut replace_file_name = use_signal(|| None::<String>);
    let mut replace_file_ct = use_signal(|| None::<String>);
    let mut replace_file_bytes = use_signal(|| None::<Vec<u8>>);
    let mut replace_file_size = use_signal(|| 0i64);
    let mut replace_title = use_signal(String::new);

    let handle_replace_file = move |evt: FormEvent| async move {
        let files = evt.files();
        if let Some(f) = files.first() {
            let name = f.name();
            let ct = f
                .content_type()
                .unwrap_or_else(|| mime_from_filename(&name));
            match f.read_bytes().await {
                Ok(bytes) => {
                    replace_file_size.set(bytes.len() as i64);
                    replace_file_bytes.set(Some(bytes.to_vec()));
                    replace_file_ct.set(Some(ct));
                    replace_file_name.set(Some(name));
                }
                Err(_) => {
                    action_error.set(Some("Failed to read file".to_string()));
                }
            }
        }
    };

    rsx! {
        Separator {}
        div { class: "mb-4",
            div { class: "flex items-center justify-between mb-3",
                h4 { class: "font-medium text-base", "Document Actions" }
                div { class: "flex gap-2",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            show_replace_form.set(false);
                            show_strike_form.set(false);
                            action_error.set(None);
                            action_success.set(None);
                            let current = *show_seal_form.read();
                            show_seal_form.set(!current);
                        },
                        if *show_seal_form.read() { "Cancel" } else { "Seal / Unseal" }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            show_seal_form.set(false);
                            show_strike_form.set(false);
                            action_error.set(None);
                            action_success.set(None);
                            let current = *show_replace_form.read();
                            show_replace_form.set(!current);
                        },
                        if *show_replace_form.read() { "Cancel" } else { "Replace File" }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| {
                            show_seal_form.set(false);
                            show_replace_form.set(false);
                            action_error.set(None);
                            action_success.set(None);
                            let current = *show_strike_form.read();
                            show_strike_form.set(!current);
                        },
                        if *show_strike_form.read() { "Cancel" } else { "Strike" }
                    }
                }
            }

            if let Some(err) = &*action_error.read() {
                div { class: "error-message mb-2", "{err}" }
            }
            if let Some(msg) = &*action_success.read() {
                div { class: "flex items-center gap-2 mb-2 p-2 rounded text-sm",
                    style: "background: var(--color-primary-surface); color: var(--color-primary);",
                    span { "\u{2713}" }
                    span { "{msg}" }
                }
            }

            // ── Seal / Unseal sub-form ──
            if *show_seal_form.read() {
                div { class: "p-3 border rounded mb-3",
                    h5 { class: "font-medium mb-2", "Seal Document" }
                    div { class: "flex flex-col gap-2",
                        FormSelect {
                            label: "Sealing Level",
                            value: "{seal_level}",
                            onchange: move |evt: Event<FormData>| seal_level.set(evt.value().to_string()),
                            option { value: "SealedCourtOnly", "Court Only" }
                            option { value: "SealedCaseParticipants", "Case Participants" }
                            option { value: "SealedAttorneysOnly", "Attorneys Only" }
                        }
                        FormSelect {
                            label: "Reason Code",
                            value: "{seal_reason}",
                            onchange: move |evt: Event<FormData>| seal_reason.set(evt.value().to_string()),
                            option { value: "", "Select reason..." }
                            option { value: "JuvenileRecord", "Juvenile Record" }
                            option { value: "TradeSecret", "Trade Secret" }
                            option { value: "InformantIdentity", "Informant Identity" }
                            option { value: "NationalSecurity", "National Security" }
                            option { value: "GrandJury", "Grand Jury" }
                            option { value: "SealedIndictment", "Sealed Indictment" }
                            option { value: "ProtectiveOrder", "Protective Order" }
                            option { value: "Other", "Other" }
                        }
                        div { class: "flex gap-2 mt-2",
                            Button {
                                variant: ButtonVariant::Primary,
                                disabled: *acting.read(),
                                onclick: {
                                    let doc_id = document_id.clone();
                                    let court = ctx.court_id.read().clone();
                                    move |_| {
                                        let court = court.clone();
                                        let did = doc_id.clone();
                                        let level = seal_level.read().clone();
                                        let reason = seal_reason.read().clone();
                                        spawn(async move {
                                            acting.set(true);
                                            action_error.set(None);
                                            match server::api::seal_document_action(
                                                court, did, level, reason, None,
                                            ).await {
                                                Ok(_) => {
                                                    show_seal_form.set(false);
                                                    action_success.set(Some("Document sealed.".to_string()));
                                                    on_updated.call(());
                                                }
                                                Err(e) => action_error.set(Some(format!("{}", e))),
                                            }
                                            acting.set(false);
                                        });
                                    }
                                },
                                if *acting.read() { "Sealing..." } else { "Seal" }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                disabled: *acting.read(),
                                onclick: {
                                    let doc_id = document_id.clone();
                                    let court = ctx.court_id.read().clone();
                                    move |_| {
                                        let court = court.clone();
                                        let did = doc_id.clone();
                                        spawn(async move {
                                            acting.set(true);
                                            action_error.set(None);
                                            match server::api::unseal_document_action(court, did).await {
                                                Ok(_) => {
                                                    show_seal_form.set(false);
                                                    action_success.set(Some("Document unsealed.".to_string()));
                                                    on_updated.call(());
                                                }
                                                Err(e) => action_error.set(Some(format!("{}", e))),
                                            }
                                            acting.set(false);
                                        });
                                    }
                                },
                                "Unseal"
                            }
                        }
                    }
                }
            }

            // ── Replace File sub-form ──
            if *show_replace_form.read() {
                div { class: "p-3 border rounded mb-3",
                    h5 { class: "font-medium mb-2", "Replace Document File" }
                    div { class: "flex flex-col gap-2",
                        div { class: "form-group",
                            label { class: "form-label", "Replacement File" }
                            form {
                                onchange: handle_replace_file,
                                input {
                                    r#type: "file",
                                    disabled: *acting.read(),
                                }
                            }
                            if let Some(ref name) = &*replace_file_name.read() {
                                div { class: "flex items-center gap-2 mt-1",
                                    span { class: "text-sm text-muted", "{name} ({format_file_size(*replace_file_size.read())})" }
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        onclick: move |_| {
                                            replace_file_name.set(None);
                                            replace_file_ct.set(None);
                                            replace_file_bytes.set(None);
                                            replace_file_size.set(0);
                                        },
                                        "Remove"
                                    }
                                }
                            }
                        }
                        Input {
                            label: "Title Override (optional)",
                            value: replace_title.read().clone(),
                            on_input: move |evt: FormEvent| replace_title.set(evt.value().to_string()),
                            placeholder: "Leave blank to keep original title",
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            disabled: *acting.read() || replace_file_bytes.read().is_none(),
                            onclick: {
                                let doc_id = document_id.clone();
                                let court = ctx.court_id.read().clone();
                                move |_| {
                                    let court = court.clone();
                                    let did = doc_id.clone();
                                    let fname = replace_file_name.read().clone();
                                    let fct = replace_file_ct.read().clone();
                                    let fbytes = replace_file_bytes.read().clone();
                                    let fsize = *replace_file_size.read();
                                    let t = replace_title.read().clone();
                                    spawn(async move {
                                        acting.set(true);
                                        action_error.set(None);
                                        if let (Some(name), Some(ct), Some(bytes)) = (fname, fct, fbytes) {
                                            let title_opt = if t.trim().is_empty() { None } else { Some(t) };
                                            match server::api::replace_document_file(
                                                court, did, name, ct, fsize, bytes, title_opt,
                                            ).await {
                                                Ok(_) => {
                                                    show_replace_form.set(false);
                                                    replace_file_name.set(None);
                                                    replace_file_ct.set(None);
                                                    replace_file_bytes.set(None);
                                                    replace_file_size.set(0);
                                                    replace_title.set(String::new());
                                                    action_success.set(Some("File replaced.".to_string()));
                                                    on_updated.call(());
                                                }
                                                Err(e) => action_error.set(Some(format!("{}", e))),
                                            }
                                        }
                                        acting.set(false);
                                    });
                                }
                            },
                            if *acting.read() { "Replacing..." } else { "Replace" }
                        }
                    }
                }
            }

            // ── Strike sub-form ──
            if *show_strike_form.read() {
                div { class: "p-3 border border-destructive rounded mb-3",
                    h5 { class: "font-medium mb-2 text-destructive", "Strike Document" }
                    p { class: "text-sm text-muted mb-3",
                        "This will permanently mark this document as stricken from the record. This action cannot be undone."
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        disabled: *acting.read(),
                        onclick: {
                            let doc_id = document_id.clone();
                            let court = ctx.court_id.read().clone();
                            move |_| {
                                let court = court.clone();
                                let did = doc_id.clone();
                                spawn(async move {
                                    acting.set(true);
                                    action_error.set(None);
                                    match server::api::strike_document_action(court, did).await {
                                        Ok(_) => {
                                            show_strike_form.set(false);
                                            action_success.set(Some("Document stricken from record.".to_string()));
                                            on_updated.call(());
                                        }
                                        Err(e) => action_error.set(Some(format!("{}", e))),
                                    }
                                    acting.set(false);
                                });
                            }
                        },
                        if *acting.read() { "Striking..." } else { "Confirm Strike" }
                    }
                }
            }
        }
    }
}

pub fn format_file_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn mime_from_filename(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.ends_with(".pdf") {
        "application/pdf".to_string()
    } else if lower.ends_with(".doc") || lower.ends_with(".docx") {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg".to_string()
    } else if lower.ends_with(".png") {
        "image/png".to_string()
    } else if lower.ends_with(".txt") {
        "text/plain".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

fn format_date(date_str: &str) -> String {
    crate::format_helpers::format_date_human(date_str)
}
