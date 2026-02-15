use dioxus::prelude::*;
use server::api::{create_product, delete_product, list_products, update_product};
use shared_types::Product;
use shared_ui::{
    use_toast, Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardHeader,
    CardTitle, Collapsible, CollapsibleContent, CollapsibleTrigger, Date, DatePicker,
    DatePickerCalendar, DatePickerInput, DatePickerPopover, Form, Input, Label, RadioGroup,
    RadioGroupItem, SearchBar, SelectContent, SelectItem, SelectRoot, SelectTrigger, SelectValue,
    Separator, Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader,
    SheetSide, SheetTitle, Skeleton, SliderRange, SliderRoot, SliderThumb, SliderTrack, SliderValue,
    TabContent, TabList, TabTrigger, Tabs, Textarea, TextareaVariant, ToastOptions, ToggleGroup,
    ToggleGroupItem,
};

/// Maximum price bound used by the slider filter.
const PRICE_SLIDER_MAX: f64 = 1000.0;

/// Step increment for the price slider.
const PRICE_SLIDER_STEP: f64 = 10.0;

/// Maps a product status string to the appropriate badge variant.
fn badge_variant_for_status(status: &str) -> BadgeVariant {
    match status {
        "active" => BadgeVariant::Primary,
        "draft" => BadgeVariant::Secondary,
        "archived" => BadgeVariant::Outline,
        _ => BadgeVariant::Secondary,
    }
}

/// Filters a product list by status tab, search query, category, price, and date.
fn filter_products(
    products: &[Product],
    tab: &str,
    search: &str,
    category: &str,
    price_max: f64,
    date_after: Option<Date>,
) -> Vec<Product> {
    let query = search.to_lowercase();
    products
        .iter()
        .filter(|p| tab == "all" || p.status == tab)
        .filter(|p| {
            query.is_empty()
                || p.name.to_lowercase().contains(&query)
                || p.description.to_lowercase().contains(&query)
        })
        .filter(|p| category == "All" || p.category == category)
        .filter(|p| p.price <= price_max)
        .filter(|p| {
            date_after.is_none_or(|d| {
                let date_str = format!("{}-{:02}-{:02}", d.year(), d.month() as u8, d.day());
                p.created_at >= date_str
            })
        })
        .cloned()
        .collect()
}

/// Products page displaying a filterable, tabbed product catalog with CRUD operations.
#[component]
pub fn Products() -> Element {
    let mut products = use_server_future(list_products)?;
    let toast = use_toast();

    let mut view_mode = use_signal(|| "grid".to_string());
    let mut search_query = use_signal(String::new);
    let mut category_filter = use_signal(|| "All".to_string());
    let mut price_max = use_signal(|| PRICE_SLIDER_MAX);
    let mut date_after = use_signal(|| None::<Date>);
    let mut show_sheet = use_signal(|| false);
    let mut editing_product = use_signal(|| Option::<Product>::None);

    let mut form_name = use_signal(String::new);
    let mut form_description = use_signal(String::new);
    let mut form_price = use_signal(String::new);
    let mut form_category = use_signal(|| "Hardware".to_string());
    let mut form_status = use_signal(|| "active".to_string());

    let open_create = move |_| {
        editing_product.set(None);
        form_name.set(String::new());
        form_description.set(String::new());
        form_price.set(String::new());
        form_category.set("Hardware".to_string());
        form_status.set("active".to_string());
        show_sheet.set(true);
    };

    let handle_save = move |_: FormEvent| {
        let name = form_name();
        let description = form_description();
        let price_str = form_price();
        let category = form_category();
        let status = form_status();
        let editing = editing_product();

        spawn(async move {
            let parsed_price: f64 = price_str.parse().unwrap_or(0.0);

            let result = if let Some(existing) = editing {
                update_product(
                    existing.id,
                    name,
                    description,
                    parsed_price,
                    category,
                    status,
                )
                .await
            } else {
                create_product(name, description, parsed_price, category, status).await
            };

            match result {
                Ok(_) => {
                    products.restart();
                    show_sheet.set(false);
                    toast.success(
                        "Product saved successfully".to_string(),
                        ToastOptions::new(),
                    );
                }
                Err(err) => {
                    toast.error(
                        shared_types::AppError::friendly_message(&err.to_string()),
                        ToastOptions::new(),
                    );
                }
            }
        });
    };

    let handle_delete = move |product_id: i64| {
        spawn(async move {
            match delete_product(product_id).await {
                Ok(()) => {
                    products.restart();
                    show_sheet.set(false);
                    toast.success("Product deleted".to_string(), ToastOptions::new());
                }
                Err(err) => {
                    toast.error(
                        shared_types::AppError::friendly_message(&err.to_string()),
                        ToastOptions::new(),
                    );
                }
            }
        });
    };

    let product_list = products.read();
    let all_products: Vec<Product> = match product_list.as_ref() {
        Some(Ok(list)) => list.clone(),
        _ => vec![],
    };
    let is_loading = product_list.is_none();

    let query = search_query();
    let cat = category_filter();
    let pmax = price_max();
    let dafter = date_after();

    let filtered_all = filter_products(&all_products, "all", &query, &cat, pmax, dafter);
    let filtered_active = filter_products(&all_products, "active", &query, &cat, pmax, dafter);
    let filtered_archived = filter_products(&all_products, "archived", &query, &cat, pmax, dafter);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./products.css") }

        div {
            class: "products-page",

            // Page header
            div {
                class: "page-header-responsive products-header",
                h1 {
                    class: "products-title",
                    "Products"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: open_create,
                    "New Product"
                }
            }

            Separator {}

            // Search bar
            SearchBar {
                Input {
                    value: search_query(),
                    placeholder: "Search products...",
                    label: "",
                    on_input: move |evt: FormEvent| search_query.set(evt.value()),
                }
            }

            // Filter bar inside a Collapsible
            Collapsible {
                CollapsibleTrigger {
                    Button {
                        variant: ButtonVariant::Outline,
                        "Filters"
                    }
                }
                CollapsibleContent {
                    div {
                        class: "filter-bar",

                        // Category filter
                        div {
                            class: "filter-control filter-field",
                            Label { html_for: "category-filter", "Category" }
                            SelectRoot::<String> {
                                on_value_change: move |val: Option<String>| {
                                    if let Some(v) = val {
                                        category_filter.set(v);
                                    }
                                },
                                SelectTrigger {
                                    SelectValue {}
                                }
                                SelectContent {
                                    SelectItem::<String> { value: "All", index: 0usize, "All" }
                                    SelectItem::<String> { value: "Hardware", index: 1usize, "Hardware" }
                                    SelectItem::<String> { value: "Software", index: 2usize, "Software" }
                                    SelectItem::<String> { value: "Service", index: 3usize, "Service" }
                                }
                            }
                        }

                        // Price range slider
                        div {
                            class: "filter-control filter-field",
                            Label { html_for: "price-slider", "Max Price: ${pmax:.0}" }
                            SliderRoot {
                                default_value: SliderValue::Single(PRICE_SLIDER_MAX),
                                max: PRICE_SLIDER_MAX,
                                step: PRICE_SLIDER_STEP,
                                on_value_change: move |val: SliderValue| {
                                    let SliderValue::Single(v) = val;
                                    price_max.set(v);
                                },
                                SliderTrack {
                                    SliderRange {}
                                }
                                SliderThumb {}
                            }
                        }

                        // Date picker filter
                        div {
                            class: "filter-control filter-field",
                            Label { html_for: "date-filter", "Created After" }
                            DatePicker {
                                selected_date: date_after(),
                                on_value_change: move |val: Option<Date>| {
                                    date_after.set(val);
                                },
                                DatePickerInput {}
                                DatePickerPopover {
                                    DatePickerCalendar {}
                                }
                            }
                        }

                        // Reset filters
                        div {
                            class: "filter-control filter-reset",
                            Button {
                                variant: ButtonVariant::Ghost,
                                onclick: move |_| {
                                    search_query.set(String::new());
                                    category_filter.set("All".to_string());
                                    price_max.set(PRICE_SLIDER_MAX);
                                    date_after.set(None);
                                },
                                "Reset Filters"
                            }
                        }
                    }
                }
            }

            // Tabs + view toggle row
            Tabs {
                horizontal: true,
                default_value: "all",
                div {
                    class: "tabs-toolbar",
                    TabList {
                        TabTrigger { value: "all", index: 0usize, "All" }
                        TabTrigger { value: "active", index: 1usize, "Active" }
                        TabTrigger { value: "archived", index: 2usize, "Archived" }
                    }
                    div {
                        class: "view-toggle-row",
                        ToggleGroup {
                            horizontal: true,
                            default_pressed: std::collections::HashSet::from([0]),
                            on_pressed_change: move |pressed: std::collections::HashSet<usize>| {
                                if pressed.contains(&0) {
                                    view_mode.set("grid".to_string());
                                } else if pressed.contains(&1) {
                                    view_mode.set("list".to_string());
                                }
                            },
                            ToggleGroupItem { index: 0usize, "Grid" }
                            ToggleGroupItem { index: 1usize, "List" }
                        }
                    }
                }

                TabContent { value: "all", index: 0usize,
                    if is_loading {
                        {render_skeletons()}
                    } else if filtered_all.is_empty() {
                        {render_empty_state()}
                    } else {
                        ProductGrid {
                            products: filtered_all.clone(),
                            view_mode: view_mode(),
                            editing_product,
                            form_name,
                            form_description,
                            form_price,
                            form_category,
                            form_status,
                            show_sheet,
                        }
                    }
                }

                TabContent { value: "active", index: 1usize,
                    if is_loading {
                        {render_skeletons()}
                    } else if filtered_active.is_empty() {
                        {render_empty_state()}
                    } else {
                        ProductGrid {
                            products: filtered_active.clone(),
                            view_mode: view_mode(),
                            editing_product,
                            form_name,
                            form_description,
                            form_price,
                            form_category,
                            form_status,
                            show_sheet,
                        }
                    }
                }

                TabContent { value: "archived", index: 2usize,
                    if is_loading {
                        {render_skeletons()}
                    } else if filtered_archived.is_empty() {
                        {render_empty_state()}
                    } else {
                        ProductGrid {
                            products: filtered_archived.clone(),
                            view_mode: view_mode(),
                            editing_product,
                            form_name,
                            form_description,
                            form_price,
                            form_category,
                            form_status,
                            show_sheet,
                        }
                    }
                }
            }

            // Product detail / edit Sheet
            Sheet {
                open: show_sheet(),
                on_close: move |_| show_sheet.set(false),
                side: SheetSide::Right,
                SheetContent {
                    SheetHeader {
                        SheetTitle {
                            if editing_product().is_some() { "Edit Product" } else { "New Product" }
                        }
                        SheetDescription {
                            if editing_product().is_some() {
                                "Update the product details below."
                            } else {
                                "Fill in the details to create a new product."
                            }
                        }
                        SheetClose { on_close: move |_| show_sheet.set(false) }
                    }

                    Form {
                        onsubmit: handle_save,

                        div {
                            class: "sheet-form",

                            Input {
                                label: "Name",
                                value: form_name(),
                                on_input: move |evt: FormEvent| form_name.set(evt.value()),
                                placeholder: "Product name",
                            }

                            Textarea {
                                variant: TextareaVariant::Default,
                                value: form_description(),
                                on_input: move |evt: FormEvent| form_description.set(evt.value()),
                                placeholder: "Product description",
                                label: "Description",
                            }

                            Input {
                                label: "Price",
                                value: form_price(),
                                on_input: move |evt: FormEvent| form_price.set(evt.value()),
                                placeholder: "0.00",
                            }

                            div {
                                class: "sheet-field",
                                Label { html_for: "form-category", "Category" }
                                SelectRoot::<String> {
                                    default_value: Some(form_category()),
                                    on_value_change: move |val: Option<String>| {
                                        if let Some(v) = val {
                                            form_category.set(v);
                                        }
                                    },
                                    SelectTrigger {
                                        SelectValue {}
                                    }
                                    SelectContent {
                                        SelectItem::<String> { value: "Hardware", index: 0usize, "Hardware" }
                                        SelectItem::<String> { value: "Software", index: 1usize, "Software" }
                                        SelectItem::<String> { value: "Service", index: 2usize, "Service" }
                                    }
                                }
                            }

                            div {
                                class: "sheet-field",
                                Label { html_for: "form-status", "Status" }
                                RadioGroup {
                                    default_value: form_status(),
                                    on_value_change: move |val: String| form_status.set(val),
                                    div {
                                        class: "status-radio-group",
                                        label {
                                            class: "status-radio-label",
                                            RadioGroupItem { value: "active", index: 0usize }
                                            "Active"
                                        }
                                        label {
                                            class: "status-radio-label",
                                            RadioGroupItem { value: "draft", index: 1usize }
                                            "Draft"
                                        }
                                        label {
                                            class: "status-radio-label",
                                            RadioGroupItem { value: "archived", index: 2usize }
                                            "Archived"
                                        }
                                    }
                                }
                            }
                        }

                        Separator {}

                        SheetFooter {
                            div {
                                class: "sheet-footer-actions",

                                if let Some(ref product) = editing_product() {
                                    {
                                        let product_id = product.id;
                                        rsx! {
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                onclick: move |_| handle_delete(product_id),
                                                "Delete"
                                            }
                                        }
                                    }
                                }

                                SheetClose { on_close: move |_| show_sheet.set(false) }

                                Button {
                                    variant: ButtonVariant::Primary,
                                    "Save"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Grid or list of product cards.
#[component]
fn ProductGrid(
    products: Vec<Product>,
    view_mode: String,
    mut editing_product: Signal<Option<Product>>,
    mut form_name: Signal<String>,
    mut form_description: Signal<String>,
    mut form_price: Signal<String>,
    mut form_category: Signal<String>,
    mut form_status: Signal<String>,
    mut show_sheet: Signal<bool>,
) -> Element {
    let is_grid = view_mode == "grid";
    let container_class = if is_grid {
        "product-grid"
    } else {
        "product-list"
    };

    rsx! {
        div {
            class: "{container_class}",
            for product in products.iter() {
                {
                    let p = product.clone();
                    let variant = badge_variant_for_status(&product.status);
                    rsx! {
                        div {
                            class: "product-card-link",
                            onclick: move |_| {
                                let pp = p.clone();
                                form_name.set(pp.name.clone());
                                form_description.set(pp.description.clone());
                                form_price.set(format!("{:.2}", pp.price));
                                form_category.set(pp.category.clone());
                                form_status.set(pp.status.clone());
                                editing_product.set(Some(pp));
                                show_sheet.set(true);
                            },
                            Card {
                                CardHeader {
                                    div {
                                        class: "product-card-header",
                                        CardTitle { "{product.name}" }
                                        Badge { variant: variant, "{product.status}" }
                                    }
                                }
                                CardContent {
                                    div {
                                        class: "product-card-body",
                                        p {
                                            class: "product-price",
                                            "${product.price:.2}"
                                        }
                                        p {
                                            class: "product-category",
                                            "{product.category}"
                                        }
                                        p {
                                            class: "product-description",
                                            "{product.description}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Renders placeholder skeletons while product data is loading.
fn render_skeletons() -> Element {
    rsx! {
        div {
            class: "skeleton-grid",
            for _ in 0..6 {
                Card {
                    CardHeader {
                        Skeleton { style: "height: 24px; width: 60%;" }
                    }
                    CardContent {
                        div {
                            class: "skeleton-body",
                            Skeleton { style: "height: 20px; width: 40%;" }
                            Skeleton { style: "height: 16px; width: 30%;" }
                            Skeleton { style: "height: 16px; width: 80%;" }
                        }
                    }
                }
            }
        }
    }
}

/// Renders an empty state message when no products match the current filters.
fn render_empty_state() -> Element {
    rsx! {
        div {
            class: "products-empty",
            p {
                class: "products-empty-title",
                "No products found"
            }
            p {
                class: "products-empty-subtitle",
                "Try adjusting the filters or create a new product."
            }
        }
    }
}
