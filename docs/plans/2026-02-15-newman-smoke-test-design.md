# Newman Auth-Aware Smoke Test Design

## Goal

Verify that all 396 endpoints defined in `openapi-description.json` exist in the Axum router and their handlers execute. This catches mismatches between the OpenAPI spec and the actual server.

## Approach

Use `openapi-to-postmanv2` to convert the OpenAPI spec to a Postman collection, post-process it with a Node script to inject auth and test assertions, then run with Newman.

## Pass Criteria

A request **passes** if the response status is anything other than 404 (route not found) or 405 (method not allowed). Status codes like 400, 422, and 500 are acceptable — they prove the route exists and the handler runs.

## Auth Flow

1. Collection-level pre-request script registers a test user via `POST /api/v1/auth/register` (username, email, password)
2. On conflict (user exists), falls back to `POST /api/v1/auth/login`
3. Stores `access_token` from `AuthResponse` in a collection variable
4. Collection-level `Authorization: Bearer {{access_token}}` header is set on all requests

## Environment Variables (`newman-env.json`)

- `base_url`: `http://localhost:8080`
- `test_email`, `test_password`, `test_username`: credentials for the smoke test user
- Dummy UUIDs for path parameters (`case_id`, `user_id`, `document_id`, etc.)

## Per-Request Test Script

```js
pm.test("Route is defined (not 404/405)", function () {
    pm.expect(pm.response.code).to.not.be.oneOf([404, 405]);
});
```

## File Layout

```
postman/
  convert.js          # Converts OpenAPI -> Postman collection, injects scripts
  newman-env.json     # Environment variables
  lexodus-smoke.json  # Generated collection (output of convert.js)
  results.json        # Newman run output
```

## Run Commands

```bash
node postman/convert.js
newman run postman/lexodus-smoke.json \
  -e postman/newman-env.json \
  --bail failure \
  --reporters cli,json \
  --reporter-json-export postman/results.json
```

## Dependencies

- `openapi-to-postmanv2` (npm, conversion)
- `newman` (already installed globally)
- Running server on localhost:8080 with database
