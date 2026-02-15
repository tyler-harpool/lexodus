# Portman Contract Testing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Full contract testing of all 396 API endpoints using Portman — auto-generated schema validation, ordered data seeding via `assignVariables`, and negative test variations.

**Architecture:** Portman reads `openapi-description.json` and generates a Postman collection with contract tests, ordered by entity dependency chain. A `portman-config.json` defines request ordering, variable capture, request body overwrites, and auth scripts. Newman executes the collection.

**Tech Stack:** @apideck/portman (npm), Newman (installed globally), OpenAPI 3.1 spec

---

### Task 1: Install Portman and verify it can read the spec

**Files:**
- Modify: `postman/package.json`

**Step 1: Install Portman**

```bash
cd postman && npm install @apideck/portman
```

**Step 2: Verify Portman can parse the OpenAPI spec**

```bash
cd /path/to/lexodus && npx @apideck/portman --local openapi-description.json --output postman/portman-test-output.json 2>&1
```

Expected: Collection generated without errors. Delete test output after.

```bash
rm postman/portman-test-output.json
```

**Step 3: Commit**

```bash
git add postman/package.json postman/package-lock.json
git commit -m "Add @apideck/portman dependency"
```

---

### Task 2: Create the base Portman config with auth and ordering

**Files:**
- Create: `postman/portman-config.json`

This is the core config file. It defines:
1. Global settings (base URL, collection name)
2. Auth pre-request script (register/login, capture token)
3. `orderOfOperations` — the dependency chain for data seeding
4. `assignVariables` — capture IDs from create responses

**Step 1: Write `postman/portman-config.json`**

```json
{
  "version": 1.0,
  "globals": {
    "collectionPreRequestScripts": [
      "// Auth: register or login, store access_token",
      "if (pm.collectionVariables.get('access_token')) { return; }",
      "",
      "const baseUrl = pm.environment.get('base_url') || 'http://localhost:8080';",
      "const email = pm.environment.get('test_email');",
      "const password = pm.environment.get('test_password');",
      "const username = pm.environment.get('test_username');",
      "const displayName = pm.environment.get('test_display_name');",
      "",
      "pm.sendRequest({",
      "  url: baseUrl + '/api/v1/auth/register',",
      "  method: 'POST',",
      "  header: { 'Content-Type': 'application/json' },",
      "  body: { mode: 'raw', raw: JSON.stringify({ username, email, password, display_name: displayName }) }",
      "}, function (err, res) {",
      "  if (!err && res.code >= 200 && res.code < 300) {",
      "    const body = res.json();",
      "    pm.collectionVariables.set('access_token', body.access_token);",
      "    if (body.user && body.user.id) pm.collectionVariables.set('test_user_id', body.user.id);",
      "    console.log('Registered and got token');",
      "    return;",
      "  }",
      "  pm.sendRequest({",
      "    url: baseUrl + '/api/v1/auth/login',",
      "    method: 'POST',",
      "    header: { 'Content-Type': 'application/json' },",
      "    body: { mode: 'raw', raw: JSON.stringify({ email, password }) }",
      "  }, function (err2, res2) {",
      "    if (!err2 && res2.code >= 200 && res2.code < 300) {",
      "      const body2 = res2.json();",
      "      pm.collectionVariables.set('access_token', body2.access_token);",
      "      if (body2.user && body2.user.id) pm.collectionVariables.set('test_user_id', body2.user.id);",
      "      console.log('Logged in and got token');",
      "    } else {",
      "      console.error('Auth failed:', err2 || res2.code);",
      "    }",
      "  });",
      "});"
    ],
    "securityOverwrites": {
      "bearer": {
        "token": "{{access_token}}"
      }
    }
  },
  "tests": {
    "contractTests": [
      {
        "openApiOperation": "*::/*",
        "statusSuccess": {
          "enabled": true
        }
      },
      {
        "openApiOperation": "*::/*",
        "responseTime": {
          "enabled": true,
          "maxMs": 5000
        }
      },
      {
        "openApiOperation": "*::/*",
        "contentType": {
          "enabled": true
        }
      },
      {
        "openApiOperation": "*::/*",
        "jsonBody": {
          "enabled": true
        }
      },
      {
        "openApiOperation": "*::/*",
        "schemaValidation": {
          "enabled": true
        }
      }
    ]
  },
  "assignVariables": [
    {
      "openApiOperationId": "register",
      "collectionVariables": [
        {
          "responseBodyProp": "access_token",
          "name": "access_token"
        },
        {
          "responseBodyProp": "user.id",
          "name": "test_user_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_case",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_case_id"
        },
        {
          "responseBodyProp": "case_number",
          "name": "created_case_number"
        }
      ]
    },
    {
      "openApiOperationId": "create_attorney",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_attorney_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_judge",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_judge_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_defendant",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_defendant_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_party",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_party_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_docket_entry",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_docket_entry_id"
        }
      ]
    },
    {
      "openApiOperationId": "submit_filing",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_filing_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_motion",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_motion_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_order",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_order_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_opinion",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_opinion_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_evidence",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_evidence_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_charge",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_charge_id"
        }
      ]
    },
    {
      "openApiOperationId": "schedule_event",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_event_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_deadline",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_deadline_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_sentencing",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_sentencing_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_rule",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_rule_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_conflict_check",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_conflict_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_template",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_template_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_todo",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_todo_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_case_note",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_case_note_id"
        }
      ]
    },
    {
      "openApiOperationId": "create_service_record",
      "collectionVariables": [
        {
          "responseBodyProp": "id",
          "name": "created_service_record_id"
        }
      ]
    }
  ],
  "orderOfOperations": [
    "POST::/api/v1/auth/register",
    "POST::/api/v1/auth/login",
    "GET::/health",

    "POST::/api/cases",
    "GET::/api/cases",
    "GET::/api/cases/{id}",

    "POST::/api/attorneys",
    "GET::/api/attorneys",
    "GET::/api/attorneys/{id}",

    "POST::/api/judges",
    "GET::/api/judges",
    "GET::/api/judges/{id}",

    "POST::/api/defendants",
    "GET::/api/defendants/{id}",

    "POST::/api/parties",
    "GET::/api/parties/{id}",

    "POST::/api/docket/entries",
    "GET::/api/docket/entries/{id}",

    "POST::/api/filings",
    "GET::/api/filings/{id}",

    "POST::/api/motions",
    "GET::/api/motions/{id}",

    "POST::/api/charges",
    "GET::/api/charges/{id}",

    "POST::/api/evidence",
    "GET::/api/evidence/{id}",

    "POST::/api/orders",
    "GET::/api/orders/{id}",

    "POST::/api/opinions",
    "GET::/api/opinions/{id}",

    "POST::/api/deadlines",
    "GET::/api/deadlines/{id}",

    "POST::/api/calendar/events",

    "POST::/api/sentencing",
    "GET::/api/sentencing/{id}",

    "POST::/api/rules",
    "GET::/api/rules/{id}",

    "POST::/api/conflict-checks",

    "POST::/api/templates/orders",
    "GET::/api/templates/orders/{id}",

    "POST::/api/case-notes",
    "GET::/api/case-notes/{id}",

    "POST::/api/todos",
    "GET::/api/todos/{id}",

    "POST::/api/service-records"
  ]
}
```

**Step 2: Validate JSON**

```bash
node -e "require('./postman/portman-config.json'); console.log('Valid');"
```

Expected: `Valid`

**Step 3: Commit**

```bash
git add postman/portman-config.json
git commit -m "Add Portman config with auth, ordering, and variable capture"
```

---

### Task 3: Update the environment file with missing fields

**Files:**
- Modify: `postman/newman-env.json`

The smoke test register failed because `display_name` was missing. We also need to add captured variable placeholders.

**Step 1: Add `display_name` and captured ID placeholders to `newman-env.json`**

Add these entries to the `values` array:

```json
{ "key": "test_display_name", "value": "Newman Smoke Tester", "enabled": true }
```

**Step 2: Validate JSON**

```bash
node -e "require('./postman/newman-env.json'); console.log('Valid');"
```

**Step 3: Commit**

```bash
git add postman/newman-env.json
git commit -m "Add display_name to Newman env for register endpoint"
```

---

### Task 4: Create request body overwrites for seed entities

**Files:**
- Create: `postman/overwrites.json`
- Modify: `postman/portman-config.json` (add overwrites section)

Each create endpoint needs a realistic request body. Portman's `overwrites` section injects these.

**Step 1: Add the `overwrites` array to `portman-config.json`**

Add this as a top-level key in the config, after `orderOfOperations`:

```json
"overwrites": [
  {
    "openApiOperationId": "register",
    "overwriteRequestBody": [
      {
        "key": "username",
        "value": "{{test_username}}",
        "overwrite": true
      },
      {
        "key": "email",
        "value": "{{test_email}}",
        "overwrite": true
      },
      {
        "key": "password",
        "value": "{{test_password}}",
        "overwrite": true
      },
      {
        "key": "display_name",
        "value": "{{test_display_name}}",
        "overwrite": true
      }
    ]
  },
  {
    "openApiOperationId": "login",
    "overwriteRequestBody": [
      {
        "key": "email",
        "value": "{{test_email}}",
        "overwrite": true
      },
      {
        "key": "password",
        "value": "{{test_password}}",
        "overwrite": true
      }
    ]
  },
  {
    "openApiOperationId": "create_case",
    "overwriteRequestBody": [
      { "key": "title", "value": "United States v. Rodriguez", "overwrite": true },
      { "key": "crime_type", "value": "Felony", "overwrite": true },
      { "key": "district_code", "value": "district9", "overwrite": true },
      { "key": "description", "value": "Federal narcotics conspiracy charge under 21 USC 846", "overwrite": true },
      { "key": "priority", "value": "high", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_attorney",
    "overwriteRequestBody": [
      { "key": "bar_number", "value": "NY-2019-04521", "overwrite": true },
      { "key": "first_name", "value": "Maria", "overwrite": true },
      { "key": "last_name", "value": "Martinez", "overwrite": true },
      { "key": "email", "value": "mmartinez@martinez-pllc.com", "overwrite": true },
      { "key": "phone", "value": "212-555-0142", "overwrite": true },
      { "key": "firm_name", "value": "Martinez & Associates PLLC", "overwrite": true },
      { "key": "address.street1", "value": "500 Pearl Street", "overwrite": true },
      { "key": "address.city", "value": "New York", "overwrite": true },
      { "key": "address.state", "value": "NY", "overwrite": true },
      { "key": "address.zip_code", "value": "10007", "overwrite": true },
      { "key": "address.country", "value": "USA", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_judge",
    "overwriteRequestBody": [
      { "key": "name", "value": "Hon. Catherine R. Thornton", "overwrite": true },
      { "key": "title", "value": "District Judge", "overwrite": true },
      { "key": "district", "value": "district9", "overwrite": true },
      { "key": "courtroom", "value": "3B", "overwrite": true },
      { "key": "status", "value": "active", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_defendant",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "name", "value": "Carlos Eduardo Rodriguez", "overwrite": true },
      { "key": "custody_status", "value": "detained", "overwrite": true },
      { "key": "bail_type", "value": "cash", "overwrite": true },
      { "key": "bail_amount", "value": 250000, "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_party",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "party_type", "value": "defendant", "overwrite": true },
      { "key": "name", "value": "Carlos Eduardo Rodriguez", "overwrite": true },
      { "key": "entity_type", "value": "individual", "overwrite": true },
      { "key": "pro_se", "value": false, "overwrite": true },
      { "key": "service_method", "value": "electronic", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_docket_entry",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "entry_type", "value": "Motion", "overwrite": true },
      { "key": "description", "value": "Motion to Suppress Evidence obtained during warrantless search", "overwrite": true },
      { "key": "filed_by", "value": "Maria Martinez", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "submit_filing",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "document_type", "value": "Motion", "overwrite": true },
      { "key": "title", "value": "Motion to Suppress Evidence", "overwrite": true },
      { "key": "filed_by", "value": "Maria Martinez, Esq.", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_motion",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "motion_type", "value": "suppress", "overwrite": true },
      { "key": "filed_by", "value": "Defense", "overwrite": true },
      { "key": "description", "value": "Motion to suppress evidence obtained during warrantless vehicle search on I-95", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_charge",
    "overwriteRequestBody": [
      { "key": "defendant_id", "value": "{{created_defendant_id}}", "overwrite": true },
      { "key": "count_number", "value": 1, "overwrite": true },
      { "key": "statute", "value": "21 USC 846", "overwrite": true },
      { "key": "offense_description", "value": "Conspiracy to distribute controlled substances", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_evidence",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "description", "value": "Seized narcotics from vehicle trunk - 2.3kg cocaine", "overwrite": true },
      { "key": "evidence_type", "value": "physical", "overwrite": true },
      { "key": "location", "value": "DEA Evidence Vault, SDNY", "overwrite": true },
      { "key": "seized_by", "value": "DEA Special Agent Torres", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_order",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "judge_id", "value": "{{created_judge_id}}", "overwrite": true },
      { "key": "order_type", "value": "detention", "overwrite": true },
      { "key": "title", "value": "Order of Pretrial Detention", "overwrite": true },
      { "key": "content", "value": "Defendant is ordered detained pending trial per 18 USC 3142(e). Court finds defendant poses flight risk and danger to community.", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_opinion",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "judge_id", "value": "{{created_judge_id}}", "overwrite": true },
      { "key": "opinion_type", "value": "memorandum", "overwrite": true },
      { "key": "title", "value": "Memorandum Opinion on Motion to Suppress", "overwrite": true },
      { "key": "content", "value": "The Court denies defendant's motion to suppress. The automobile exception to the Fourth Amendment warrant requirement applies.", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_deadline",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "title", "value": "Pretrial motions filing deadline", "overwrite": true },
      { "key": "due_at", "value": "2026-04-15T17:00:00Z", "overwrite": true },
      { "key": "notes", "value": "Per scheduling order, all pretrial motions must be filed by this date", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "schedule_event",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "judge_id", "value": "{{created_judge_id}}", "overwrite": true },
      { "key": "event_type", "value": "status_conference", "overwrite": true },
      { "key": "scheduled_date", "value": "2026-03-20T10:00:00Z", "overwrite": true },
      { "key": "duration_minutes", "value": 30, "overwrite": true },
      { "key": "courtroom", "value": "3B", "overwrite": true },
      { "key": "description", "value": "Initial status conference - case scheduling and discovery deadlines", "overwrite": true },
      { "key": "participants", "value": ["Defense counsel", "AUSA", "Defendant"], "overwrite": true },
      { "key": "is_public", "value": true, "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_sentencing",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "defendant_id", "value": "{{created_defendant_id}}", "overwrite": true },
      { "key": "judge_id", "value": "{{created_judge_id}}", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_rule",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "rule_number", "value": "16", "overwrite": true },
      { "key": "title", "value": "Discovery and Inspection", "overwrite": true },
      { "key": "description", "value": "Federal Rule of Criminal Procedure 16 - Government and defendant disclosure obligations", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_conflict_check",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "attorney_id", "value": "{{created_attorney_id}}", "overwrite": true },
      { "key": "check_type", "value": "initial", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_template",
    "overwriteRequestBody": [
      { "key": "name", "value": "Standard Detention Order", "overwrite": true },
      { "key": "content", "value": "ORDERED that defendant {{defendant_name}} is detained pending trial pursuant to 18 USC 3142(e).", "overwrite": true },
      { "key": "template_type", "value": "detention", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_case_note",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "content", "value": "Defense counsel requests 30-day continuance for additional discovery review. Government does not oppose.", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_todo",
    "overwriteRequestBody": [
      { "key": "title", "value": "Review Brady material production from AUSA", "overwrite": true },
      { "key": "user_id", "value": "{{test_user_id}}", "overwrite": true },
      { "key": "description", "value": "Government disclosed 450 pages of discovery per Rule 16. Review for exculpatory material.", "overwrite": true }
    ]
  },
  {
    "openApiOperationId": "create_service_record",
    "overwriteRequestBody": [
      { "key": "case_id", "value": "{{created_case_id}}", "overwrite": true },
      { "key": "document_id", "value": "{{created_filing_id}}", "overwrite": true },
      { "key": "served_to", "value": "AUSA Jennifer Walsh", "overwrite": true },
      { "key": "service_method", "value": "electronic", "overwrite": true }
    ]
  }
]
```

**Step 2: Validate the config**

```bash
node -e "require('./postman/portman-config.json'); console.log('Valid');"
```

**Step 3: Commit**

```bash
git add postman/portman-config.json
git commit -m "Add request body overwrites with realistic federal court data"
```

---

### Task 5: Add X-Court-District header overwrites

**Files:**
- Modify: `postman/portman-config.json`

Many endpoints require the `X-Court-District` header. We need to add header overwrites for all operations that need it.

**Step 1: Add `operationPreRequestScripts` to inject the header**

Add to the config's top level, after `overwrites`:

```json
"operationPreRequestScripts": [
  {
    "openApiOperation": "*::/*",
    "scripts": [
      "pm.request.headers.add({ key: 'X-Court-District', value: pm.environment.get('district') || 'district9' });"
    ]
  }
]
```

**Step 2: Commit**

```bash
git add postman/portman-config.json
git commit -m "Add X-Court-District header injection for all requests"
```

---

### Task 6: Run Portman and generate the contract test collection

**Files:**
- Output: `postman/lexodus-contract.json`

**Step 1: Run Portman**

```bash
npx @apideck/portman \
  --local openapi-description.json \
  --portmanConfigFile postman/portman-config.json \
  --envFile postman/newman-env.json \
  --output postman/lexodus-contract.json \
  --includeTests
```

Expected: Collection generated with contract tests injected.

**Step 2: Validate output**

```bash
node -e "
const c = require('./postman/lexodus-contract.json');
let count = 0;
function walk(items) { items.forEach(i => i.item ? walk(i.item) : count++); }
walk(c.item || []);
console.log('Total requests:', count);
console.log('Auth:', c.auth ? c.auth.type : 'none');
"
```

Expected: ~396 requests with bearer auth.

**Step 3: Commit**

```bash
git add postman/lexodus-contract.json
git commit -m "Generate Portman contract test collection with schema validation"
```

---

### Task 7: Run Newman and analyze results

**Prerequisite:** Server running on localhost:8080 with database.

**Step 1: Verify server**

```bash
curl -s http://localhost:8080/health
```

**Step 2: Run Newman**

```bash
newman run postman/lexodus-contract.json \
  -e postman/newman-env.json \
  --reporters cli,json \
  --reporter-json-export postman/results.json
```

**Step 3: Analyze results**

```bash
node -e "
const r = require('./postman/results.json');
console.log('Total requests:', r.run.stats.requests.total);
console.log('Assertions total:', r.run.stats.assertions.total);
console.log('Assertions passed:', r.run.stats.assertions.total - r.run.stats.assertions.failed);
console.log('Assertions failed:', r.run.stats.assertions.failed);
console.log('Pass rate:', ((1 - r.run.stats.assertions.failed / r.run.stats.assertions.total) * 100).toFixed(1) + '%');

const fails = r.run.executions.filter(e => e.assertions && e.assertions.some(a => a.error));
if (fails.length) {
    console.log('\nFailing endpoints:');
    const seen = new Set();
    fails.forEach(f => {
        const key = (f.request ? f.request.method : '?') + ' ' + f.item.name;
        if (!seen.has(key)) {
            seen.add(key);
            const code = f.response ? f.response.code : 'none';
            const errors = f.assertions.filter(a => a.error).map(a => a.error.message).join('; ');
            console.log('  ' + code + ' ' + key + ' -- ' + errors);
        }
    });
}
"
```

**Step 4: Record pass rate and commit results summary**

If there are failures, document them as issues to fix in a follow-up task. The goal is iterative improvement toward 100%.

```bash
git add postman/lexodus-contract.json
git commit -m "Update contract test collection after initial Portman run"
```

---

### Task 8: Iterate on failures

This task is iterative. For each batch of failures from Task 7:

1. Identify the failure category (missing overwrite, wrong field name, schema mismatch, missing header)
2. Fix the `portman-config.json` (add/update overwrites, ordering, variables)
3. Regenerate: `npx @apideck/portman --local openapi-description.json --portmanConfigFile postman/portman-config.json --output postman/lexodus-contract.json --includeTests`
4. Re-run Newman
5. Repeat until pass rate reaches target

**Commit after each iteration:**

```bash
git add postman/portman-config.json postman/lexodus-contract.json
git commit -m "Fix contract test failures: [describe what was fixed]"
```

---

### Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Install Portman | `postman/package.json` |
| 2 | Base config (auth, ordering, variable capture) | `postman/portman-config.json` |
| 3 | Update env with display_name | `postman/newman-env.json` |
| 4 | Request body overwrites | `postman/portman-config.json` |
| 5 | X-Court-District header injection | `postman/portman-config.json` |
| 6 | Generate collection | `postman/lexodus-contract.json` |
| 7 | Run Newman, analyze results | `postman/results.json` |
| 8 | Iterate on failures | `postman/portman-config.json` |
