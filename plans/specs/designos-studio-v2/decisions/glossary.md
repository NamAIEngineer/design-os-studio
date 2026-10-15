---
id: glossary
type: glossary
title: "Domain Glossary & Banned Wording"
updated: 2026-10-15
---

## Terms

| Term | Anchor | Definition | Source |
|---|---|---|---|
| Spec Format v2 | spec-format-v2 | Canonical multi-artifact specification bundle (`spec.manifest.json`, `PRD.md`, `US-*.md`, `ui-contract.md`, `review/gates/*.md`). | D3, D5 |
| Copy cho AI | copy-cho-ai | Deterministic Markdown citation block carrying exact source file path, line range, section breadcrumb, reader view, and project revision. | D4 |
| UI Contract | ui-contract | 12-column behavioral selector table mapping `Screen × State` to `US`, `Element`, `Components`, and `SCN`. | D5 |
| Kiwi v106 | kiwi-v106 | Figma binary container format preserving unknown schema fields in `raw_fields` across load/save cycles. | D1 |

## Ban list

| Banned | Replacement | Reason |
|---|---|---|
| Emoji icons in UI chrome | `crate::icons::Icon` vector Lucide glyphs | Violates `es:designer` craft-floor §2 |
| Nested cards-in-cards | `1px` hairline divider (`#2A2B31`) or `#26272C` table header row | Violates `es:designer` craft-floor §2 |
| Unverified "seamless" claim | Cite concrete `SCN-xx` test and `CURRENT` gate digest | No unmeasured claims |
