import { cpSync, existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const frontendPublic = resolve(dirname(fileURLToPath(import.meta.url)), "../public/benchmark-data");
const processed = join(root, "results/processed/dashboard.json");
const plotsDir = join(root, "results/plots");

mkdirSync(join(frontendPublic, "plots"), { recursive: true });

const dashboardOut = join(frontendPublic, "dashboard.json");
if (existsSync(processed)) {
  cpSync(processed, dashboardOut);
  console.log("Copied dashboard.json");
} else {
  writeFileSync(
    dashboardOut,
    JSON.stringify({ generatedAt: null, cold: [], load: [], memory: [], plots: [] }, null, 2)
  );
  console.warn("No dashboard.json in results — wrote empty stub (run: make analyze)");
}

if (existsSync(plotsDir)) {
  for (const name of readdirSync(plotsDir)) {
    if (name.endsWith(".png")) {
      cpSync(join(plotsDir, name), join(frontendPublic, "plots", name), { force: true });
    }
  }
  console.log("Copied plot PNGs");
}
