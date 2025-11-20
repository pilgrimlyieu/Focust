#!/usr/bin/env pwsh
# Release automation script for Focust
# Usage:
#   ./scripts/release.ps1 1.2.3         # Specify exact version
#   ./scripts/release.ps1 --patch       # Bump patch version (0.2.11 -> 0.2.12)
#   ./scripts/release.ps1 --minor       # Bump minor version (0.2.11 -> 0.3.0)
#   ./scripts/release.ps1 --major       # Bump major version (0.2.11 -> 1.0.0)

param(
    [Parameter(Position = 0)]
    [string]$Version,
    [switch]$Patch,
    [switch]$Minor,
    [switch]$Major,
    [switch]$NoPush
)

# Color output functions
function Write-Info { Write-Host "ℹ️  $args" -ForegroundColor Cyan }
function Write-Success { Write-Host "✅ $args" -ForegroundColor Green }
function Write-Warning { Write-Host "⚠️  $args" -ForegroundColor Yellow }
function Write-Error { Write-Host "❌ $args" -ForegroundColor Red }

# Get current version from package.json
function Get-CurrentVersion {
    $packageJson = Get-Content -Path "package.json" -Raw | ConvertFrom-Json
    return $packageJson.version
}

# Calculate new version based on bump type
function Get-NewVersion {
    param([string]$Current, [string]$BumpType)
    
    $parts = $Current.Split('.')
    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]
    
    switch ($BumpType) {
        "major" { return "$($major + 1).0.0" }
        "minor" { return "$major.$($minor + 1).0" }
        "patch" { return "$major.$minor.$($patch + 1)" }
    }
}

# Validate version format
function Test-VersionFormat {
    param([string]$Ver)
    return $Ver -match '^\d+\.\d+\.\d+$'
}

# Update version in a JSON file
function Update-JsonVersion {
    param([string]$Path, [string]$NewVersion)
    
    $content = Get-Content -Path $Path -Raw
    # Use regex to replace only the version field, preserving all formatting
    $content = $content -replace '("version"\s*:\s*")([^"]+)(")', "`${1}$NewVersion`${3}"
    [System.IO.File]::WriteAllText($Path, $content, (New-Object System.Text.UTF8Encoding $false))
    
    Write-Success "Updated version in $Path to $NewVersion"
}

# Extract release notes content (after comment separator)
function Get-ReleaseNotesContent {
    $content = Get-Content -Path "RELEASE_NOTE.md" -Raw
    
    # Extract content after the separator comment ((?s) enables single-line mode for .* to match newlines)
    if ($content -match '(?s)<!--\s*Release notes content starts here\s*-->\s*(.*)$') {
        return $matches[1].Trim()
    }
    
    # Fallback: if no separator, return all content
    Write-Warning "No separator comment found in RELEASE_NOTE.md, using all content"
    return $content.Trim()
}

# Update CHANGELOG.md with new version
function Update-Changelog {
    param([string]$NewVersion, [string]$ReleaseNotes)
    
    $date = Get-Date -Format "yyyy.MM.dd"
    $changelogPath = "CHANGELOG.md"
    $content = Get-Content -Path $changelogPath -Raw
    
    # Convert ## to ### for changelog format (only at line start)
    $changelogEntry = $ReleaseNotes -replace '(?m)^## ', '### '
    
    # Insert new version after [Unreleased]
    $newEntry = "`n`n## $NewVersion ($date)`n`n$changelogEntry`n"
    $content = $content -replace '(\[Unreleased\])', "`$1$newEntry"
    
    [System.IO.File]::WriteAllText($changelogPath, $content, (New-Object System.Text.UTF8Encoding $false))
    Write-Success "Updated CHANGELOG.md with version $NewVersion"
}

# Main script
try {
    # Determine version
    $currentVersion = Get-CurrentVersion
    $newVersion = ""
    
    if ($Version) {
        if (-not (Test-VersionFormat $Version)) {
            Write-Error "Invalid version format: $Version. Expected format: X.Y.Z"
            exit 1
        }
        $newVersion = $Version
    }
    elseif ($Patch) {
        $newVersion = Get-NewVersion $currentVersion "patch"
    }
    elseif ($Minor) {
        $newVersion = Get-NewVersion $currentVersion "minor"
    }
    elseif ($Major) {
        $newVersion = Get-NewVersion $currentVersion "major"
    }
    else {
        Write-Error "No version specified. Use: ./scripts/release.ps1 <version> OR --patch/--minor/--major"
        exit 1
    }
    
    Write-Info "Current version: $currentVersion"
    Write-Info "New version: $newVersion"
    
    # Confirm with user
    $confirm = Read-Host "Continue with release v$newVersion? (y/N)"
    if ($confirm -ne 'y' -and $confirm -ne 'Y') {
        Write-Warning "Release cancelled."
        exit 0
    }
    
    # Check if RELEASE_NOTE.md exists
    if (-not (Test-Path "RELEASE_NOTE.md")) {
        Write-Error "RELEASE_NOTE.md not found. Please create it first."
        exit 1
    }
    
    # Step 1: Update version in package.json and tauri.conf.json
    Write-Info "Step 1: Updating version numbers..."
    Update-JsonVersion "package.json" $newVersion
    Update-JsonVersion "src-tauri/tauri.conf.json" $newVersion
    
    # Step 2: Update CHANGELOG.md
    Write-Info "Step 2: Updating CHANGELOG.md..."
    $releaseNotes = Get-ReleaseNotesContent
    Update-Changelog $newVersion $releaseNotes
    
    # Step 3: Commit changes
    Write-Info "Step 3: Committing changes..."
    git add package.json src-tauri/tauri.conf.json CHANGELOG.md RELEASE_NOTE.md
    git commit -m "chore: bump version to v$newVersion"
    Write-Success "Changes committed with message: chore: bump version to v$newVersion"
    
    # Step 4: Create tag
    Write-Info "Step 4: Creating tag v$newVersion..."
    git tag "v$newVersion"
    Write-Success "Tag v$newVersion created"
    
    # Step 5: Push (with confirmation)
    if (-not $NoPush) {
        $pushConfirm = Read-Host "Push commit and tag to remote? (y/N)"
        if ($pushConfirm -eq 'y' -or $pushConfirm -eq 'Y') {
            Write-Info "Step 5: Pushing changes..."
            git push origin main
            git push origin "v$newVersion"
            Write-Success "Changes and tag pushed to remote"
        }
        else {
            Write-Warning "Push skipped. Run manually: git push origin main && git push origin v$newVersion"
        }
    }
    else {
        Write-Warning "Push skipped (--NoPush flag). Run manually: git push origin main && git push origin v$newVersion"
    }
    
    Write-Success "Release v$newVersion completed! 🎉"
    Write-Info "Don't forget to sign your commit with GPG if needed: git commit --amend -S --no-edit"
}
catch {
    Write-Error "Release failed: $_"
    exit 1
}
