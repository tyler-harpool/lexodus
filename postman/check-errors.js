const r = JSON.parse(require("fs").readFileSync("postman/results.json", "utf8"));
for (const ex of r.run.executions) {
  const req = ex.request;
  if (!req || !req.url || !req.url.path) continue;
  const p = req.url.path.join("/");
  const code = ex.response ? ex.response.code : 0;
  const targets = [
    "api/admin/court-memberships",
    "api/service-records",
  ];
  const isPUT = req.method === "PUT" && p === "api/admin/court-memberships";
  const isPOSTsr = req.method === "POST" && p === "api/service-records";
  const isConflictClear = p.includes("conflict-checks") && p.includes("clear");
  const isDocReplace = p.includes("documents") && p.includes("replace");
  const isDocSeal = p.includes("documents") && p.includes("seal");
  const isBarNumber = p.includes("bar-number");
  const isUserTier = p.includes("users") && p.includes("tier");

  if (isPUT || isPOSTsr || isConflictClear || isDocReplace || isDocSeal || isBarNumber || isUserTier) {
    console.log(code, req.method, "/" + p);
    if (ex.response && ex.response.stream) console.log("  Resp:", Buffer.from(ex.response.stream.data).toString().substring(0, 250));
    if (req.body && req.body.raw) console.log("  Body:", req.body.raw.substring(0, 250));
    console.log();
  }
}
