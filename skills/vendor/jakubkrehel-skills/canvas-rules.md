<!-- Portions of this folder are copied from or derived from jakubkrehel/skills (https://github.com/jakubkrehel/skills), commit 4c66b1fa86e9ab70187b43d7da9b6fdb29f2ef63, Copyright (c) 2026 Jakub Krehel, MIT License (see LICENSE). -->

# jakubkrehel UI Rules (Canvas Edition)

> Portions of this folder are copied from or derived from jakubkrehel/skills (https://github.com/jakubkrehel/skills), commit 4c66b1fa86e9ab70187b43d7da9b6fdb29f2ef63, Copyright (c) 2026 Jakub Krehel, MIT License (see LICENSE).

## better-accessibility

- **`better-accessibility/forms#1`** (`check:surface.web-app.required-states`): Design explicit error state frames with inline helper text bound to semantic danger tokens below the invalid input.
- **`better-accessibility/forms#2`** (`check:surface.web-app.required-states`): Design explicit error state frames with inline helper text bound to semantic danger tokens below the invalid input.
- **`better-accessibility/forms#3`** (`check:surface.web-app.required-states`): Design explicit error state frames with inline helper text bound to semantic danger tokens below the invalid input.
- **`better-accessibility/forms#4`** (`check:surface.web-app.required-states`): Design explicit error state frames with inline helper text bound to semantic danger tokens below the invalid input.
- **`better-accessibility/forms#5`** (`check:surface.web-app.required-states`): Design explicit error state frames with inline helper text bound to semantic danger tokens below the invalid input.
- **`better-accessibility/forms#6`** (`lint:placeholder-text`): Every form input frame must have a persistent visible label node above or beside the field rather than relying on placeholder text alone.
- **`better-accessibility/forms#8`** (`check:surface.web-app.required-states`): Design explicit error state frames with inline helper text bound to semantic danger tokens below the invalid input.
- **`better-accessibility/hit-areas#1`** (`lint:tap-target-size`): Interactive controls and icon buttons must have a bounding box of at least 24x24 px (32x32 px desktop, 44x44 px touch); check with `designos_lint` tap-target-size.
- **`better-accessibility/motion-and-zoom#1`** (`rubric:layout`): Include explicit close or pause controls on banners, toasts, and media frames on the canvas instead of relying on auto-dismissal.
- **`better-accessibility/motion-and-zoom#2`** (`rubric:layout`): Include explicit close or pause controls on banners, toasts, and media frames on the canvas instead of relying on auto-dismissal.
- **`better-accessibility/motion-and-zoom#3`** (`rubric:layout`): Include explicit close or pause controls on banners, toasts, and media frames on the canvas instead of relying on auto-dismissal.
- **`better-accessibility/motion-and-zoom#4`** (`rubric:layout`): Include explicit close or pause controls on banners, toasts, and media frames on the canvas instead of relying on auto-dismissal.
- **`better-accessibility/motion-and-zoom#5`** (`lint:text-truncated`): Size text frames with Auto-Resize Height or sufficient container headroom so copy never truncates when scaled; verify with `designos_lint` text-truncated.
- **`better-accessibility/motion-and-zoom#6`** (`lint:overflow-parent`): Ensure narrow 320 px mobile frames reflow vertically in auto-layout without horizontal child overflow; verify with `designos_lint` overflow-parent.
- **`better-accessibility/semantics-and-aria#7`** (`lint:non-scale-font-size`): Establish heading hierarchy through the document type scale and text styles so visual levels match structural importance.
- **`better-accessibility/semantics-and-aria#10`** (`rubric:iconography`): Give icon-only button frames a descriptive layer name (such as `Button / Close`) so exported specs and accessibility inspection carry an explicit label.
- **`better-accessibility/semantics-and-aria#17`** (`lint:contrast`): Active controls must pass `designos_lint` WCAG contrast against their resolved backdrop; disabled state frames use muted tokens intentionally.

## better-colors

- **`better-colors/color-formats#5`** (`skill:generate-library/tokens`): Model derived surface tints and light/dark appearances as document Variable Collections and modes bound via `var:` references.
- **`better-colors/color-formats#6`** (`skill:generate-library/tokens`): Model derived surface tints and light/dark appearances as document Variable Collections and modes bound via `var:` references.
- **`better-colors/color-formats#7`** (`skill:generate-library/tokens`): Model derived surface tints and light/dark appearances as document Variable Collections and modes bound via `var:` references.
- **`better-colors/contrast#1`** (`lint:contrast`): Avoid mid-lightness surface backgrounds under body text; shift the surface token toward light or dark extremes so `designos_lint` contrast reaches AA/AAA.
- **`better-colors/contrast#2`** (`lint:contrast`): Body text needs 4.5:1 against the colour actually behind it; designos_lint measures the resolved backdrop and keeps adjusted colours within sRGB gamut.
- **`better-colors/contrast#3`** (`lint:contrast`): Verify foreground-on-surface contrast across both Light and Dark variable modes with `designos_lint` contrast.
- **`better-colors/contrast#4`** (`lint:contrast`): On translucent or background-blur frames, keep fill opacity high enough that text passes `designos_lint` resolved-backdrop contrast over any underlying layer.
- **`better-colors/contrast#5`** (`lint:contrast`): Measure contrast on the final composited canvas pixels (including layer opacity and blend modes) via `designos_lint` rather than inspecting raw fill hex values.
- **`better-colors/contrast#6`** (`lint:contrast`): Place a dark or light scrim overlay behind text sitting over image fills so `designos_lint` worst-region backdrop contrast passes AA.
- **`better-colors/palette-generation#1`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#1).
- **`better-colors/palette-generation#2`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#2).
- **`better-colors/palette-generation#3`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#3).
- **`better-colors/palette-generation#4`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#4).
- **`better-colors/palette-generation#5`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#5).
- **`better-colors/palette-generation#6`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#6).
- **`better-colors/palette-generation#7`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#7).
- **`better-colors/palette-generation#8`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#8).
- **`better-colors/palette-generation#9`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#9).
- **`better-colors/palette-generation#10`** (`rubric:color`): Build colour variable ramps with perceptual OKLCH lightness progression and consistent hue across steps (better-colors/palette-generation#10).
- **`better-colors/palette-generation#11`** (`lint:contrast`): Re-run `designos_lint` contrast on both Light and Dark variable modes because mirrored ramp steps do not preserve perceived contrast.
- **`better-colors/palette-structure#1`** (`skill:generate-library/tokens`): Structure colour variable collections into clear semantic roles (background, subtle surface, border, solid accent, foreground text) with distinct status hues (better-colors/palette-structure#1).
- **`better-colors/palette-structure#2`** (`skill:generate-library/tokens`): Structure colour variable collections into clear semantic roles (background, subtle surface, border, solid accent, foreground text) with distinct status hues (better-colors/palette-structure#2).
- **`better-colors/palette-structure#3`** (`skill:generate-library/tokens`): Structure colour variable collections into clear semantic roles (background, subtle surface, border, solid accent, foreground text) with distinct status hues (better-colors/palette-structure#3).
- **`better-colors/palette-structure#4`** (`skill:generate-library/tokens`): Structure colour variable collections into clear semantic roles (background, subtle surface, border, solid accent, foreground text) with distinct status hues (better-colors/palette-structure#4).
- **`better-colors/palette-structure#5`** (`lint:hardcoded-color`): Replace raw hex fills and near-duplicate colour literals with semantic document variables (`var:`) verified by `designos_lint` hardcoded-color.
- **`better-colors/palette-structure#6`** (`skill:generate-library/tokens`): Structure colour variable collections into clear semantic roles (background, subtle surface, border, solid accent, foreground text) with distinct status hues (better-colors/palette-structure#6).
- **`better-colors/palette-structure#7`** (`lint:hardcoded-color`): Replace raw hex fills and near-duplicate colour literals with semantic document variables (`var:`) verified by `designos_lint` hardcoded-color.
- **`better-colors/palette-structure#8`** (`lint:hardcoded-color`): Replace raw hex fills and near-duplicate colour literals with semantic document variables (`var:`) verified by `designos_lint` hardcoded-color.
- **`better-colors/palette-structure#9`** (`skill:generate-library/tokens`): Structure colour variable collections into clear semantic roles (background, subtle surface, border, solid accent, foreground text) with distinct status hues (better-colors/palette-structure#9).

## better-interface

- **`better-interface/SKILL#1`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#1).
- **`better-interface/SKILL#2`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#2).
- **`better-interface/SKILL#3`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#3).
- **`better-interface/SKILL#4`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#4).
- **`better-interface/SKILL#5`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#5).
- **`better-interface/SKILL#6`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#6).
- **`better-interface/SKILL#7`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#7).
- **`better-interface/SKILL#8`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#8).
- **`better-interface/SKILL#9`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#9).
- **`better-interface/SKILL#14`** (`lint:clipped-partial`): Verify that no frame content is clipped, overlapping, or overflowing at compact widths using `designos_lint` (`clipped-partial`, `sibling-overlap`, `overflow-parent`).
- **`better-interface/SKILL#15`** (`lint:contrast`): Ensure all body and control text passes `designos_lint` WCAG contrast against its resolved backdrop and never conveys status by colour alone.
- **`better-interface/SKILL#16`** (`lint:contrast`): Ensure all body and control text passes `designos_lint` WCAG contrast against its resolved backdrop and never conveys status by colour alone.
- **`better-interface/SKILL#17`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#17).
- **`better-interface/SKILL#18`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#18).
- **`better-interface/SKILL#19`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#19).
- **`better-interface/SKILL#20`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#20).
- **`better-interface/SKILL#21`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#21).
- **`better-interface/SKILL#22`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#22).
- **`better-interface/SKILL#23`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#23).
- **`better-interface/SKILL#24`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#24).
- **`better-interface/SKILL#25`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#25).
- **`better-interface/SKILL#26`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#26).
- **`better-interface/SKILL#27`** (`skill:critique/workflow`): Evaluate canvas frames systematically across accessibility, layout, typography, colour, and state completeness using `designos_critique` (better-interface/SKILL#27).
- **`better-interface/review-format#1`** (`skill:critique/workflow`): Report each canvas critique finding with target node IDs, severity (`blocker`, `major`, `minor`), concrete issue description, and repair target (better-interface/review-format#1).
- **`better-interface/review-format#2`** (`skill:critique/workflow`): Report each canvas critique finding with target node IDs, severity (`blocker`, `major`, `minor`), concrete issue description, and repair target (better-interface/review-format#2).
- **`better-interface/review-format#3`** (`skill:critique/workflow`): Report each canvas critique finding with target node IDs, severity (`blocker`, `major`, `minor`), concrete issue description, and repair target (better-interface/review-format#3).
- **`better-interface/review-format#4`** (`skill:critique/workflow`): Report each canvas critique finding with target node IDs, severity (`blocker`, `major`, `minor`), concrete issue description, and repair target (better-interface/review-format#4).
- **`better-interface/review-format#5`** (`skill:critique/workflow`): Report each canvas critique finding with target node IDs, severity (`blocker`, `major`, `minor`), concrete issue description, and repair target (better-interface/review-format#5).
- **`better-interface/review-format#6`** (`skill:critique/workflow`): Report each canvas critique finding with target node IDs, severity (`blocker`, `major`, `minor`), concrete issue description, and repair target (better-interface/review-format#6).
- **`better-interface/review-format#7`** (`skill:critique/workflow`): Report each canvas critique finding with target node IDs, severity (`blocker`, `major`, `minor`), concrete issue description, and repair target (better-interface/review-format#7).

## better-layout

- **`better-layout/grouping-and-alignment#1`** (`lint:off-scale`): Group related elements using on-scale auto-layout `itemSpacing` and `padding` steps without stray off-grid offsets; check with `designos_lint` off-scale.
- **`better-layout/grouping-and-alignment#2`** (`rubric:layout`): Use filled card frames for selectable units and 1 px hairline strokes for dense data rows rather than nesting redundant borders (better-layout/grouping-and-alignment#2).
- **`better-layout/grouping-and-alignment#3`** (`rubric:layout`): Use filled card frames for selectable units and 1 px hairline strokes for dense data rows rather than nesting redundant borders (better-layout/grouping-and-alignment#3).
- **`better-layout/grouping-and-alignment#4`** (`lint:off-scale`): Group related elements using on-scale auto-layout `itemSpacing` and `padding` steps without stray off-grid offsets; check with `designos_lint` off-scale.
- **`better-layout/grouping-and-alignment#5`** (`lint:off-scale`): Group related elements using on-scale auto-layout `itemSpacing` and `padding` steps without stray off-grid offsets; check with `designos_lint` off-scale.
- **`better-layout/grouping-and-alignment#6`** (`rubric:typography`): Right-align numeric columns and enable tabular numerals (`tnum`) in table frames while left-aligning text columns.
- **`better-layout/spacing-and-adaptivity#1`** (`rubric:layout`): Keep primary action bars in fixed header or footer zones and show clear disclosure or scroll affordances on bounded containers (better-layout/spacing-and-adaptivity#1).
- **`better-layout/spacing-and-adaptivity#2`** (`rubric:layout`): Keep primary action bars in fixed header or footer zones and show clear disclosure or scroll affordances on bounded containers (better-layout/spacing-and-adaptivity#2).
- **`better-layout/spacing-and-adaptivity#3`** (`lint:text-truncated`): Avoid silent text truncation on the canvas; size containers to fit copy or include an explicit expand affordance, verified by `designos_lint` text-truncated.
- **`better-layout/spacing-and-adaptivity#5`** (`rubric:layout`): Keep primary action bars in fixed header or footer zones and show clear disclosure or scroll affordances on bounded containers (better-layout/spacing-and-adaptivity#5).
- **`better-layout/spacing-and-adaptivity#6`** (`rubric:layout`): Keep primary action bars in fixed header or footer zones and show clear disclosure or scroll affordances on bounded containers (better-layout/spacing-and-adaptivity#6).

## better-typography

- **`better-typography/SKILL#1`** (`lint:text-unexpected-wrap`): Prevent orphan words and unexpected two-line wraps on badges, buttons, and headings; verify with `designos_lint` text-unexpected-wrap.
- **`better-typography/SKILL#2`** (`lint:text-unexpected-wrap`): Prevent orphan words and unexpected two-line wraps on badges, buttons, and headings; verify with `designos_lint` text-unexpected-wrap.
- **`better-typography/SKILL#3`** (`lint:text-unexpected-wrap`): Prevent orphan words and unexpected two-line wraps on badges, buttons, and headings; verify with `designos_lint` text-unexpected-wrap.
- **`better-typography/SKILL#4`** (`lint:text-unexpected-wrap`): Prevent orphan words and unexpected two-line wraps on badges, buttons, and headings; verify with `designos_lint` text-unexpected-wrap.
- **`better-typography/SKILL#5`** (`check:surface.mobile.body-min-size`): Keep mobile body and input text at or above 15–16 px so controls stay legible without pinch-zoom.
- **`better-typography/variable-fonts-and-opentype#1`** (`rubric:typography`): Use variable font weight axes and OpenType features (small caps, tabular numerals, proper superscripts/subscripts) on canvas text styles (better-typography/variable-fonts-and-opentype#1).
- **`better-typography/variable-fonts-and-opentype#2`** (`rubric:typography`): Use variable font weight axes and OpenType features (small caps, tabular numerals, proper superscripts/subscripts) on canvas text styles (better-typography/variable-fonts-and-opentype#2).
- **`better-typography/variable-fonts-and-opentype#3`** (`rubric:typography`): Use variable font weight axes and OpenType features (small caps, tabular numerals, proper superscripts/subscripts) on canvas text styles (better-typography/variable-fonts-and-opentype#3).
- **`better-typography/variable-fonts-and-opentype#4`** (`rubric:typography`): Use variable font weight axes and OpenType features (small caps, tabular numerals, proper superscripts/subscripts) on canvas text styles (better-typography/variable-fonts-and-opentype#4).
- **`better-typography/wrapping-and-punctuation#1`** (`rubric:typography`): Align multi-line paragraphs according to script direction and keep line lengths within 60–75 characters (better-typography/wrapping-and-punctuation#1).
- **`better-typography/wrapping-and-punctuation#2`** (`rubric:typography`): Align multi-line paragraphs according to script direction and keep line lengths within 60–75 characters (better-typography/wrapping-and-punctuation#2).

## better-ui

- **`better-ui/enter-exit#1`** (`rubric:motion`): Group prototype transition layers semantically (header, body, actions) with staggered smart-animate timing (better-ui/enter-exit#1).
- **`better-ui/enter-exit#2`** (`rubric:motion`): Group prototype transition layers semantically (header, body, actions) with staggered smart-animate timing (better-ui/enter-exit#2).
- **`better-ui/enter-exit#3`** (`rubric:motion`): Group prototype transition layers semantically (header, body, actions) with staggered smart-animate timing (better-ui/enter-exit#3).
- **`better-ui/enter-exit#4`** (`rubric:motion`): Group prototype transition layers semantically (header, body, actions) with staggered smart-animate timing (better-ui/enter-exit#4).
- **`better-ui/icons#1`** (`rubric:iconography`): Use a single consistent SVG vector icon set per surface, sized to match its pixel grid (`16`, `20`, or `24` px) and proportional to adjacent text (better-ui/icons#1).
- **`better-ui/icons#2`** (`rubric:iconography`): Use a single consistent SVG vector icon set per surface, sized to match its pixel grid (`16`, `20`, or `24` px) and proportional to adjacent text (better-ui/icons#2).
- **`better-ui/icons#3`** (`rubric:iconography`): Use a single consistent SVG vector icon set per surface, sized to match its pixel grid (`16`, `20`, or `24` px) and proportional to adjacent text (better-ui/icons#3).
- **`better-ui/icons#4`** (`rubric:iconography`): Use a single consistent SVG vector icon set per surface, sized to match its pixel grid (`16`, `20`, or `24` px) and proportional to adjacent text (better-ui/icons#4).
- **`better-ui/icons#5`** (`rubric:iconography`): Use a single consistent SVG vector icon set per surface, sized to match its pixel grid (`16`, `20`, or `24` px) and proportional to adjacent text (better-ui/icons#5).
- **`better-ui/icons#6`** (`rubric:iconography`): Use a single consistent SVG vector icon set per surface, sized to match its pixel grid (`16`, `20`, or `24` px) and proportional to adjacent text (better-ui/icons#6).

## better-writing

- **`better-writing/SKILL#1`** (`skill:generate-design/content`): Keep product terminology, noun/verb pairs, and capitalization consistent across all frame labels and headings (better-writing/SKILL#1).
- **`better-writing/SKILL#2`** (`skill:generate-design/content`): Keep product terminology, noun/verb pairs, and capitalization consistent across all frame labels and headings (better-writing/SKILL#2).
- **`better-writing/SKILL#3`** (`skill:generate-design/content`): Keep product terminology, noun/verb pairs, and capitalization consistent across all frame labels and headings (better-writing/SKILL#3).
- **`better-writing/SKILL#4`** (`lint:placeholder-text`): Write concrete, action-specific UI copy (verb + object on buttons and confirmation titles) and never leave generic placeholder strings; check with `designos_lint` placeholder-text (better-writing/SKILL#4).
- **`better-writing/SKILL#5`** (`lint:placeholder-text`): Write concrete, action-specific UI copy (verb + object on buttons and confirmation titles) and never leave generic placeholder strings; check with `designos_lint` placeholder-text (better-writing/SKILL#5).
- **`better-writing/SKILL#6`** (`lint:placeholder-text`): Write concrete, action-specific UI copy (verb + object on buttons and confirmation titles) and never leave generic placeholder strings; check with `designos_lint` placeholder-text (better-writing/SKILL#6).
- **`better-writing/SKILL#7`** (`lint:placeholder-text`): Write concrete, action-specific UI copy (verb + object on buttons and confirmation titles) and never leave generic placeholder strings; check with `designos_lint` placeholder-text (better-writing/SKILL#7).

## build-design

- **`build-design/SKILL#1`** (`skill:designos-use/workflow`): Reconcile design frames against domain rules, component props, and lint checks before handoff (build-design/SKILL#1).
- **`build-design/SKILL#2`** (`check:surface.web-app.required-states`): Include explicit frames for loading, empty, error, and disabled states alongside the default state.
- **`build-design/SKILL#3`** (`skill:designos-use/workflow`): Reconcile design frames against domain rules, component props, and lint checks before handoff (build-design/SKILL#3).
- **`build-design/figma#1`** (`lint:hardcoded-value`): Bind fills, strokes, spacing, and radii to semantic document variables (`var:`) instead of raw literals; verify with `designos_lint` hardcoded-value.
- **`build-design/figma#2`** (`lint:detached-instance`): Instantiate published library and local components rather than detaching or duplicating raw layer trees; verify with `designos_lint` detached-instance.
- **`build-design/figma#3`** (`lint:absolute-in-auto-layout`): Express spacing intent with Auto-Layout gap and padding rather than absolute child positioning; verify with `designos_lint` absolute-in-auto-layout.
- **`build-design/figma#4`** (`skill:designos-use/workflow`): Store raster and SVG assets directly in the document asset store rather than referencing ephemeral external URLs.

## explain-interface

- **`explain-interface/find-the-effect#1`** (`lint:overflow-parent`): Compare child bounds against the parent frame box to catch unintentional overflow; verify with `designos_lint` overflow-parent.
- **`explain-interface/find-the-effect#2`** (`lint:contrast`): Account for paint order, background blur, and blend modes when inspecting visual hierarchy and resolved backdrop contrast (explain-interface/find-the-effect#2).
- **`explain-interface/find-the-effect#3`** (`lint:contrast`): Account for paint order, background blur, and blend modes when inspecting visual hierarchy and resolved backdrop contrast (explain-interface/find-the-effect#3).
- **`explain-interface/find-the-effect#4`** (`lint:contrast`): Account for paint order, background blur, and blend modes when inspecting visual hierarchy and resolved backdrop contrast (explain-interface/find-the-effect#4).
- **`explain-interface/from-an-image#1`** (`rubric:color`): Evaluate palette ramps by perceptual OKLCH lightness progression, constant hue, and subtle neutral tinting (explain-interface/from-an-image#1).
- **`explain-interface/from-an-image#2`** (`rubric:color`): Evaluate palette ramps by perceptual OKLCH lightness progression, constant hue, and subtle neutral tinting (explain-interface/from-an-image#2).
- **`explain-interface/from-an-image#3`** (`rubric:color`): Evaluate palette ramps by perceptual OKLCH lightness progression, constant hue, and subtle neutral tinting (explain-interface/from-an-image#3).
- **`explain-interface/from-an-image#4`** (`lint:contrast`): Measure foreground/background contrast across every text and control pair on the canvas using `designos_lint` contrast.
- **`explain-interface/from-an-image#5`** (`rubric:color`): Evaluate palette ramps by perceptual OKLCH lightness progression, constant hue, and subtle neutral tinting (explain-interface/from-an-image#5).
- **`explain-interface/from-an-image#6`** (`rubric:typography`): Audit typeface category, modular type scale steps, weight separation, and 60–75 character line measure on the canvas (explain-interface/from-an-image#6).
- **`explain-interface/from-an-image#7`** (`rubric:typography`): Audit typeface category, modular type scale steps, weight separation, and 60–75 character line measure on the canvas (explain-interface/from-an-image#7).
- **`explain-interface/from-an-image#8`** (`rubric:typography`): Audit typeface category, modular type scale steps, weight separation, and 60–75 character line measure on the canvas (explain-interface/from-an-image#8).
- **`explain-interface/from-an-image#9`** (`rubric:typography`): Audit typeface category, modular type scale steps, weight separation, and 60–75 character line measure on the canvas (explain-interface/from-an-image#9).
- **`explain-interface/from-an-image#10`** (`rubric:typography`): Audit typeface category, modular type scale steps, weight separation, and 60–75 character line measure on the canvas (explain-interface/from-an-image#10).
- **`explain-interface/from-an-image#11`** (`check:surface.web-app.required-states`): Verify interactive state coverage (default, hover, focus, disabled, loading, empty, error) and light/dark appearance frames (explain-interface/from-an-image#11).
- **`explain-interface/from-an-image#12`** (`check:surface.web-app.required-states`): Verify interactive state coverage (default, hover, focus, disabled, loading, empty, error) and light/dark appearance frames (explain-interface/from-an-image#12).
- **`explain-interface/from-an-image#13`** (`check:surface.web-app.required-states`): Verify interactive state coverage (default, hover, focus, disabled, loading, empty, error) and light/dark appearance frames (explain-interface/from-an-image#13).
- **`explain-interface/from-an-image#14`** (`check:surface.web-app.required-states`): Verify interactive state coverage (default, hover, focus, disabled, loading, empty, error) and light/dark appearance frames (explain-interface/from-an-image#14).
- **`explain-interface/from-an-image#15`** (`check:surface.web-app.required-states`): Verify interactive state coverage (default, hover, focus, disabled, loading, empty, error) and light/dark appearance frames (explain-interface/from-an-image#15).

## interface-review

- **`interface-review/SKILL#11`** (`check:surface.web-app.required-states`): Check that new components and screens include complete state matrices (empty, loading, error, disabled) and consistent peer controls (interface-review/SKILL#11).
- **`interface-review/SKILL#13`** (`check:surface.web-app.required-states`): Check that new components and screens include complete state matrices (empty, loading, error, disabled) and consistent peer controls (interface-review/SKILL#13).
- **`interface-review/SKILL#14`** (`check:surface.web-app.required-states`): Check that new components and screens include complete state matrices (empty, loading, error, disabled) and consistent peer controls (interface-review/SKILL#14).
- **`interface-review/removed-signals#5`** (`lint:hardcoded-color`): Replacing a raw hex colour literal with a bound variable token (`var:`) of the same value is a valid tokenisation fix.

## state-machine

- **`state-machine/switcher#1`** (`skill:generate-library/components`): Organize component state matrices and showcase frames in a consistent, labeled group order with clear headings (state-machine/switcher#1).
- **`state-machine/switcher#2`** (`skill:generate-library/components`): Organize component state matrices and showcase frames in a consistent, labeled group order with clear headings (state-machine/switcher#2).
- **`state-machine/switcher#10`** (`skill:generate-library/components`): Organize component state matrices and showcase frames in a consistent, labeled group order with clear headings (state-machine/switcher#10).
- **`state-machine/switcher#11`** (`skill:generate-library/components`): Organize component state matrices and showcase frames in a consistent, labeled group order with clear headings (state-machine/switcher#11).

## technical-drawing

- **`technical-drawing/SKILL#1`** (`rubric:layout`): Construct technical vector illustrations from outer silhouette and major volumes down to functional joint details, removing extraneous construction lines (technical-drawing/SKILL#1).
- **`technical-drawing/SKILL#2`** (`rubric:layout`): Construct technical vector illustrations from outer silhouette and major volumes down to functional joint details, removing extraneous construction lines (technical-drawing/SKILL#2).
- **`technical-drawing/SKILL#3`** (`rubric:layout`): Construct technical vector illustrations from outer silhouette and major volumes down to functional joint details, removing extraneous construction lines (technical-drawing/SKILL#3).
- **`technical-drawing/SKILL#4`** (`rubric:layout`): Construct technical vector illustrations from outer silhouette and major volumes down to functional joint details, removing extraneous construction lines (technical-drawing/SKILL#4).

## variant

- **`variant/SKILL#4`** (`lint:clipped-partial`): Ensure variant frames have zero clipping or sibling overlap across target widths; check with `designos_lint` clipped-partial and sibling-overlap.
- **`variant/SKILL#5`** (`lint:contrast`): Every explored variant must pass `designos_lint` WCAG contrast on body and control text against its resolved backdrop.
- **`variant/SKILL#6`** (`rubric:consistency`): Keep explored design variants grounded in the active design system's semantic tokens, density, and surface hierarchy (variant/SKILL#6).
- **`variant/SKILL#7`** (`rubric:consistency`): Keep explored design variants grounded in the active design system's semantic tokens, density, and surface hierarchy (variant/SKILL#7).
- **`variant/SKILL#8`** (`lint:text-truncated`): Avoid unrecoverable text truncation in variant frames; check with `designos_lint` text-truncated.
- **`variant/SKILL#9`** (`rubric:consistency`): Keep explored design variants grounded in the active design system's semantic tokens, density, and surface hierarchy (variant/SKILL#9).
- **`variant/SKILL#10`** (`rubric:consistency`): Keep explored design variants grounded in the active design system's semantic tokens, density, and surface hierarchy (variant/SKILL#10).
- **`variant/SKILL#11`** (`rubric:consistency`): Keep explored design variants grounded in the active design system's semantic tokens, density, and surface hierarchy (variant/SKILL#11).
- **`variant/SKILL#12`** (`rubric:consistency`): Keep explored design variants grounded in the active design system's semantic tokens, density, and surface hierarchy (variant/SKILL#12).
- **`variant/SKILL#13`** (`lint:hardcoded-value`): Bind colour, spacing, radius, and typography in every variant to document variables (`var:`); check with `designos_lint` hardcoded-value.
- **`variant/SKILL#14`** (`rubric:consistency`): Keep explored design variants grounded in the active design system's semantic tokens, density, and surface hierarchy (variant/SKILL#14).
- **`variant/SKILL#15`** (`rubric:consistency`): Keep explored design variants grounded in the active design system's semantic tokens, density, and surface hierarchy (variant/SKILL#15).
- **`variant/picker#1`** (`skill:generate-design/variants`): Lay out side-by-side variant comparison frames in a labeled, ordered grid on the canvas (variant/picker#1).
- **`variant/picker#2`** (`skill:generate-design/variants`): Lay out side-by-side variant comparison frames in a labeled, ordered grid on the canvas (variant/picker#2).
- **`variant/picker#10`** (`skill:generate-design/variants`): Lay out side-by-side variant comparison frames in a labeled, ordered grid on the canvas (variant/picker#10).
- **`variant/picker#11`** (`skill:generate-design/variants`): Lay out side-by-side variant comparison frames in a labeled, ordered grid on the canvas (variant/picker#11).
