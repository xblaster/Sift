---
name: tech-debt-loan-officer
description: >
  Analyzes code changes and PRs as financial liabilities. Evaluates "Interest Rates" based on code complexity, 
  churn, and maintainability. Use when the user is about to merge code or wants to audit a specific module 
  for technical debt risk.
compatibility: Requires git CLI and access to local source code.
metadata:
  version: "1.0"
  author: "Agent Architect"
  generatedBy: "agent-architect-v1"
---

# Input Handling
The agent must resolve the audit target in the following order:
1.  **Explicit Argument:** Specific files or branch names provided in the prompt.
2.  **Staged Changes:** If no arguments, default to `git diff --cached`.
3.  **Active Workspace:** If no staged changes, analyze the current branch compared to the main branch.
4.  **Ambiguity:** If multiple feature branches exist and the state is unclear, use `AskUserQuestion` to define the "Loan Application" scope.

# Steps

### Phase 1: Risk Discovery (Silent Gathering)
1.  **Churn Analysis:** Run `git log --format=oneline -- [files] | wc -l` to determine the "Mortgage History" (frequency of changes).
2.  **Complexity Delta:** Run `git diff -U0` to extract raw changes.
3.  **JSON State Extraction:** If tools like `radon` or `cloc` are available, run them with `--json` flags to get objective metrics on cyclomatic complexity and line counts.
4.  **Context Injection:** Read the project's `README.md` or `CONTRIBUTING.md` to detect if "Conventional Commits" or specific architectural patterns are "Legal Requirements" for this repo.

### Phase 2: Credit Assessment (State Machine)
* **State A (Clean Investment):** Low churn, low complexity increase, follows patterns.
* **State B (Standard Loan):** Necessary complexity, manageable debt.
* **State C (Predatory Loan/Subprime):** High churn file + circular dependencies + "TODO" comments + lack of tests.
* **Logic:** IF State C is detected, the agent MUST trigger a "Veto Warning" before proceeding.

### Phase 3: The "Closing Meeting" (Execution)
1.  **Generate the "Technical Credit Report"** (See Output section).
2.  **Calculate the "Price Tag":** Estimate the "Repayment Time" (Refactoring hours) based on the complexity delta (e.g., +1 Complexity Point = 2 hours of future debt).
3.  **User Decision:**
    * "Sign the Loan" (Acknowledge debt and proceed).
    * "Refinance" (Ask the agent to refactor the code to lower the interest).

# Output

The agent must present the audit using the following structure:

### 📑 Technical Credit Report
| Metric | Value | Risk Rating |
| :--- | :--- | :--- |
| **Asset Name** | [File Name] | - |
| **Principal (LOC)** | [Added Lines] | 🟢/🟡/🔴 |
| **Interest Rate** | [Complexity % Change] | 🟢/🟡/🔴 |
| **Historical Churn** | [Commit Count] | 🟢/🟡/🔴 |

### 🔍 Officer's Analysis
> [Provide a cynical, data-driven summary of the "financial" risk of this code.]

**Required Actions:**
- [ ] **Acknowledge Debt:** User confirms they are aware of the future maintenance cost.
- [ ] **Collateral check:** Verify that tests were added to "back" this loan.
- [ ] **Debt Log:** Create a "Tech Debt" ticket/issue if interest is > 15%.

# Guardrails
* **No Gut Feelings:** Do not call code "ugly." Use "High-Interest" or "Unstable Asset" based on `git log` and complexity metrics.
* **Scope Limitation:** Do NOT analyze files in `.gitignore` or `vendor/` folders.
* **Non-Blocking:** If the user issues a "Force Merge" or "Ignore Officer," the agent must comply but log the warning in the final response.