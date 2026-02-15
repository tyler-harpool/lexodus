# Newman Auth-Aware Smoke Test Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Convert the OpenAPI spec to a Postman collection and run Newman to verify all 396 endpoints are routable and their handlers execute.

**Architecture:** A Node.js script uses `openapi-to-postmanv2` to convert `openapi-description.json` into a Postman collection, then post-processes it to inject auth pre-request scripts, per-request test assertions, and an `Authorization` header. Newman runs the collection against the live server.

**Tech Stack:** Node.js, openapi-to-postmanv2 (npm), Newman CLI (already installed)

---

### Task 1: Initialize postman directory and install dependencies

**Files:**
- Create: `postman/package.json`

**Step 1: Create the postman directory and init npm**

```bash
mkdir -p postman
cd postman
npm init -y
npm install openapi-to-postmanv2
```

**Step 2: Verify installation**

Run: `node -e "require('openapi-to-postmanv2'); console.log('OK');"` from `postman/`
Expected: `OK`

**Step 3: Commit**

```bash
git add postman/package.json postman/package-lock.json
git commit -m "Add postman directory with openapi-to-postmanv2 dependency"
```

---

### Task 2: Create the Newman environment file

**Files:**
- Create: `postman/newman-env.json`

**Step 1: Write the environment file**

This file provides `base_url`, test credentials, and realistic federal court values for all 48 path parameters. Uses test districts `district9` and `district12` per project conventions. String-type params use realistic federal court terminology.

```json
{
  "id": "lexodus-smoke-env",
  "name": "Lexodus Smoke Test",
  "values": [
    { "key": "base_url", "value": "http://localhost:8080", "enabled": true },
    { "key": "test_email", "value": "newman-smoke@test.com", "enabled": true },
    { "key": "test_password", "value": "TestPassword123!", "enabled": true },
    { "key": "test_username", "value": "newman_smoke_user", "enabled": true },

    { "key": "case_id", "value": "a1b2c3d4-e5f6-7890-abcd-ef1234567890", "enabled": true },
    { "key": "user_id", "value": "b2c3d4e5-f6a7-8901-bcde-f12345678901", "enabled": true },
    { "key": "document_id", "value": "c3d4e5f6-a7b8-9012-cdef-123456789012", "enabled": true },
    { "key": "attorney_id", "value": "d4e5f6a7-b8c9-0123-defa-234567890123", "enabled": true },
    { "key": "defendant_id", "value": "e5f6a7b8-c9d0-1234-efab-345678901234", "enabled": true },
    { "key": "judge_id", "value": "f6a7b8c9-d0e1-2345-fabc-456789012345", "enabled": true },
    { "key": "party_id", "value": "a7b8c9d0-e1f2-3456-abcd-567890123456", "enabled": true },
    { "key": "event_id", "value": "b8c9d0e1-f2a3-4567-bcde-678901234567", "enabled": true },
    { "key": "filing_id", "value": "c9d0e1f2-a3b4-5678-cdef-789012345678", "enabled": true },
    { "key": "order_id", "value": "d0e1f2a3-b4c5-6789-defa-890123456789", "enabled": true },
    { "key": "opinion_id", "value": "e1f2a3b4-c5d6-7890-efab-901234567890", "enabled": true },
    { "key": "evidence_id", "value": "f2a3b4c5-d6e7-8901-fabc-012345678901", "enabled": true },
    { "key": "deadline_id", "value": "a3b4c5d6-e7f8-9012-abcd-123456789012", "enabled": true },
    { "key": "docket_entry_id", "value": "b4c5d6e7-f8a9-0123-bcde-234567890123", "enabled": true },
    { "key": "reminder_id", "value": "c5d6e7f8-a9b0-1234-cdef-345678901234", "enabled": true },
    { "key": "attachment_id", "value": "d6e7f8a9-b0c1-2345-defa-456789012345", "enabled": true },
    { "key": "template_id", "value": "e7f8a9b0-c1d2-3456-efab-567890123456", "enabled": true },
    { "key": "conflict_id", "value": "f8a9b0c1-d2e3-4567-fabc-678901234567", "enabled": true },
    { "key": "victim_id", "value": "a9b0c1d2-e3f4-5678-abcd-789012345678", "enabled": true },
    { "key": "extension_id", "value": "b0c1d2e3-f4a5-6789-bcde-890123456789", "enabled": true },
    { "key": "draft_id", "value": "c1d2e3f4-a5b6-7890-cdef-901234567890", "enabled": true },
    { "key": "comment_id", "value": "d2e3f4a5-b6c7-8901-defa-012345678901", "enabled": true },
    { "key": "recusal_id", "value": "e3f4a5b6-c7d8-9012-efab-123456789012", "enabled": true },
    { "key": "product_id", "value": "f4a5b6c7-d8e9-0123-fabc-234567890123", "enabled": true },
    { "key": "entry_id", "value": "a5b6c7d8-e9f0-1234-abcd-345678901234", "enabled": true },
    { "key": "court_id", "value": "district9", "enabled": true },
    { "key": "id", "value": "b6c7d8e9-f0a1-2345-bcde-456789012345", "enabled": true },

    { "key": "bar_number", "value": "NY-2019-04521", "enabled": true },
    { "key": "case_number", "value": "1:26-cr-00042", "enabled": true },
    { "key": "district", "value": "district9", "enabled": true },
    { "key": "court", "value": "district9", "enabled": true },
    { "key": "cja_district", "value": "district12", "enabled": true },
    { "key": "jurisdiction", "value": "federal", "enabled": true },
    { "key": "courtroom", "value": "3B", "enabled": true },
    { "key": "status", "value": "active", "enabled": true },
    { "key": "state", "value": "open", "enabled": true },
    { "key": "area", "value": "criminal", "enabled": true },
    { "key": "category", "value": "motion-to-suppress", "enabled": true },
    { "key": "deadline_type", "value": "pretrial-motion", "enabled": true },
    { "key": "entry_type", "value": "minute-entry", "enabled": true },
    { "key": "offense_type", "value": "felony", "enabled": true },
    { "key": "feature_path", "value": "case-management", "enabled": true },
    { "key": "trigger", "value": "nef-received", "enabled": true },
    { "key": "format", "value": "json", "enabled": true },
    { "key": "text", "value": "Brady material request", "enabled": true },
    { "key": "firm_name", "value": "Martinez-and-Associates-PLLC", "enabled": true },
    { "key": "party_name", "value": "United-States-v-Rodriguez", "enabled": true },
    { "key": "recipient", "value": "clerk@district9.uscourts.gov", "enabled": true }
  ]
}
```

**Step 2: Validate JSON**

Run: `node -e "require('./postman/newman-env.json'); console.log('Valid');"` from project root
Expected: `Valid`

**Step 3: Commit**

```bash
git add postman/newman-env.json
git commit -m "Add Newman environment file with realistic federal court test data"
```

---

### Task 3: Create the OpenAPI-to-Postman conversion script

**Files:**
- Create: `postman/convert.js`

This is the core script. It:
1. Reads `openapi-description.json`
2. Converts it to a Postman collection via `openapi-to-postmanv2`
3. Post-processes the collection to:
   - Set `{{base_url}}` as the collection base URL
   - Replace OpenAPI path param syntax `{param}` with Postman syntax `{{param}}`
   - Add collection-level `Authorization: Bearer {{access_token}}` header
   - Add a collection-level pre-request script that registers/logs in a test user
   - Add a per-request test script asserting status is not 404/405
4. Writes `postman/lexodus-smoke.json`

**Step 1: Write `postman/convert.js`**

```js
const fs = require('fs');
const path = require('path');
const Converter = require('openapi-to-postmanv2');

const OPENAPI_PATH = path.join(__dirname, '..', 'openapi-description.json');
const OUTPUT_PATH = path.join(__dirname, 'lexodus-smoke.json');

const openapiSpec = fs.readFileSync(OPENAPI_PATH, 'utf8');

// Pre-request script: register or login, store access_token
const AUTH_PRE_REQUEST = `
// Only run auth once per collection run
if (pm.collectionVariables.get("access_token")) {
    return;
}

const baseUrl = pm.environment.get("base_url");
const email = pm.environment.get("test_email");
const password = pm.environment.get("test_password");
const username = pm.environment.get("test_username");

// Try register first
pm.sendRequest({
    url: baseUrl + "/api/v1/auth/register",
    method: "POST",
    header: { "Content-Type": "application/json" },
    body: {
        mode: "raw",
        raw: JSON.stringify({ username: username, email: email, password: password })
    }
}, function (err, res) {
    if (!err && res.code >= 200 && res.code < 300) {
        const body = res.json();
        pm.collectionVariables.set("access_token", body.access_token);
        console.log("Registered and got token");
        return;
    }
    // Register failed (user exists?), try login
    pm.sendRequest({
        url: baseUrl + "/api/v1/auth/login",
        method: "POST",
        header: { "Content-Type": "application/json" },
        body: {
            mode: "raw",
            raw: JSON.stringify({ email: email, password: password })
        }
    }, function (err2, res2) {
        if (!err2 && res2.code >= 200 && res2.code < 300) {
            const body2 = res2.json();
            pm.collectionVariables.set("access_token", body2.access_token);
            console.log("Logged in and got token");
        } else {
            console.error("Auth failed:", err2 || res2.code, res2 ? res2.text() : "");
        }
    });
});
`.trim();

// Per-request test: route must exist
const ROUTE_TEST = `
pm.test("Route is defined (not 404/405)", function () {
    pm.expect(pm.response.code).to.not.be.oneOf([404, 405]);
});
`.trim();

Converter.convert(
    { type: 'string', data: openapiSpec },
    {},
    function (err, result) {
        if (err) {
            console.error('Conversion error:', err);
            process.exit(1);
        }

        if (!result.result) {
            console.error('Conversion failed:', result.reason);
            process.exit(1);
        }

        const collection = result.output[0].data;

        // 1. Add collection-level auth header
        if (!collection.auth) {
            collection.auth = {
                type: 'bearer',
                bearer: [{ key: 'token', value: '{{access_token}}', type: 'string' }]
            };
        }

        // 2. Add collection-level pre-request script (auth)
        if (!collection.event) collection.event = [];
        collection.event.push({
            listen: 'prerequest',
            script: {
                type: 'text/javascript',
                exec: AUTH_PRE_REQUEST.split('\n')
            }
        });

        // 3. Walk all items and:
        //    - Replace {param} with {{param}} in URLs
        //    - Set host to {{base_url}}
        //    - Add per-request test script
        function processItems(items) {
            for (const item of items) {
                if (item.item) {
                    processItems(item.item);
                    continue;
                }

                if (!item.request) continue;

                const req = item.request;

                // Fix URL: replace path params {x} -> {{x}}
                if (req.url) {
                    // Handle URL as object (Postman SDK format)
                    if (typeof req.url === 'object') {
                        // Set host to base_url variable
                        req.url.host = ['{{base_url}}'];
                        req.url.protocol = undefined;

                        // Fix path segments: :param -> {{param}}
                        if (req.url.path) {
                            req.url.path = req.url.path.map(function (seg) {
                                if (seg.startsWith(':')) {
                                    return '{{' + seg.slice(1) + '}}';
                                }
                                return seg.replace(/\{([^}]+)\}/g, '{{$1}}');
                            });
                        }

                        // Fix raw URL too
                        if (req.url.raw) {
                            req.url.raw = req.url.raw
                                .replace(/^https?:\/\/[^/]+/, '{{base_url}}')
                                .replace(/\{([^}]+)\}/g, '{{$1}}')
                                .replace(/:([a-zA-Z_][a-zA-Z0-9_]*)/g, '{{$1}}');
                        }
                    } else if (typeof req.url === 'string') {
                        req.url = req.url
                            .replace(/^https?:\/\/[^/]+/, '{{base_url}}')
                            .replace(/\{([^}]+)\}/g, '{{$1}}');
                    }
                }

                // Add per-request test
                if (!item.event) item.event = [];
                item.event.push({
                    listen: 'test',
                    script: {
                        type: 'text/javascript',
                        exec: ROUTE_TEST.split('\n')
                    }
                });
            }
        }

        processItems(collection.item || []);

        // 4. Add collection variable placeholder for access_token
        if (!collection.variable) collection.variable = [];
        collection.variable.push({
            key: 'access_token',
            value: '',
            type: 'string'
        });

        // Write output
        fs.writeFileSync(OUTPUT_PATH, JSON.stringify(collection, null, 2));

        // Count requests
        let count = 0;
        function countItems(items) {
            for (const item of items) {
                if (item.item) countItems(item.item);
                else if (item.request) count++;
            }
        }
        countItems(collection.item || []);

        console.log('Collection written to:', OUTPUT_PATH);
        console.log('Total requests:', count);
    }
);
```

**Step 2: Run the conversion**

Run from project root:
```bash
node postman/convert.js
```

Expected output:
```
Collection written to: /path/to/postman/lexodus-smoke.json
Total requests: ~396
```

**Step 3: Validate the generated collection is valid JSON**

Run: `node -e "const c = require('./postman/lexodus-smoke.json'); console.log('Items:', c.item.length, 'Auth:', c.auth.type);"`
Expected: `Items: <number> Auth: bearer`

**Step 4: Commit**

```bash
git add postman/convert.js
git commit -m "Add OpenAPI to Postman collection conversion script"
```

---

### Task 4: Run Newman and verify results

**Prerequisite:** Server must be running on localhost:8080 with database available.

**Step 1: Start the server** (if not already running)

```bash
dx serve
```

Wait until server is accepting connections (check `curl http://localhost:8080/health`).

**Step 2: Run Newman**

```bash
newman run postman/lexodus-smoke.json \
  -e postman/newman-env.json \
  --reporters cli,json \
  --reporter-json-export postman/results.json
```

**Step 3: Analyze results**

Check the Newman CLI output for:
- Total requests executed
- Total assertions passed/failed
- Any 404/405 failures (these indicate spec-vs-router mismatches)

For a quick summary of failures:
```bash
node -e "
const r = require('./postman/results.json');
const fails = r.run.executions.filter(e =>
    e.assertions && e.assertions.some(a => a.error)
);
console.log('Total:', r.run.stats.requests.total);
console.log('Passed:', r.run.stats.assertions.total - r.run.stats.assertions.failed);
console.log('Failed:', r.run.stats.assertions.failed);
if (fails.length) {
    console.log('\nFailing endpoints:');
    fails.forEach(f => {
        const name = f.item.name;
        const code = f.response ? f.response.code : 'no response';
        console.log('  ' + code + ' ' + name);
    });
}
"
```

**Step 4: Add results.json to .gitignore and commit**

```bash
echo 'postman/results.json' >> .gitignore
echo 'postman/node_modules/' >> .gitignore
git add .gitignore postman/lexodus-smoke.json
git commit -m "Add generated Postman smoke test collection"
```

---

### Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Init postman dir, install deps | `postman/package.json` |
| 2 | Create Newman env file | `postman/newman-env.json` |
| 3 | Write conversion script | `postman/convert.js` |
| 4 | Run Newman, analyze results | `postman/results.json` (gitignored) |
