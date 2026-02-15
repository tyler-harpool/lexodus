const r = JSON.parse(require("fs").readFileSync("postman/results.json","utf8"));
let failCount = 0;
for (const ex of r.run.executions) {
  for (const a of (ex.assertions || [])) {
    if (a.error) {
      const req = ex.request;
      const method = req ? req.method : "?";
      const url = req && req.url ? "/" + req.url.path.join("/") : "?";
      const code = ex.response ? ex.response.code : "?";
      console.log(code + " " + method + " " + url);
      console.log("  ASSERT: " + a.assertion);
      console.log("  ERROR: " + a.error.message.substring(0, 200));
      console.log();
      failCount++;
    }
  }
}
console.log("Total failing assertions:", failCount);
