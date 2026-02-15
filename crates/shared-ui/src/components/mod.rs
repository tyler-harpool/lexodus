// Phase 1: Standalone components (no primitives)
pub mod badge;
pub mod button;
pub mod card;
pub mod data_table;
pub mod detail_list;
pub mod form;
pub mod form_select;
pub mod input;
pub mod page_header;
pub mod pagination;
pub mod search_bar;
pub mod sheet;
pub mod skeleton;
pub mod textarea;

// Phase 2A: Simple primitive wrappers
pub mod aspect_ratio;
pub mod checkbox;
pub mod label;
pub mod progress;
pub mod separator;
pub mod switch;
pub mod toggle;

// Phase 2B: Compound primitive wrappers
pub mod accordion;
pub mod collapsible;
pub mod radio_group;
pub mod scroll_area;
pub mod tabs;
pub mod toggle_group;
pub mod toolbar;

// Phase 2C: Overlay/popup wrappers
pub mod alert_dialog;
pub mod context_menu;
pub mod dialog;
pub mod dropdown_menu;
pub mod hover_card;
pub mod popover;
pub mod tooltip;

// Phase 2D: Navigation & complex
pub mod menubar;
pub mod navbar;
pub mod select;
pub mod slider;

// Phase 2E: Special
pub mod avatar;
pub mod calendar;
pub mod date_picker;
pub mod toast;

// Phase 1 (last): Depends on button, sheet, separator, tooltip
pub mod sidebar;

// Re-exports for convenience
pub use accordion::*;
pub use alert_dialog::*;
pub use aspect_ratio::*;
pub use avatar::*;
pub use badge::*;
pub use button::*;
pub use calendar::*;
pub use card::*;
pub use checkbox::*;
pub use data_table::*;
pub use detail_list::*;
pub use collapsible::*;
pub use context_menu::*;
pub use date_picker::*;
pub use dialog::*;
pub use dropdown_menu::*;
pub use form::*;
pub use form_select::*;
pub use hover_card::*;
pub use input::*;
pub use label::*;
pub use menubar::*;
pub use navbar::*;
pub use page_header::*;
pub use pagination::*;
pub use popover::*;
pub use progress::*;
pub use radio_group::*;
pub use scroll_area::*;
pub use search_bar::*;
pub use select::*;
pub use separator::*;
pub use sheet::*;
pub use sidebar::*;
pub use skeleton::*;
pub use slider::*;
pub use switch::*;
pub use tabs::*;
pub use textarea::*;
pub use toast::*;
pub use toggle::*;
pub use toggle_group::*;
pub use toolbar::*;
pub use tooltip::*;
