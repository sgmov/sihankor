# SiHankor Document Style Guide

## Character Constraints

- Use only ASCII characters and CJK characters. Do not use emojis or other non-ASCII symbols.
- Replace em-dash (U+2014) with fullwidth colon (U+FF1A) when the em-dash acts as a connector between Chinese words or clauses. Example: `A——B` → `A：B`.
- Replace middle dot (U+00B7) with ASCII hyphen `-`.
- Replace left curly quote (U+201C) and right curly quote (U+201D) with straight double quotes `"`.
- Replace right arrow (U+2192) with `->` and left arrow (U+2190) with `<-`.
- Replace not-equal sign (U+2260) with `!=`.
- Permitted CJK punctuation: U+3001, U+3002, U+FF0C, U+FF1A, U+FF1B, U+FF08, U+FF09, U+300A, U+300B, U+300C, U+300D.
- Apply all character replacement rules **only to main narrative text**. Do not modify content inside code fences, Mermaid blocks or frontmatter.
- If a character cannot be converted to standard ASCII or CJK characters without altering the original meaning, retain the original character only when necessary and add a brief note for this exception. Do not create arbitrary replacements.

## Structure Constraints

- Horizontal rules (`---`) are prohibited in the main body. Horizontal rules may only be used as opening and closing delimiters for frontmatter.
- Use level-2 headings (`##`) for section separation; do not use horizontal rules.
- Tables are limited to a maximum of 3 columns. Split wide tables into bullet lists or subsections.
- All fenced code blocks must specify a valid language tag: `mermaid`, `text`, `yaml`, `json`, `rust`.
- Empty fenced code blocks are not allowed. Every code block must contain a valid language tag and actual content.
- If a fenced code block has an unsupported language tag or contains no content, convert it to a valid code block with one of the permitted languages listed above. If conversion is not feasible, remove the code block and describe its content with plain text.
- ASCII art diagrams are prohibited. Use Mermaid `flowchart` for all flowcharts, relationship diagrams and structural diagrams.

## Typography Constraints

- Use bold (`**`) solely for term definition statements and highlighted numeric values. Do not apply bold to regular body text or examples.
- Keep each list item concise and limited to one single concept. Use paragraphs instead of bullet points for lengthy content.
- Do not create deeply nested lists; the maximum nesting level is 2.

## Frontmatter

Frontmatter must be valid YAML wrapped between `---` delimiters. Mandatory fields: `id`, `type`, `stage`. The `---` delimiters for frontmatter are the only permitted horizontal rules across all documents.

## Mermaid Diagrams

- Adopt `flowchart` for all flow and relationship diagrams.
- Keep node labels brief. Use `<br/>` for line breaks within labels.
- Keep edge labels under 10 characters in length.
