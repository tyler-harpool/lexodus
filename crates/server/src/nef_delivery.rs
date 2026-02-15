//! NEF (Notice of Electronic Filing) delivery via email (Mailgun) and SMS (Twilio).
//!
//! Called fire-and-forget after a filing is submitted — errors are logged but
//! never fail the filing itself.

use shared_types::Nef;
use sqlx::{Pool, Postgres};
use std::collections::HashSet;

use crate::repo::party::PartyServiceInfo;

/// Deliver a NEF via email and SMS to all electronic-service recipients.
///
/// Sends to both party contacts and their attorneys (via `representations`).
/// Respects feature flags — skips email if `!mailgun`, skips SMS if `!twilio`.
pub async fn deliver_nef(
    pool: &Pool<Postgres>,
    court_id: &str,
    nef: &Nef,
    document_title: &str,
    case_number: &str,
    parties: &[PartyServiceInfo],
) {
    let flags = crate::config::feature_flags();
    let email_enabled = flags.mailgun;
    let sms_enabled = flags.twilio;

    if !email_enabled && !sms_enabled {
        tracing::debug!("NEF delivery skipped — both mailgun and twilio disabled");
        return;
    }

    let subject = format!("[Lexodus] NEF — {}: {}", case_number, document_title);
    let html_body = nef_email_html(
        nef.html_snapshot.as_deref().unwrap_or(""),
        case_number,
    );
    let sms_message = format!(
        "[Lexodus] Filing in {}: {}. Check your email for the full Notice of Electronic Filing.",
        case_number, document_title,
    );

    // Track emails/phones already contacted to avoid duplicates
    let mut emailed: HashSet<String> = HashSet::new();
    let mut texted: HashSet<String> = HashSet::new();

    // --- Deliver to parties with electronic service ---
    for party in parties {
        let method = party.service_method.as_deref().unwrap_or("Electronic");
        if method != "Electronic" {
            continue;
        }

        if email_enabled {
            if let Some(ref email) = party.email {
                if emailed.insert(email.to_lowercase()) {
                    deliver_email(email, &subject, &html_body, &party.name).await;
                }
            }
        }

        if sms_enabled && party.nef_sms_opt_in {
            if let Some(ref phone) = party.phone {
                if texted.insert(phone.clone()) {
                    deliver_sms(phone, &sms_message, &party.name).await;
                }
            }
        }
    }

    // --- Deliver to attorneys representing parties on the case ---
    match crate::repo::party::list_attorney_contacts_by_case(pool, court_id, nef.case_id).await {
        Ok(attorneys) => {
            for atty in &attorneys {
                if email_enabled {
                    if let Some(ref email) = atty.email {
                        if emailed.insert(email.to_lowercase()) {
                            deliver_email(email, &subject, &html_body, &atty.attorney_name).await;
                        }
                    }
                }

                if sms_enabled && atty.nef_sms_opt_in {
                    if let Some(ref phone) = atty.phone {
                        if texted.insert(phone.clone()) {
                            deliver_sms(phone, &sms_message, &atty.attorney_name).await;
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!(error = %e, case_id = %nef.case_id, "Failed to look up attorney contacts for NEF delivery");
        }
    }

    tracing::info!(
        nef_id = %nef.id,
        case_number = case_number,
        emails_sent = emailed.len(),
        sms_sent = texted.len(),
        "NEF delivery complete"
    );
}

/// Send a single NEF email, logging any errors.
async fn deliver_email(to: &str, subject: &str, html_body: &str, recipient_name: &str) {
    match crate::mailgun::send_email(to, subject, html_body).await {
        Ok(()) => {
            tracing::info!(to = to, name = recipient_name, "NEF email sent");
        }
        Err(e) => {
            tracing::error!(error = %e, to = to, name = recipient_name, "NEF email failed");
        }
    }
}

/// Send a single NEF SMS, logging any errors.
async fn deliver_sms(to: &str, message: &str, recipient_name: &str) {
    match crate::twilio::send_sms(to, message).await {
        Ok(()) => {
            tracing::info!(to = to, name = recipient_name, "NEF SMS sent");
        }
        Err(e) => {
            tracing::error!(error = %e, to = to, name = recipient_name, "NEF SMS failed");
        }
    }
}

/// Wrap the NEF HTML snapshot in an email template matching Lexodus styling.
fn nef_email_html(html_snapshot: &str, case_number: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: 'Courier New', monospace; background: #0a0a0f; color: #e0e0e0; padding: 20px;">
  <div style="max-width: 600px; margin: 0 auto; border: 1px solid #00f0ff; padding: 30px;">
    <h1 style="color: #00f0ff; text-align: center;">Notice of Electronic Filing</h1>
    <p style="color: #888; text-align: center;">Case {case_number}</p>
    <hr style="border-color: #333;" />
    {html_snapshot}
    <hr style="border-color: #333;" />
    <p style="color: #888; font-size: 12px;">This is an automated notice from the Lexodus CM/ECF system.
    You are receiving this because you are a registered participant in this case.</p>
  </div>
</body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nef_email_html_contains_case_and_snapshot() {
        let html = nef_email_html("<p>Test filing</p>", "1:25-cr-00042");
        assert!(html.contains("1:25-cr-00042"));
        assert!(html.contains("<p>Test filing</p>"));
        assert!(html.contains("Notice of Electronic Filing"));
        assert!(html.contains("Lexodus CM/ECF"));
    }

    #[test]
    fn nef_email_html_handles_empty_snapshot() {
        let html = nef_email_html("", "1:25-cr-00001");
        assert!(html.contains("1:25-cr-00001"));
        assert!(html.contains("Notice of Electronic Filing"));
    }
}
