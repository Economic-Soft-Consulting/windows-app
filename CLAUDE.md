# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

eSoft Facturi - A Windows desktop invoice management application built with Tauri 2 and Next.js 16. The app bundles a Next.js frontend (compiled to static HTML) inside a Rust-based native window.

## Commands

```bash
# Development - starts Next.js dev server and Tauri window with hot reload
npm run tauri dev

# Build production app (creates Windows installer in src-tauri/target/release/bundle/)
npm run tauri build

# Frontend only (for testing React components without Tauri)
npm run dev          # Next.js dev server at localhost:3000
npm run build        # Build static export to /out

# Lint
npm run lint
```

## Architecture

### Two-Process Model
- **Frontend Process**: Next.js app compiled to static HTML (`/out` directory), rendered in a webview
- **Backend Process**: Rust binary (`src-tauri/`) handles native OS operations and auto-updates

### Communication Pattern
Frontend and backend communicate via Tauri's event system (not commands):
- Rust emits events using `app.emit("event-name", payload)`
- React listens with `listen<T>("event-name", callback)` from `@tauri-apps/api/event`

Currently implemented events: `update-checking`, `update-downloading`, `update-done`

### Key Directories
- `app/` - Next.js App Router pages and components
- `src-tauri/src/` - Rust source (lib.rs for app setup, updater.rs for auto-update logic)
- `src-tauri/tauri.conf.json` - App metadata, window config, updater endpoints

### Build Flow
1. `npm run build` compiles Next.js to static files in `/out`
2. Tauri bundles `/out` with the Rust binary
3. Output: Windows installer with auto-update capability

### Auto-Update System
Updates are fetched from GitHub Releases. The updater checks `latest.json` at the configured endpoint, downloads new versions in the background, and restarts the app. See `src-tauri/src/updater.rs` for implementation.

## CI/CD

GitHub Actions workflow (`.github/workflows/build.yml`) builds on Windows and creates GitHub releases with signed installers. Requires `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets.
