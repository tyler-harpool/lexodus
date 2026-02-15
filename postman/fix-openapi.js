/**
 * Post-process the OpenAPI spec to add:
 * 1. servers field (so Portman generates correct base URL)
 * 2. Missing operationIds for all operations
 * 3. Enum constraints for server-validated fields
 */
const fs = require("fs");

const specPath = process.argv[2] || "openapi-description.json";
const spec = JSON.parse(fs.readFileSync(specPath, "utf8"));

let fixes = 0;

// 1. Add servers field if missing
if (!spec.servers || spec.servers.length === 0) {
  spec.servers = [{ url: "http://localhost:8080", description: "Local dev" }];
  fixes++;
  console.log("Added servers field");
}

// 2. Add missing operationIds
function pathToOperationId(method, path) {
  // Convert /api/cases/{id}/plea  →  post_cases_id_plea
  const clean = path
    .replace(/^\/api\//, "")
    .replace(/\{[^}]+\}/g, "by_id")
    .replace(/[/-]/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_|_$/g, "");
  return method.toLowerCase() + "_" + clean;
}

if (spec.paths) {
  for (const [path, methods] of Object.entries(spec.paths)) {
    for (const method of ["get", "post", "put", "patch", "delete"]) {
      if (methods[method] && !methods[method].operationId) {
        methods[method].operationId = pathToOperationId(method, path);
        fixes++;
      }
    }
  }
  console.log("Added missing operationIds");
}

// 3. Add enum constraints to schemas
const enumDefs = {
  // Case enums
  crime_type: [
    "fraud", "drug_offense", "racketeering", "cybercrime",
    "tax_offense", "money_laundering", "immigration", "firearms", "other"
  ],
  priority: ["low", "medium", "high", "critical"],
  // status field is too generic — only add for specific schemas

  // Defendant enums
  citizenship_status: [
    "Citizen", "Permanent Resident", "Visa Holder", "Undocumented", "Unknown"
  ],
  custody_status: [
    "In Custody", "Released", "Bail", "Bond", "Fugitive",
    "Supervised Release", "Unknown"
  ],
  bail_type: [
    "Cash", "Surety", "Property", "Personal Recognizance",
    "Unsecured", "Denied", "None"
  ],

  // Charge enums
  plea: [
    "Not Guilty", "Guilty", "No Contest", "Alford", "Not Yet Entered"
  ],
  verdict: [
    "", "Guilty", "Not Guilty", "Dismissed", "Mistrial", "Acquitted", "Hung Jury"
  ],

  // Motion enums
  motion_type: [
    "Dismiss", "Suppress", "Compel", "Summary Judgment", "Continuance",
    "Change of Venue", "Reconsideration", "Limine", "Severance",
    "Joinder", "Discovery", "New Trial", "Other"
  ],

  // Evidence enums
  evidence_type: [
    "Physical", "Documentary", "Digital", "Testimonial",
    "Demonstrative", "Forensic", "Other"
  ],

  // Docket entry types
  entry_type: [
    "complaint", "indictment", "information", "criminal_complaint",
    "answer", "motion", "response", "reply", "notice", "order",
    "minute_order", "scheduling_order", "protective_order", "sealing_order",
    "discovery_request", "discovery_response", "deposition", "interrogatories",
    "exhibit", "witness_list", "expert_report", "hearing_notice",
    "hearing_minutes", "transcript", "judgment", "verdict", "sentence",
    "summons", "subpoena", "service_return", "appearance", "withdrawal",
    "substitution", "notice_of_appeal", "appeal_brief", "appellate_order",
    "letter", "status", "other"
  ],

  // Note types
  note_type: [
    "General", "Legal Research", "Procedural", "Confidential",
    "Bench Note", "Clerk Note", "Other"
  ],

  // Order types
  order_type: [
    "Scheduling", "Protective", "Restraining", "Dismissal", "Sentencing",
    "Detention", "Release", "Discovery", "Sealing", "Contempt",
    "Procedural", "Standing", "Other"
  ],

  // Opinion types
  opinion_type: [
    "Majority", "Concurrence", "Dissent", "Per Curiam",
    "Memorandum", "En Banc", "Summary", "Other"
  ],

  // Party types
  party_type: [
    "Plaintiff", "Defendant", "Appellant", "Appellee", "Petitioner",
    "Respondent", "Intervenor", "Amicus Curiae", "Third Party",
    "Government", "Witness", "Counter-Claimant", "Cross-Claimant", "Other", "Unknown"
  ],

  // Party roles
  party_role: [
    "Lead", "Co-Defendant", "Co-Plaintiff", "Cross-Claimant", "Counter-Claimant",
    "Garnishee", "Real Party in Interest", "Principal", "Co-Party", "Representative",
    "Guardian", "Trustee", "Executor", "Administrator", "Next Friend", "Other"
  ],

  // Entity types
  entity_type: [
    "Individual", "Corporation", "Partnership", "LLC", "Government",
    "Non-Profit", "Trust", "Estate", "Other"
  ],

  // Document types (includes PDF-generated types)
  document_type: [
    "Motion", "Order", "Brief", "Memorandum", "Declaration", "Affidavit",
    "Exhibit", "Transcript", "Notice", "Subpoena", "Warrant", "Indictment",
    "Plea Agreement", "Judgment", "Verdict", "Other",
    "Conditions of Release", "Court Order", "Criminal Judgment",
    "Minute Entry", "Rule 16(b) Scheduling Order", "Waiver of Indictment"
  ],

  // Vote types
  vote_type: [
    "Join", "Concur", "Concur in Part", "Dissent", "Dissent in Part",
    "Recused", "Not Participating"
  ],

  // Citation types
  citation_type: [
    "Followed", "Distinguished", "Overruled", "Cited", "Discussed",
    "Criticized", "Questioned", "Harmonized", "Parallel", "Other"
  ],

  // Representation types
  representation_type: [
    "Private", "Court Appointed", "Pro Bono", "Public Defender",
    "CJA Panel", "Government", "General", "Limited", "Pro Hac Vice",
    "Standby", "Other"
  ],

  // Service methods
  service_method: [
    "Electronic", "Mail", "Personal Service", "Waiver", "Publication",
    "Certified Mail", "Express Mail", "ECF", "Other"
  ],

  // Departure/variance types
  departure_type: ["Upward", "Downward", "None"],
  variance_type: ["Upward", "Downward", "None"],

  // Criminal history category
  criminal_history_category: ["I", "II", "III", "IV", "V", "VI"],

  // Victim types
  victim_type: [
    "Individual", "Organization", "Government", "Minor", "Deceased", "Anonymous"
  ],

  // Event kind
  event_kind: ["text_entry", "filing", "promote_attachment"],

  // Disposition (opinions)
  disposition: [
    "Affirmed", "Reversed", "Remanded", "Vacated", "Dismissed",
    "Modified", "Certified"
  ],

  // Conflict types (judge conflicts)
  conflict_type: [
    "Financial", "Familial", "Professional", "Prior Representation",
    "Organizational", "Other"
  ],

  // Disciplinary action types
  action_type: [
    "Reprimand", "Censure", "Suspension", "Disbarment",
    "Reinstatement", "Probation", "Other"
  ],

  // Reminder types
  reminder_type: ["Email", "SMS", "In-App", "Push", "Fax"],

  // Assignment types
  assignment_type: [
    "Initial", "Reassignment", "Temporary", "Related Case", "Emergency"
  ],

  // Checkout types (billing)
  checkout_type: ["subscription", "onetime"]
};

// Judge-specific enums (can't use generic "title" or "status")
const judgeEnums = {
  title: [
    "Chief Judge", "Judge", "Senior Judge", "Magistrate Judge", "Visiting Judge"
  ],
  status: ["Active", "Senior", "Inactive", "Retired", "Deceased"]
};

// Motion-specific status enum
const motionStatusEnum = [
  "Pending", "Granted", "Denied", "Withdrawn", "Moot",
  "Deferred", "Partially Granted"
];

function addEnumsToSchema(schemaName, schema) {
  if (!schema || !schema.properties) return;

  for (const [propName, prop] of Object.entries(schema.properties)) {
    if (prop.type === "string" && !prop.enum) {
      // Check judge-specific schemas
      if (schemaName.toLowerCase().includes("judge") && judgeEnums[propName]) {
        prop.enum = judgeEnums[propName];
        fixes++;
        continue;
      }

      // Check motion status specifically
      if (schemaName.toLowerCase().includes("motion") && propName === "status") {
        prop.enum = motionStatusEnum;
        fixes++;
        continue;
      }

      // Skip certain schemas for specific fields
      // TimelineEntry.entry_type can be docket types, document event types, or "nef"
      if (schemaName === "TimelineEntry" && propName === "entry_type") continue;

      // General enum mapping
      if (enumDefs[propName]) {
        prop.enum = enumDefs[propName];
        fixes++;
      }
    }
  }
}

// Walk all schemas in components
if (spec.components && spec.components.schemas) {
  for (const [name, schema] of Object.entries(spec.components.schemas)) {
    addEnumsToSchema(name, schema);

    // Also check allOf/oneOf/anyOf compositions
    if (schema.allOf) {
      for (const sub of schema.allOf) {
        addEnumsToSchema(name, sub);
      }
    }
  }
  console.log("Added enum constraints to schemas");
}

fs.writeFileSync(specPath, JSON.stringify(spec, null, 2));
console.log("Total fixes applied: " + fixes);
