use std::sync::LazyLock;

use chrono::Datelike;
use ecow::EcoVec;
use shared_types::AppError;
use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

/// Parameters for generating a generic court document PDF.
pub struct DocumentParams {
    pub court_name: String,
    pub doc_type: String,
    pub case_id: String,
    pub title: String,
    pub content_body: String,
    pub show_signature: bool,
    pub signer_id: String,
    pub document_date: String,
}

/// Escape special Typst characters inside string literals (`\`, `"`, `#`).
pub fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('#', "\\#")
}

/// Build a complete Typst source by prepending `#let` variable bindings
/// to the generic `document.typ` template.
pub fn build_document_source(params: &DocumentParams) -> String {
    let bindings = format!(
        r##"#let court_name = "{court_name}"
#let doc_type = "{doc_type}"
#let case_id = "{case_id}"
#let title = "{title}"
#let content_body = "{content_body}"
#let show_signature = {show_sig}
#let signer_id = "{signer_id}"
#let document_date = "{document_date}"

"##,
        court_name = escape_typst(&params.court_name),
        doc_type = escape_typst(&params.doc_type),
        case_id = escape_typst(&params.case_id),
        title = escape_typst(&params.title),
        content_body = escape_typst(&params.content_body),
        show_sig = params.show_signature,
        signer_id = escape_typst(&params.signer_id),
        document_date = escape_typst(&params.document_date),
    );

    let template = include_str!("../../../templates/document.typ");
    format!("{bindings}{template}")
}

// ---------------------------------------------------------------------------
// Civil-specific param structs and source builders
// ---------------------------------------------------------------------------

/// Parameters for generating a JS-44 Civil Cover Sheet.
pub struct CivilCoverSheetParams {
    pub court_name: String,
    pub case_number: String,
    pub plaintiff_name: String,
    pub defendant_name: String,
    pub county: String,
    pub attorney_info: String,
    pub jurisdiction_basis: String,
    pub nature_of_suit: String,
    pub nos_description: String,
    pub cause_of_action: String,
    pub class_action: bool,
    pub jury_demand: String,
    pub amount_in_controversy: String,
    pub document_date: String,
}

/// Build a complete Typst source for the JS-44 Civil Cover Sheet by prepending
/// `#let` variable bindings to the `js-44-cover-sheet.typ` template.
pub fn build_civil_cover_sheet_source(params: &CivilCoverSheetParams) -> String {
    let bindings = format!(
        r##"#let court_name = "{court_name}"
#let case_number = "{case_number}"
#let plaintiff_name = "{plaintiff_name}"
#let defendant_name = "{defendant_name}"
#let county = "{county}"
#let attorney_info = "{attorney_info}"
#let jurisdiction_basis = "{jurisdiction_basis}"
#let nature_of_suit = "{nature_of_suit}"
#let nos_description = "{nos_description}"
#let cause_of_action = "{cause_of_action}"
#let class_action = {class_action}
#let jury_demand = "{jury_demand}"
#let amount_in_controversy = "{amount_in_controversy}"
#let document_date = "{document_date}"

"##,
        court_name = escape_typst(&params.court_name),
        case_number = escape_typst(&params.case_number),
        plaintiff_name = escape_typst(&params.plaintiff_name),
        defendant_name = escape_typst(&params.defendant_name),
        county = escape_typst(&params.county),
        attorney_info = escape_typst(&params.attorney_info),
        jurisdiction_basis = escape_typst(&params.jurisdiction_basis),
        nature_of_suit = escape_typst(&params.nature_of_suit),
        nos_description = escape_typst(&params.nos_description),
        cause_of_action = escape_typst(&params.cause_of_action),
        class_action = params.class_action,
        jury_demand = escape_typst(&params.jury_demand),
        amount_in_controversy = escape_typst(&params.amount_in_controversy),
        document_date = escape_typst(&params.document_date),
    );

    let template = include_str!("../../../templates/js-44-cover-sheet.typ");
    format!("{bindings}{template}")
}

/// Parameters for generating a Civil Summons.
pub struct CivilSummonsParams {
    pub court_name: String,
    pub case_number: String,
    pub plaintiff_name: String,
    pub defendant_name: String,
    pub attorney_info: String,
    pub document_date: String,
}

/// Build a complete Typst source for a Civil Summons by prepending
/// `#let` variable bindings to the `civil-summons.typ` template.
pub fn build_civil_summons_source(params: &CivilSummonsParams) -> String {
    let bindings = format!(
        r##"#let court_name = "{court_name}"
#let case_number = "{case_number}"
#let plaintiff_name = "{plaintiff_name}"
#let defendant_name = "{defendant_name}"
#let attorney_info = "{attorney_info}"
#let document_date = "{document_date}"

"##,
        court_name = escape_typst(&params.court_name),
        case_number = escape_typst(&params.case_number),
        plaintiff_name = escape_typst(&params.plaintiff_name),
        defendant_name = escape_typst(&params.defendant_name),
        attorney_info = escape_typst(&params.attorney_info),
        document_date = escape_typst(&params.document_date),
    );

    let template = include_str!("../../../templates/civil-summons.typ");
    format!("{bindings}{template}")
}

/// Build a complete Typst source for a Civil Scheduling Order by prepending
/// `#let` variable bindings to the `civil-scheduling-order.typ` template.
pub fn build_civil_scheduling_order_source(params: &DocumentParams) -> String {
    let bindings = format!(
        r##"#let court_name = "{court_name}"
#let case_id = "{case_id}"
#let content_body = "{content_body}"
#let show_signature = {show_sig}
#let signer_id = "{signer_id}"
#let document_date = "{document_date}"

"##,
        court_name = escape_typst(&params.court_name),
        case_id = escape_typst(&params.case_id),
        content_body = escape_typst(&params.content_body),
        show_sig = params.show_signature,
        signer_id = escape_typst(&params.signer_id),
        document_date = escape_typst(&params.document_date),
    );

    let template = include_str!("../../../templates/civil-scheduling-order.typ");
    format!("{bindings}{template}")
}

/// Build a complete Typst source for a Civil Judgment by prepending
/// `#let` variable bindings to the `civil-judgment.typ` template.
pub fn build_civil_judgment_source(params: &DocumentParams) -> String {
    let bindings = format!(
        r##"#let court_name = "{court_name}"
#let case_id = "{case_id}"
#let content_body = "{content_body}"
#let show_signature = {show_sig}
#let signer_id = "{signer_id}"
#let document_date = "{document_date}"

"##,
        court_name = escape_typst(&params.court_name),
        case_id = escape_typst(&params.case_id),
        content_body = escape_typst(&params.content_body),
        show_sig = params.show_signature,
        signer_id = escape_typst(&params.signer_id),
        document_date = escape_typst(&params.document_date),
    );

    let template = include_str!("../../../templates/civil-judgment.typ");
    format!("{bindings}{template}")
}

// ---------------------------------------------------------------------------
// Static singletons — initialized once, reused across all requests
// ---------------------------------------------------------------------------

static FONTS: LazyLock<Vec<Font>> = LazyLock::new(|| {
    typst_assets::fonts()
        .flat_map(|data| Font::iter(Bytes::new(data)))
        .collect()
});

static FONT_BOOK: LazyLock<LazyHash<FontBook>> = LazyLock::new(|| {
    LazyHash::new(FontBook::from_fonts(FONTS.iter()))
});

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| {
    LazyHash::new(Library::default())
});

// ---------------------------------------------------------------------------
// World implementation for in-process Typst compilation
// ---------------------------------------------------------------------------

struct LexodusWorld {
    source: Source,
}

impl LexodusWorld {
    fn new(source_text: &str) -> Self {
        Self {
            source: Source::detached(source_text),
        }
    }
}

impl World for LexodusWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &FONT_BOOK
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rooted_path().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(id.vpath().as_rooted_path().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        FONTS.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = chrono::Utc::now();
        let naive = if let Some(hours) = offset {
            let tz = chrono::FixedOffset::east_opt((hours as i32) * 3600)?;
            now.with_timezone(&tz).naive_local()
        } else {
            now.naive_utc()
        };
        Datetime::from_ymd(
            naive.year(),
            (naive.month0() + 1) as u8,
            (naive.day0() + 1) as u8,
        )
    }
}

// ---------------------------------------------------------------------------
// Public compilation entry point
// ---------------------------------------------------------------------------

/// Compile a Typst source string into PDF bytes using the in-process library.
///
/// Compilation is offloaded to a blocking thread since it is CPU-bound.
pub async fn compile_typst(source: &str) -> Result<Vec<u8>, AppError> {
    let source = source.to_owned();

    tokio::task::spawn_blocking(move || compile_typst_sync(&source))
        .await
        .map_err(|e| AppError::internal(format!("Typst task panicked: {e}")))?
}

fn compile_typst_sync(source: &str) -> Result<Vec<u8>, AppError> {
    let world = LexodusWorld::new(source);

    let warned = typst::compile::<PagedDocument>(&world);
    let document = warned.output.map_err(|diagnostics| {
        format_diagnostics("Typst compilation failed", &diagnostics)
    })?;

    typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).map_err(|diagnostics| {
        format_diagnostics("PDF export failed", &diagnostics)
    })
}

fn format_diagnostics(prefix: &str, diagnostics: &EcoVec<SourceDiagnostic>) -> AppError {
    let msgs: Vec<String> = diagnostics
        .iter()
        .map(|d| d.message.to_string())
        .collect();
    AppError::internal(format!("{prefix}: {}", msgs.join("; ")))
}
