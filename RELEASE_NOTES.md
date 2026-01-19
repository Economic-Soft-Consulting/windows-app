# Release v0.5.2

# Release v0.5.1 - Invoice Numbering & Enhanced Details

## 🎯 Major Features

### ⚡ Invoice Numbering Configuration
- **Configurable number range**: Set start and end numbers for invoice numbering in Settings
- **Auto-increment**: Current number automatically updates with each new invoice
- **Custom series display**: Invoice prints show "Seria: {SimbolCarnet}  Nr: {number}" format
- **Validation**: Prevents exceeding configured maximum number
- **Settings-based**: Replaces database auto-increment with user-controlled numbering

### 📊 Enhanced Invoice Detail View
- **Three-line price breakdown** for each product:
  - Price without VAT (gray)
  - TVA amount with percentage (blue) - uses actual VAT from database
  - Price with VAT (bold)
- **Actual VAT rates**: Each product displays its real VAT percentage from database (no more hardcoded 19%)
- **Issue & Due dates**: Clearly displayed in separate cards
- **Smart due date calculation**: Uses partner's actual payment terms from database
- **Center-aligned price column**: Better visual balance
- **Accurate totals**: Footer calculates total VAT across all items correctly

### 🎨 UI Enhancements
- **Partner cards**: Show payment terms (scadență) with 7-day default
- **Product cards**: Display VAT percentage badges
- **Cleaner table layout**: Removed redundant columns, focused on essential information

## 🔧 Technical Improvements

### Database
- Migration 8: Added invoice numbering fields to agent_settings
- Partner payment terms now included in invoice queries
- Enhanced data retrieval for accurate calculations

### Backend
- Smart numbering initialization: Auto-sets current number to start number
- Invoice creation now uses settings-based numbering
- All Invoice models updated with partner_payment_term field
- Improved logging for debugging payment terms and numbering

### Frontend
- TypeScript types updated with new fields
- Commands updated to support invoice numbering
- Settings page enhanced with invoice numbering UI
- Invoice detail dialog completely redesigned

## 🐛 Bug Fixes
- Fixed payment term defaults (7 days instead of 10)
- Corrected VAT calculations to use actual product data
- Fixed TypeScript type errors for partner and invoice fields
- Resolved compilation errors in Rust backend

## 📝 Changes Summary

**Settings Page:**
- New "Numerotare Facturi" section with:
  - Număr Start (configurable)
  - Număr Final (configurable)
  - Număr Curent (read-only, auto-updating)

**Invoice Details:**
- Removed: Redundant Total and TVA columns
- Added: Comprehensive price breakdown with labels
- Improved: Date display with issue and due dates
- Enhanced: Footer shows accurate total with VAT

**Invoice Printing:**
- Changed from "KARIN-F-000001" format
- Now shows "Seria: FFEAPP  Nr: 15000" (using actual settings)

## 🚀 Upgrade Notes
- Database will automatically migrate to add new fields
- Existing invoices maintain their numbers
- Set your preferred invoice number range in Settings before creating new invoices
- Default invoice numbering starts at 1 if not configured

## 🔍 What's Next?
This release focuses on invoice management and display enhancements. Future updates will continue improving the user experience and adding more configuration options.

---

**Full Changelog**: https://github.com/your-repo/compare/v0.4.0...v0.5.0
