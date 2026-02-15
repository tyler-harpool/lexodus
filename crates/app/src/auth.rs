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
    pub core: bool,
    pub case_management: bool,
    pub court_operations: bool,
    pub legal_documents: bool,
    pub people_orgs: bool,
    pub administration: bool,
}

pub fn use_sidebar_visibility() -> SidebarVisibility {
    let role = use_user_role();
    match role {
        UserRole::Admin => SidebarVisibility {
            core: true,
            case_management: true,
            court_operations: true,
            legal_documents: true,
            people_orgs: true,
            administration: true,
        },
        UserRole::Clerk => SidebarVisibility {
            core: true,
            case_management: true,
            court_operations: true,
            legal_documents: true,
            people_orgs: true,
            administration: true,
        },
        UserRole::Judge => SidebarVisibility {
            core: true,
            case_management: true,
            court_operations: true,
            legal_documents: true,
            people_orgs: false,
            administration: false,
        },
        UserRole::Attorney => SidebarVisibility {
            core: true,
            case_management: true,
            court_operations: false,
            legal_documents: true,
            people_orgs: false,
            administration: false,
        },
        UserRole::Public => SidebarVisibility {
            core: true,
            case_management: false,
            court_operations: false,
            legal_documents: false,
            people_orgs: false,
            administration: false,
        },
    }
}
