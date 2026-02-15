const c = require("./lexodus-contract.json");
const seen = {};
function walk(items) {
  for (const item of items) {
    if (item.item) { walk(item.item); continue; }
    if (item.request && item.request.url && item.request.url.variable) {
      for (const v of item.request.url.variable) {
        if (v.value && v.value.indexOf("{{") === -1) {
          var sig = v.key + "=" + v.value;
          if (!seen[sig]) { seen[sig] = true; console.log(":" + v.key + " = " + JSON.stringify(v.value)); }
        }
      }
    }
  }
}
walk(c.item || []);
