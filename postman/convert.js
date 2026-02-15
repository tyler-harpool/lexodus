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
