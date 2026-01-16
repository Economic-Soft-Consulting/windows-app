# Test print functionality
# This script simulates creating an invoice and testing print

$API_URL = "http://localhost:3000"

# Get partners
Write-Host "Getting partners..."
$partners = curl -s "$API_URL/api/partners" | ConvertFrom-Json

if ($partners -and $partners.Count -gt 0) {
    $partner = $partners[0]
    Write-Host "Using partner: $($partner.name)"
    
    # Get products
    Write-Host "Getting products..."
    $products = curl -s "$API_URL/api/products" | ConvertFrom-Json
    
    if ($products -and $products.Count -gt 0) {
        $product = $products[0]
        Write-Host "Using product: $($product.name)"
        
        # Create invoice
        Write-Host "Creating invoice..."
        $invoiceData = @{
            partner_id = $partner.id
            location_id = $partner.locations[0].id
            items = @(@{
                product_id = $product.id
                quantity = 1
                unit_price = 100
            })
        } | ConvertTo-Json
        
        $invoice = curl -s -X POST "$API_URL/api/invoices" -ContentType "application/json" -Body $invoiceData | ConvertFrom-Json
        
        if ($invoice -and $invoice.id) {
            Write-Host "Invoice created: $($invoice.id)"
            
            # Check if HTML was generated
            Start-Sleep -Seconds 2
            $htmlFiles = Get-ChildItem "$env:TEMP\factura_$($invoice.id).html" -ErrorAction SilentlyContinue
            
            if ($htmlFiles) {
                Write-Host "SUCCESS: HTML file generated!"
                Write-Host "File path: $($htmlFiles.FullName)"
                Get-Content $htmlFiles.FullName | Select-Object -First 50
            } else {
                Write-Host "HTML file not found in temp"
            }
        } else {
            Write-Host "Failed to create invoice"
        }
    }
}
