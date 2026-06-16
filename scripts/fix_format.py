#!/usr/bin/env python3
"""Fix format violations in .sih.md files: character replacements per AGENTS.md style guide.
Skips fenced code blocks and frontmatter."""
import sys, os, re
from pathlib import Path

REPLACEMENTS = [
    # (description, regex pattern, replacement, skip_in_blocks)
    # em-dash → fullwidth colon (when acting as connector between Chinese)
    ('em-dash→fullwidth colon', '\u2014', '\uff1a', True),
    # right arrow → ->
    ('right arrow→->', '\u2192', '->', True),
    # left arrow → <-
    ('left arrow→<-', '\u2190', '<-', True),
    # curly quotes → straight
    ('left curly quote→"', '\u201c', '"', True),
    ('right curly quote→"', '\u201d', '"', True),
    # middle dot → hyphen
    ('middle dot→-', '\u00b7', '-', True),
    # section sign → $
    ('section sign→$', '\u00a7', '$', True),
    # not-equal → !=
    ('not-equal→!=', '\u2260', '!=', True),
    # bidirectional arrow →
    ('bidirectional arrow→<->', '\u2194', '<->', True),
    # horizontal ellipsis → ...
    ('horizontal ellipsis→...', '\u2026', '...', True),
    # circled digits → parenthesized
    ('circled 1→(1)', '\u2460', '(1)', True),
    ('circled 4→(4)', '\u2463', '(4)', True),
    # multiplication sign → x
    ('multiplication sign→x', '\u00d7', 'x', True),
    # ≥ → >=
    ('≥→>=', '\u2265', '>=', True),
    # ≤ → <=
    ('≤→<=', '\u2264', '<=', True),
    # subscript numbers → regular
    ('subscript 1→1', '\u2081', '1', True),
    ('subscript 2→2', '\u2082', '2', True),
    ('subscript 3→3', '\u2083', '3', True),
    # checkmark → [OK]
    ('checkmark→[OK]', '\u2713', '[OK]', True),
    # cross mark → [X]
    ('cross mark→[X]', '\u274c', '[X]', True),
]

def fix_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.split('\n')
    new_lines = []
    in_frontmatter = False
    in_codeblock = False
    frontmatter_closed = False
    changes = 0

    for line in lines:
        stripped = line.strip()

        # Track frontmatter
        if not frontmatter_closed:
            if len(new_lines) == 0 and stripped == '---':
                in_frontmatter = True
                new_lines.append(line)
                continue
            if in_frontmatter and stripped == '---':
                in_frontmatter = False
                frontmatter_closed = True
                new_lines.append(line)
                continue
            if in_frontmatter:
                new_lines.append(line)
                continue

        # Track code blocks
        if stripped.startswith('```'):
            in_codeblock = not in_codeblock
            new_lines.append(line)
            continue

        # Skip code block content and frontmatter
        if in_codeblock:
            new_lines.append(line)
            continue

        # Apply replacements
        new_line = line
        for desc, pattern, replacement, skip_in_blocks in REPLACEMENTS:
            if pattern in new_line:
                count = new_line.count(pattern)
                new_line = new_line.replace(pattern, replacement)
                changes += count

        new_lines.append(new_line)

    if changes > 0:
        new_content = '\n'.join(new_lines)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"  {filepath}: {changes} replacements")
        return changes
    return 0

def main():
    root = sys.argv[1] if len(sys.argv) > 1 else 'docs'
    total_changes = 0
    files_fixed = 0

    for filepath in Path(root).rglob('*.sih.md'):
        changes = fix_file(str(filepath))
        if changes > 0:
            files_fixed += 1
            total_changes += changes

    # Also fix AGENTS.md
    agents = 'AGENTS.md'
    if os.path.exists(agents):
        changes = fix_file(agents)
        if changes > 0:
            files_fixed += 1
            total_changes += changes

    # Fix README.md
    readme = 'README.md'
    if os.path.exists(readme):
        changes = fix_file(readme)
        if changes > 0:
            files_fixed += 1
            total_changes += changes

    print(f"\nTotal: {total_changes} replacements in {files_fixed} files")

if __name__ == '__main__':
    main()
