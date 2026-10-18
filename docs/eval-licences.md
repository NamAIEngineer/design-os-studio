# Evaluation, Model Weight, Design Kit & Skill Licence Register

This register is the single source of truth for every dataset, model weight, design kit, skill, knowledge repository, and font referenced by `designos-eval` and the `design:os` skills pack (Issue #611 / Task E15-23).

- Run `python3 scripts/check_eval_licences.py` to validate the register, verify `THIRD_PARTY_LICENSES.md`, and enforce the client-file ban.
- Run `python3 scripts/check_eval_licences.py --write` after editing any `ship-*` entry to regenerate `THIRD_PARTY_LICENSES.md`.

## Summary Table

| ID | Name | Kind | Licence | Use | Review | Pinned |
|---|---|---|---|---|---|---|
| `design2code` | Design2Code test set | `dataset` | `ODC-By-1.0` | `eval-only-cache` | `approved` | `e2f61f6e9d9b5ee201b81d8e0ae4d37aa13f8f25` |
| `design2code-code` | Design2Code metric reference code | `dataset` | `MIT` | `eval-only-cache` | `approved` | `3a0e4b906c4d5b683b659a4a9d82c4f8f5437c6d` |
| `websight` | WebSight synthetic web pages | `dataset` | `CC-BY-4.0` | `eval-only-cache` | `approved` | `b11f8172f89c992b56ac702319e02c428cca4a4e` |
| `uicrit` | UICrit critiques and ratings | `dataset` | `CC-BY-4.0` | `eval-only-cache` | `approved` | `6f14a5620c9a5e63f6f7b92c7a483a71f72b5e41` |
| `rico-screenshots` | RICO mobile UI screenshots | `dataset` | `Custom-Academic-Eval` | `eval-only-cache` | `approved` | `rico-v1-2017` |
| `uiclip-weights` | UIClip model weights | `weights` | `MIT` | `eval-only-cache` | `approved` | `a4d98d12b5e04c71f918e3b2f6c8a10d4e219c03` |
| `openai-clip` | OpenAI CLIP ViT-B/32 weights | `weights` | `MIT` | `eval-only-cache` | `approved` | `dcba3cb2e2827b402d2701e7e1c7d9fed8a20ef1` |
| `shadcn-ui` | shadcn/ui design system reference | `kit` | `MIT` | `eval-redistribute` | `approved` | `9b2e5e32d18f74d82e39a8e17c519a6b4f08d312` |
| `ibm-carbon` | IBM Carbon Design System | `kit` | `Apache-2.0` | `eval-redistribute` | `approved` | `e8c41d75f0218a93c7e5d2104b9f6a12c38d90ef` |
| `jakubkrehel-skills` | jakubkrehel/skills UI craft rules | `skills` | `MIT` | `ship-in-skills` | `approved` | `4c66b1fa86e9ab70187b43d7da9b6fdb29f2ef63` |
| `transitions-dev` | transitions.dev skills | `skills` | `NONE` | `excluded` | `rejected` | `` |
| `ease-design` | ease-design first-party knowledge | `knowledge` | `MIT` | `ship-in-skills` | `approved` | `1f8a9c42e7b03d61a5c89e204b7d1e3f5a902c14` |
| `font-inter` | Inter typeface | `font` | `OFL-1.1` | `ship-in-app` | `approved` | `v4.0` |

## Machine-Readable Register

```toml
[[source]]
id = "design2code"
name = "Design2Code test set"
kind = "dataset"
url = "https://huggingface.co/datasets/SALT-NLP/Design2Code"
pinned = "e2f61f6e9d9b5ee201b81d8e0ae4d37aa13f8f25"
licence = "ODC-By-1.0"
use = "eval-only-cache"
attribution = "Design2Code (Si et al., 2024), SALT-NLP, ODC-BY 1.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: approved as eval-only-cache only; pages derive from third-party websites and are fetched into user cache at run time, never committed or shipped."
consumers = ["E15-27"]

[[source]]
id = "design2code-code"
name = "Design2Code metric reference code"
kind = "dataset"
url = "https://github.com/NoviScl/Design2Code"
pinned = "3a0e4b906c4d5b683b659a4a9d82c4f8f5437c6d"
licence = "MIT"
use = "eval-only-cache"
attribution = "Copyright (c) 2024 SALT-NLP / NoviScl (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Reference metric implementation for E15-27."
consumers = ["E15-27"]

[[source]]
id = "websight"
name = "WebSight synthetic web pages"
kind = "dataset"
url = "https://huggingface.co/datasets/HuggingFaceM4/WebSight"
pinned = "b11f8172f89c992b56ac702319e02c428cca4a4e"
licence = "CC-BY-4.0"
use = "eval-only-cache"
attribution = "WebSight (Laurencon et al., 2024), HuggingFaceM4, CC BY 4.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Synthetic HTML/CSS pages; kept in eval cache only until embedded photo sources are verified."
consumers = ["E15-27"]

[[source]]
id = "uicrit"
name = "UICrit critiques and ratings"
kind = "dataset"
url = "https://github.com/google-research-datasets/uicrit"
pinned = "6f14a5620c9a5e63f6f7b92c7a483a71f72b5e41"
licence = "CC-BY-4.0"
use = "eval-only-cache"
attribution = "UICrit (Duan et al., UIST 2024), Google Research, CC BY 4.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: approved as eval-only-cache; critiques and ratings are CC BY 4.0 while underlying RICO screenshots are governed by rico-screenshots."
consumers = ["E15-32"]

[[source]]
id = "rico-screenshots"
name = "RICO mobile UI screenshots"
kind = "dataset"
url = "http://interactionmining.org/rico"
pinned = "rico-v1-2017"
licence = "Custom-Academic-Eval"
use = "eval-only-cache"
attribution = "RICO: A Mobile App Dataset for Building Data-Driven Design Applications (Deka et al., UIST 2017)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: RICO dataset terms provide mobile UI screenshots for research/evaluation; raw screenshots are never redistributed or committed to this repo and remain strictly in local user eval cache."
consumers = ["E15-32"]

[[source]]
id = "uiclip-weights"
name = "UIClip model weights"
kind = "weights"
url = "https://huggingface.co/biglab/uiclip_jitteredwebsites-2-224-paraphrased"
pinned = "a4d98d12b5e04c71f918e3b2f6c8a10d4e219c03"
licence = "MIT"
use = "eval-only-cache"
attribution = "UIClip: A Data-driven Model for Assessing User Interface Design (Wu et al., UIST 2024), MIT License."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19 & R4.13: weights are downloaded to user cache for optional eval and never bundled in the application."
consumers = ["E15-31"]

[[source]]
id = "openai-clip"
name = "OpenAI CLIP ViT-B/32 weights"
kind = "weights"
url = "https://github.com/openai/CLIP"
pinned = "dcba3cb2e2827b402d2701e7e1c7d9fed8a20ef1"
licence = "MIT"
use = "eval-only-cache"
attribution = "Copyright (c) 2021 OpenAI (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "ViT-B/32 weights cached in user directory for eval only (R4.13)."
consumers = ["E15-31"]

[[source]]
id = "shadcn-ui"
name = "shadcn/ui design system reference"
kind = "kit"
url = "https://github.com/shadcn-ui/ui"
pinned = "9b2e5e32d18f74d82e39a8e17c519a6b4f08d312"
licence = "MIT"
use = "eval-redistribute"
attribution = "Copyright (c) 2023 shadcn (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Open design system used by S2 brief-to-screen and the deterministic large-document generator (E15-24, E15-28)."
consumers = ["E15-24", "E15-28"]

[[source]]
id = "ibm-carbon"
name = "IBM Carbon Design System"
kind = "kit"
url = "https://github.com/carbon-design-system/carbon"
pinned = "e8c41d75f0218a93c7e5d2104b9f6a12c38d90ef"
licence = "Apache-2.0"
use = "eval-redistribute"
attribution = "Copyright IBM Corp. 2016, 2024. Licensed under the Apache License, Version 2.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Open design system used by S2 brief-to-screen and the deterministic large-document generator (E15-24, E15-28). Preserve NOTICE if any upstream file is copied."
consumers = ["E15-24", "E15-28"]

[[source]]
id = "jakubkrehel-skills"
name = "jakubkrehel/skills UI craft rules"
kind = "skills"
url = "https://github.com/jakubkrehel/skills"
pinned = "4c66b1fa86e9ab70187b43d7da9b6fdb29f2ef63"
licence = "MIT"
use = "ship-in-skills"
attribution = "Copyright (c) 2026 Jakub Krehel (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Adopted by E15-46 with full MIT licence text in skills/vendor/ and THIRD_PARTY_LICENSES.md."
consumers = ["E15-46"]

[[source]]
id = "transitions-dev"
name = "transitions.dev skills"
kind = "skills"
url = "https://transitions.dev"
pinned = ""
licence = "NONE"
use = "excluded"
attribution = ""
review = "rejected"
reviewer = "owner"
reviewed_on = "2026-10-18"
notes = "No licence file found; excluded per owner ruling."
consumers = []

[[source]]
id = "ease-design"
name = "ease-design first-party knowledge"
kind = "knowledge"
url = "https://github.com/jangtrinh/ease-design"
pinned = "1f8a9c42e7b03d61a5c89e204b7d1e3f5a902c14"
licence = "MIT"
use = "ship-in-skills"
attribution = "Copyright (c) 2026 ease-design contributors (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner's own design knowledge base; source for E15-18, E15-21, E15-31, and E15-47."
consumers = ["E15-18", "E15-21", "E15-31", "E15-47"]

[[source]]
id = "font-inter"
name = "Inter typeface"
kind = "font"
url = "https://github.com/rsms/inter"
pinned = "v4.0"
licence = "OFL-1.1"
use = "ship-in-app"
attribution = "Copyright 2020 The Inter Project Authors (SIL Open Font License 1.1)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Bundled UI and default canvas font; additional archetype fonts are fetched on demand by designos-fonts."
consumers = ["E15-18"]

[ban]
patterns = ['(?i)\bvsf\b', '(?i)\bpcp\b', '(?i)untitled ui', '^/Users/', '^/home/', '^[A-Za-z]:\\Users\\']
```
