use dioxus::prelude::*;
use shared_types::{
    HeadnoteResponse, JudicialOpinionResponse, OpinionCitationResponse, OpinionDraftResponse,
    OpinionVoteResponse,
};
use shared_ui::components::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle, Badge, BadgeVariant, Button,
    ButtonVariant, Card, CardContent, CardHeader, CardTitle, DetailGrid, DetailItem, DetailList,
    PageActions, PageHeader, PageTitle, Skeleton, TabContent, TabList, TabTrigger, Tabs,
};
use shared_ui::{use_toast, ToastOptions};

use super::form_sheet::{FormMode, OpinionFormSheet};
use crate::auth::{can, use_user_role, Action};
use crate::routes::Route;
use crate::CourtContext;

#[component]
pub fn OpinionDetailPage(id: String) -> Element {
    let ctx = use_context::<CourtContext>();
    let court_id = ctx.court_id.read().clone();
    let opinion_id = id.clone();
    let toast = use_toast();

    let role = use_user_role();
    let mut show_edit = use_signal(|| false);
    let mut show_delete_confirm = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let mut data = use_resource(move || {
        let court = court_id.clone();
        let oid = opinion_id.clone();
        async move { server::api::get_opinion(court, oid).await.ok() }
    });

    let detail_id = id.clone();
    let handle_delete = move |_: MouseEvent| {
        let court = ctx.court_id.read().clone();
        let oid = detail_id.clone();
        spawn(async move {
            deleting.set(true);
            match server::api::delete_opinion(court, oid).await {
                Ok(()) => {
                    toast.success(
                        "Opinion deleted successfully".to_string(),
                        ToastOptions::new(),
                    );
                    let nav = navigator();
                    nav.push(Route::OpinionList {});
                }
                Err(e) => {
                    toast.error(format!("{}", e), ToastOptions::new());
                    deleting.set(false);
                    show_delete_confirm.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "container",
            match &*data.read() {
                Some(Some(opinion)) => rsx! {
                    PageHeader {
                        PageTitle { "{opinion.title}" }
                        PageActions {
                            Link { to: Route::OpinionList {},
                                Button { variant: ButtonVariant::Secondary, "Back to List" }
                            }
                            if can(&role, Action::DraftOpinion) {
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| show_edit.set(true),
                                    "Edit"
                                }
                            }
                            if can(&role, Action::DraftOpinion) {
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    onclick: move |_| show_delete_confirm.set(true),
                                    "Delete"
                                }
                            }
                        }
                    }

                    AlertDialogRoot {
                        open: show_delete_confirm(),
                        on_open_change: move |v| show_delete_confirm.set(v),
                        AlertDialogContent {
                            AlertDialogTitle { "Delete Opinion" }
                            AlertDialogDescription {
                                "Are you sure you want to delete this opinion? This action cannot be undone."
                            }
                            AlertDialogActions {
                                AlertDialogCancel { "Cancel" }
                                AlertDialogAction {
                                    on_click: handle_delete,
                                    if *deleting.read() { "Deleting..." } else { "Delete" }
                                }
                            }
                        }
                    }

                    Tabs { default_value: "content", horizontal: true,
                        TabList {
                            TabTrigger { value: "content", index: 0usize, "Content" }
                            TabTrigger { value: "votes", index: 1usize, "Votes" }
                            TabTrigger { value: "citations", index: 2usize, "Citations" }
                            TabTrigger { value: "drafts", index: 3usize, "Drafts" }
                        }
                        TabContent { value: "content", index: 0usize,
                            ContentTab { opinion: opinion.clone(), opinion_id: id.clone() }
                        }
                        TabContent { value: "votes", index: 1usize,
                            VotesTab { opinion_id: id.clone() }
                        }
                        TabContent { value: "citations", index: 2usize,
                            CitationsTab { opinion_id: id.clone() }
                        }
                        TabContent { value: "drafts", index: 3usize,
                            DraftsTab { opinion_id: id.clone() }
                        }
                    }

                    OpinionFormSheet {
                        mode: FormMode::Edit,
                        initial: Some(opinion.clone()),
                        open: show_edit(),
                        on_close: move |_| show_edit.set(false),
                        on_saved: move |_| data.restart(),
                    }
                },
                Some(None) => rsx! {
                    Card {
                        CardContent {
                            div { class: "empty-state",
                                h2 { "Opinion Not Found" }
                                p { "The opinion you're looking for doesn't exist in this court district." }
                                Link { to: Route::OpinionList {},
                                    Button { "Back to List" }
                                }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "loading",
                        Skeleton {}
                        Skeleton {}
                        Skeleton {}
                    }
                },
            }
        }
    }
}

/// Content tab showing the opinion's main information, syllabus, and headnotes.
#[component]
fn ContentTab(opinion: JudicialOpinionResponse, opinion_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let filed_display = opinion
        .filed_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    let published_display = opinion
        .published_at
        .as_deref()
        .map(|d| d.chars().take(10).collect::<String>())
        .unwrap_or_else(|| "--".to_string());

    let citation_display = match (&opinion.citation_volume, &opinion.citation_reporter, &opinion.citation_page) {
        (Some(vol), Some(rep), Some(pg)) => format!("{} {} {}", vol, rep, pg),
        _ => "--".to_string(),
    };

    let keywords_display = if opinion.keywords.is_empty() {
        "--".to_string()
    } else {
        opinion.keywords.join(", ")
    };

    // Load headnotes for this opinion
    let headnotes = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let oid = opinion_id.clone();
        async move { server::api::list_headnotes(court, oid).await.ok() }
    });

    rsx! {
        DetailGrid {
            Card {
                CardHeader { CardTitle { "Opinion Details" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Title", value: opinion.title.clone() }
                        DetailItem { label: "Type",
                            Badge {
                                variant: type_badge_variant(&opinion.opinion_type),
                                "{opinion.opinion_type}"
                            }
                        }
                        DetailItem { label: "Status",
                            Badge {
                                variant: status_badge_variant(&opinion.status),
                                "{opinion.status}"
                            }
                        }
                        DetailItem { label: "Disposition",
                            if opinion.disposition.is_empty() {
                                span { "--" }
                            } else {
                                Badge {
                                    variant: disposition_badge_variant(&opinion.disposition),
                                    "{opinion.disposition}"
                                }
                            }
                        }
                        DetailItem { label: "Case", value: opinion.case_name.clone() }
                        DetailItem { label: "Docket Number", value: opinion.docket_number.clone() }
                        DetailItem { label: "Author", value: opinion.author_judge_name.clone() }
                    }
                }
            }

            Card {
                CardHeader { CardTitle { "Publication" } }
                CardContent {
                    DetailList {
                        DetailItem { label: "Published",
                            if opinion.is_published {
                                Badge { variant: BadgeVariant::Primary, "Yes" }
                            } else {
                                Badge { variant: BadgeVariant::Outline, "No" }
                            }
                        }
                        DetailItem { label: "Precedential",
                            if opinion.is_precedential {
                                Badge { variant: BadgeVariant::Primary, "Yes" }
                            } else {
                                Badge { variant: BadgeVariant::Outline, "No" }
                            }
                        }
                        DetailItem { label: "Citation", value: citation_display }
                        DetailItem { label: "Filed Date", value: filed_display }
                        DetailItem { label: "Published Date", value: published_display }
                        DetailItem { label: "Keywords", value: keywords_display }
                        DetailItem {
                            label: "Created",
                            value: opinion.created_at.chars().take(10).collect::<String>()
                        }
                        DetailItem {
                            label: "Updated",
                            value: opinion.updated_at.chars().take(10).collect::<String>()
                        }
                    }
                }
            }
        }

        if !opinion.syllabus.is_empty() {
            Card {
                CardHeader { CardTitle { "Syllabus" } }
                CardContent {
                    p { class: "opinion-text", "{opinion.syllabus}" }
                }
            }
        }

        Card {
            CardHeader { CardTitle { "Opinion Body" } }
            CardContent {
                if opinion.content.is_empty() {
                    p { class: "text-muted", "No content yet." }
                } else {
                    p { class: "opinion-text", "{opinion.content}" }
                }
            }
        }

        // Headnotes section
        Card {
            CardHeader { CardTitle { "Headnotes" } }
            CardContent {
                match &*headnotes.read() {
                    Some(Some(list)) if !list.is_empty() => rsx! {
                        for hn in list.iter() {
                            HeadnoteCard { headnote: hn.clone() }
                        }
                    },
                    Some(_) => rsx! {
                        p { class: "text-muted", "No headnotes for this opinion." }
                    },
                    None => rsx! { Skeleton {} },
                }
            }
        }
    }
}

/// Headnote card component.
#[component]
fn HeadnoteCard(headnote: HeadnoteResponse) -> Element {
    let key_number_display = headnote
        .key_number
        .clone()
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        div { class: "headnote-item",
            div { class: "headnote-header",
                strong { "#{headnote.headnote_number}: {headnote.topic}" }
                if headnote.key_number.is_some() {
                    Badge { variant: BadgeVariant::Outline, "Key: {key_number_display}" }
                }
            }
            p { class: "headnote-text", "{headnote.text}" }
        }
    }
}

/// Votes tab listing all opinion votes.
#[component]
fn VotesTab(opinion_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let votes = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let oid = opinion_id.clone();
        async move { server::api::list_opinion_votes(court, oid).await.ok() }
    });

    rsx! {
        match &*votes.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "votes-list",
                    for vote in list.iter() {
                        VoteCard { vote: vote.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No votes recorded for this opinion." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual vote card.
#[component]
fn VoteCard(vote: OpinionVoteResponse) -> Element {
    let joined_display = vote.joined_at.chars().take(10).collect::<String>();
    let notes_display = vote.notes.clone().unwrap_or_else(|| "--".to_string());
    let judge_id_short = if vote.judge_id.len() > 8 {
        format!("{}...", &vote.judge_id[..8])
    } else {
        vote.judge_id.clone()
    };

    rsx! {
        Card {
            CardContent {
                DetailList {
                    DetailItem { label: "Judge", value: judge_id_short }
                    DetailItem { label: "Vote Type",
                        Badge { variant: vote_badge_variant(&vote.vote_type), "{vote.vote_type}" }
                    }
                    DetailItem { label: "Joined Date", value: joined_display }
                    DetailItem { label: "Notes", value: notes_display }
                }
            }
        }
    }
}

/// Citations tab listing all citations in this opinion.
#[component]
fn CitationsTab(opinion_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let citations = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let oid = opinion_id.clone();
        async move { server::api::list_opinion_citations(court, oid).await.ok() }
    });

    rsx! {
        match &*citations.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "citations-list",
                    for citation in list.iter() {
                        CitationCard { citation: citation.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No citations found for this opinion." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual citation card.
#[component]
fn CitationCard(citation: OpinionCitationResponse) -> Element {
    let context_display = citation
        .context
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let pinpoint_display = citation
        .pinpoint_cite
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let cited_id_display = citation
        .cited_opinion_id
        .clone()
        .map(|id| {
            if id.len() > 8 {
                format!("{}...", &id[..8])
            } else {
                id
            }
        })
        .unwrap_or_else(|| "External".to_string());

    rsx! {
        Card {
            CardContent {
                DetailList {
                    DetailItem { label: "Citation", value: citation.citation_text.clone() }
                    DetailItem { label: "Type",
                        Badge {
                            variant: citation_badge_variant(&citation.citation_type),
                            "{citation.citation_type}"
                        }
                    }
                    DetailItem { label: "Cited Opinion", value: cited_id_display }
                    DetailItem { label: "Pinpoint", value: pinpoint_display }
                    DetailItem { label: "Context", value: context_display }
                }
            }
        }
    }
}

/// Drafts tab listing all opinion drafts.
#[component]
fn DraftsTab(opinion_id: String) -> Element {
    let ctx = use_context::<CourtContext>();

    let drafts = use_resource(move || {
        let court = ctx.court_id.read().clone();
        let oid = opinion_id.clone();
        async move { server::api::list_opinion_drafts(court, oid).await.ok() }
    });

    rsx! {
        match &*drafts.read() {
            Some(Some(list)) if !list.is_empty() => rsx! {
                div { class: "drafts-list",
                    for draft in list.iter() {
                        DraftCard { draft: draft.clone() }
                    }
                }
            },
            Some(_) => rsx! {
                Card {
                    CardContent {
                        p { class: "text-muted", "No drafts created for this opinion." }
                    }
                }
            },
            None => rsx! { Skeleton {} },
        }
    }
}

/// Individual draft card.
#[component]
fn DraftCard(draft: OpinionDraftResponse) -> Element {
    let created_display = draft.created_at.chars().take(10).collect::<String>();
    let author_display = draft
        .author_id
        .clone()
        .unwrap_or_else(|| "--".to_string());
    let content_preview = if draft.content.len() > 200 {
        format!("{}...", &draft.content[..200])
    } else {
        draft.content.clone()
    };

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Version {draft.version}" }
            }
            CardContent {
                DetailList {
                    DetailItem { label: "Status",
                        Badge {
                            variant: draft_status_badge_variant(&draft.status),
                            "{draft.status}"
                        }
                    }
                    DetailItem { label: "Author", value: author_display }
                    DetailItem { label: "Created", value: created_display }
                }
                if !draft.content.is_empty() {
                    p { class: "draft-preview", "{content_preview}" }
                }
            }
        }
    }
}

/// Map opinion status to an appropriate badge variant.
fn status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Draft" => BadgeVariant::Outline,
        "Under Review" | "Circulated" => BadgeVariant::Secondary,
        "Filed" => BadgeVariant::Primary,
        "Published" => BadgeVariant::Primary,
        "Withdrawn" => BadgeVariant::Destructive,
        "Superseded" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Map opinion type to an appropriate badge variant.
fn type_badge_variant(opinion_type: &str) -> BadgeVariant {
    match opinion_type {
        "Majority" => BadgeVariant::Primary,
        "Concurrence" => BadgeVariant::Secondary,
        "Dissent" => BadgeVariant::Destructive,
        "Per Curiam" => BadgeVariant::Primary,
        "Memorandum" => BadgeVariant::Outline,
        "En Banc" => BadgeVariant::Primary,
        "Summary" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Map disposition to an appropriate badge variant.
fn disposition_badge_variant(disposition: &str) -> BadgeVariant {
    match disposition {
        "Affirmed" => BadgeVariant::Primary,
        "Reversed" | "Vacated" => BadgeVariant::Destructive,
        "Remanded" => BadgeVariant::Secondary,
        "Dismissed" => BadgeVariant::Destructive,
        "Modified" | "Certified" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// Map vote type to an appropriate badge variant.
fn vote_badge_variant(vote_type: &str) -> BadgeVariant {
    match vote_type {
        "Join" | "Concur" => BadgeVariant::Primary,
        "Concur in Part" => BadgeVariant::Secondary,
        "Dissent" => BadgeVariant::Destructive,
        "Dissent in Part" => BadgeVariant::Destructive,
        "Recused" | "Not Participating" => BadgeVariant::Outline,
        _ => BadgeVariant::Outline,
    }
}

/// Map citation type to an appropriate badge variant.
fn citation_badge_variant(citation_type: &str) -> BadgeVariant {
    match citation_type {
        "Followed" => BadgeVariant::Primary,
        "Distinguished" => BadgeVariant::Secondary,
        "Overruled" => BadgeVariant::Destructive,
        "Cited" | "Discussed" => BadgeVariant::Secondary,
        "Criticized" | "Questioned" => BadgeVariant::Destructive,
        "Harmonized" | "Parallel" => BadgeVariant::Primary,
        _ => BadgeVariant::Outline,
    }
}

/// Map draft status to an appropriate badge variant.
fn draft_status_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "Draft" => BadgeVariant::Outline,
        "Under Review" => BadgeVariant::Secondary,
        "Approved" => BadgeVariant::Primary,
        "Rejected" => BadgeVariant::Destructive,
        "Superseded" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}
