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
│   ├── settings/           # Settings page with printer configuration
│   └── layout.tsx          # Sidebar + header layout
├── components/             # Page-specific components
│   ├── invoices/           # Invoice cards, wizard steps
│   ├── layout/             # Sidebar, Header, NetworkIndicator
│   └── sync/               # SyncButton, FirstRunOverlay
components/ui/              # shadcn/ui components
hooks/                      # React hooks (useInvoices, usePartners, etc.)
lib/tauri/                  # TypeScript types and Tauri command wrappers
src-tauri/
├── resources/              # Bundled resources (SumatraPDF.exe for printing)
└── src/
    ├── lib.rs              # Tauri app initialization
    ├── commands.rs         # All Tauri commands (sync, partners, products, invoices, printing)
    ├── database.rs         # SQLite setup with rusqlite + migrations
    ├── models.rs           # Rust data structures
    ├── mock_api.rs         # Mock external service (50% failure rate for testing)
    ├── print_invoice.rs    # HTML invoice template generation (80mm thermal)
    └── updater.rs          # Auto-update logic
```

### Data Flow
1. **First Run**: App shows overlay, fetches partners/products from mock API, stores in SQLite
2. **Subsequent Runs**: Uses cached SQLite data immediately, background sync if online
3. **Invoice Creation**: 4-step wizard (Partner → Location → Products → Review)
4. **Invoice Sending**: Saved to SQLite first, then attempted send with 50% mock failure rate

### Database Schema (SQLite via rusqlite)
- `partners` - Partner companies with CIF and Reg.Com (synced from external service)
- `locations` - Partner locations (one-to-many with partners)
- `products` - Available products (synced from external service)
- `invoices` - Created invoices with status (pending/sending/sent/failed), includes partner CIF/Reg.Com
- `invoice_items` - Line items for each invoice
- `sync_metadata` - Tracks last sync timestamps

**Database Migrations**: Automatic ALTER TABLE execution for schema updates (e.g., adding CIF/reg_com columns to existing databases)

### Printing System
**Technology Stack**:
- **PDF Generation**: Microsoft Edge headless (--headless --print-to-pdf)
- **PDF Printing**: SumatraPDF portable (~5MB, bundled in installer)
- **Template**: Custom HTML optimized for 80mm x 297mm thermal paper
- **Storage**: PDFs archived in `%APPDATA%\facturi.softconsulting.com\invoices\`

**Print Flow**:
1. Generate HTML from template (`print_invoice.rs`)
2. Convert HTML → PDF via Edge headless (800ms)
3. Send PDF to printer via SumatraPDF with `-print-settings shrink` (silent, no preview)
4. Auto-print on invoice creation (configurable in Settings)

**Printer Detection**: Windows WMI via PowerShell (`Get-WmiObject Win32_Printer`)

**Settings Page** (`app/(dashboard)/settings/page.tsx`):
- Printer selection dropdown
- Number of copies (1-10)
- Paper width (58mm/80mm/A4)
- Auto-print toggle
- Preview PDF toggle
- Settings saved in localStorage as JSON

**SumatraPDF Bundling**:
- Placed in `src-tauri/resources/SumatraPDF.exe`
- Automatically included in installer via `tauri.conf.json` resources
- Fallback search paths: bundled → AppData\Local → Program Files

**Invoice Template Features**:
- Company info (KARIN FASHION SRL) with CIF, Reg.Com, bank details
- Customer info with CIF, Reg.Com, location
- Product list with unit prices, quantities, VAT breakdown
- Total calculations (subtotal, VAT 24%, grand total)
- Payment terms and due date (+10 days default)
- Signature blocks
- Footer branding "printed by ESOFT APP"
- Optimized layout: 68mm width, 10.5px font, 0.5mm left margin for perfect centering

### UI Components
Uses shadcn/ui (New York style) with Tailwind CSS v4. Key components:
- Tabs, Cards, Dialog, AlertDialog for layout
- Table for invoice details
- RadioGroup for location selection
- Input with search for partner/product filtering
- Select for printer and settings configuration
- Switch for toggle settings (auto-print, preview)
- Sonner for toast notifications

## New Features Added

### Printing System (January 2026)
- **Silent PDF Printing**: Full printing pipeline with SumatraPDF integration
- **Automatic Print on Save**: Configurable in Settings page
- **Printer Selection**: Windows printer enumeration via WMI
- **PDF Archival**: All invoices saved to AppData for record-keeping
- **Thermal Paper Support**: Template optimized for 80mm receipt printers
- **Print Speed Optimization**: Sub-2-second print jobs (Edge 800ms + SumatraPDF instant)

### Database Enhancements
- **CIF/Reg.Com Fields**: Added Romanian company registration fields to partners and invoices
- **Automatic Migrations**: ALTER TABLE logic for backward compatibility with existing databases
- **Enhanced Partner Data**: Full company information for legal compliance

### Settings Page
- **Complete Print Configuration**: Printer, copies, paper width, auto-print, preview
- **Persistent Settings**: localStorage with JSON serialization
- **Real-time Printer Detection**: Refresh printer list on demand
- **App Information**: Version, storage paths, capabilities

### Developer Experience
- **SumatraPDF Auto-Bundle**: Resources folder with automatic installer inclusion
- **Improved Type Safety**: Fixed TypeScript errors in UpdateNotification
- **Build Optimization**: Faster compilation with better error handling

## Technical Notes

### Printing Architecture
The printing system uses a multi-stage pipeline:
1. **HTML Generation** (Rust): Template in `print_invoice.rs` with company/partner data
2. **PDF Conversion** (Edge): Headless Chrome-based rendering with precise page dimensions
3. **PDF Output** (File System): Persistent storage in AppData
4. **Print Job** (SumatraPDF): Silent printing with shrink-to-fit for proper alignment
5. **Cleanup** (Optional): Old invoices can be archived/deleted via app settings

**Why SumatraPDF?**
- Lightweight (~5MB portable executable)
- Command-line interface perfect for automation
- Print settings flags: `-silent`, `-exit-when-done`, `-exit-on-print`, `-print-settings shrink`
- No dependencies or system integration required
- Works on all Windows versions (7+)

### Thermal Printer Optimization
80mm thermal printers have specific constraints:
- **Physical margins**: ~4mm unprintable on each side
- **Page size**: 80mm width × 297mm length (A4 height)
- **Content width**: 68mm safe area (72mm with risk of clipping)
- **Font scaling**: 10.5-11px optimal for readability vs density
- **Alignment**: Asymmetric margins (0.5mm left, 6mm right) compensate for printer hardware offset

## CI/CD

GitHub Actions workflow (`.github/workflows/build.yml`) builds on Windows and creates GitHub releases with signed installers. Requires `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets.

### Auto-Update System
- **Check on Startup**: App queries GitHub releases for `latest.json`
- **Background Download**: New versions download silently
- **Cryptographic Signing**: All updates verified with ed25519 signatures
- **Silent Installation**: Updates install and restart app automatically
- **Rollback Safety**: Old version preserved until successful update

**Release Process**:
1. Run `npm run tauri build`
2. Upload to GitHub: `*.exe`, `*.exe.sig`, `latest.json`
3. All installed apps auto-update within 24 hours
