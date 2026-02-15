use dioxus::prelude::*;

use crate::routes::Route;

/// Privacy Policy page — publicly accessible, no auth required.
#[component]
pub fn Privacy() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./privacy.css") }

        div { class: "legal-page",
            div { class: "legal-container",
                div { class: "legal-header",
                    h1 { class: "legal-title", "Privacy Policy" }
                    p { class: "legal-updated", "Last updated: February 9, 2026" }
                }

                div { class: "legal-section",
                    h2 { "1. Information We Collect" }
                    p { "When you create an account or use Lexodus, we may collect the following personal information:" }
                    ul {
                        li { "Name and display name" }
                        li { "Email address" }
                        li { "Phone number" }
                        li { "Account credentials (passwords are stored securely hashed)" }
                    }
                }

                div { class: "legal-section",
                    h2 { "2. How We Use Your Information" }
                    p { "We use the information we collect to:" }
                    ul {
                        li { "Provide, maintain, and improve our services" }
                        li { "Authenticate your identity and secure your account" }
                        li { "Send verification codes via SMS to confirm your phone number" }
                        li { "Send security alerts related to your account (e.g., password changes, suspicious login attempts)" }
                        li { "Send billing and transactional notifications" }
                        li { "Communicate with you about service updates" }
                    }
                }

                div { class: "legal-section",
                    h2 { "3. SMS Messaging" }
                    p { "By providing your phone number you consent to receive SMS messages from Lexodus. These messages may include:" }
                    ul {
                        li { "One-time verification codes" }
                        li { "Security alerts" }
                        li { "Billing notifications" }
                    }
                    p { "Message frequency varies based on account activity. Message and data rates may apply. You can opt out at any time by replying STOP to any message. Reply HELP for assistance." }
                }

                div { class: "legal-section",
                    h2 { "4. Data Sharing" }
                    p { "We do not sell, rent, or share your personal information with third parties for their marketing purposes. We may share data with:" }
                    ul {
                        li { "Service providers who assist in operating our platform (e.g., SMS delivery, payment processing)" }
                        li { "Law enforcement or regulatory bodies when required by law" }
                    }
                }

                div { class: "legal-section",
                    h2 { "5. Data Retention" }
                    p { "We retain your personal information for as long as your account is active or as needed to provide our services. You may request deletion of your account and associated data at any time by contacting us." }
                }

                div { class: "legal-section",
                    h2 { "6. Your Rights" }
                    p { "You have the right to:" }
                    ul {
                        li { "Access the personal data we hold about you" }
                        li { "Request correction of inaccurate information" }
                        li { "Request deletion of your account and data" }
                        li { "Opt out of SMS communications by replying STOP" }
                    }
                }

                div { class: "legal-section",
                    h2 { "7. Security" }
                    p { "We implement industry-standard security measures to protect your personal information, including encrypted storage of credentials and secure transmission of data." }
                }

                div { class: "legal-section",
                    h2 { "8. Contact Us" }
                    p { "If you have questions about this Privacy Policy, please contact us at support@lexodus.app." }
                }

                Link { to: Route::Login { redirect: None },
                    class: "legal-back-link",
                    "Back to Login"
                }
            }
        }
    }
}
