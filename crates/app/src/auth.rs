use dioxus::prelude::*;
use shared_types::{AuthUser, UserRole};

/// Global authentication state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AuthState {
    pub current_user: Signal<Option<AuthUser>>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            current_user: Signal::new(None),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.current_user.read().is_some()
    }

    pub fn set_user(&mut self, user: AuthUser) {
        self.current_user.set(Some(user));
    }

    pub fn clear_auth(&mut self) {
        self.current_user.set(None);
    }
}

/// Hook to access auth state.
pub fn use_auth() -> AuthState {
    use_context::<AuthState>()
}

/// Hook to check if the current user has the admin role.
pub fn use_is_admin() -> bool {
    let auth = use_auth();
    let binding = auth.current_user.read();
    let is_admin = binding.as_ref().map(|u| u.role == "admin").unwrap_or(false);
    is_admin
}

/// Check if the current user can manage court memberships for the active court.
/// True if the user is a platform admin OR a clerk in the active court.
pub fn use_can_manage_memberships() -> bool {
    let auth = use_auth();
    let ctx = use_context::<crate::CourtContext>();
    let court = ctx.court_id.read().clone();
    let user = auth.current_user.read().clone();
    user.map(|u| {
        if u.role == "admin" {
            return true;
        }
        u.court_roles
            .get(&court)
            .map(|r| r == "clerk")
            .unwrap_or(false)
    })
    .unwrap_or(false)
}

/// Get the user's effective role for the currently selected court.
/// Computed reactively from auth state and court context — no intermediate signal.
pub fn use_user_role() -> UserRole {
    let auth = use_auth();
    let ctx = use_context::<crate::CourtContext>();
    let court = ctx.court_id.read().clone();
    let user = auth.current_user.read().clone();
    user.and_then(|u| {
        if u.role == "admin" {
            return Some(UserRole::Admin);
        }
        u.court_roles
            .get(&court)
            .map(|r| UserRole::from_str_or_default(r))
    })
    .unwrap_or(UserRole::Public)
}

/// Determine which sidebar groups are visible for the current user's role.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarVisibility {
    pub work: bool,    // Queue
    pub court: bool,   // Cases, Schedule, Deadlines
    pub admin: bool,   // Compliance, Rules, Users, Settings
}

pub fn use_sidebar_visibility() -> SidebarVisibility {
    let role = use_user_role();
    match role {
        UserRole::Admin => SidebarVisibility {
            work: true,
            court: true,
            admin: true,
        },
        UserRole::Clerk => SidebarVisibility {
            work: true,
            court: true,
            admin: true,
        },
        UserRole::Judge => SidebarVisibility {
            work: true,
            court: true,
            admin: false,
        },
        UserRole::Attorney => SidebarVisibility {
            work: true,
            court: true,
            admin: false,
        },
        UserRole::Public => SidebarVisibility {
            work: false,
            court: true,
            admin: false,
        },
    }
}

/// Actions that can be role-gated in the UI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    // Court record management (clerk/admin)
    ManageAttorneys,
    ManageJudges,
    ManageRules,

    // Case workflow
    CreateCase,
    EditCase,
    DeleteCase,

    // Docket & Filing
    CreateDocketEntry,
    FileFiling,

    // Judicial actions
    SignOrder,
    IssueOrder,
    DraftOpinion,

    // Document control
    SealDocument,
    StrikeDocument,

    // Evidence & Sentencing
    ManageEvidence,
    EnterSentencing,

    // Universal
    GeneratePdf,
}

/// Check if a role is permitted to perform an action.
pub fn can(role: &UserRole, action: Action) -> bool {
    match action {
        // Clerk/Admin only
        Action::ManageAttorneys | Action::ManageJudges | Action::ManageRules => {
            matches!(role, UserRole::Clerk | UserRole::Admin)
        }
        Action::CreateCase | Action::CreateDocketEntry => {
            matches!(role, UserRole::Clerk | UserRole::Admin)
        }
        // Admin only
        Action::DeleteCase => matches!(role, UserRole::Admin),
        // Clerk/Admin/Judge
        Action::EditCase | Action::EnterSentencing => {
            matches!(role, UserRole::Clerk | UserRole::Judge | UserRole::Admin)
        }
        // Attorney/Clerk/Admin
        Action::FileFiling => {
            matches!(role, UserRole::Attorney | UserRole::Clerk | UserRole::Admin)
        }
        // Judge/Admin
        Action::SignOrder | Action::DraftOpinion => {
            matches!(role, UserRole::Judge | UserRole::Admin)
        }
        // Clerk/Admin
        Action::IssueOrder => matches!(role, UserRole::Clerk | UserRole::Admin),
        // Judge/Clerk/Admin
        Action::SealDocument | Action::StrikeDocument => {
            matches!(role, UserRole::Judge | UserRole::Clerk | UserRole::Admin)
        }
        Action::ManageEvidence => {
            matches!(role, UserRole::Clerk | UserRole::Admin)
        }
        // All except public
        Action::GeneratePdf => !matches!(role, UserRole::Public),
    }
}
