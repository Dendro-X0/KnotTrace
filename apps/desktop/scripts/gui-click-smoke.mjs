/**
 * GUI click-smoke against a running KnotTrace Tauri webview.
 *
 * Prerequisites:
 *   1. RC.EXE on PATH (Windows SDK bin/.../x64)
 *   2. Start app with CDP:
 *        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9223 --remote-allow-origins=*"
 *        npm run tauri dev
 *   3. npm run smoke:gui
 */
import { chromium } from "playwright-core";

const CDP = process.env.KNOTTRACE_CDP || "http://127.0.0.1:9223";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function visible(page, text) {
  return page
    .getByText(text, { exact: false })
    .first()
    .isVisible()
    .catch(() => false);
}

async function palette(page, query) {
  await page.keyboard.press("Control+K");
  await sleep(350);
  await page.keyboard.type(query, { delay: 15 });
  await sleep(250);
  await page.keyboard.press("Enter");
}

const browser = await chromium.connectOverCDP(CDP);
const context = browser.contexts()[0];
if (!context) throw new Error("No CDP browser context — is tauri running with port 9223?");
const page =
  context.pages().find((p) => p.url().includes("localhost:1420")) ||
  context.pages()[0];
if (!page) throw new Error("No KnotTrace page attached to CDP");

console.log("PAGE", page.url(), await page.title());
await page.waitForSelector("body", { timeout: 15000 });
await sleep(2000);

const overviewMarkers = [
  "Path break",
  "Integrity",
  "Egress timeline",
  "Explain locally",
  "Explain with AI",
  "Diagnostic export",
];
for (const marker of overviewMarkers) {
  console.log("OVERVIEW", marker, await visible(page, marker));
}

await palette(page, "Run health check");
console.log("TRIGGERED health check");
await sleep(8000);

const explain = page.getByRole("button", { name: /^Explain$/ });
if (await explain.count()) {
  await explain.first().click();
  console.log("CLICKED Explain locally");
  await sleep(1500);
}

const copyJson = page.getByRole("button", { name: /Copy JSON/i });
if (await copyJson.count()) {
  await copyJson.first().click();
  console.log("CLICKED Copy JSON");
  await sleep(500);
}

await palette(page, "Network");
await sleep(1500);
for (const marker of ["Path break", "Integrity", "Egress timeline", "Tunnel compare"]) {
  console.log("NETWORK", marker, await visible(page, marker));
}

await browser.close();
console.log("SMOKE_DONE");
