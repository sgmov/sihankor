#!/usr/bin/env python3
"""
DSR2 Fix Script: Repair stage values and missing frontmatter fields in .sih.md files.

Run from SiHankor-A project directory (CWD = /Users/moc/projects/SiHankor-A/).
The sih-docs path is inferred as ./sih-docs relative to CWD by default.

Usage:
    python dsr2_fix.py                          # dry-run preview (default)
    python dsr2_fix.py --dry-run                # explicit dry-run
    python dsr2_fix.py --no-dry-run             # execute actual writes
    python dsr2_fix.py --sih-docs /path/to/sih-docs  # custom sih-docs path
"""

import argparse
import os
import re
import shutil
import sys
from datetime import datetime

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

BACKUP_ROOT = "/tmp/dsr2-fix-backup"

# Archive directories (relative to sih-docs root)
ARCHIVE_PREFIXES = ["archive/v1/", "archive/v2/", "archive/v3/"]

# Specific files with stage: 0
SPECIFIC_FIXES_STAGE0 = [
    "actions/plan/sihmd-format-spec-fixes-20260603-001.sih.md",
    "rebuild/designs/spec/sihmd-format-specification-20260602-001.sih.md",
]

# File specifically missing the id field
FILE_MISSING_ID = "mcp-governance-interface-design-20260608-001.sih.md"

# ---------------------------------------------------------------------------
# Frontmatter helpers
# ---------------------------------------------------------------------------


def parse_frontmatter(content):
    """Parse frontmatter from .sih.md content.

    Returns:
        (frontmatter_lines, body_lines, has_frontmatter, error_msg)
    """
    lines = content.split("\n")

    # File must start with ---
    if not lines or lines[0].strip() != "---":
        return None, lines, False, None

    # Find closing ---
    end_idx = None
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            end_idx = i
            break

    if end_idx is None:
        return None, lines, False, "unclosed frontmatter delimiter"

    frontmatter_lines = lines[1:end_idx]
    body_lines = lines[end_idx + 1 :]
    return frontmatter_lines, body_lines, True, None


def rebuild_content(frontmatter_lines, body_lines):
    """Rebuild full file content from frontmatter and body lines."""
    parts = ["---"]
    parts.extend(frontmatter_lines)
    parts.append("---")
    parts.extend(body_lines)
    return "\n".join(parts)


def get_field(frontmatter_lines, field):
    """Return (value, index) of a field in frontmatter, or (None, None)."""
    pattern = re.compile(rf"^{re.escape(field)}\s*:\s*(.*)$", re.IGNORECASE)
    for idx, line in enumerate(frontmatter_lines):
        m = pattern.match(line)
        if m:
            return m.group(1).strip(), idx
    return None, None


def has_field(frontmatter_lines, field):
    """Check if a field exists in frontmatter."""
    val, _ = get_field(frontmatter_lines, field)
    return val is not None


def replace_field_value(frontmatter_lines, field, old_value, new_value):
    """Replace the value of a field if it matches old_value.

    Returns (new_lines, changed: bool).
    """
    pattern = re.compile(
        rf"^({re.escape(field)}\s*:\s*){re.escape(old_value)}\s*$", re.IGNORECASE
    )
    new_lines = []
    changed = False
    for line in frontmatter_lines:
        if pattern.match(line):
            new_lines.append(f"{field}: {new_value}")
            changed = True
        else:
            new_lines.append(line)
    return new_lines, changed


def set_field(frontmatter_lines, field, value):
    """Set field to value. Replaces if exists, otherwise appends.

    Returns (new_lines, added: bool, replaced: bool).
    """
    pattern = re.compile(rf"^({re.escape(field)}\s*:).*$", re.IGNORECASE)
    new_lines = []
    replaced = False
    for line in frontmatter_lines:
        if pattern.match(line):
            new_lines.append(f"{field}: {value}")
            replaced = True
        else:
            new_lines.append(line)

    if not replaced:
        # Try to insert after the `id:` line
        id_idx = None
        for i, line in enumerate(new_lines):
            if re.match(r"^id\s*:", line, re.IGNORECASE):
                id_idx = i
                break
        if id_idx is not None:
            new_lines.insert(id_idx + 1, f"{field}: {value}")
        else:
            # No id line, append to frontmatter
            new_lines.append(f"{field}: {value}")

    return new_lines, not replaced, replaced


# ---------------------------------------------------------------------------
# Fix logic per file
# ---------------------------------------------------------------------------


def classify_path(rel_path):
    """Determine which rules apply to a given relative path.

    Returns a list of fix descriptions (strings).
    """
    fixes = []

    # Normalize to forward slashes
    rel_path = rel_path.replace("\\", "/")

    # Rule 1: Archive dirs -- propose/ratify -> X
    for prefix in ARCHIVE_PREFIXES:
        if rel_path.startswith(prefix):
            fixes.append("archive_stage_propose_ratify_to_x")
            break

    # Rule 2 & 3: Specific files with stage: 0 -> X
    for sp in SPECIFIC_FIXES_STAGE0:
        if rel_path == sp:
            fixes.append("specific_stage_0_to_x")
            break

    # Rule 4: Check for missing stage (applied later after parsing)
    fixes.append("maybe_add_stage_x")

    # Rule 5: Specific file missing id
    if rel_path == FILE_MISSING_ID:
        fixes.append("add_missing_id")

    return fixes


def fix_file(file_path, rel_path, dry_run):
    """Apply all applicable fixes to a single .sih.md file.

    Returns a dict with change info or None if no changes.
    """
    basename = os.path.basename(file_path)

    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()

    original_content = content

    # Parse frontmatter
    fm_lines, body_lines, has_fm, fm_error = parse_frontmatter(content)
    if not has_fm:
        return {
            "path": rel_path,
            "status": "skipped",
            "reason": fm_error or "no frontmatter found",
            "changes": [],
        }

    # Determine applicable rules
    fixes = classify_path(rel_path)
    changes = []
    modified = False

    # -- Rule 1: archive stage propose/ratify -> X --
    if "archive_stage_propose_ratify_to_x" in fixes:
        for old_val in ["propose", "ratify"]:
            new_lines, ch = replace_field_value(
                fm_lines, "stage", old_val, "X"
            )
            if ch:
                changes.append(f"stage: {old_val} -> stage: X (archive rule)")
                fm_lines = new_lines
                modified = True

    # -- Rule 2 & 3: specific files stage: 0 -> X --
    if "specific_stage_0_to_x" in fixes:
        new_lines, ch = replace_field_value(fm_lines, "stage", "0", "X")
        if ch:
            changes.append("stage: 0 -> stage: X (specific file rule)")
            fm_lines = new_lines
            modified = True

    # -- Rule 4: missing stage field --
    if "maybe_add_stage_x" in fixes:
        if not has_field(fm_lines, "stage"):
            # Determine id value: use filename without .sih.md suffix
            file_stem = basename
            if file_stem.endswith(".sih.md"):
                file_stem = file_stem[: -len(".sih.md")]

            # Ensure id field exists
            if not has_field(fm_lines, "id"):
                new_lines, added, replaced = set_field(
                    fm_lines, "id", file_stem
                )
                if added or replaced:
                    changes.append(
                        f"added missing id field -> id: {file_stem}"
                    )
                    fm_lines = new_lines
                    modified = True

            # Add stage: X after id:
            new_lines, added, replaced = set_field(fm_lines, "stage", "X")
            if added:
                changes.append("added missing stage field -> stage: X")
                fm_lines = new_lines
                modified = True

    # -- Rule 5: specific file missing id --
    if "add_missing_id" in fixes:
        if not has_field(fm_lines, "id"):
            new_lines, added, replaced = set_field(
                fm_lines,
                "id",
                "mcp-governance-interface-design-20260608-001",
            )
            if added or replaced:
                changes.append(
                    "added missing id field -> id: mcp-governance-interface-design-20260608-001"
                )
                fm_lines = new_lines
                modified = True

    if not modified:
        return {
            "path": rel_path,
            "status": "unchanged",
            "changes": [],
        }

    # Rebuild content
    new_content = rebuild_content(fm_lines, body_lines)

    if dry_run:
        return {
            "path": rel_path,
            "status": "would_modify",
            "changes": changes,
            "diff_preview": _make_diff_preview(original_content, new_content),
        }

    # -- Backup --
    backup_path = _backup_file(file_path, original_content)

    # -- Write --
    with open(file_path, "w", encoding="utf-8") as f:
        f.write(new_content)

    return {
        "path": rel_path,
        "status": "modified",
        "changes": changes,
        "backup": backup_path,
    }


def _make_diff_preview(original, modified):
    """Generate a compact diff preview for --dry-run output."""
    orig_lines = original.split("\n")
    new_lines = modified.split("\n")

    # Show only frontmatter region (up to and including closing ---)
    fm_end = 0
    for i, line in enumerate(new_lines):
        if i > 0 and line.strip() == "---":
            fm_end = i + 1
            break

    preview_lines = []
    max_lines = max(len(orig_lines), len(new_lines))
    for i in range(min(max_lines, fm_end + 2)):
        o = orig_lines[i] if i < len(orig_lines) else ""
        n = new_lines[i] if i < len(new_lines) else ""
        if o != n:
            preview_lines.append(f"  - |{o}|")
            preview_lines.append(f"  + |{n}|")

    # If preview is empty but we know there's a change, show last few lines
    if not preview_lines:
        for i in range(max(0, min(len(orig_lines), len(new_lines)) - 3),
                       min(len(orig_lines), len(new_lines))):
            o = orig_lines[i]
            n = new_lines[i]
            if o != n:
                preview_lines.append(f"  - |{o}|")
                preview_lines.append(f"  + |{n}|")

    return "\n".join(preview_lines) if preview_lines else "(frontmatter change)"


def _backup_file(file_path, content):
    """Backup file content to /tmp/dsr2-fix-backup/ preserving directory structure.

    Returns the backup path.
    """
    # Derive relative path from .../sih-docs/ or fallback to basename
    # We use the full absolute path to create a unique backup location
    abs_path = os.path.abspath(file_path)
    # Strip leading / to create a relative path under backup root
    rel_storage = abs_path.lstrip("/")
    backup_path = os.path.join(BACKUP_ROOT, rel_storage) + ".bak"

    os.makedirs(os.path.dirname(backup_path), exist_ok=True)
    with open(backup_path, "w", encoding="utf-8") as f:
        f.write(content)

    return backup_path


# ---------------------------------------------------------------------------
# Scanning
# ---------------------------------------------------------------------------


def scan_sih_files(sih_docs_root):
    """Recursively find all .sih.md files under sih_docs_root."""
    matches = []
    for root, dirs, files in os.walk(sih_docs_root):
        for fn in files:
            if fn.endswith(".sih.md"):
                full_path = os.path.join(root, fn)
                rel_path = os.path.relpath(full_path, sih_docs_root)
                matches.append((full_path, rel_path))
    return sorted(matches, key=lambda x: x[1])


# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------


def print_report(results, dry_run):
    """Print a formatted summary of all results."""
    modified_count = 0
    unchanged_count = 0
    skipped_count = 0

    print("=" * 72)
    mode = "DRY-RUN (no changes written)" if dry_run else "LIVE (changes written)"
    print(f"DSR2 Fix Report -- {mode}")
    print("=" * 72)

    for r in results:
        status = r["status"]
        path = r["path"]

        if status == "unchanged":
            unchanged_count += 1
            continue

        if status == "skipped":
            skipped_count += 1
            reason = r.get("reason", "")
            print(f"\n  [SKIP] {path}")
            if reason:
                print(f"         reason: {reason}")
            continue

        if status in ("modified", "would_modify"):
            modified_count += 1
            label = "[WOULD]" if dry_run else "[FIX]"
            print(f"\n  {label} {path}")
            for ch in r["changes"]:
                print(f"         {ch}")
            if dry_run and r.get("diff_preview"):
                print(f"         diff preview:")
                print(f"{r['diff_preview']}")
            if not dry_run and r.get("backup"):
                print(f"         backup: {r['backup']}")

    print()
    print("-" * 72)
    print(
        f"Summary: {modified_count} modified, "
        f"{unchanged_count} unchanged, "
        f"{skipped_count} skipped"
    )
    if dry_run:
        print("No files were written. Run without --dry-run to apply.")
    print("=" * 72)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Fix stage values and missing frontmatter fields in .sih.md files."
    )
    parser.add_argument(
        "--sih-docs",
        default=None,
        help="Path to sih-docs directory (default: ./sih-docs relative to CWD)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        default=None,
        help="Preview changes without writing (default mode)",
    )
    parser.add_argument(
        "--no-dry-run",
        action="store_true",
        help="Actually write changes (overrides --dry-run default)",
    )

    args = parser.parse_args(argv)

    # Resolve sih-docs path
    if args.sih_docs is None:
        args.sih_docs = os.path.normpath(
            os.path.join(os.getcwd(), "sih-docs")
        )

    # Dry-run logic: --no-dry-run flag disables; default is dry-run
    if args.no_dry_run:
        args.dry_run = False
    elif args.dry_run is None:
        args.dry_run = True

    args.sih_docs = os.path.abspath(args.sih_docs)

    return args


def main():
    args = parse_args()

    sih_docs = args.sih_docs
    dry_run = args.dry_run

    # Validate sih-docs directory
    if not os.path.isdir(sih_docs):
        print(f"ERROR: sih-docs directory not found: {sih_docs}", file=sys.stderr)
        print("Use --sih-docs PATH to specify the correct location.", file=sys.stderr)
        sys.exit(1)

    print(f"Scanning: {sih_docs}")
    print(f"Mode:     {'DRY-RUN' if dry_run else 'LIVE'}")

    # Find all .sih.md files
    files = scan_sih_files(sih_docs)
    print(f"Found {len(files)} .sih.md file(s)")
    print()

    # Process each file
    results = []
    for full_path, rel_path in files:
        result = fix_file(full_path, rel_path, dry_run)
        results.append(result)

    # Print report
    print_report(results, dry_run)

    # Exit code: non-zero if any errors
    errors = [r for r in results if r.get("status") == "error"]
    if errors:
        sys.exit(1)


if __name__ == "__main__":
    main()
