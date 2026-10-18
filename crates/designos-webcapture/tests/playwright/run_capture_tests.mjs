import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import assert from "node:assert/strict";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const crateRoot = path.resolve(__dirname, "../..");
const captureJsPath = path.join(crateRoot, "js", "capture.js");
const cardHtmlPath = path.join(__dirname, "fixtures", "card.html");
const cardCaptureOutPath = path.join(__dirname, "fixtures", "card.capture.json");
const truncatedCaptureOutPath = path.join(__dirname, "fixtures", "card.truncated.capture.json");

const captureScript = fs.readFileSync(captureJsPath, "utf8");

async function launchBrowser() {
  try {
    return await chromium.launch({ headless: true });
  } catch (_err) {
    return await chromium.launch({ channel: "chrome", headless: true });
  }
}

function indexById(root, map = new Map()) {
  if (!root) return map;
  if (root.id) {
    map.set(root.id, root);
  }
  if (Array.isArray(root.children)) {
    for (const child of root.children) {
      indexById(child, map);
    }
  }
  return map;
}

async function main() {
  const writeFixtures = process.argv.includes("--write-fixtures");
  const browser = await launchBrowser();
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    const page = await context.newPage();
    await page.goto(pathToFileURL(cardHtmlPath).href, { waitUntil: "load" });
    await page.addScriptTag({ content: captureScript });

    // 1. Standard full capture
    const capture = await page.evaluate(async () => {
      return await window.__designosCapture();
    });

    assert.equal(capture.schema, "designos-webcapture");
    assert.equal(capture.version, 1);
    assert.equal(capture.viewport.width, 1280);
    assert.equal(capture.viewport.height, 800);
    assert.equal(capture.truncated, false);
    assert.ok(capture.root, "capture.root must exist");

    const byId = indexById(capture.root);

    // 1a. boxes_styles_runs_images
    const card = byId.get("pricing-card");
    assert.ok(card, "pricing-card element must be captured");
    assert.ok(card.box.width >= 320, `card box width must be >= 320, got ${card.box.width}`);
    assert.ok(card.box.height > 50, `card box height must be > 50, got ${card.box.height}`);
    assert.equal(card.styles["display"], "flex");
    assert.equal(card.styles["flex-direction"], "column");
    assert.equal(card.styles["border-top-left-radius"], "12px");
    assert.ok(card.pseudo && card.pseudo.before, "card ::before pseudo box must be captured");
    assert.equal(card.pseudo.before.content, "PRO");

    const desc = byId.get("desc");
    assert.ok(desc, "desc element must be captured");
    assert.ok(Array.isArray(desc.text_runs) && desc.text_runs.length >= 1, "desc should have text_runs");
    assert.ok(
      desc.text_runs[0].lines && desc.text_runs[0].lines.length >= 2,
      "multi-line paragraph should produce >= 2 wrapped line rects"
    );
    for (const line of desc.text_runs[0].lines) {
      assert.ok(line.width > 0 && line.height > 0, "line rect must have positive width/height");
    }

    const avatarImg = byId.get("avatar-img");
    assert.ok(avatarImg && avatarImg.image, "avatar-img must have image ref");
    assert.ok(!avatarImg.image.blocked, "avatar-img must not be blocked");
    assert.ok(
      typeof avatarImg.image.data_url === "string" && avatarImg.image.data_url.startsWith("data:image/"),
      "avatar-img must have inline data_url"
    );

    const badgeSvg = byId.get("badge-svg");
    assert.ok(badgeSvg && badgeSvg.image, "badge-svg must have image ref");
    assert.ok(
      (typeof badgeSvg.image.svg === "string" && badgeSvg.image.svg.includes("<svg")) ||
        (typeof badgeSvg.image.data_url === "string" && badgeSvg.image.data_url.startsWith("data:image/svg")),
      "badge-svg must have inline svg or SVG data_url"
    );

    const sparkCanvas = byId.get("spark-canvas");
    assert.ok(sparkCanvas && sparkCanvas.image, "spark-canvas must have image ref");
    assert.ok(
      typeof sparkCanvas.image.data_url === "string" && sparkCanvas.image.data_url.startsWith("data:image/png"),
      "spark-canvas must have PNG data_url"
    );

    const bgBanner = byId.get("bg-banner");
    assert.ok(bgBanner && (bgBanner.image || (bgBanner.images && bgBanner.images.length > 0)), "bg-banner with CSS background-image must have image ref");

    assert.equal(byId.has("secret-hidden"), false, "display:none element must be skipped when include_hidden is false");

    // 2. css_var_refs
    assert.ok(capture.root.css_vars_declared, "html/:root must record css_vars_declared");
    assert.equal(capture.root.css_vars_declared["--brand"], "#0d99ff");

    const ctaBtn = byId.get("cta-btn");
    assert.ok(ctaBtn && ctaBtn.css_var_refs, "cta-btn must record css_var_refs");
    assert.deepEqual(ctaBtn.css_var_refs["background-color"], ["--brand"]);

    // 3. component_hint
    assert.equal(card.component_hint, "PricingCard");

    // 4. cross_origin_image_blocked_recorded
    const blockedImg = byId.get("blocked-img");
    assert.ok(blockedImg && blockedImg.image, "blocked-img must have image ref");
    assert.equal(blockedImg.image.blocked, true);
    assert.ok(blockedImg.image.src.includes("unreachable-cross-origin.png"));

    // 5. truncation
    const truncated = await page.evaluate(async () => {
      return await window.__designosCapture({ max_elements: 4 });
    });
    assert.equal(truncated.stats.elements, 4);
    assert.equal(truncated.truncated, true);
    assert.ok(truncated.stats.skipped_elements > 0);

    if (writeFixtures) {
      fs.writeFileSync(cardCaptureOutPath, JSON.stringify(capture, null, 2) + "\n", "utf8");
      fs.writeFileSync(truncatedCaptureOutPath, JSON.stringify(truncated, null, 2) + "\n", "utf8");
    }

    // 6. Performance budget: 5,000-element page completes in <= 1.5s
    const perfPage = await context.newPage();
    await perfPage.setContent(`<!DOCTYPE html><html><body><div id="root"></div><script>
      const root = document.getElementById("root");
      const frag = document.createDocumentFragment();
      for (let i = 0; i < 4995; i++) {
        const d = document.createElement("div");
        d.style.display = "inline-block";
        d.style.width = "16px";
        d.style.height = "16px";
        d.textContent = "x";
        frag.appendChild(d);
      }
      root.appendChild(frag);
    </script></body></html>`);
    await perfPage.addScriptTag({ content: captureScript });
    const perfCapture = await perfPage.evaluate(async () => {
      return await window.__designosCapture({ include_images: false });
    });
    assert.ok(perfCapture.stats.elements >= 4995, `expected >= 4995 elements, got ${perfCapture.stats.elements}`);
    assert.ok(
      perfCapture.stats.elapsed_ms <= 1500,
      `5,000-element capture took ${perfCapture.stats.elapsed_ms}ms (budget <= 1500ms)`
    );

    console.log(
      JSON.stringify({
        ok: true,
        elements: capture.stats.elements,
        perf_5000_ms: perfCapture.stats.elapsed_ms,
      })
    );
  } finally {
    await browser.close();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
