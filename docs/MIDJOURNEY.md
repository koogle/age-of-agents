# Midjourney asset tools

The project's FAL generation scripts remain available under `assets/*/tools/`.
For Midjourney, use the unofficial [ehmo/mj](https://github.com/ehmo/mj) CLI and
MCP server with your own account. The installer pins version **0.11.1** and
verifies the release archive against the published SHA-256 checksum.

```bash
bash scripts/install_midjourney.sh
codex mcp add midjourney -- "$HOME/.local/bin/mj-mcp"
```

Restart the Codex session after adding an MCP server. On a machine with a visible
browser, sign in with Google or Discord, wait for the Midjourney Create page,
then press Enter in the terminal:

```bash
"$HOME/.local/bin/mj" login --i-understand
"$HOME/.local/bin/mj" doctor
```

The `--i-understand` flag acknowledges mj's warning that Midjourney prohibits
automated access and may ban accounts. The first browser command downloads
Camoufox. Login stays in the tool's local browser profile; do not copy account
tokens or profiles into this repository.

For a standalone candidate asset:

```bash
"$HOME/.local/bin/mj" imagine "small Greek marble town hall, clean straight terracotta gable roof, soft two-tone cel shading, thin pen lines, three-quarter RTS view, white background, no text --ar 1:1" --wait --download /tmp/towncenter-candidates
```

Review candidates at gameplay size before replacing an atlas. Preserve the
existing frame rectangles, footprint anchor, scale, camera and construction
states in `assets/sprites/towncenter.json`.

## Historical cloud setup

The following records an earlier environment and is not a current readiness check. Consult [the asset guide](knowledge/asset-pipeline.md) and inspect the active environment before using these paths or treating a provider as blocked.

Both binaries were installed in `/workspace/bin` and the `midjourney` MCP entry
points to `/workspace/bin/mj-mcp`. Installation and registration are verified;
the MCP file root is restricted to `/workspace/age-of-agents/assets` so reference
images can be uploaded without exposing unrelated local files.
Browser setup and account login are incomplete. This environment's enforced
network policy blocks `midjourney.com` and `fal.run`, despite a configured FAL
key. It also lacks a visible login browser. Complete sign-in on a desktop with
network access, or configure the cloud environment to allow the provider and
its authentication, browser-download and asset-hosting domains. Do not bypass
the environment proxy.
