---
description: "ease-design ui-init — Initialise ease-design for this project"
---

# ui-init

Run the ease-design initialiser for this runtime:

// turbo
```bash
ui init --runtime antigravity
```

Pass `--force` to overwrite an existing installation.

After `ui init` finishes:
1. Run `ui onboard` and walk its checklist with the user.
2. For any pending step, ask the user's approval before running the suggested
   setup/install command — never install silently. `ui` only reports what is
   missing; the host agent is the one that acts.
3. Show `ui guide` so the user sees what design:os can do.
4. For the full sequence (entry-point routing, soul, heartbeat, Figma), defer
   to the `onboard` journey skill.
