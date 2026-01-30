# Developer Documentation - eSoft Facturi v0.6.8

## Table of Contents

1. [Project Overview](#project-overview)
2. [Technology Stack](#technology-stack)
3. [Project Structure](#project-structure)
4. [Architecture](#architecture)
5. [Data Models](#data-models)
6. [Tauri Commands API](#tauri-commands-api)
7. [React Components](#react-components)
8. [State Management](#state-management)
9. [Build & Deployment](#build--deployment)
10. [Development Workflow](#development-workflow)

---

## Project Overview

**eSoft Facturi** is a desktop invoicing application built with Tauri (Rust backend) and Next.js (React frontend). It enables offline-first invoice creation with synchronization to WinMentor ERP system.

### Key Features
- Offline-first architecture with local SQLite database
- Role-based access (Agent vs Administrator)
- 4-step invoice creation wizard
- Thermal printer support (80mm/58mm)
- Automatic data synchronization

---

## Technology Stack

### Frontend
| Technology | Version | Purpose |
|------------|---------|---------|
| Next.js | 15.x | React framework with App Router |
| React | 19.x | UI library |
| TypeScript | 5.x | Type safety |
| Tailwind CSS | 4.x | Styling |
| shadcn/ui | Latest | UI component library |
| Radix UI | Latest | Accessible primitives |
| Lucide React | Latest | Icons |

### Backend (Tauri)
| Technology | Version | Purpose |
|------------|---------|---------|
| Tauri | 2.x | Desktop runtime |
| Rust | 1.75+ | Backend logic |
| SQLite | 3.x | Local database |
| Reqwest | Latest | HTTP client for sync |
| SumatraPDF | Latest | PDF printing |

---

## Project Structure

```
f:\Proiecteprogramare\Karin-aplicatie\
├── app/                          # Next.js App Router
│   ├── (dashboard)/              # Protected dashboard routes
│   │   ├── page.tsx              # Dashboard home
│   │   ├── layout.tsx            # Dashboard layout with sidebar
│   │   ├── invoices/             # Invoice pages
│   │   │   ├── page.tsx          # Invoice list
│   │   │   └── new/page.tsx      # New invoice wizard
│   │   ├── data/page.tsx         # Partners & Products
│   │   └── settings/page.tsx     # Admin settings
│   ├── login/page.tsx            # Login page
│   ├── contexts/                 # React contexts
│   │   └── AuthContext.tsx       # Authentication state
│   ├── components/               # App-specific components
│   │   ├── layout/               # Sidebar, Header
│   │   ├── invoices/             # Invoice-related components
│   │   │   └── wizard/           # 4-step wizard components
│   │   └── sync/                 # First-run overlay
│   ├── globals.css               # Global styles
│   └── layout.tsx                # Root layout
├── components/ui/                # shadcn/ui components
├── hooks/                        # Custom React hooks
│   ├── useInvoices.ts
│   ├── usePartners.ts
│   ├── useProducts.ts
│   ├── useSyncStatus.ts
│   └── useOnlineStatus.ts
├── lib/
│   ├── tauri/
│   │   ├── commands.ts           # Tauri invoke wrappers
│   │   └── types.ts              # TypeScript interfaces
│   └── utils.ts                  # Utility functions
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── commands/             # Tauri command handlers
│   │   ├── db/                   # SQLite operations
│   │   └── sync/                 # API sync logic
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── public/                       # Static assets
├── package.json                  # Node dependencies
└── docs/                         # Documentation
```

---

## Architecture

### Frontend-Backend Communication

```
┌─────────────────────────────────────────────────────────────┐
│                     Next.js Frontend                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Pages     │  │  Components │  │  Hooks              │  │
│  │  (App Dir)  │  │  (UI/Logic) │  │  (useInvoices etc)  │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                    │              │
│         └────────────────┼────────────────────┘              │
│                          │                                   │
│                          ▼                                   │
│              ┌───────────────────────┐                      │
│              │  lib/tauri/commands   │                      │
│              │  (invoke wrappers)    │                      │
│              └───────────┬───────────┘                      │
└──────────────────────────┼──────────────────────────────────┘
                           │ Tauri IPC
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Backend (Rust)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Commands   │  │  Database   │  │  Sync Service       │  │
│  │  (Handlers) │  │  (SQLite)   │  │  (HTTP to WinMentor)│  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **User creates invoice** → Frontend wizard collects data
2. **Submit** → `createInvoice()` command invoked
3. **Backend** → Saves to local SQLite with status "pending"
4. **Sync** → When online, sends to WinMentor API
5. **Response** → Updates status to "sent" or "failed"

---

## Data Models

### Core Types (lib/tauri/types.ts)

#### Partner
```typescript
interface Partner {
  id: string;
  name: string;
  cif?: string;                    // Tax ID (CUI)
  reg_com?: string;                // Trade Registry
  cod_intern?: string;             // Internal code
  cod_extern?: string;             // External code
  tip_partener?: string;           // Partner type
  credit_client?: string;          // Credit limit
  moneda?: string;                 // Currency
  // ... more fields
}

interface PartnerWithLocations extends Partner {
  locations: Location[];
}
```

#### Location
```typescript
interface Location {
  id: string;
  partner_id: string;
  name: string;
  address: string | null;
  localitate?: string;             // City
  judet?: string;                  // County
  strada?: string;                 // Street
  telefon?: string;
  email?: string;
}
```

#### Product
```typescript
interface Product {
  id: string;
  name: string;
  unit_of_measure: string;
  price: number;
  class: string | null;
  tva_percent: number | null;      // VAT percentage
}
```

#### Invoice
```typescript
type InvoiceStatus = "pending" | "sending" | "sent" | "failed";

interface Invoice {
  id: string;
  partner_id: string;
  partner_name: string;
  location_id: string;
  location_name: string;
  status: InvoiceStatus;
  total_amount: number;
  item_count: number;
  notes: string | null;
  created_at: string;
  sent_at: string | null;
  error_message: string | null;
}

interface InvoiceItem {
  id: string;
  invoice_id: string;
  product_id: string;
  product_name: string;
  quantity: number;
  unit_price: number;
  unit_of_measure: string;
  total_price: number;
  tva_percent: number | null;
}
```

#### Agent Settings
```typescript
interface AgentSettings {
  agent_name: string | null;
  carnet_series: string | null;
  simbol_carnet_livr: string | null;
  simbol_gestiune_livrare: string | null;
  cod_carnet: string | null;
  cod_carnet_livr: string | null;
  delegate_name: string | null;
  delegate_act: string | null;
  invoice_number_start: number | null;
  invoice_number_end: number | null;
  invoice_number_current: number | null;
}
```

---

## Tauri Commands API

All commands are defined in `lib/tauri/commands.ts` and invoke Rust handlers.

### Sync Commands
```typescript
clearDatabase(): Promise<void>           // Reset all data
checkFirstRun(): Promise<boolean>        // Check if first launch
getSyncStatus(): Promise<SyncStatus>     // Get sync timestamps
syncAllData(): Promise<SyncStatus>       // Trigger full sync
checkOnlineStatus(): Promise<boolean>    // Check internet
```

### Partner Commands
```typescript
getPartners(): Promise<PartnerWithLocations[]>
searchPartners(query: string): Promise<PartnerWithLocations[]>
```

### Product Commands
```typescript
getProducts(partnerId?: string): Promise<Product[]>
searchProducts(query: string, partnerId?: string): Promise<Product[]>
```

### Invoice Commands
```typescript
createInvoice(request: CreateInvoiceRequest): Promise<Invoice>
getInvoices(statusFilter?: InvoiceStatus): Promise<Invoice[]>
getInvoiceDetail(invoiceId: string): Promise<InvoiceDetail>
sendInvoice(invoiceId: string): Promise<Invoice>
sendAllPendingInvoices(): Promise<string[]>
cancelInvoiceSending(invoiceId: string): Promise<Invoice>
deleteInvoice(invoiceId: string): Promise<void>
```

### Print Commands
```typescript
getAvailablePrinters(): Promise<string[]>
printInvoiceToHtml(invoiceId: string, printerName?: string): Promise<string>
```

### Settings Commands
```typescript
getAgentSettings(): Promise<AgentSettings>
saveAgentSettings(...args): Promise<AgentSettings>
```

---

## React Components

### Page Components
| Component | Path | Description |
|-----------|------|-------------|
| `HomePage` | `app/(dashboard)/page.tsx` | Dashboard with stats |
| `InvoicesPage` | `app/(dashboard)/invoices/page.tsx` | Invoice list |
| `NewInvoicePage` | `app/(dashboard)/invoices/new/page.tsx` | Invoice wizard |
| `DataPage` | `app/(dashboard)/data/page.tsx` | Partners & Products |
| `SettingsPage` | `app/(dashboard)/settings/page.tsx` | Admin config |
| `LoginPage` | `app/login/page.tsx` | Authentication |

### Invoice Wizard Components
| Component | Purpose |
|-----------|---------|
| `InvoiceWizard` | Main wizard container with step navigation |
| `PartnerStep` | Partner search and selection |
| `LocationStep` | Location selection from partner |
| `ProductStep` | Product search and cart management |
| `ReviewStep` | Final review before submission |

### Layout Components
| Component | Purpose |
|-----------|---------|
| `Sidebar` | Navigation sidebar with role-based links |
| `Header` | Top header with sync status |
| `FirstRunOverlay` | Initial sync overlay on first launch |

---

## State Management

### Authentication Context

```typescript
// app/contexts/AuthContext.tsx
type UserRole = "admin" | "agent" | null;

interface AuthContextType {
  userRole: UserRole;
  isAuthenticated: boolean;
  login: (role: UserRole, password?: string) => boolean;
  logout: () => void;
  isAdmin: boolean;
  isAgent: boolean;
}
```

**Key behaviors:**
- Agent login persists in localStorage
- Admin login does NOT persist (security)
- Admin requires password: `esoft2026`

### Custom Hooks

#### useInvoices
```typescript
const { invoices, isLoading, refetch } = useInvoices(statusFilter?);
```

#### usePartners
```typescript
const { partners, isLoading, searchPartners } = usePartners();
```

#### useProducts
```typescript
const { products, isLoading, searchProducts } = useProducts(partnerId?);
```

#### useSyncStatus
```typescript
const { status, isSyncing, triggerSync, checkIsFirstRun } = useSyncStatus();
```

#### useOnlineStatus
```typescript
const { isOnline } = useOnlineStatus();
```

---

## Build & Deployment

### Development
```bash
npm install                    # Install dependencies
npm run tauri dev             # Start development server
```

### Production Build
```bash
npm run tauri build           # Build production bundle
```

**Output files:**
- `src-tauri/target/release/bundle/msi/*.msi` - Windows installer
- `src-tauri/target/release/bundle/nsis/*-setup.exe` - NSIS installer

### GitHub Actions Release

The repository uses GitHub Actions for automated releases:

1. Tag a new version: `git tag v0.6.8`
2. Push tag: `git push origin v0.6.8`
3. GitHub Actions builds and creates release

**Required secrets:**
- `TAURI_SIGNING_PRIVATE_KEY` - For code signing

---

## Development Workflow

### Adding a New Tauri Command

1. **Rust handler** (`src-tauri/src/commands/`)
```rust
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    // Implementation
}
```

2. **Register command** (`src-tauri/src/main.rs`)
```rust
.invoke_handler(tauri::generate_handler![
    my_command,
    // ... other commands
])
```

3. **TypeScript wrapper** (`lib/tauri/commands.ts`)
```typescript
export async function myCommand(param: string): Promise<string> {
  return invoke<string>("my_command", { param });
}
```

### Adding a New Page

1. Create file in `app/(dashboard)/my-page/page.tsx`
2. Add navigation link in `Sidebar.tsx` (with role check if needed)
3. Page automatically gets dashboard layout

### Styling Guidelines

- Use Tailwind CSS classes
- Follow shadcn/ui component patterns
- Responsive design with `sm:`, `md:`, `lg:` breakpoints
- Dark mode supported via CSS variables

### Known Issues & Workarounds

#### Dropdown Positioning with Zoom
The app uses `zoom: 0.8` for 10" screen optimization. Radix UI portals require counter-zoom:
```css
[data-radix-popper-content-wrapper] {
  zoom: 1.25 !important;
}
```

---

## Contact

- **Repository**: Economic-Soft-Consulting/windows-app
- **Website**: [softconsulting.ro](https://www.softconsulting.ro)
- **Version**: 0.6.8
