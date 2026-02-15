const r = require("./results.json");
r.run.executions.forEach(e => {
  const method = e.request && e.request.method ? e.request.method : "?";
  const code = e.response ? e.response.code : "no-resp";
  if (code < 400 && code !== "no-resp") return;
  const name = e.item ? e.item.name : "?";
  const path = e.request && e.request.url && e.request.url.path
    ? "/" + e.request.url.path.join("/") : "?";
  const body = e.response && e.response.body ? e.response.body.substring(0, 300) : "(empty)";
  const reqBody = e.request && e.request.body && e.request.body.raw ? e.request.body.raw.substring(0, 300) : "(none)";
  console.log("---");
  console.log(code + " " + method + " " + path);
  console.log("  Req: " + reqBody);
  console.log("  Res: " + body);
});
