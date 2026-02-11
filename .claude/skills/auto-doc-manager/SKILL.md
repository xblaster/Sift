---
name: auto-doc-manager
description: >
  Synchronize and generate technical documentation (JSDoc, Docstrings, README, OpenAPI) from source code.
  Use when the user needs to document new functions, update a README based on recent changes, 
  or regenerate API specs. Handles multi-language detection and consistency checks.
compatibility: Requires file system access (ls, grep, cat), and optionally language-specific CLI (jsdoc, swagger-jsdoc).
metadata:
  version: "1.0"
  author: "Agent Architect"
  generatedBy: "agent-architect-v1"
---

# Auto-Doc Manager: The "Live Documentation" Module

Maintain a perfectly synchronized documentation ecosystem. This skill prevents "Doc Decay" by analyzing the source code before proposing updates.

## Input Handling
1. **Explicit Target**: User specifies a file or directory (e.g., `doc-writer-sync "src/api/"`).
2. **Contextual Inference**: If no target is provided, scan the current git diff to identify modified but undocumented files.
3. **Ambiguity Resolution**: 
   - IF multiple documentation types are possible (e.g., updating JSDoc OR OpenAPI): 
     - Call `AskUserQuestion` with options: [JSDoc/Docstrings, README.md, OpenAPI/Swagger].

## Steps

### Phase 1: Silent Discovery & State Analysis
1. **Tech Stack Identification**: Run `ls -F` and parse `package.json` or `requirements.txt` via `--json` to detect the environment (Node.js, Python, Go).
2. **Missing Doc Detection**: 
   - For JS/TS: Use `grep` or an AST-based parser to find exported functions without `/** ... */` blocks.
   - For Python: Identify classes/functions without `""" ... """`.
   - For APIs: Search for route patterns (e.g., `@app.get`, `router.post`) and check if they exist in `swagger.json` or `openapi.yaml`.
3. **State Check**: Compare timestamps of source files vs. documentation files.

### Phase 2: Strategy Definition (SMAG)
- **State A**: Code exists, Documentation is missing/outdated.
- **Transition**: Generate specific documentation block based on code signature (params, return types).
- **State B**: Documentation proposed and pending validation.

### Phase 3: Generation & Feedback (Verbose)
1. **Drafting**: Create the docstring/spec using the project's established style.
2. **User Review**:
   - Display a side-by-side comparison using a Markdown table:
     | Context | Change Proposed |
     | :--- | :--- |
     | `function add(a, b)` | Added `@param {number} a`, `@param {number} b` |
3. **Confirmation**: Use `AskUserQuestion` to confirm: "Apply these documentation updates to [X] files? (y/n)".

### Phase 4: Execution
- Update files using atomic writes.
- **Verification**: Run a final lint check on the documentation if tools are available (e.g., `eslint-plugin-jsdoc`).

## Output Formatting
- Use **Checklists** to show progress:
  - [ ] Analysis of `src/controllers/`
  - [ ] JSDoc Generation
  - [ ] OpenAPI Schema Sync
- Summary of changes:
  - `✓` 4 Docstrings added.
  - `⚠` 1 README section requires manual review (Logic too complex for auto-sync).

## Guardrails
- **No Hallucination**: Never invent API parameters or function purposes. If the code logic is ambiguous, flag it as `[TODO: Manual Description Required]`.
- **Scope Restriction**: DO NOT modify functional code logic. Limit edits strictly to comment blocks and `.md`/`.yaml`/`.json` doc files.
- **Preservation**: Do not overwrite existing manual documentation without explicit user confirmation.
- **Size Limit**: If the diff is > 500 lines, stop and ask the user to process by subdirectory.