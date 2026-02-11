---
name: git-master
description: >
  Advanced Git workflow orchestrator. Specialized in generating Conventional Commits, 
  crafting descriptive Pull Requests, and guiding users through Merge Conflict resolution. 
  Use when the user needs to commit changes, open a PR on GitHub, or fix broken merges 
  while maintaining high standards of repository hygiene.
compatibility: Requires git CLI, gh (GitHub) CLI, and access to a repository.
metadata:
  version: "1.0"
  author: "Agent Architect"
  generatedBy: "agent-architect-v1"
---

# Git Master: Cognitive Module for Repository Excellence

Orchestrate the end-to-end lifecycle of a contribution, from staged changes to resolved conflicts and PR creation.

## 1. Input Handling & Ambiguity Resolution
- **Context Detection**: Always check `git status --porcelain` and `git branch --show-current` first.
- **Ambiguity Hierarchy**:
    1. Check for staged files. If none, ask: "Which files should I stage for this commit?"
    2. Check for current PRs. If one exists for the branch, pivot to "Update PR" mode.
    3. If multiple commit types (feat, fix, chore) seem applicable, use `AskUserQuestion`.

---

## 2. Steps

### Phase 1: Conventional Commit Generation (Discovery)
1.  **State Check**: Run `git diff --cached --name-only` to see what is staged.
    - *Logic*: IF output is empty, run `git status` and ask the user to stage files.
2.  **Analysis**: Run `git diff --cached` to analyze changes.
    - *Constraint*: Parse the diff to identify the scope (e.g., file paths, function names).
3.  **Drafting**: Propose a message following the pattern: `<type>(<scope>): <description>`.
    - Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`.
4.  **Verification**: Ask the user: "Does this commit message accurately reflect your intent?"

### Phase 2: Pull Request Orchestration (State-Machine)
1.  **Duplicate Check**: Run `gh pr list --head $(git branch --show-current) --json number,url`.
    - *Logic*: IF a PR exists, provide the link and ask to update the description instead of creating a new one.
2.  **Metadata Gathering**:
    - **Templates**: Search `.github/PULL_REQUEST_TEMPLATE/*.md`.
    - **Base Branch**: Run `gh repo view --json defaultBranch`.
3.  **Disambiguation**: IF multiple templates exist, call `AskUserQuestion` to select the right one.
4.  **Creation**: Generate the PR body by summarizing the commits since the base branch.
    - *Logic*: Run `git log <base-branch>..HEAD --oneline` to feed the summary.

### Phase 3: Conflict Management (Recovery)
1.  **Detection**: Run `git status --porcelain`.
    - *Logic*: Look for lines starting with `UU`, `AA`, or `DD`.
2.  **Reporting**: Create a table of conflicted files.
    - | File | Status | Conflict Type |
      | :--- | :--- | :--- |
3.  **Resolution Guide**: For each file:
    - Display the conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`).
    - Explain the difference between "Current Change" and "Incoming Change".
    - Ask: "Which version should we keep, or should we merge them manually?"

---

## 3. Output & Feedback
- Use **Checklists** to show progress:
    - [x] Staged changes analyzed.
    - [ ] Conventional Commit message generated.
    - [ ] PR Template selected.
- Use **Emojis** for status:
    - ✅ Commit successful: `<short_sha>`.
    - ⚠️ Conflict detected in `<filename>`.
    - 🚀 PR Created: `<url>`.

---

## 4. Guardrails
- **No Force-Push**: Never use `--force` or `--hard` unless specifically and separately confirmed by the user after a warning.
- **Main Branch Protection**: NEVER commit directly to `main` or `master` without explicit user override.
- **JSON First**: Always use `--json` flags with `gh` to avoid parsing raw terminal strings.
- **Scope Limitation**: Do not modify `.gitignore` or hidden config files unless specifically requested.