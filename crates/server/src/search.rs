use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, STORED, TEXT};
use tantivy::schema::Value;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};

/// Global search index, initialized once during server startup.
static SEARCH_INDEX: OnceLock<SearchIndex> = OnceLock::new();

/// A single search result returned by the global search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub entity_type: String,
    pub title: String,
    pub subtitle: String,
}

/// Schema field handles for the Tantivy index.
struct SearchFields {
    id: Field,
    entity_type: Field,
    title: Field,
    subtitle: Field,
    court_id: Field,
}

/// In-memory Tantivy search index for global court entity search.
pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    fields: SearchFields,
}

impl SearchIndex {
    /// Create a new in-RAM search index with the standard schema.
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();
        let id = schema_builder.add_text_field("id", STORED);
        let entity_type = schema_builder.add_text_field("entity_type", STORED);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let subtitle = schema_builder.add_text_field("subtitle", TEXT | STORED);
        let court_id = schema_builder.add_text_field("court_id", STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema);
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .expect("failed to create index reader");

        SearchIndex {
            index,
            reader,
            fields: SearchFields {
                id,
                entity_type,
                title,
                subtitle,
                court_id,
            },
        }
    }

    /// Full-text search filtered by court_id. Returns up to `limit` results.
    pub fn search(&self, query_str: &str, court_id: &str, limit: usize) -> Vec<SearchResult> {
        let searcher = self.reader.searcher();
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.fields.title, self.fields.subtitle]);

        let query = match query_parser.parse_query(query_str) {
            Ok(q) => q,
            Err(_) => return Vec::new(),
        };

        let top_docs = match searcher.search(&query, &TopDocs::with_limit(limit * 3)) {
            Ok(docs) => docs,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc: tantivy::TantivyDocument = match searcher.doc(doc_address) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let stored_court = doc
                .get_first(self.fields.court_id)
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if stored_court != court_id {
                continue;
            }

            let id = doc
                .get_first(self.fields.id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let entity_type = doc
                .get_first(self.fields.entity_type)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = doc
                .get_first(self.fields.title)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let subtitle = doc
                .get_first(self.fields.subtitle)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(SearchResult {
                id,
                entity_type,
                title,
                subtitle,
            });

            if results.len() >= limit {
                break;
            }
        }

        results
    }

    /// Acquire an IndexWriter for bulk indexing. Caller must commit.
    fn writer(&self) -> IndexWriter {
        self.index
            .writer(50_000_000)
            .expect("failed to create index writer")
    }
}

/// Get the global SearchIndex. Panics if not yet initialized.
pub fn get_search() -> &'static SearchIndex {
    SEARCH_INDEX
        .get()
        .expect("SearchIndex not initialized — call init_search() first")
}

/// Initialize the global SearchIndex. Should be called once at startup.
pub fn init_search() -> &'static SearchIndex {
    SEARCH_INDEX.get_or_init(SearchIndex::new)
}

/// Row type for lightweight criminal case queries used during indexing.
struct CaseRow {
    id: uuid::Uuid,
    court_id: String,
    case_number: String,
    title: String,
    crime_type: String,
}

/// Row type for lightweight civil case queries used during indexing.
struct CivilCaseRow {
    id: uuid::Uuid,
    court_id: String,
    case_number: String,
    title: String,
    nature_of_suit: String,
    jurisdiction_basis: String,
}

/// Row type for lightweight attorney queries used during indexing.
struct AttorneyRow {
    id: uuid::Uuid,
    court_id: String,
    first_name: String,
    last_name: String,
    bar_number: String,
    firm_name: Option<String>,
}

/// Row type for lightweight judge queries used during indexing.
struct JudgeRow {
    id: uuid::Uuid,
    court_id: String,
    name: String,
    title: String,
}

/// Row type for lightweight docket entry queries used during indexing.
struct DocketRow {
    id: uuid::Uuid,
    court_id: String,
    entry_number: i32,
    entry_type: String,
    description: String,
    case_number: String,
}

/// Row type for lightweight calendar event queries used during indexing.
struct CalendarRow {
    id: uuid::Uuid,
    court_id: String,
    event_type: String,
    description: Option<String>,
    case_title: String,
}

/// Row type for lightweight deadline queries used during indexing.
struct DeadlineRow {
    id: uuid::Uuid,
    court_id: String,
    title: String,
    status: String,
    case_title: String,
}

/// Row type for lightweight judicial order queries used during indexing.
struct OrderRow {
    id: uuid::Uuid,
    court_id: String,
    title: String,
    order_type: String,
    case_title: String,
}

/// Row type for lightweight judicial opinion queries used during indexing.
struct OpinionRow {
    id: uuid::Uuid,
    court_id: String,
    title: String,
    author_judge_name: String,
}

/// Build the full-text search index from all court entities in the database.
/// Should be called once at server startup after migrations complete.
pub async fn build_index(pool: &Pool<Postgres>, search: &SearchIndex) {
    let mut writer = search.writer();
    let f = &search.fields;

    // Index cases
    let cases = sqlx::query_as!(
        CaseRow,
        "SELECT id, court_id, case_number, title, crime_type FROM criminal_cases"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &cases {
        let display_title = format!("{} - {}", row.case_number, row.title);
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "case",
                f.title => display_title,
                f.subtitle => row.crime_type.as_str(),
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    // Index civil cases
    let civil_cases = sqlx::query_as!(
        CivilCaseRow,
        "SELECT id, court_id, case_number, title, nature_of_suit, jurisdiction_basis FROM civil_cases"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &civil_cases {
        let display_title = format!("{} - {}", row.case_number, row.title);
        let display_subtitle = format!("NOS {} | {}", row.nature_of_suit, row.jurisdiction_basis.replace('_', " "));
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "civil_case",
                f.title => display_title,
                f.subtitle => display_subtitle,
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    // Index attorneys
    let attorneys = sqlx::query_as!(
        AttorneyRow,
        "SELECT id, court_id, first_name, last_name, bar_number, firm_name FROM attorneys"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &attorneys {
        let display_title = format!("{} {}", row.first_name, row.last_name);
        let firm = row.firm_name.as_deref().unwrap_or("N/A");
        let display_subtitle = format!("{} - {}", row.bar_number, firm);
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "attorney",
                f.title => display_title,
                f.subtitle => display_subtitle,
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    // Index judges
    let judges = sqlx::query_as!(
        JudgeRow,
        "SELECT id, court_id, name, title FROM judges"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &judges {
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "judge",
                f.title => row.name.as_str(),
                f.subtitle => row.title.as_str(),
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    // Index docket entries (joined with case for case_number, both criminal and civil)
    let dockets = sqlx::query_as!(
        DocketRow,
        r#"
        SELECT d.id as "id!", d.court_id as "court_id!", d.entry_number as "entry_number!",
               d.entry_type as "entry_type!", d.description as "description!",
               c.case_number as "case_number!"
        FROM docket_entries d
        JOIN criminal_cases c ON c.id = d.case_id AND c.court_id = d.court_id
        WHERE d.case_type = 'criminal'
        UNION ALL
        SELECT d.id as "id!", d.court_id as "court_id!", d.entry_number as "entry_number!",
               d.entry_type as "entry_type!", d.description as "description!",
               cv.case_number as "case_number!"
        FROM docket_entries d
        JOIN civil_cases cv ON cv.id = d.case_id AND cv.court_id = d.court_id
        WHERE d.case_type = 'civil'
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &dockets {
        let desc_truncated: String = row.description.chars().take(80).collect();
        let display_title = format!("Dkt #{}: {}", row.entry_number, desc_truncated);
        let display_subtitle = format!("{} - case {}", row.entry_type, row.case_number);
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "docket",
                f.title => display_title,
                f.subtitle => display_subtitle,
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    // Index calendar events (joined with case for title, both criminal and civil)
    let events = sqlx::query_as!(
        CalendarRow,
        r#"
        SELECT e.id as "id!", e.court_id as "court_id!", e.event_type as "event_type!",
               e.description as "description!", c.title as "case_title!"
        FROM calendar_events e
        JOIN criminal_cases c ON c.id = e.case_id AND c.court_id = e.court_id
        WHERE e.case_type = 'criminal'
        UNION ALL
        SELECT e.id as "id!", e.court_id as "court_id!", e.event_type as "event_type!",
               e.description as "description!", cv.title as "case_title!"
        FROM calendar_events e
        JOIN civil_cases cv ON cv.id = e.case_id AND cv.court_id = e.court_id
        WHERE e.case_type = 'civil'
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &events {
        let desc = row.description.as_deref().unwrap_or("");
        let display_title = format!("{}: {}", row.event_type, desc);
        let display_subtitle = format!("case {}", row.case_title);
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "calendar",
                f.title => display_title,
                f.subtitle => display_subtitle,
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    // Index deadlines (joined with case for title, both criminal and civil)
    let deadlines = sqlx::query_as!(
        DeadlineRow,
        r#"
        SELECT d.id as "id!", d.court_id as "court_id!", d.title as "title!",
               d.status as "status!", c.title as "case_title!"
        FROM deadlines d
        JOIN criminal_cases c ON c.id = d.case_id AND c.court_id = d.court_id
        WHERE d.case_type = 'criminal'
        UNION ALL
        SELECT d.id as "id!", d.court_id as "court_id!", d.title as "title!",
               d.status as "status!", cv.title as "case_title!"
        FROM deadlines d
        JOIN civil_cases cv ON cv.id = d.case_id AND cv.court_id = d.court_id
        WHERE d.case_type = 'civil'
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &deadlines {
        let display_subtitle = format!("{} - case {}", row.status, row.case_title);
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "deadline",
                f.title => row.title.as_str(),
                f.subtitle => display_subtitle,
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    // Index judicial orders (joined with case for title, both criminal and civil)
    let orders = sqlx::query_as!(
        OrderRow,
        r#"
        SELECT o.id as "id!", o.court_id as "court_id!", o.title as "title!",
               o.order_type as "order_type!", c.title as "case_title!"
        FROM judicial_orders o
        JOIN criminal_cases c ON c.id = o.case_id AND c.court_id = o.court_id
        WHERE o.case_type = 'criminal'
        UNION ALL
        SELECT o.id as "id!", o.court_id as "court_id!", o.title as "title!",
               o.order_type as "order_type!", cv.title as "case_title!"
        FROM judicial_orders o
        JOIN civil_cases cv ON cv.id = o.case_id AND cv.court_id = o.court_id
        WHERE o.case_type = 'civil'
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &orders {
        let display_subtitle = format!("{} - case {}", row.order_type, row.case_title);
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "order",
                f.title => row.title.as_str(),
                f.subtitle => display_subtitle,
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    // Index judicial opinions
    let opinions = sqlx::query_as!(
        OpinionRow,
        "SELECT id, court_id, title, author_judge_name FROM judicial_opinions"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for row in &opinions {
        let display_subtitle = format!("by {}", row.author_judge_name);
        writer
            .add_document(doc!(
                f.id => row.id.to_string(),
                f.entity_type => "opinion",
                f.title => row.title.as_str(),
                f.subtitle => display_subtitle,
                f.court_id => row.court_id.as_str(),
            ))
            .ok();
    }

    writer.commit().expect("failed to commit search index");
    // Reload the reader so searches pick up the freshly committed segments.
    search.reader.reload().ok();
}
