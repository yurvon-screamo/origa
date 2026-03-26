<#
.SYNOPSIS
    Generates all required Tauri icons from a source image.

.DESCRIPTION
    This script generates all icon sizes required by Tauri for desktop and mobile platforms.
    Uses System.Drawing for high-quality image resizing.

.PARAMETER SourceImage
    Path to the source logo image. Default: origa_ui\public\new_logo.png

.PARAMETER OutputDir
    Output directory for generated icons. Default: tauri\icons

.PARAMETER Backup
    Create backup of existing icons before generation.

.EXAMPLE
    .\generate_icons.ps1
    .\generate_icons.ps1 -Backup
    .\generate_icons.ps1 -SourceImage "path\to\logo.png"
#>

param(
    [string]$SourceImage = "$PSScriptRoot\..\origa_ui\public\new_logo.png",
    [string]$OutputDir = "$PSScriptRoot\..\tauri\icons",
    [switch]$Backup
)

# Error handling
$ErrorActionPreference = "Stop"

# Resolve paths
$SourceImage = (Resolve-Path $SourceImage -ErrorAction Stop).Path
$OutputDir = (New-Item -ItemType Directory -Path $OutputDir -Force).FullName
$IosDir = Join-Path $OutputDir "ios"
$AndroidDir = "$PSScriptRoot\..\tauri\gen\android\app\src\main\res"

Write-Host "=== Tauri Icon Generator ===" -ForegroundColor Cyan
Write-Host "Source: $SourceImage" -ForegroundColor Gray
Write-Host "Output: $OutputDir" -ForegroundColor Gray
Write-Host ""

# Load System.Drawing assembly
Add-Type -AssemblyName System.Drawing

# Icon size definitions
$IconSizes = @{
    # Standard Tauri icons
    "32x32.png"          = 32
    "128x128.png"        = 128
    "128x128@2x.png"     = 256
    "icon-32.png"        = 32
    "icon-64.png"        = 64
    "icon-128.png"       = 128
    "icon-256.png"       = 256
    "icon-512.png"       = 512
    "icon.png"           = 1024
    "square-icon.png"    = 512
    
    # Windows Store icons
    "StoreLogo.png"      = 50
    "Square30x30Logo.png" = 30
    "Square44x44Logo.png" = 44
    "Square71x71Logo.png" = 71
    "Square89x89Logo.png" = 89
    "Square107x107Logo.png" = 107
    "Square142x142Logo.png" = 142
    "Square150x150Logo.png" = 150
    "Square284x284Logo.png" = 284
    "Square310x310Logo.png" = 310
}

$IosIconSizes = @{
    # 20x20 series
    "AppIcon-20x20@1x.png"     = 20
    "AppIcon-20x20@2x.png"     = 40
    "AppIcon-20x20@2x-1.png"   = 40
    "AppIcon-20x20@3x.png"     = 60
    
    # 29x29 series
    "AppIcon-29x29@1x.png"     = 29
    "AppIcon-29x29@2x.png"     = 58
    "AppIcon-29x29@2x-1.png"   = 58
    "AppIcon-29x29@3x.png"     = 87
    
    # 40x40 series
    "AppIcon-40x40@1x.png"     = 40
    "AppIcon-40x40@2x.png"     = 80
    "AppIcon-40x40@2x-1.png"   = 80
    "AppIcon-40x40@3x.png"     = 120
    
    # 60x60 series
    "AppIcon-60x60@2x.png"     = 120
    "AppIcon-60x60@3x.png"     = 180
    
    # 76x76 series
    "AppIcon-76x76@1x.png"     = 76
    "AppIcon-76x76@2x.png"     = 152
    
    # Others
    "AppIcon-83.5x83.5@2x.png" = 167
    "AppIcon-512@2x.png"       = 1024
}

# Android mipmap icon sizes (folder name -> pixel size)
$AndroidSizes = @{
    "mipmap-mdpi"    = 48
    "mipmap-hdpi"    = 72
    "mipmap-xhdpi"   = 96
    "mipmap-xxhdpi"  = 144
    "mipmap-xxxhdpi" = 192
}

# Additional standalone icons (64x64 is sometimes referenced separately)
$AdditionalSizes = @{
    "64x64.png" = 64
}

function Resize-Image {
    param(
        [string]$SourcePath,
        [string]$OutputPath,
        [int]$Size
    )
    
    try {
        # Load source image
        $sourceImage = [System.Drawing.Image]::FromFile($SourcePath)
        
        # Check if resize is needed
        if ($sourceImage.Width -eq $Size -and $sourceImage.Height -eq $Size) {
            $sourceImage.Dispose()
            Copy-Item $SourcePath $OutputPath -Force
            return $true
        }
        
        # Create new bitmap with target size
        $bitmap = New-Object System.Drawing.Bitmap($Size, $Size)
        
        # Set high quality settings
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
        $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
        
        # Draw resized image
        $graphics.DrawImage($sourceImage, 0, 0, $Size, $Size)
        
        # Ensure output directory exists
        $outputDir = Split-Path $OutputPath -Parent
        if (-not (Test-Path $outputDir)) {
            New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
        }
        
        # Save as PNG
        $bitmap.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
        
        # Cleanup
        $graphics.Dispose()
        $bitmap.Dispose()
        $sourceImage.Dispose()
        
        return $true
    }
    catch {
        Write-Warning "Failed to generate $OutputPath : $($_.Exception.Message)"
        return $false
    }
}

function New-IcoFile {
    param(
        [string]$SourcePath,
        [string]$OutputPath,
        [int]$Size = 256
    )
    
    try {
        # Load source image
        $sourceImage = [System.Drawing.Image]::FromFile($SourcePath)
        
        # Resize to target size for ICO
        $bitmap = New-Object System.Drawing.Bitmap($Size, $Size)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
        $graphics.DrawImage($sourceImage, 0, 0, $Size, $Size)
        
        # Get icon from bitmap
        $icon = [System.Drawing.Icon]::FromHandle($bitmap.GetHicon())
        
        # Save as ICO
        $fileStream = [System.IO.File]::Create($OutputPath)
        $icon.Save($fileStream)
        $fileStream.Close()
        
        # Cleanup
        $icon.Dispose()
        $graphics.Dispose()
        $bitmap.Dispose()
        $sourceImage.Dispose()
        
        return $true
    }
    catch {
        Write-Warning "Failed to generate $OutputPath : $($_.Exception.Message)"
        return $false
    }
}

function Backup-ExistingIcons {
    param([string]$OutputDir)
    
    $backupDir = Join-Path $OutputDir "backup_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
    
    if (Test-Path $OutputDir) {
        $existingIcons = Get-ChildItem -Path $OutputDir -Filter "*.png" -Recurse
        if ($existingIcons.Count -gt 0) {
            New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
            
            foreach ($icon in $existingIcons) {
                $relativePath = $icon.FullName.Substring($OutputDir.Length + 1)
                $destPath = Join-Path $backupDir $relativePath
                $destDir = Split-Path $destPath -Parent
                
                if (-not (Test-Path $destDir)) {
                    New-Item -ItemType Directory -Path $destDir -Force | Out-Null
                }
                
                Copy-Item $icon.FullName $destPath -Force
            }
            
            Write-Host "Created backup at: $backupDir" -ForegroundColor Green
        }
    }
}

function Copy-LargestForIcns {
    param(
        [string]$SourcePath,
        [string]$OutputPath
    )
    
    try {
        # For .icns, we just copy the largest PNG (1024x1024 or source)
        # .icns generation requires macOS-specific tools
        $largestIcon = Join-Path $OutputDir "icon.png"
        
        if (Test-Path $largestIcon) {
            Copy-Item $largestIcon (Join-Path $OutputDir "icon.icns") -Force
            Write-Host "  [COPied] icon.icns <- icon.png (use icon.png for .icns generation on macOS)" -ForegroundColor Yellow
            return $true
        }
        
        return $false
    }
    catch {
        Write-Warning "Failed to copy for .icns: $($_.Exception.Message)"
        return $false
    }
}

# Main execution
try {
    # Verify source image exists
    if (-not (Test-Path $SourceImage)) {
        throw "Source image not found: $SourceImage"
    }
    
    # Create backup if requested
    if ($Backup) {
        Backup-ExistingIcons -OutputDir $OutputDir
        if (Test-Path $AndroidDir) {
            Backup-ExistingIcons -OutputDir $AndroidDir
        }
    }
    
    # Ensure output directories exist
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    New-Item -ItemType Directory -Path $IosDir -Force | Out-Null
    
    Write-Host "Generating icons..." -ForegroundColor Cyan
    $successCount = 0
    $failCount = 0
    
    # Generate standard icons
    Write-Host "`n[Standard Icons]" -ForegroundColor Yellow
    foreach ($iconName in $IconSizes.Keys) {
        $size = $IconSizes[$iconName]
        $outputPath = Join-Path $OutputDir $iconName
        
        Write-Host "  Generating $iconName ($($size)x$($size))..." -NoNewline
        
        if (Resize-Image -SourcePath $SourceImage -OutputPath $outputPath -Size $size) {
            Write-Host " OK" -ForegroundColor Green
            $successCount++
        }
        else {
            Write-Host " FAILED" -ForegroundColor Red
            $failCount++
        }
    }
    
    # Generate additional standalone icons
    Write-Host "`n[Additional Icons]" -ForegroundColor Yellow
    foreach ($iconName in $AdditionalSizes.Keys) {
        $size = $AdditionalSizes[$iconName]
        $outputPath = Join-Path $OutputDir $iconName
        
        Write-Host "  Generating $iconName ($($size)x$($size))..." -NoNewline
        
        if (Resize-Image -SourcePath $SourceImage -OutputPath $outputPath -Size $size) {
            Write-Host " OK" -ForegroundColor Green
            $successCount++
        }
        else {
            Write-Host " FAILED" -ForegroundColor Red
            $failCount++
        }
    }
    
    # Generate iOS icons
    Write-Host "`n[iOS Icons]" -ForegroundColor Yellow
    foreach ($iconName in $IosIconSizes.Keys) {
        $size = $IosIconSizes[$iconName]
        $outputPath = Join-Path $IosDir $iconName
        
        Write-Host "  Generating $iconName ($($size)x$($size))..." -NoNewline
        
        if (Resize-Image -SourcePath $SourceImage -OutputPath $outputPath -Size $size) {
            Write-Host " OK" -ForegroundColor Green
            $successCount++
        }
        else {
            Write-Host " FAILED" -ForegroundColor Red
            $failCount++
        }
    }
    
    # Generate Android mipmap icons
    Write-Host "`n[Android Icons]" -ForegroundColor Yellow
    foreach ($mipmapFolder in $AndroidSizes.Keys) {
        $size = $AndroidSizes[$mipmapFolder]
        $outputFolder = Join-Path $AndroidDir $mipmapFolder
        
        # Create mipmap folder if it doesn't exist
        if (-not (Test-Path $outputFolder)) {
            New-Item -ItemType Directory -Path $outputFolder -Force | Out-Null
        }
        
        # Generate three icon variants for each density
        $iconVariants = @(
            "ic_launcher.png",
            "ic_launcher_round.png",
            "ic_launcher_foreground.png"
        )
        
        foreach ($iconVariant in $iconVariants) {
            $outputPath = Join-Path $outputFolder $iconVariant
            
            Write-Host "  Generating $mipmapFolder/$iconVariant ($($size)x$($size))..." -NoNewline
            
            if (Resize-Image -SourcePath $SourceImage -OutputPath $outputPath -Size $size) {
                Write-Host " OK" -ForegroundColor Green
                $successCount++
            }
            else {
                Write-Host " FAILED" -ForegroundColor Red
                $failCount++
            }
        }
    }
    
    # Generate .ico file
    Write-Host "`n[Windows ICO]" -ForegroundColor Yellow
    $icoPath = Join-Path $OutputDir "icon.ico"
    Write-Host "  Generating icon.ico (256x256)..." -NoNewline
    
    if (New-IcoFile -SourcePath $SourceImage -OutputPath $icoPath -Size 256) {
        Write-Host " OK" -ForegroundColor Green
        $successCount++
    }
    else {
        Write-Host " FAILED" -ForegroundColor Red
        $failCount++
    }
    
    # Handle .icns (copy largest PNG as reference)
    Write-Host "`n[macOS ICNS]" -ForegroundColor Yellow
    Write-Host "  Note: .icns requires macOS tools. Copying icon.png for reference..." -NoNewline
    $icnsSource = Join-Path $OutputDir "icon.png"
    $icnsDest = Join-Path $OutputDir "icon.icns"
    
    if (Test-Path $icnsSource) {
        Copy-Item $icnsSource $icnsDest -Force
        Write-Host " OK (placeholder)" -ForegroundColor Yellow
        Write-Host "  Tip: On macOS, use: 'iconutil -c icns icons.iconset' to generate proper .icns" -ForegroundColor Gray
        $successCount++
    }
    else {
        Write-Host " FAILED" -ForegroundColor Red
        $failCount++
    }
    
    # Summary
    Write-Host "`n================================" -ForegroundColor Cyan
    Write-Host "Generation complete!" -ForegroundColor Green
    Write-Host "  Success: $successCount" -ForegroundColor Green
    Write-Host "  Failed:  $failCount" -ForegroundColor $(if ($failCount -gt 0) { "Red" } else { "Green" })
    Write-Host "================================`n" -ForegroundColor Cyan
    
    # List generated files
    Write-Host "Generated files:" -ForegroundColor Yellow
    Get-ChildItem -Path $OutputDir -Filter "*.png" -Recurse | ForEach-Object {
        $relativePath = $_.FullName.Substring($OutputDir.Length + 1)
        Write-Host "  $relativePath" -ForegroundColor Gray
    }
    Get-ChildItem -Path $OutputDir -Filter "*.ico" | ForEach-Object {
        Write-Host "  $($_.Name)" -ForegroundColor Gray
    }
    Get-ChildItem -Path $OutputDir -Filter "*.icns" | ForEach-Object {
        Write-Host "  $($_.Name) (placeholder - use macOS to generate)" -ForegroundColor Yellow
    }
    
    # List Android icons if directory exists
    if (Test-Path $AndroidDir) {
        Write-Host "`nAndroid mipmap icons:" -ForegroundColor Yellow
        foreach ($mipmapFolder in $AndroidSizes.Keys) {
            $folderPath = Join-Path $AndroidDir $mipmapFolder
            if (Test-Path $folderPath) {
                Get-ChildItem -Path $folderPath -Filter "*.png" | ForEach-Object {
                    Write-Host "  $mipmapFolder/$($_.Name)" -ForegroundColor Gray
                }
            }
        }
    }
    
    exit $(if ($failCount -gt 0) { 1 } else { 0 })
}
catch {
    Write-Error "Error: $($_.Exception.Message)"
    Write-Error $_.ScriptStackTrace
    exit 1
}