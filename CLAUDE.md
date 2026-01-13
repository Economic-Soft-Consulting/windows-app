# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

eSoft Facturi - A Windows desktop invoice management application built with Tauri 2 and Next.js 16. The app manages invoices for partners with offline-first SQLite storage and sync capabilities.

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
- **Backend Process**: Rust binary (`src-tauri/`) handles SQLite database, native OS operations, and auto-updates

### Communication Pattern
Frontend and backend communicate via Tauri commands:
- Rust defines commands with `#[tauri::command]` in `src-tauri/src/commands.rs`
- Frontend calls commands via `invoke<T>("command_name", { args })` from `@tauri-apps/api/core`
- TypeScript wrappers in `lib/tauri/commands.ts` provide typed interfaces

### Key Directories
```
app/                         # Next.js App Router
├── (dashboard)/            # Dashboard layout group
│   ├── invoices/           # Invoice list and creation pages
│   ├── data/               # Partners & products view
│   └── layout.tsx          # Sidebar + header layout
├── components/             # Page-specific components
│   ├── invoices/           # Invoice cards, wizard steps
│   ├── layout/             # Sidebar, Header, NetworkIndicator
│   └── sync/               # SyncButton, FirstRunOverlay
components/ui/              # shadcn/ui components
hooks/                      # React hooks (useInvoices, usePartners, etc.)
lib/tauri/                  # TypeScript types and Tauri command wrappers
src-tauri/src/
├── lib.rs                  # Tauri app initialization
├── commands.rs             # All Tauri commands (sync, partners, products, invoices)
├── database.rs             # SQLite setup with rusqlite
├── models.rs               # Rust data structures
├── mock_api.rs             # Mock external service (50% failure rate for testing)
└── updater.rs              # Auto-update logic
```

### Data Flow
1. **First Run**: App shows overlay, fetches partners/products from mock API, stores in SQLite
2. **Subsequent Runs**: Uses cached SQLite data immediately, background sync if online
3. **Invoice Creation**: 4-step wizard (Partner → Location → Products → Review)
4. **Invoice Sending**: Saved to SQLite first, then attempted send with 50% mock failure rate

### Database Schema (SQLite via rusqlite)
- `partners` - Partner companies (synced from external service)
- `locations` - Partner locations (one-to-many with partners)
- `products` - Available products (synced from external service)
- `invoices` - Created invoices with status (pending/sending/sent/failed)
- `invoice_items` - Line items for each invoice
- `sync_metadata` - Tracks last sync timestamps

### UI Components
Uses shadcn/ui (New York style) with Tailwind CSS v4. Key components:
- Tabs, Cards, Dialog, AlertDialog for layout
- Table for invoice details
- RadioGroup for location selection
- Input with search for partner/product filtering
- Sonner for toast notifications

## CI/CD

GitHub Actions workflow (`.github/workflows/build.yml`) builds on Windows and creates GitHub releases with signed installers. Requires `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets.
