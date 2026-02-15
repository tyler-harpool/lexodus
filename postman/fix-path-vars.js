/**
 * Post-process a Portman-generated Postman collection to replace
 * default path variable placeholders ("nost") with captured collection
 * variable references based on the endpoint path context.
 */
const fs = require("fs");

const collectionPath = process.argv[2] || "postman/lexodus-contract.json";
const collection = JSON.parse(fs.readFileSync(collectionPath, "utf8"));

// Map: path variable key → collection variable name
// For "id", we need context-based resolution (see idByPrefix below)
const namedVarMap = {
  case_id: "{{created_case_id}}",
  attorney_id: "{{created_attorney_id}}",
  judge_id: "{{created_judge_id}}",
  defendant_id: "{{created_defendant_id}}",
  party_id: "{{created_party_id}}",
  entry_id: "{{created_docket_entry_id}}",
  event_id: "{{created_event_id}}",
  deadline_id: "{{created_deadline_id}}",
  evidence_id: "{{created_evidence_id}}",
  charge_id: "{{created_charge_id}}",
  motion_id: "{{created_motion_id}}",
  order_id: "{{created_order_id}}",
  opinion_id: "{{created_opinion_id}}",
  sentencing_id: "{{created_sentencing_id}}",
  rule_id: "{{created_rule_id}}",
  template_id: "{{created_template_id}}",
  todo_id: "{{created_todo_id}}",
  filing_id: "{{created_filing_id}}",
  conflict_id: "{{created_conflict_id}}",
  docket_entry_id: "{{created_docket_entry_id}}",
  document_id: "{{created_document_id}}",
  case_number: "{{created_case_number}}",
  bar_number: "{{created_bar_number}}",
  user_id: "{{test_user_id}}",
  product_id: "{{product_id}}",
  recusal_id: "{{created_recusal_id}}",
  extension_id: "{{created_extension_id}}",
  attachment_id: "{{attachment_id}}",
  reminder_id: "{{reminder_id}}",
  draft_id: "{{created_draft_id}}",
  comment_id: "{{created_comment_id}}",
  victim_id: "{{created_victim_id}}",
};

// Static path variable values (not IDs)
const staticVarMap = {
  // bar_number handled in namedVarMap as {{created_bar_number}}
  state: "NY",
  court: "district9",
  cja_district: "district12",
  area: "criminal",
  status: "Active",
  district: "district9",
  // case_number handled in namedVarMap as {{created_case_number}}
  courtroom: "3B",
  entry_type: "motion",
  deadline_type: "open",
  text: "motion",
  firm_name: "Martinez-and-Associates-PLLC",
  party_name: "United-States-v-Rodriguez",
  category: "motion-to-suppress",
  jurisdiction: "federal",
  trigger: "nef-received",
  feature_path: "case-management",
  format: "json",
  court_id: "district9",
  offense_type: "felony",
  recipient: "clerk@district9.uscourts.gov",
};

// Context-based "id" resolution: path prefix → collection variable
const idByPrefix = [
  ["/api/attorneys", "{{created_attorney_id}}"],
  ["/api/cases", "{{created_case_id}}"],
  ["/api/judges", "{{created_judge_id}}"],
  ["/api/defendants", "{{created_defendant_id}}"],
  ["/api/parties", "{{created_party_id}}"],
  ["/api/docket", "{{created_docket_entry_id}}"],
  ["/api/calendar", "{{created_event_id}}"],
  ["/api/deadlines", "{{created_deadline_id}}"],
  ["/api/evidence", "{{created_evidence_id}}"],
  ["/api/charges", "{{created_charge_id}}"],
  ["/api/motions", "{{created_motion_id}}"],
  ["/api/orders", "{{created_order_id}}"],
  ["/api/opinions", "{{created_opinion_id}}"],
  ["/api/sentencing", "{{created_sentencing_id}}"],
  ["/api/rules", "{{created_rule_id}}"],
  ["/api/templates", "{{created_template_id}}"],
  ["/api/todos", "{{created_todo_id}}"],
  ["/api/case-notes", "{{created_case_note_id}}"],
  ["/api/service-records", "{{created_service_record_id}}"],
  ["/api/filings/upload", "{{created_upload_id}}"],
  ["/api/filings", "{{created_filing_id}}"],
  ["/api/conflict-checks", "{{created_conflict_id}}"],
  ["/api/documents", "{{created_document_id}}"],
  ["/api/representations", "{{created_representation_id}}"],
  ["/api/assignments", "{{created_assignment_id}}"],
  ["/api/custody-transfers", "{{created_custody_transfer_id}}"],
  ["/api/nef", "{{id}}"],
  ["/api/speedy-trial", "{{id}}"],
  ["/api/extensions", "{{created_extension_id}}"],
  ["/api/admin/court-role-requests", "{{id}}"],
  ["/api/recusals", "{{created_recusal_id}}"],
];

function getPath(urlObj) {
  if (urlObj && urlObj.path) {
    return "/" + urlObj.path.join("/");
  }
  return "";
}

function resolveId(path) {
  for (const [prefix, varRef] of idByPrefix) {
    if (path.startsWith(prefix)) return varRef;
  }
  return "{{id}}";
}

// Context-sensitive status path variable resolution
const statusByPrefix = [
  ["/api/cases/count-by-status", "filed"],
  ["/api/attorneys/status", "Active"],
  ["/api/judges/status", "Active"],
];

function resolveStatus(path) {
  for (const [prefix, val] of statusByPrefix) {
    if (path.startsWith(prefix)) return val;
  }
  return "Active";
}

let fixCount = 0;

function walkItems(items) {
  for (const item of items) {
    if (item.item) {
      walkItems(item.item);
      continue;
    }
    if (item.request && item.request.url && item.request.url.variable) {
      const path = getPath(item.request.url);
      for (const v of item.request.url.variable) {
        const key = v.key;
        // Match any Portman-generated placeholder: lorem ipsum words, random numbers, UUIDs, known defaults
        const isPlaceholder = (val) => {
          if (!val) return false;
          if (val === "<string>" || val === "nost") return true;
          if (/^-?\d{3,}$/.test(val)) return true;  // numeric placeholders
          // UUID format (Portman generates random UUIDs for UUID params)
          if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(val)) return true;
          // urn:uuid: prefixed UUIDs
          if (/^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(val)) return true;
          // Latin/lorem ipsum: 1-5 lowercase words
          if (/^[a-z]+( [a-z]+){0,4}$/i.test(val) && val.length < 50) return true;
          return false;
        };
        if (isPlaceholder(v.value)) {
          let newVal = null;
          if (key === "id") {
            newVal = resolveId(path);
          } else if (key === "status") {
            newVal = resolveStatus(path);
          } else if (namedVarMap[key]) {
            newVal = namedVarMap[key];
          } else if (staticVarMap[key]) {
            newVal = staticVarMap[key];
          }
          if (newVal) {
            v.value = newVal;
            fixCount++;
          }
        }
      }
    }
  }
}

walkItems(collection.item || []);

fs.writeFileSync(collectionPath, JSON.stringify(collection, null, 2));
console.log("Fixed " + fixCount + " path variable values");
