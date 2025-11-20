#!/usr/bin/env bash
# Release automation script for Focust
# Usage:
#   ./scripts/release.sh 1.2.3         # Specify exact version
#   ./scripts/release.sh --patch       # Bump patch version (0.2.11 -> 0.2.12)
#   ./scripts/release.sh --minor       # Bump minor version (0.2.11 -> 0.3.0)
#   ./scripts/release.sh --major       # Bump major version (0.2.11 -> 1.0.0)

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() { echo -e "${CYAN}ℹ️  $*${NC}"; }
log_success() { echo -e "${GREEN}✅ $*${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $*${NC}"; }
log_error() { echo -e "${RED}❌ $*${NC}"; }

# Get current version from package.json
get_current_version() {
    grep -oP '"version":\s*"\K[^"]+' package.json
}

# Calculate new version based on bump type
get_new_version() {
    local current=$1
    local bump_type=$2
    
    IFS='.' read -r major minor patch <<< "$current"
    
    case $bump_type in
        major) echo "$((major + 1)).0.0" ;;
        minor) echo "$major.$((minor + 1)).0" ;;
        patch) echo "$major.$minor.$((patch + 1))" ;;
    esac
}

# Validate version format
validate_version() {
    [[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
}

# Update version in a JSON file
update_json_version() {
    local file=$1
    local new_version=$2
    
    # Use sed to replace only the version field, preserving formatting
    sed -i.bak -E "s/(\"version\"[[:space:]]*:[[:space:]]*\")[^\"]+/\1$new_version/" "$file"
    rm -f "${file}.bak"
    log_success "Updated version in $file to $new_version"
}

# Extract release notes content (after comment separator)
get_release_notes_content() {
    local content=$(cat RELEASE_NOTE.md)
    
    # Extract content after the separator comment
    if echo "$content" | grep -q '<!--.*Release notes content starts here.*-->'; then
        echo "$content" | sed -n '/<!--.*Release notes content starts here.*-->/,${/<!--.*Release notes content starts here.*-->/!p;}'
    else
        # Fallback: if no separator, return all content
        log_warning "No separator comment found in RELEASE_NOTE.md, using all content"
        echo "$content"
    fi
}

# Update CHANGELOG.md with new version
update_changelog() {
    local new_version=$1
    local release_notes=$2
    local date=$(date +%Y.%m.%d)
    
    # Convert ## to ### for changelog format (only at line start)
    release_notes=$(echo "$release_notes" | sed 's/^## /### /g')
    
    # Create new entry with proper spacing
    local new_entry=$'\n\n'"## $new_version ($date)"$'\n\n'"$release_notes"$'\n'
    
    # Insert after [Unreleased] using awk to preserve formatting
    awk -v entry="$new_entry" '/\[Unreleased\]/ {print; print entry; next} 1' CHANGELOG.md > CHANGELOG.md.tmp
    mv CHANGELOG.md.tmp CHANGELOG.md
    
    log_success "Updated CHANGELOG.md with version $new_version"
}

# Main script
main() {
    local version=""
    local bump_type=""
    local no_push=0
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --patch) bump_type="patch"; shift ;;
            --minor) bump_type="minor"; shift ;;
            --major) bump_type="major"; shift ;;
            --no-push) no_push=1; shift ;;
            *) version=$1; shift ;;
        esac
    done
    
    # Determine version
    local current_version=$(get_current_version)
    local new_version=""
    
    if [[ -n $version ]]; then
        if ! validate_version "$version"; then
            log_error "Invalid version format: $version. Expected format: X.Y.Z"
            exit 1
        fi
        new_version=$version
    elif [[ -n $bump_type ]]; then
        new_version=$(get_new_version "$current_version" "$bump_type")
    else
        log_error "No version specified. Use: ./scripts/release.sh <version> OR --patch/--minor/--major"
        exit 1
    fi
    
    log_info "Current version: $current_version"
    log_info "New version: $new_version"
    
    # Confirm with user
    read -p "Continue with release v$new_version? (y/N): " confirm
    if [[ $confirm != "y" && $confirm != "Y" ]]; then
        log_warning "Release cancelled."
        exit 0
    fi
    
    # Check if RELEASE_NOTE.md exists
    if [[ ! -f RELEASE_NOTE.md ]]; then
        log_error "RELEASE_NOTE.md not found. Please create it first."
        exit 1
    fi
    
    # Step 1: Update version in package.json and tauri.conf.json
    log_info "Step 1: Updating version numbers..."
    update_json_version "package.json" "$new_version"
    update_json_version "src-tauri/tauri.conf.json" "$new_version"
    
    # Step 2: Update CHANGELOG.md
    log_info "Step 2: Updating CHANGELOG.md..."
    local release_notes=$(get_release_notes_content)
    update_changelog "$new_version" "$release_notes"
    
    # Step 3: Commit changes
    log_info "Step 3: Committing changes..."
    git add package.json src-tauri/tauri.conf.json CHANGELOG.md RELEASE_NOTE.md
    git commit -m "chore: bump version to v$new_version"
    log_success "Changes committed with message: chore: bump version to v$new_version"
    
    # Step 4: Create tag
    log_info "Step 4: Creating tag v$new_version..."
    git tag "v$new_version"
    log_success "Tag v$new_version created"
    
    # Step 5: Push (with confirmation)
    if [[ $no_push -eq 0 ]]; then
        read -p "Push commit and tag to remote? (y/N): " push_confirm
        if [[ $push_confirm == "y" || $push_confirm == "Y" ]]; then
            log_info "Step 5: Pushing changes..."
            git push origin main
            git push origin "v$new_version"
            log_success "Changes and tag pushed to remote"
        else
            log_warning "Push skipped. Run manually: git push origin main && git push origin v$new_version"
        fi
    else
        log_warning "Push skipped (--no-push flag). Run manually: git push origin main && git push origin v$new_version"
    fi
    
    log_success "Release v$new_version completed! 🎉"
    log_info "Don't forget to sign your commit with GPG if needed: git commit --amend -S --no-edit"
}

main "$@"
