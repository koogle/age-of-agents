# Painted ground textures

One seamless, top-down painted texture per biome, sampled by `frontend/ground-paint.js` (`/assets/terrain/<name>.webp`, falling back to `.png`). Each name ships as a 512×512 `.webp` (the runtime file) plus the 1024×1024 `.png` master. One repeat covers 4×4 cells in game.

Work in progress: meadow, prairie and beach are done; the rest follow in the next commits. Full provenance lands with them.
