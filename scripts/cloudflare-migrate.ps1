# Cloudflare DNS Migration Script for uwuwu.net
# Usage:
#   $env:CF_TOKEN = "your-token-here"
#   .\scripts\cloudflare-migrate.ps1

$ErrorActionPreference = "Stop"

if (-not $env:CF_TOKEN) {
    Write-Host "ERROR: Set CF_TOKEN first:" -ForegroundColor Red
    Write-Host '  $env:CF_TOKEN = "your-cloudflare-api-token"'
    exit 1
}

$api = "https://api.cloudflare.com/client/v4"

function Cf-Request {
    param(
        [string]$Method,
        [string]$Uri,
        [string]$Body
    )
    $tmpFile = $null
    $allArgs = @("-sS", "-X", $Method, $Uri,
        "-H", "Authorization: Bearer $env:CF_TOKEN",
        "-H", "Content-Type: application/json")
    if ($Body) {
        $tmpFile = [System.IO.Path]::GetTempFileName()
        [System.IO.File]::WriteAllText($tmpFile, $Body, [System.Text.UTF8Encoding]::new($false))
        $allArgs += "-d", "@$tmpFile"
    }
    $output = & curl.exe @allArgs
    if ($tmpFile) { Remove-Item $tmpFile -Force -ErrorAction SilentlyContinue }
    return ($output | Out-String) | ConvertFrom-Json
}

# --- Step 1: Get zone + account ID ---
Write-Host "`n=== Step 1: Getting zone info ===" -ForegroundColor Cyan
$zoneCheck = Cf-Request -Method GET -Uri "$api/zones?name=uwuwu.net"
if ($zoneCheck.result.Count -gt 0) {
    $zoneId = $zoneCheck.result[0].id
    $accountId = $zoneCheck.result[0].account.id
    Write-Host "Zone already exists (status: $($zoneCheck.result[0].status))"
} else {
    # Token lacks /accounts access, try creating zone without account.id
    Write-Host "Zone not found, creating..." -ForegroundColor Yellow
}

# --- Step 2: Create zone (if needed) ---
if (-not $zoneId) {
    Write-Host "`n=== Step 2: Creating zone uwuwu.net ===" -ForegroundColor Cyan
    $zoneBody = @{ name = "uwuwu.net"; type = "full" } | ConvertTo-Json -Depth 3
    $zoneResp = Cf-Request -Method POST -Uri "$api/zones" -Body $zoneBody
    if (-not $zoneResp.success) {
        if ($zoneResp.errors[0].code -eq 1061) {
            Write-Host "Zone already exists, fetching existing..." -ForegroundColor Yellow
            $zones = Cf-Request -Method GET -Uri "$api/zones?name=uwuwu.net"
            $zoneId = $zones.result[0].id
            $accountId = $zones.result[0].account.id
        } else {
            Write-Host "ERROR creating zone: $($zoneResp.errors | ConvertTo-Json -Depth 5)" -ForegroundColor Red
            exit 1
        }
    } else {
        $zoneId = $zoneResp.result.id
        $accountId = $zoneResp.result.account.id
    }
}
Write-Host "Account ID: $accountId"
Write-Host "Zone ID: $zoneId"

# --- Step 3: Create DNS records ---
Write-Host "`n=== Step 3: Creating DNS records ===" -ForegroundColor Cyan

$records = @(
    @{ type="CNAME"; name="origa";                    content="c2qj368z.up.railway.app"; proxied=$true;  ttl=1 }
    @{ type="CNAME"; name="app.origa";                content="9fmm6y4e.up.railway.app"; proxied=$false; ttl=1 }
    @{ type="CNAME"; name="s3.origa";                 content="sltxm1ip.up.railway.app"; proxied=$false; ttl=1 }
    @{ type="CNAME"; name="pass";                     content="vcce37wa.up.railway.app"; proxied=$false; ttl=1 }
    @{ type="TXT";   name="_railway-verify.origa";    content="railway-verify=84343282d94e247b3f50fd414f78ce3d3a6fe1da4a8d4efb886543539575de5a"; ttl=300 }
    @{ type="TXT";   name="_railway-verify.s3.origa"; content="railway-verify=6fd8268110336b172171e5bead945782f9e9eaac165b88f1091d730d8053d4cb"; ttl=300 }
    @{ type="TXT";   name="_railway-verify.app.origa";content="railway-verify=2e3aa92460f8b63574ab28b0cb9302b3c6e070229bee6bc90bb2a723817981c1"; ttl=300 }
    @{ type="TXT";   name="_railway-verify.pass";     content="railway-verify=0568c9a326fbea6ee4c477b567e9850dbaed4c68edf8fe92eb2792f288914140"; ttl=300 }
)

foreach ($r in $records) {
    $body = $r | ConvertTo-Json -Compress
    $resp = Cf-Request -Method POST -Uri "$api/zones/$zoneId/dns_records" -Body $body
    if ($resp.success) {
        $proxy = if ($r.proxied) { "PROXIED" } else { "DNS-only" }
        Write-Host "  OK: $($r.type) $($r.name) -> $($r.content) [$proxy]" -ForegroundColor Green
    } else {
        $errCode = $resp.errors[0].code
        if ($errCode -eq 81053) {
            Write-Host "  SKIP (already exists): $($r.type) $($r.name)" -ForegroundColor Yellow
        } else {
            Write-Host "  FAIL: $($r.type) $($r.name) -> $($resp.errors | ConvertTo-Json -Depth 3)" -ForegroundColor Red
        }
    }
}

# --- Step 4: Delete legacy/auto-imported records ---
Write-Host "`n=== Step 4: Cleaning up legacy records ===" -ForegroundColor Cyan
$allRecords = Cf-Request -Method GET -Uri "$api/zones/$zoneId/dns_records?per_page=100"

foreach ($rec in $allRecords.result) {
    $delete = $false
    $reason = ""

    # Legacy apex A
    if ($rec.type -eq "A" -and $rec.name -eq "uwuwu.net") { $delete = $true; $reason = "legacy apex A" }
    # Legacy smtp
    if ($rec.type -eq "A" -and $rec.name -eq "smtp.uwuwu.net") { $delete = $true; $reason = "legacy smtp A" }
    # Legacy MX
    if ($rec.type -eq "MX") { $delete = $true; $reason = "legacy MX" }
    # Legacy SPF
    if ($rec.type -eq "TXT" -and $rec.content -like "*v=spf1*") { $delete = $true; $reason = "legacy SPF TXT" }
    # Duplicate non-proxied origa CNAME (we want the proxied one)
    if ($rec.type -eq "CNAME" -and $rec.name -eq "origa.uwuwu.net" -and -not $rec.proxied) { $delete = $true; $reason = "duplicate non-proxied origa CNAME" }

    if ($delete) {
        $delResp = Cf-Request -Method DELETE -Uri "$api/zones/$zoneId/dns_records/$($rec.id)"
        Write-Host "  DELETED: $($rec.type) $($rec.name) ($reason)" -ForegroundColor Yellow
    }
}

# --- Step 5: Show final records ---
Write-Host "`n=== Step 5: Final DNS records ===" -ForegroundColor Cyan
$finalRecords = Cf-Request -Method GET -Uri "$api/zones/$zoneId/dns_records?per_page=100"
$finalRecords.result | ForEach-Object {
    $proxy = if ($_.proxied) { "PROXIED" } else { "DNS-only" }
    Write-Host "  $($_.type.PadRight(6)) $($_.name.PadRight(35)) $($_.content) [$proxy]"
}

# --- Step 6: Show nameservers ---
Write-Host "`n=== Step 6: CLOUDFLARE NAMESERVERS (set these at OnlineNIC) ===" -ForegroundColor Green
$zoneInfo = Cf-Request -Method GET -Uri "$api/zones/$zoneId"
Write-Host ""
Write-Host "  NS 1: $($zoneInfo.result.name_servers[0])" -ForegroundColor White
Write-Host "  NS 2: $($zoneInfo.result.name_servers[1])" -ForegroundColor White
Write-Host ""
Write-Host "Set these nameservers at your registrar (OnlineNIC)." -ForegroundColor Green
Write-Host "Cloudflare will activate the zone once the NS change propagates." -ForegroundColor Green
Write-Host ""
Write-Host "To revoke the token after migration: dash.cloudflare.com/profile/api-tokens" -ForegroundColor DarkGray
