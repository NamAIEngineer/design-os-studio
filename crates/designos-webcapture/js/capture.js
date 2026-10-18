/**
 * design:os studio — Self-contained ES2020 Web Capture Script (T8.25 / Issue #276)
 *
 * Exposes `window.__designosCapture(options) -> Promise<Capture>`.
 * Uses only standard DOM APIs, never mutates the page, and captures references to
 * native globals at load time so pages that override `Array.prototype` or `window.fetch`
 * cannot break capture (T8.25 §3.1, §8.6).
 */
(function () {
  "use strict";

  const g = typeof window !== "undefined" ? window : globalThis;
  const nativeFetch =
    typeof g.fetch === "function" ? Function.prototype.bind.call(g.fetch, g) : null;
  const nativeSlice = Array.prototype.slice;
  const nativeFrom = Array.from;
  const nativeKeys = Object.keys;
  const nativeNow =
    typeof performance !== "undefined" && typeof performance.now === "function"
      ? () => performance.now()
      : () => Date.now();

  const SKIPPED_TAGS = new Set(["script", "style", "template", "noscript", "head"]);

  // Exact computed style subset specified in T8.25 §3.4.
  const CAPTURED_STYLE_PROPS = [
    "display",
    "position",
    "top",
    "right",
    "bottom",
    "left",
    "z-index",
    "flex-direction",
    "flex-wrap",
    "justify-content",
    "align-items",
    "align-content",
    "align-self",
    "flex-grow",
    "flex-shrink",
    "flex-basis",
    "gap",
    "row-gap",
    "column-gap",
    "grid-template-columns",
    "grid-template-rows",
    "grid-column",
    "grid-row",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
    "border-top-style",
    "border-right-style",
    "border-bottom-style",
    "border-left-style",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "border-top-left-radius",
    "border-top-right-radius",
    "border-bottom-right-radius",
    "border-bottom-left-radius",
    "background-color",
    "background-image",
    "background-size",
    "background-position",
    "background-repeat",
    "box-shadow",
    "opacity",
    "overflow-x",
    "overflow-y",
    "transform",
    "transform-origin",
    "color",
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "line-height",
    "letter-spacing",
    "text-align",
    "text-decoration-line",
    "text-transform",
    "white-space",
    "word-break",
    "text-overflow",
    "-webkit-line-clamp",
    "object-fit",
    "mix-blend-mode",
    "filter",
    "backdrop-filter",
    "visibility",
  ];

  const TEXT_RUN_STYLE_PROPS = [
    "color",
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "line-height",
    "letter-spacing",
    "text-align",
    "text-decoration-line",
    "text-transform",
    "white-space",
  ];

  // CSS shorthand expansion map so declarations like `background: var(--brand)`
  // populate `css_var_refs["background-color"]` (T8.25 §3.7, §10).
  const SHORTHAND_EXPANSIONS = {
    background: ["background-color", "background-image"],
    "border-radius": [
      "border-top-left-radius",
      "border-top-right-radius",
      "border-bottom-right-radius",
      "border-bottom-left-radius",
    ],
    padding: ["padding-top", "padding-right", "padding-bottom", "padding-left"],
    margin: ["margin-top", "margin-right", "margin-bottom", "margin-left"],
    "border-color": [
      "border-top-color",
      "border-right-color",
      "border-bottom-color",
      "border-left-color",
    ],
    "border-width": [
      "border-top-width",
      "border-right-width",
      "border-bottom-width",
      "border-left-width",
    ],
    border: [
      "border-top-width",
      "border-right-width",
      "border-bottom-width",
      "border-left-width",
      "border-top-color",
      "border-right-color",
      "border-bottom-color",
      "border-left-color",
    ],
    gap: ["gap", "row-gap", "column-gap"],
    overflow: ["overflow-x", "overflow-y"],
  };

  function parsePx(val) {
    if (!val) return 0;
    const n = parseFloat(val);
    return Number.isFinite(n) ? n : 0;
  }

  function roundCoord(n) {
    return Math.round(n * 100) / 100;
  }

  /**
   * Computes a simple CSS selector specificity tuple `[idCount, classAttrPseudoCount, tagCount]`
   * encoded as a single integer for deterministic rule ordering (T8.25 §3.7).
   */
  function selectorSpecificity(selector) {
    const cleaned = String(selector || "")
      .replace(/:not\(([^)]*)\)/g, " $1 ")
      .replace(/::[a-zA-Z-]+/g, " ");
    const ids = (cleaned.match(/#[a-zA-Z0-9_-]+/g) || []).length;
    const classesAttrs = (
      cleaned.match(/(\.[a-zA-Z0-9_-]+|\[[^\]]+\]|:[a-zA-Z-]+(\([^)]*\))?)/g) || []
    ).length;
    const tags = (
      cleaned
        .replace(/#[a-zA-Z0-9_-]+/g, " ")
        .replace(/\.[a-zA-Z0-9_-]+/g, " ")
        .replace(/\[[^\]]+\]/g, " ")
        .replace(/:[a-zA-Z-]+(\([^)]*\))?/g, " ")
        .match(/\b[a-zA-Z][a-zA-Z0-9-]*/g) || []
    ).length;
    return ids * 1000000 + classesAttrs * 1000 + tags;
  }

  /**
   * Extracts all `--*` variable names referenced inside a CSS value string containing `var(...)`.
   */
  function extractVarNames(rawVal) {
    if (!rawVal || typeof rawVal !== "string" || !rawVal.includes("var(")) {
      return [];
    }
    const out = [];
    const re = /var\(\s*(--[a-zA-Z0-9_-]+)/g;
    let m;
    while ((m = re.exec(rawVal)) !== null) {
      if (!out.includes(m[1])) {
        out.push(m[1]);
      }
    }
    return out;
  }

  /**
   * Indexes readable CSSStyleRules in `doc.styleSheets` and records unreadable cross-origin
   * stylesheet URLs in `unavailableSet` (T8.25 §3.7).
   */
  function collectStylesheetRules(doc, unavailableSet) {
    const rules = [];
    let order = 0;

    function visitRuleList(ruleList) {
      for (let i = 0; i < ruleList.length; i++) {
        const rule = ruleList[i];
        if (!rule) continue;
        // STYLE_RULE === 1
        if (rule.type === 1 && rule.selectorText && rule.style) {
          const decls = {};
          const customProps = {};
          const style = rule.style;
          for (let j = 0; j < style.length; j++) {
            const prop = style[j];
            const val = style.getPropertyValue(prop);
            const prio = style.getPropertyPriority(prop) === "important" ? 1 : 0;
            if (prop.startsWith("--")) {
              customProps[prop] = { value: val ? val.trim() : "", important: prio };
            } else if (val && val.includes("var(")) {
              decls[prop] = { value: val.trim(), important: prio };
            }
          }
          // Check shorthand properties on `rule.style` because browsers keep unexpanded `var(...)`
          // on shorthand keys or expose them via `cssText`.
          for (const shortProp of nativeKeys(SHORTHAND_EXPANSIONS)) {
            const sv = style.getPropertyValue(shortProp);
            if (sv && sv.includes("var(")) {
              const prio = style.getPropertyPriority(shortProp) === "important" ? 1 : 0;
              decls[shortProp] = { value: sv.trim(), important: prio };
            }
          }
          if (rule.cssText && rule.cssText.includes("var(")) {
            const bodyMatch = rule.cssText.match(/\{([\s\S]*)\}/);
            if (bodyMatch) {
              const parts = bodyMatch[1].split(";");
              for (const part of parts) {
                const colon = part.indexOf(":");
                if (colon <= 0) continue;
                const pName = part.slice(0, colon).trim().toLowerCase();
                let pVal = part.slice(colon + 1).trim();
                const imp = /!important\s*$/i.test(pVal) ? 1 : 0;
                if (imp) pVal = pVal.replace(/!important\s*$/i, "").trim();
                if (pName.startsWith("--")) {
                  customProps[pName] = { value: pVal, important: imp };
                } else if (pVal.includes("var(")) {
                  decls[pName] = { value: pVal, important: imp };
                }
              }
            }
          }
          if (nativeKeys(decls).length > 0 || nativeKeys(customProps).length > 0) {
            rules.push({
              selectorText: rule.selectorText,
              decls,
              customProps,
              order: order++,
            });
          }
        } else if (rule.cssRules) {
          try {
            visitRuleList(rule.cssRules);
          } catch (_e) {
            // Ignore nested unreadable rules
          }
        }
      }
    }

    const sheets = doc.styleSheets ? nativeSlice.call(doc.styleSheets) : [];
    for (const sheet of sheets) {
      let cssRules;
      try {
        cssRules = sheet.cssRules;
      } catch (_err) {
        if (sheet.href) {
          unavailableSet.add(sheet.href);
        }
        continue;
      }
      if (cssRules) {
        visitRuleList(cssRules);
      }
    }
    return rules;
  }

  /**
   * Computes `css_vars_declared`, `css_var_refs`, and `declaration_sources` for `el` (T8.25 §3.7).
   */
  function resolveElementCssVars(el, cs, parentCs, rules, inheritedCustomDefs) {
    const winningDecls = {};
    const localCustomDefs = Object.assign({}, inheritedCustomDefs);
    const candidateVarNames = new Set();

    for (const rule of rules) {
      const selectors = rule.selectorText.split(",");
      let bestSpec = -1;
      for (const rawSel of selectors) {
        const sel = rawSel.trim();
        if (!sel) continue;
        try {
          if (el.matches(sel)) {
            const spec = selectorSpecificity(sel);
            if (spec > bestSpec) bestSpec = spec;
          }
        } catch (_e) {
          // Ignore pseudo-element or unsupported selectors in `el.matches`
        }
      }
      if (bestSpec < 0) continue;

      for (const k of nativeKeys(rule.customProps)) {
        const entry = rule.customProps[k];
        candidateVarNames.add(k);
        localCustomDefs[k] = entry.value;
      }

      for (const prop of nativeKeys(rule.decls)) {
        const entry = rule.decls[prop];
        const score = entry.important * 1e9 + bestSpec * 1e3 + rule.order;
        const targets = SHORTHAND_EXPANSIONS[prop] || [prop];
        for (const targetProp of targets) {
          if (
            prop === "background" &&
            targetProp === "background-image" &&
            !entry.value.includes("url(") &&
            !entry.value.includes("gradient(")
          ) {
            continue;
          }
          const prev = winningDecls[targetProp];
          if (!prev || score >= prev.score) {
            winningDecls[targetProp] = { value: entry.value, score };
          }
        }
      }
    }

    // Check inline `el.style`
    if (el.style) {
      const inlineCssText = el.getAttribute("style") || "";
      if (inlineCssText) {
        for (const part of inlineCssText.split(";")) {
          const colon = part.indexOf(":");
          if (colon <= 0) continue;
          const pName = part.slice(0, colon).trim();
          let pVal = part.slice(colon + 1).trim();
          const imp = /!important\s*$/i.test(pVal) ? 1 : 0;
          if (imp) pVal = pVal.replace(/!important\s*$/i, "").trim();
          if (pName.startsWith("--")) {
            candidateVarNames.add(pName);
            localCustomDefs[pName] = pVal;
          } else if (pVal.includes("var(")) {
            const lower = pName.toLowerCase();
            const score = (imp ? 2e9 : 1.5e9) + 999999;
            const targets = SHORTHAND_EXPANSIONS[lower] || [lower];
            for (const targetProp of targets) {
              if (
                lower === "background" &&
                targetProp === "background-image" &&
                !pVal.includes("url(") &&
                !pVal.includes("gradient(")
              ) {
                continue;
              }
              winningDecls[targetProp] = { value: pVal, score };
            }
          }
        }
      }
    }

    // (a) `css_vars_declared`: custom properties whose computed value on `el` differs from parent's.
    const cssVarsDeclared = {};
    for (const varName of candidateVarNames) {
      const val = cs.getPropertyValue(varName).trim();
      if (!val) continue;
      const parentVal = parentCs ? parentCs.getPropertyValue(varName).trim() : "";
      if (!parentCs || val !== parentVal) {
        cssVarsDeclared[varName] = val;
      }
    }

    // (b) `css_var_refs`: resolve chains `prop -> ["--brand", ...]`
    const cssVarRefs = {};
    const declarationSources = {};
    for (const prop of nativeKeys(winningDecls)) {
      const rawVal = winningDecls[prop].value;
      const directVars = extractVarNames(rawVal);
      if (directVars.length === 0) continue;
      const chain = [];
      const visited = new Set();
      const queue = nativeFrom(directVars);
      while (queue.length > 0) {
        const vName = queue.shift();
        if (visited.has(vName)) continue;
        visited.add(vName);
        chain.push(vName);
        const defVal = localCustomDefs[vName];
        if (defVal && defVal.includes("var(")) {
          for (const nextVar of extractVarNames(defVal)) {
            if (!visited.has(nextVar)) {
              queue.push(nextVar);
            }
          }
        }
      }
      if (chain.length > 0) {
        cssVarRefs[prop] = chain;
        declarationSources[prop] = rawVal;
      }
    }

    return { cssVarsDeclared, cssVarRefs, declarationSources, localCustomDefs };
  }

  /**
   * Detects the nearest framework component name on `el` (T8.25 §3.8, E15-44 §3.1).
   */
  function extractComponentHint(el) {
    try {
      const keys = nativeKeys(el);
      // 1. React Fiber (`__reactFiber$*` or `__reactInternalInstance$*`)
      for (const k of keys) {
        if (k.startsWith("__reactFiber$") || k.startsWith("__reactInternalInstance$")) {
          let fiber = el[k];
          let depth = 0;
          while (fiber && depth < 32) {
            depth++;
            const t = fiber.type || fiber.elementType;
            if (t) {
              if (typeof t === "function") {
                const name = t.displayName || t.name;
                if (name) return String(name);
              } else if (typeof t === "object") {
                const name =
                  t.displayName ||
                  t.name ||
                  (t.render && (t.render.displayName || t.render.name)) ||
                  (t.type && (t.type.displayName || t.type.name));
                if (name) return String(name);
              }
            }
            fiber = fiber.return;
          }
        }
      }

      // 2. Vue 3 / Vue 2
      if (el.__vueParentComponent && el.__vueParentComponent.type) {
        const vt = el.__vueParentComponent.type;
        const vName = vt.name || vt.__name;
        if (vName) return String(vName);
      }
      if (el.__vue__ && el.__vue__.$options) {
        const v2Name = el.__vue__.$options.name || el.__vue__.$options._componentTag;
        if (v2Name) return String(v2Name);
      }

      // 3. Svelte dev metadata
      if (el.__svelte_meta) {
        if (el.__svelte_meta.componentName) {
          return String(el.__svelte_meta.componentName);
        }
        if (el.__svelte_meta.loc && el.__svelte_meta.loc.file) {
          const file = String(el.__svelte_meta.loc.file);
          const base = file.split("/").pop().replace(/\.svelte$/i, "");
          if (base) return base;
        }
      }

      // 4. Explicit `data-component` attribute
      const dataComp = el.getAttribute("data-component");
      if (dataComp && dataComp.trim()) {
        return dataComp.trim();
      }

      // 5. Angular (`ng-version` or `_nghost-*` custom element selector)
      const hasNgHost = nativeFrom(el.attributes || []).some(
        (a) => a.name === "ng-version" || a.name.startsWith("_nghost-")
      );
      if (hasNgHost && el.tagName && el.tagName.includes("-")) {
        return el.tagName.toLowerCase();
      }
    } catch (_e) {
      // Ignore property access errors on exotic host objects
    }
    return null;
  }

  function extractStyleSubset(cs, propList) {
    const out = {};
    for (const prop of propList) {
      const val = cs.getPropertyValue(prop);
      if (val !== undefined && val !== null && val !== "") {
        out[prop] = val;
      }
    }
    return out;
  }

  function capturePseudo(el, which, borderBox) {
    try {
      const pcs = g.getComputedStyle(el, which);
      if (!pcs) return null;
      const content = pcs.getPropertyValue("content");
      const display = pcs.getPropertyValue("display");
      if (display === "none") return null;
      const bgColor = pcs.getPropertyValue("background-color");
      const bgImage = pcs.getPropertyValue("background-image");
      const hasContent =
        content &&
        content !== "none" &&
        content !== "normal" &&
        content !== '""' &&
        content !== "''";
      const hasBox =
        (bgColor && bgColor !== "rgba(0, 0, 0, 0)" && bgColor !== "transparent") ||
        (bgImage && bgImage !== "none");
      if (!hasContent && !hasBox) return null;
      const w = parsePx(pcs.getPropertyValue("width")) || borderBox.width;
      const h = parsePx(pcs.getPropertyValue("height")) || borderBox.height;
      return {
        content: hasContent ? content.replace(/^["']|["']$/g, "") : "",
        box: {
          x: borderBox.x,
          y: borderBox.y,
          width: roundCoord(w),
          height: roundCoord(h),
        },
        styles: extractStyleSubset(pcs, CAPTURED_STYLE_PROPS),
      };
    } catch (_e) {
      return null;
    }
  }

  function captureTextRunsForElement(el, cs, offsetX, offsetY, stats) {
    const runs = [];
    const doc = el.ownerDocument;
    const childNodes = el.childNodes ? nativeSlice.call(el.childNodes) : [];
    for (const node of childNodes) {
      if (!node || node.nodeType !== 3) continue; // Node.TEXT_NODE === 3
      const rawText = node.nodeValue || "";
      if (!rawText.trim()) continue;
      const range = doc.createRange();
      range.selectNodeContents(node);
      const clientRects = range.getClientRects ? nativeSlice.call(range.getClientRects()) : [];
      const lines = [];
      let linesTruncated = false;
      for (let i = 0; i < clientRects.length; i++) {
        const r = clientRects[i];
        if (r.width <= 0 && r.height <= 0) continue;
        if (lines.length >= 2000) {
          linesTruncated = true;
          break;
        }
        lines.push({
          x: roundCoord(r.left + offsetX),
          y: roundCoord(r.top + offsetY),
          width: roundCoord(r.width),
          height: roundCoord(r.height),
        });
      }
      const run = {
        text: rawText,
        styles: extractStyleSubset(cs, TEXT_RUN_STYLE_PROPS),
        lines,
      };
      if (linesTruncated) {
        run.lines_truncated = true;
      }
      runs.push(run);
      stats.text_runs++;
    }
    return runs;
  }

  async function waitWithTimeout(promise, timeoutMs) {
    let timer = null;
    const timeoutPromise = new Promise((resolve) => {
      timer = setTimeout(() => resolve("timeout"), timeoutMs);
    });
    try {
      return await Promise.race([promise.then(() => "ok").catch(() => "error"), timeoutPromise]);
    } finally {
      if (timer !== null) clearTimeout(timer);
    }
  }

  async function fetchAsDataUrl(url, timeoutMs) {
    if (!nativeFetch) return null;
    const controller = typeof AbortController !== "undefined" ? new AbortController() : null;
    const timer = controller ? setTimeout(() => controller.abort(), timeoutMs) : null;
    try {
      const resp = await nativeFetch(url, {
        mode: "cors",
        signal: controller ? controller.signal : undefined,
      });
      if (!resp || !resp.ok) return null;
      const blob = await resp.blob();
      return await new Promise((resolve) => {
        const reader = new FileReader();
        reader.onloadend = () =>
          resolve(typeof reader.result === "string" ? reader.result : null);
        reader.onerror = () => resolve(null);
        reader.readAsDataURL(blob);
      });
    } catch (_e) {
      return null;
    } finally {
      if (timer !== null) clearTimeout(timer);
    }
  }

  function drawImageElementToPngDataUrl(img, doc) {
    try {
      const w = img.naturalWidth || img.width || 0;
      const h = img.naturalHeight || img.height || 0;
      if (w <= 0 || h <= 0) return null;
      const canvas = doc.createElement("canvas");
      canvas.width = w;
      canvas.height = h;
      const ctx = canvas.getContext("2d");
      if (!ctx) return null;
      ctx.drawImage(img, 0, 0, w, h);
      return canvas.toDataURL("image/png");
    } catch (_e) {
      return null;
    }
  }

  async function captureImageFromUrl(src, imgEl, kind, doc, budgetState, options) {
    if (!src) return null;
    const timeoutMs = options.image_timeout_ms ?? 3000;

    if (src.startsWith("data:")) {
      const byteLen = src.length;
      if (budgetState.imageBytes + byteLen > budgetState.maxImageBytes) {
        return { src, skipped: "size limit", kind };
      }
      budgetState.imageBytes += byteLen;
      budgetState.images++;
      return { src, data_url: src, kind };
    }

    if (imgEl && !imgEl.complete && typeof imgEl.decode === "function") {
      const status = await waitWithTimeout(imgEl.decode(), timeoutMs);
      if (status === "timeout" && !imgEl.complete) {
        budgetState.images++;
        return { src, pending: true, kind };
      }
    }

    let dataUrl = imgEl ? drawImageElementToPngDataUrl(imgEl, doc) : null;
    if (!dataUrl) {
      dataUrl = await fetchAsDataUrl(src, Math.min(timeoutMs, 1500));
    }

    if (!dataUrl) {
      budgetState.images++;
      return { src, blocked: true, kind };
    }

    const byteLen = dataUrl.length;
    if (budgetState.imageBytes + byteLen > budgetState.maxImageBytes) {
      return { src, skipped: "size limit", kind };
    }
    budgetState.imageBytes += byteLen;
    budgetState.images++;
    return { src, data_url: dataUrl, kind };
  }

  function extractCssUrls(bgImageValue) {
    if (!bgImageValue || bgImageValue === "none") return [];
    const urls = [];
    const re = /url\(\s*(['"]?)(.*?)\1\s*\)/gi;
    let m;
    while ((m = re.exec(bgImageValue)) !== null) {
      if (m[2]) urls.push(m[2]);
    }
    return urls;
  }

  function collectFonts(doc) {
    const out = [];
    const seen = new Set();
    try {
      if (doc.fonts && typeof doc.fonts.forEach === "function") {
        doc.fonts.forEach((face) => {
          const family = String(face.family || "").replace(/^["']|["']$/g, "");
          const weight = String(face.weight || "400");
          const style = String(face.style || "normal");
          const key = `${family}|${weight}|${style}`;
          if (!family || seen.has(key)) return;
          seen.add(key);
          out.push({ family, weight, style });
        });
      }
    } catch (_e) {
      // Ignore font iteration failures
    }
    return out;
  }

  async function __designosCapture(options = {}) {
    const startTime = nativeNow();
    const doc = document;
    const win = g;
    const includeHidden = Boolean(options.include_hidden);
    const maxElements =
      typeof options.max_elements === "number" && options.max_elements > 0
        ? Math.floor(options.max_elements)
        : 20000;
    const maxImageBytes =
      typeof options.max_image_bytes === "number" && options.max_image_bytes >= 0
        ? options.max_image_bytes
        : 50 * 1024 * 1024;

    const rootEl =
      typeof options.root === "string"
        ? doc.querySelector(options.root) || doc.documentElement
        : options.root instanceof Element
        ? options.root
        : doc.documentElement;

    const unavailableSet = new Set();
    const sheetRules = collectStylesheetRules(doc, unavailableSet);

    const stats = {
      elements: 0,
      skipped_elements: 0,
      text_runs: 0,
      images: 0,
      image_bytes: 0,
      elapsed_ms: 0,
    };
    const budgetState = {
      maxElements,
      maxImageBytes,
      imageBytes: 0,
      images: 0,
      truncated: false,
    };

    let paintCounter = 0;

    async function walkElement(el, parentCs, inheritedCustomDefs, pageOffsetX, pageOffsetY) {
      const tag = (el.tagName || "div").toLowerCase();
      if (SKIPPED_TAGS.has(tag)) {
        return null;
      }

      const elWin = el.ownerDocument.defaultView || win;
      const cs = elWin.getComputedStyle(el);
      const isHidden =
        cs.getPropertyValue("display") === "none" ||
        cs.getPropertyValue("visibility") === "hidden";
      if (isHidden && !includeHidden) {
        return null;
      }

      if (stats.elements >= budgetState.maxElements) {
        budgetState.truncated = true;
        stats.skipped_elements++;
        return null;
      }
      stats.elements++;

      const scrollX =
        cs.getPropertyValue("position") === "fixed" ? 0 : elWin.scrollX || elWin.pageXOffset || 0;
      const scrollY =
        cs.getPropertyValue("position") === "fixed" ? 0 : elWin.scrollY || elWin.pageYOffset || 0;
      const offX = pageOffsetX + scrollX;
      const offY = pageOffsetY + scrollY;

      const rect = el.getBoundingClientRect();
      const borderBox = {
        x: roundCoord(rect.left + offX),
        y: roundCoord(rect.top + offY),
        width: roundCoord(rect.width),
        height: roundCoord(rect.height),
      };

      const bt = parsePx(cs.getPropertyValue("border-top-width"));
      const br = parsePx(cs.getPropertyValue("border-right-width"));
      const bb = parsePx(cs.getPropertyValue("border-bottom-width"));
      const bl = parsePx(cs.getPropertyValue("border-left-width"));
      const pt = parsePx(cs.getPropertyValue("padding-top"));
      const pr = parsePx(cs.getPropertyValue("padding-right"));
      const pb = parsePx(cs.getPropertyValue("padding-bottom"));
      const pl = parsePx(cs.getPropertyValue("padding-left"));

      const contentBox = {
        x: roundCoord(borderBox.x + bl + pl),
        y: roundCoord(borderBox.y + bt + pt),
        width: roundCoord(Math.max(0, borderBox.width - bl - pl - br - pr)),
        height: roundCoord(Math.max(0, borderBox.height - bt - pt - bb - pb)),
      };

      const zIndexRaw = cs.getPropertyValue("z-index");
      const zIndexNum = parseInt(zIndexRaw, 10);
      const z = paintCounter++ + (Number.isFinite(zIndexNum) ? zIndexNum * 1000 : 0);

      const styles = extractStyleSubset(cs, CAPTURED_STYLE_PROPS);
      const { cssVarsDeclared, cssVarRefs, declarationSources, localCustomDefs } =
        resolveElementCssVars(el, cs, parentCs, sheetRules, inheritedCustomDefs);
      const componentHint = extractComponentHint(el);

      const beforePseudo = capturePseudo(el, "::before", borderBox);
      const afterPseudo = capturePseudo(el, "::after", borderBox);
      const pseudo =
        beforePseudo || afterPseudo
          ? {
              ...(beforePseudo ? { before: beforePseudo } : {}),
              ...(afterPseudo ? { after: afterPseudo } : {}),
            }
          : undefined;

      const textRuns =
        tag === "svg" ? [] : captureTextRunsForElement(el, cs, offX, offY, stats);

      let primaryImage = null;
      const extraImages = [];

      if (tag === "img") {
        const src = el.currentSrc || el.getAttribute("src") || "";
        primaryImage = await captureImageFromUrl(
          src,
          el,
          "img",
          el.ownerDocument,
          budgetState,
          options
        );
      } else if (tag === "canvas" && typeof el.toDataURL === "function") {
        try {
          const dataUrl = el.toDataURL("image/png");
          if (budgetState.imageBytes + dataUrl.length <= budgetState.maxImageBytes) {
            budgetState.imageBytes += dataUrl.length;
            budgetState.images++;
            primaryImage = { src: "canvas", data_url: dataUrl, kind: "canvas" };
          } else {
            primaryImage = { src: "canvas", skipped: "size limit", kind: "canvas" };
          }
        } catch (_e) {
          primaryImage = { src: "canvas", blocked: true, kind: "canvas" };
        }
      } else if (tag === "svg") {
        const svgHtml = el.outerHTML || "";
        budgetState.images++;
        primaryImage = { src: "inline-svg", svg: svgHtml, kind: "svg" };
      } else if (tag === "video") {
        const poster = el.getAttribute("poster");
        if (poster) {
          primaryImage = await captureImageFromUrl(
            poster,
            null,
            "video-poster",
            el.ownerDocument,
            budgetState,
            options
          );
        }
      }

      const bgUrls = extractCssUrls(styles["background-image"]);
      for (const bgUrl of bgUrls) {
        const bgRef = await captureImageFromUrl(
          bgUrl,
          null,
          "background",
          el.ownerDocument,
          budgetState,
          options
        );
        if (bgRef) {
          if (!primaryImage) {
            primaryImage = bgRef;
          } else {
            extraImages.push(bgRef);
          }
        }
      }

      const children = [];
      let hasOpenShadow = false;
      const hasClosedShadow = Boolean(el.__designosClosedShadow);

      if (tag !== "svg") {
        if (el.shadowRoot) {
          hasOpenShadow = true;
          const shadowChildren = el.shadowRoot.children
            ? nativeSlice.call(el.shadowRoot.children)
            : [];
          for (const sc of shadowChildren) {
            const childCap = await walkElement(sc, cs, localCustomDefs, pageOffsetX, pageOffsetY);
            if (childCap) children.push(childCap);
          }
        }

        if (tag === "iframe") {
          let iframeDoc = null;
          try {
            iframeDoc = el.contentDocument;
          } catch (_e) {
            iframeDoc = null;
          }
          if (iframeDoc && iframeDoc.documentElement) {
            const subCap = await walkElement(
              iframeDoc.documentElement,
              cs,
              localCustomDefs,
              borderBox.x,
              borderBox.y
            );
            if (subCap) children.push(subCap);
          } else if (el.getAttribute("src")) {
            return {
              tag,
              ...(el.id ? { id: el.id } : {}),
              classes: el.classList ? nativeFrom(el.classList) : [],
              box: borderBox,
              content_box: contentBox,
              z,
              visible: !isHidden,
              styles,
              cross_origin: true,
              children: [],
            };
          }
        } else {
          const domChildren = el.children ? nativeSlice.call(el.children) : [];
          for (const child of domChildren) {
            const childCap = await walkElement(
              child,
              cs,
              localCustomDefs,
              pageOffsetX,
              pageOffsetY
            );
            if (childCap) children.push(childCap);
          }
        }
      }

      const out = {
        tag,
        ...(el.id ? { id: el.id } : {}),
        classes: el.classList ? nativeFrom(el.classList) : [],
        ...(el.getAttribute("role") ? { role: el.getAttribute("role") } : {}),
        ...(el.getAttribute("aria-label") ? { aria_label: el.getAttribute("aria-label") } : {}),
        ...(el.getAttribute("alt") ? { alt: el.getAttribute("alt") } : {}),
        box: borderBox,
        content_box: contentBox,
        z,
        visible: !isHidden,
        styles,
        ...(pseudo ? { pseudo } : {}),
        ...(textRuns.length > 0 ? { text_runs: textRuns } : {}),
        ...(primaryImage ? { image: primaryImage } : {}),
        ...(extraImages.length > 0 ? { images: extraImages } : {}),
        ...(nativeKeys(cssVarsDeclared).length > 0
          ? { css_vars_declared: cssVarsDeclared }
          : {}),
        ...(nativeKeys(cssVarRefs).length > 0 ? { css_var_refs: cssVarRefs } : {}),
        ...(nativeKeys(declarationSources).length > 0
          ? { declaration_sources: declarationSources }
          : {}),
        ...(componentHint
          ? { component_hint: componentHint, framework_name: componentHint }
          : {}),
        ...(hasOpenShadow ? { shadow: true } : {}),
        ...(hasClosedShadow ? { closed_shadow: true } : {}),
        children,
      };
      return out;
    }

    const rootCapture =
      (await walkElement(rootEl, null, {}, 0, 0)) || {
        tag: "html",
        classes: [],
        box: { x: 0, y: 0, width: 0, height: 0 },
        content_box: { x: 0, y: 0, width: 0, height: 0 },
        z: 0,
        visible: true,
        styles: {},
        children: [],
      };

    stats.images = budgetState.images;
    stats.image_bytes = budgetState.imageBytes;
    stats.elapsed_ms = Math.round((nativeNow() - startTime) * 100) / 100;

    return {
      schema: "designos-webcapture",
      version: 1,
      url: String(doc.location ? doc.location.href : ""),
      title: String(doc.title || ""),
      viewport: {
        width: win.innerWidth || doc.documentElement.clientWidth || 0,
        height: win.innerHeight || doc.documentElement.clientHeight || 0,
        dpr: win.devicePixelRatio || 1,
      },
      scroll: {
        width: Math.max(
          doc.documentElement ? doc.documentElement.scrollWidth : 0,
          doc.body ? doc.body.scrollWidth : 0
        ),
        height: Math.max(
          doc.documentElement ? doc.documentElement.scrollHeight : 0,
          doc.body ? doc.body.scrollHeight : 0
        ),
      },
      captured_at: new Date().toISOString(),
      fonts: collectFonts(doc),
      ...(unavailableSet.size > 0
        ? { css_vars_unavailable: nativeFrom(unavailableSet) }
        : {}),
      root: rootCapture,
      truncated: budgetState.truncated,
      stats,
    };
  }

  g.__designosCapture = __designosCapture;
  if (typeof module !== "undefined" && module.exports) {
    module.exports = { __designosCapture };
  }
})();
