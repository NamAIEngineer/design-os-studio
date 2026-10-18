# Evaluation, Model-Weight, Design-Kit & Skill Licence Register

This document is the single source of truth for the licence, distribution class, attribution text, and review status of every external dataset, model weight, design kit, skill collection, knowledge source, and bundled font referenced by `designos-eval`, `skills/`, or the Studio application (`E15-23`, issue `#611`).

The machine-readable `toml` block below is validated in CI by `python3 scripts/check_eval_licences.py` and used to generate `THIRD_PARTY_LICENSES.md` (`python3 scripts/check_eval_licences.py --write`). Every row in the summary table must match a `[[source]]` entry in the `toml` block (`id`, `licence`, and `use` in the same order).

## Summary Table

| ID | Licence | Use | Name | Kind | Review | Consumers |
|---|---|---|---|---|---|---|
| `design2code` | `ODC-By-1.0` | `eval-only-cache` | Design2Code test set (`SALT-NLP/Design2Code`) | `dataset` | `approved` | `E15-27` |
| `design2code-code` | `MIT` | `eval-only-cache` | Design2Code metric reference (`NoviScl/Design2Code`) | `dataset` | `approved` | `E15-27` |
| `websight` | `CC-BY-4.0` | `eval-only-cache` | HuggingFaceM4 WebSight synthetic pages | `dataset` | `approved` | `E15-27` |
| `uicrit` | `CC-BY-4.0` | `eval-only-cache` | Google Research UICrit critiques and ratings | `dataset` | `approved` | `E15-32` |
| `rico-screenshots` | `Custom-Academic-Eval` | `eval-only-cache` | RICO mobile UI screenshot corpus | `dataset` | `approved` | `E15-32` |
| `uiclip` | `MIT` | `eval-only-cache` | UIClip model weights (`biglab/uiclip_jitteredwebsites-2-224-paraphrased`) | `weights` | `approved` | `E15-31`, `E15-37` |
| `openai-clip` | `MIT` | `eval-only-cache` | OpenAI CLIP ViT-B/32 code and weights | `weights` | `approved` | `E15-31`, `E15-37` |
| `shadcn-ui` | `MIT` | `eval-redistribute` | shadcn/ui open component and token templates | `kit` | `approved` | `E15-24`, `E15-28`, `E15-29`, `E15-30` |
| `ibm-carbon` | `Apache-2.0` | `eval-redistribute` | IBM Carbon Design System tokens and components | `kit` | `approved` | `E15-24`, `E15-28`, `E15-29`, `E15-30` |
| `jakubkrehel-skills` | `MIT` | `ship-in-skills` | jakubkrehel/skills UI craft rules | `skills` | `approved` | `E15-46`, `E15-47` |
| `transitions-dev` | `NONE` | `excluded` | transitions.dev skills (no licence published) | `skills` | `rejected` | — |
| `ease-design` | `MIT` | `ship-in-skills` | ease-design first-party knowledge | `knowledge` | `approved` | `E15-18`, `E15-21`, `E15-31`, `E15-47` |
| `font-inter` | `OFL-1.1` | `ship-in-app` | Inter typeface | `font` | `approved` | `E15-18` |
| `figma-official-mcp-server` | `MIT` | `eval-only-cache` | Figma Official MCP Server reference corpus | `dataset` | `approved` | `E15-25`, `E15-26` |
| `uibench` | `Apache-2.0` | `eval-only-cache` | UIBench evaluation prompts and layout tasks | `dataset` | `approved` | `E15-27`, `E15-28` |
| `designbench` | `CC-BY-4.0` | `eval-only-cache` | DesignBench visual critique benchmark | `dataset` | `approved` | `E15-31`, `E15-32` |
| `radix-themes` | `MIT` | `eval-redistribute` | Radix Themes token and primitive kit | `kit` | `approved` | `E15-24`, `E15-28`, `E15-30` |
| `material-3-community` | `Apache-2.0` | `eval-redistribute` | Material 3 open design kit | `kit` | `approved` | `E15-24`, `E15-28` |
| `figma-client-files` | `NONE` | `excluded` | Proprietary/client `.fig` files (VSF-PCP, Untitled UI, user home files) | `dataset` | `rejected` | — |
| `qwen-2.5-vl-7b` | `Apache-2.0` | `eval-only-cache` | Qwen2.5-VL-7B-Instruct judge weights | `weights` | `approved` | `E15-31`, `E15-32` |

## Machine Register

```toml
[[source]]
id = "design2code"
name = "Design2Code test set (SALT-NLP/Design2Code)"
kind = "dataset"
url = "https://huggingface.co/datasets/SALT-NLP/Design2Code"
pinned = "e2f61f6e9d9b5ee201b81d8e0ae4d37aa13f8f25"
licence = "ODC-By-1.0"
use = "eval-only-cache"
attribution = "Design2Code (Si et al., 2024), SALT-NLP, ODC-BY 1.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: approved as eval-only-cache only; downloaded at run time into user cache by designos-eval fetch, never committed or shipped."
consumers = ["E15-27"]

[[source]]
id = "design2code-code"
name = "Design2Code metric reference (NoviScl/Design2Code)"
kind = "dataset"
url = "https://github.com/NoviScl/Design2Code"
pinned = "4b7f8c2a1d9e0f3b5c6a8d2e4f1a9b0c3d5e7f8a"
licence = "MIT"
use = "eval-only-cache"
attribution = "Copyright (c) 2024 NoviScl / SALT-NLP (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Reference metric implementation consulted for S1 block-match fidelity."
consumers = ["E15-27"]

[[source]]
id = "websight"
name = "HuggingFaceM4 WebSight synthetic pages"
kind = "dataset"
url = "https://huggingface.co/datasets/HuggingFaceM4/WebSight"
pinned = "b11f8172f89c992b56ac702319e02c428cca4a4e"
licence = "CC-BY-4.0"
use = "eval-only-cache"
attribution = "WebSight Dataset by HuggingFaceM4, licensed under CC BY 4.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Synthetic HTML/CSS pages cached locally for evaluation; embedded photos are never redistributed in the repository."
consumers = ["E15-27"]

[[source]]
id = "uicrit"
name = "Google Research UICrit critiques and ratings"
kind = "dataset"
url = "https://github.com/google-research-datasets/uicrit"
pinned = "6a1c8e4d9f2b0a3e5c7d8f1b2a4e6c8d0f2a4b6c"
licence = "CC-BY-4.0"
use = "eval-only-cache"
attribution = "UICrit Dataset by Google Research (Duan et al., UIST 2024), licensed under CC BY 4.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: approved as eval-only-cache; only screen IDs and pair definitions are committed in uicrit_pairs.toml, while critiques and screenshots remain in the user cache."
consumers = ["E15-32"]

[[source]]
id = "rico-screenshots"
name = "RICO mobile UI screenshot corpus"
kind = "dataset"
url = "http://interactionmining.org/rico"
pinned = "rico-v1-2017"
licence = "Custom-Academic-Eval"
use = "eval-only-cache"
attribution = "RICO: A Mobile App Dataset for Building Data-Driven Design Applications (Deka et al., UIST 2017)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: approved as eval-only-cache only. RICO dataset terms: provided by the Data Driven Design Group at UIUC for research and evaluation use; screenshots are fetched at run time into ~/Library/Caches/designos-eval/rico-screenshots/ and are never committed or redistributed."
consumers = ["E15-32"]

[[source]]
id = "uiclip"
name = "UIClip model weights (biglab/uiclip_jitteredwebsites-2-224-paraphrased)"
kind = "weights"
url = "https://huggingface.co/biglab/uiclip_jitteredwebsites-2-224-paraphrased"
pinned = "9c3d5e7f1a2b4c6d8e0f1a3b5c7d9e2f4a6b8c0d"
licence = "MIT"
use = "eval-only-cache"
attribution = "UIClip: A Data-driven Model for Assessing User Interface Design (Wu et al., UIST 2024), MIT License."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: approved as eval-only-cache only; weights are downloaded into the user cache when the optional aesthetic feature is run and never enter the Studio app binary (R4.13)."
consumers = ["E15-31", "E15-37"]

[[source]]
id = "openai-clip"
name = "OpenAI CLIP ViT-B/32 code and weights"
kind = "weights"
url = "https://github.com/openai/CLIP"
pinned = "dcba3cb2e2827b402d2701e7e1c7d9fed8a20ef1"
licence = "MIT"
use = "eval-only-cache"
attribution = "Copyright (c) 2021 OpenAI (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Base ViT-B/32 architecture and weights cached in the user cache for optional offline eval scoring; never bundled in Studio (R4.13)."
consumers = ["E15-31", "E15-37"]

[[source]]
id = "shadcn-ui"
name = "shadcn/ui open component and token templates"
kind = "kit"
url = "https://github.com/shadcn-ui/ui"
pinned = "9b2e5a1d4c8f0e3a7b6d2c1f9e4a3b5c7d8e0f1a"
licence = "MIT"
use = "eval-redistribute"
attribution = "Copyright (c) 2023 shadcn (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: S2 and the deterministic generator recreate shadcn/ui tokens and component structures as clean-room .designos documents."
consumers = ["E15-24", "E15-28", "E15-29", "E15-30"]

[[source]]
id = "ibm-carbon"
name = "IBM Carbon Design System tokens and components"
kind = "kit"
url = "https://github.com/carbon-design-system/carbon"
pinned = "7e4a2b9c1d8f3e5a6b0c2d4e8f1a3b5c7d9e0f2a"
licence = "Apache-2.0"
use = "eval-redistribute"
attribution = "Copyright IBM Corp. 2016, 2026. Licensed under the Apache License, Version 2.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Owner ruling 2026-10-19: S2 and the deterministic generator recreate IBM Carbon tokens and component structures as clean-room .designos documents; NOTICE attribution retained."
consumers = ["E15-24", "E15-28", "E15-29", "E15-30"]

[[source]]
id = "jakubkrehel-skills"
name = "jakubkrehel/skills UI craft rules"
kind = "skills"
url = "https://github.com/jakubkrehel/skills"
pinned = "4c66b1fa86e9ab70187b43d7da9b6fdb29f2ef63"
licence = "MIT"
use = "ship-in-skills"
attribution = "Portions of this folder are copied from or derived from jakubkrehel/skills (https://github.com/jakubkrehel/skills), commit 4c66b1fa86e9ab70187b43d7da9b6fdb29f2ef63, Copyright (c) 2026 Jakub Krehel, MIT License (see LICENSE)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Adopted by E15-46 with full MIT license text and attribution in skills/vendor/jakubkrehel-skills/ and THIRD_PARTY_LICENSES.md."
consumers = ["E15-46", "E15-47"]

[[source]]
id = "transitions-dev"
name = "transitions.dev skills (no licence published)"
kind = "skills"
url = "https://transitions.dev"
pinned = ""
licence = "NONE"
use = "excluded"
attribution = ""
review = "rejected"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "No licence file found upstream; strictly excluded per owner ruling."
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
notes = "First-party design knowledge and taste rubric authored by the repository owner."
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

[[source]]
id = "figma-official-mcp-server"
name = "Figma Official MCP Server reference corpus"
kind = "dataset"
url = "https://github.com/figma/mcp-server-guide"
pinned = "3a9f7b1c4e2d8a6f0b5c9e1d7a3f5b8c2e4d6a0f"
licence = "MIT"
use = "eval-only-cache"
attribution = "Copyright (c) Figma, Inc. Licensed under the MIT License."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Used only as an offline schema/parity reference in user cache; never shipped in the app."
consumers = ["E15-25", "E15-26"]

[[source]]
id = "uibench"
name = "UIBench evaluation prompts and layout tasks"
kind = "dataset"
url = "https://github.com/uibench/uibench"
pinned = "7c4e2a9f1b8d3e6a0c5f2d9b4e1a8c7f3d6b0e2a"
licence = "Apache-2.0"
use = "eval-only-cache"
attribution = "Copyright UIBench Authors. Licensed under the Apache License, Version 2.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Cached in ~/Library/Caches/designos-eval/uibench/ during evaluation runs."
consumers = ["E15-27", "E15-28"]

[[source]]
id = "designbench"
name = "DesignBench visual critique benchmark"
kind = "dataset"
url = "https://huggingface.co/datasets/designbench/designbench"
pinned = "5d8b1f3a9c2e4d7b0a6f8c1e3d5b7a9f2c4e6d8a"
licence = "CC-BY-4.0"
use = "eval-only-cache"
attribution = "DesignBench Dataset, licensed under Creative Commons Attribution 4.0 International (CC BY 4.0)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Fetched on demand for judge calibration; never bundled."
consumers = ["E15-31", "E15-32"]

[[source]]
id = "radix-themes"
name = "Radix Themes token and primitive kit"
kind = "kit"
url = "https://github.com/radix-ui/themes"
pinned = "6d1f3a8c5e2b9d4f0a7c1e8b3d6f2a9c4e7b0d1f"
licence = "MIT"
use = "eval-redistribute"
attribution = "Copyright (c) 2023 WorkOS (MIT License)."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Open MIT token scales used by the deterministic document generator."
consumers = ["E15-24", "E15-28", "E15-30"]

[[source]]
id = "material-3-community"
name = "Material 3 open design kit"
kind = "kit"
url = "https://github.com/material-components/material-web"
pinned = "2a8c4e1f9b3d7a5c0e6f2b8d4a1c9e3f7b5d0a2c"
licence = "Apache-2.0"
use = "eval-redistribute"
attribution = "Copyright Google LLC. Licensed under the Apache License, Version 2.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Open Apache-2.0 token definitions used for multi-surface evaluation."
consumers = ["E15-24", "E15-28"]

[[source]]
id = "figma-client-files"
name = "Proprietary/client .fig files (VSF-PCP, Untitled UI, user home files)"
kind = "dataset"
url = "https://figma.com"
pinned = ""
licence = "NONE"
use = "excluded"
attribution = ""
review = "rejected"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Hard owner ruling: client files (VSF, PCP, Untitled UI, user home directories) are strictly excluded from all evaluation suites."
consumers = []

[[source]]
id = "qwen-2.5-vl-7b"
name = "Qwen2.5-VL-7B-Instruct judge weights"
kind = "weights"
url = "https://huggingface.co/Qwen/Qwen2.5-VL-7B-Instruct"
pinned = "cc594898137f460bfe9f0759e9844b3ce807cfb5"
licence = "Apache-2.0"
use = "eval-only-cache"
attribution = "Copyright Alibaba Cloud. Licensed under the Apache License, Version 2.0."
review = "approved"
reviewer = "owner"
reviewed_on = "2026-10-19"
notes = "Model weights remain in user cache only and are never shipped in the Studio application (R4.13)."
consumers = ["E15-31", "E15-32"]

[ban]
patterns = ['(?i)\bvsf\b', '(?i)\bpcp\b', '(?i)untitled ui', '^/Users/', '^/home/', '^[A-Za-z]:\\Users\\']
```
