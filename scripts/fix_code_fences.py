#!/usr/bin/env python3
"""Add missing 'text' language tag to unlabeled code fences in .sih.md files."""
import sys, re
from pathlib import Path

def fix_unlabeled_fences(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.split('\n')
    new_lines = []
    in_frontmatter = False
    frontmatter_closed = False
    in_codeblock = False
    changes = 0

    for i, line in enumerate(lines):
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

        # Check if this is an opening fence without language tag
        if stripped == '```' and not in_codeblock:
            # Opening fence without language tag — add 'text'
            new_line = line.replace('```', '```text')
            new_lines.append(new_line)
            in_codeblock = True
            changes += 1
        elif stripped == '```':
            # Closing fence
            in_codeblock = False
            new_lines.append(line)
        elif stripped.startswith('```'):
            # Has a language tag — track in_codeblock
            in_codeblock = not in_codeblock
            new_lines.append(line)
        else:
            new_lines.append(line)

    if changes > 0:
        new_content = '\n'.join(new_lines)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"  {filepath}: {changes} unlabeled fences fixed")
        return changes
    return 0

def main():
    root = sys.argv[1] if len(sys.argv) > 1 else 'docs'
    total = 0
    for filepath in Path(root).rglob('*.sih.md'):
        total += fix_unlabeled_fences(str(filepath))
    print(f"\nTotal: {total} fences fixed")

if __name__ == '__main__':
    main()
