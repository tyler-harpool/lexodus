use dioxus::prelude::*;

use crate::routes::Route;

/// Terms of Service page — publicly accessible, no auth required.
#[component]
pub fn Terms() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./terms.css") }

        div { class: "legal-page",
            div { class: "legal-container",
                div { class: "legal-header",
                    h1 { class: "legal-title", "Terms of Service" }
                    p { class: "legal-updated", "Last updated: February 9, 2026" }
                }

                div { class: "legal-section",
                    h2 { "1. Acceptance of Terms" }
                    p { "By accessing or using Lexodus, you agree to be bound by these Terms of Service. If you do not agree to these terms, do not use the service." }
                }

                div { class: "legal-section",
                    h2 { "2. Service Description" }
                    p { "Lexodus provides a web-based platform for managing users, products, and account settings. Features include user authentication, billing management, and administrative tools." }
                }

                div { class: "legal-section",
                    h2 { "3. Account Responsibilities" }
                    p { "You are responsible for:" }
                    ul {
                        li { "Maintaining the confidentiality of your account credentials" }
                        li { "All activity that occurs under your account" }
                        li { "Providing accurate and up-to-date information" }
                        li { "Notifying us immediately of any unauthorized use of your account" }
                    }
                }

                div { class: "legal-section",
                    h2 { "4. SMS Communications" }
                    p { "By providing your phone number and using Lexodus, you consent to receive SMS messages related to your account. Details:" }
                    ul {
                        li { "Program name: Lexodus" }
                        li { "Message types: verification codes, security alerts, billing notifications" }
                        li { "Message frequency varies based on your account activity" }
                        li { "Message and data rates may apply" }
                        li { "Reply STOP to opt out of SMS messages at any time" }
                        li { "Reply HELP for customer support information" }
                        li { "Carriers are not liable for delayed or undelivered messages" }
                    }
                    p { "Opting out of SMS may limit certain account features such as phone verification and two-factor authentication." }
                }

                div { class: "legal-section",
                    h2 { "5. Prohibited Conduct" }
                    p { "You agree not to:" }
                    ul {
                        li { "Use the service for any unlawful purpose" }
                        li { "Attempt to gain unauthorized access to any part of the service" }
                        li { "Interfere with or disrupt the service or its infrastructure" }
                        li { "Impersonate any person or entity" }
                    }
                }

                div { class: "legal-section",
                    h2 { "6. Intellectual Property" }
                    p { "All content, features, and functionality of Lexodus are owned by us and are protected by copyright, trademark, and other intellectual property laws." }
                }

                div { class: "legal-section",
                    h2 { "7. Limitation of Liability" }
                    p { "To the maximum extent permitted by law, Lexodus and its operators shall not be liable for any indirect, incidental, special, consequential, or punitive damages arising from your use of the service. The service is provided \"as is\" without warranties of any kind." }
                }

                div { class: "legal-section",
                    h2 { "8. Termination" }
                    p { "We reserve the right to suspend or terminate your account at our discretion, with or without notice, for conduct that we determine violates these terms or is harmful to other users or the service." }
                }

                div { class: "legal-section",
                    h2 { "9. Changes to Terms" }
                    p { "We may update these terms from time to time. Continued use of the service after changes constitutes acceptance of the updated terms. We will notify users of material changes via email or in-app notification." }
                }

                div { class: "legal-section",
                    h2 { "10. Contact Us" }
                    p { "If you have questions about these Terms of Service, please contact us at support@lexodus.app." }
                }

                Link { to: Route::Login { redirect: None },
                    class: "legal-back-link",
                    "Back to Login"
                }
            }
        }
    }
}
