/**
 * Post-process a Portman-generated Postman collection to fix execution order:
 *
 * 1. Extract primary POST creates → "Setup" folder at beginning (after auth)
 * 2. Leave all GET/PUT/PATCH in their original folders
 * 3. Extract ALL DELETE requests → "Cleanup" folder at the end
 *
 * This fixes Portman's folder-based grouping which causes:
 * - DELETE operations running before downstream creates (FK violations)
 * - Collection variables not being set before they're referenced
 */
const fs = require("fs");

const collectionPath = process.argv[2] || "postman/lexodus-contract.json";
const collection = JSON.parse(fs.readFileSync(collectionPath, "utf8"));

// Primary create operations (order matters — dependencies go first)
const primaryCreates = [
  "POST::/api/v1/auth/register",
  "POST::/api/v1/auth/login",
  "POST::/api/cases",
  "POST::/api/attorneys",
  "POST::/api/judges",
  "POST::/api/defendants",
  "POST::/api/parties",
  "POST::/api/docket/entries",
  "POST::/api/filings",
  "POST::/api/motions",
  "POST::/api/charges",
  "POST::/api/evidence",
  "POST::/api/orders",
  "POST::/api/opinions",
  "POST::/api/deadlines",
  "POST::/api/calendar/events",
  "POST::/api/sentencing",
  "POST::/api/rules",
  "POST::/api/conflict-checks",
  "POST::/api/templates/orders",
  "POST::/api/case-notes",
  "POST::/api/todos",
  "POST::/api/service-records",
  "POST::/api/judges/assignments",
  "POST::/api/custody-transfers",
  "POST::/api/representations",
  "POST::/api/signatures",
  // Secondary creates (depend on primary entities)
  "POST::/api/cases/{id}/victims",
  "POST::/api/judges/{judge_id}/conflicts",
  "POST::/api/judges/{judge_id}/recusals",
  "POST::/api/opinions/{opinion_id}/drafts",
  "POST::/api/opinions/{opinion_id}/drafts/{draft_id}/comments",
  "POST::/api/cases/{id}/speedy-trial/start",
  "POST::/api/filings/upload/init",
  "POST::/api/features/override",
  "POST::/api/events",
];

function getOperationKey(item) {
  if (!item.request) return null;
  const method = item.request.method;
  const path = item.request.url && item.request.url.path
    ? "/" + item.request.url.path.join("/").replace(/:([^/]+)/g, "{$1}")
    : null;
  if (!path) return null;
  return method + "::" + path;
}

const setupItems = [];
const deleteItems = [];
let extractedSetup = 0;
let extractedDelete = 0;

function extractItems(items) {
  const kept = [];
  for (const item of items) {
    if (item.item) {
      item.item = extractItems(item.item);
      kept.push(item);
      continue;
    }

    const opKey = getOperationKey(item);

    // Extract DELETE requests
    if (item.request && item.request.method === "DELETE") {
      deleteItems.push(item);
      extractedDelete++;
      continue;
    }

    // Extract primary creates
    if (opKey && primaryCreates.includes(opKey)) {
      setupItems.push({ item, order: primaryCreates.indexOf(opKey) });
      extractedSetup++;
      continue;
    }

    kept.push(item);
  }
  return kept;
}

// Extract items from all folders
collection.item = extractItems(collection.item || []);

// Remove empty folders
collection.item = collection.item.filter(
  (f) => !f.item || f.item.length > 0
);

// Sort setup items by dependency order
setupItems.sort((a, b) => a.order - b.order);

// Build final collection: Setup → original folders → Cleanup
const finalItems = [];

// Setup folder with ordered creates
if (setupItems.length > 0) {
  finalItems.push({
    name: "Setup - Entity Creates",
    description: {
      content: "Primary create operations in dependency order",
      type: "text/plain",
    },
    item: setupItems.map((s) => s.item),
  });
}

// Original folders (with creates and deletes removed)
finalItems.push(...collection.item);

// Order DELETEs: children before parents to avoid FK violations
const deleteOrder = [
  "/api/opinions/", "/api/sentencing/", "/api/orders/",
  "/api/conflict-checks/", "/api/representations/",
  "/api/service-records/", "/api/custody-transfers/",
  "/api/assignments/", "/api/judges/conflicts/",
  "/api/speedy-trial/", "/api/docket/", "/api/deadlines/",
  "/api/calendar/", "/api/case-notes/", "/api/evidence/",
  "/api/motions/", "/api/charges/", "/api/defendants/",
  "/api/parties/", "/api/templates/", "/api/rules/",
  "/api/todos/", "/api/features/", "/api/config/",
  "/api/cases/", "/api/attorneys/", "/api/judges/",
  "/api/",
];

function getDeletePriority(item) {
  const path = item.request && item.request.url && item.request.url.path
    ? "/" + item.request.url.path.join("/")
    : "";
  for (let i = 0; i < deleteOrder.length; i++) {
    if (path.includes(deleteOrder[i])) return i;
  }
  return deleteOrder.length;
}

deleteItems.sort((a, b) => getDeletePriority(a) - getDeletePriority(b));

// Cleanup folder with ordered deletes (children before parents)
if (deleteItems.length > 0) {
  finalItems.push({
    name: "Cleanup - Deletes",
    description: {
      content: "DELETE operations at end, children before parents",
      type: "text/plain",
    },
    item: deleteItems,
  });
}

collection.item = finalItems;

fs.writeFileSync(collectionPath, JSON.stringify(collection, null, 2));
console.log(
  "Extracted " + extractedSetup + " creates to Setup folder, " +
  extractedDelete + " deletes to Cleanup folder"
);
