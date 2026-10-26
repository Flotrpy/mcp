# Site

A static website and dashboard for the five servers. There's no framework, build step or runtime dependency: just HTML, modern CSS and a few small ES modules.

```sh
cd site && npm run serve     # http://localhost:8000
npm test                     # JS unit tests (sag formula parity, config generators, search)
```

**Live:** https://flotrpy.github.io/mcp/ (GitHub Pages, served from the `gh-pages` branch). `.github/workflows/pages.yml` republishes it when `site/` changes on `main`. To publish by hand, run `sh site/tools/build-pages.sh dist` and push `dist/` to `gh-pages`.

Alternatively, deploy to Vercel by pointing a project's root at `site/`. `vercel.json` sets clean URLs, caching and a strict Content Security Policy (no inline scripts or styles).

## Pages
- **`index.html`:**
  - Hero with a replay of **real recorded sessions**.
  - Server bento, with the joinery tile showing a real cut diagram.
  - **Shelf-sag calculator** (a port of joinery's formula, with a draggable support driven by a spring).
  - **Config builder** for Claude Code, Claude Desktop, Cursor and Codex, with URL state.
  - Live grade scoreboard.
  - **⌘K palette**.
- **`dashboard.html`:** the latest lighthouse audit, with grades, protocol support, handshake latency, lockfile status and token cost per tool. Every chart has a text description and a data table.

## Data
`data/transcripts.json`, `data/dashboard.json` and `data/joinery-layout.svg` are generated from the release binaries by `tools/record.py`. Nothing on the site is hand-written output.

## Design rules
The rules are recorded in [`docs/UI-RESEARCH.md`](../docs/UI-RESEARCH.md):
- **Glass:** only on functional chrome (nav, palette).
- **Motion:** springs only for direct manipulation; CSS transitions under 300 ms for everything else.
- **Tracking:** size-specific.
- **Hit targets:** 44 px on touch.
- **Status:** never colour alone.
- **Accessibility preferences:** reduced motion, reduced transparency and increased contrast are all honoured.
