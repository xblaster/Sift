---
name: security-audit-tool
description: >
  Performs a comprehensive security audit of a specific directory or file using 
  automated scanning tools (like Semgrep, Snyk, or Trivy). Use when the user 
  needs to identify vulnerabilities, secrets leakage, or misconfigurations. 
  Handles tool discovery, state-based risk assessment, and structured reporting.
compatibility: Requires security-focused CLI tools (semgrep, snyk, or trivy) installed in PATH.
metadata:
  version: "1.0.0"
  author: "Agent Architect"
  generatedBy: "agent-architect-v1"
---

# Skill: Security Auditing

## Input Handling & Resolution
1.  **Target Path**: Identify the directory or file to audit.
    * If provided in prompt: Use it.
    * If not: Check current working directory.
    * If ambiguous: Use `AskUserQuestion` to confirm the target path.
2.  **Tool Selection**:
    * Priority 1: Tool specified by the user.
    * Priority 2: Detect project type (e.g., `package.json` -> Snyk/Semgrep, `Dockerfile` -> Trivy).
    * Last Resort: `AskUserQuestion` to select from available tools on the system.

## Steps

### Phase 1: Discovery & Pre-flight (Silent Gathering)
* **Tool Verification**: Run `<tool> --version --json` (or equivalent) to ensure the binary exists and is functional.
* **Environment Check**: Verify read permissions on the target path.
* **Existing State**: Check for existing audit reports (e.g., `sast-results.json`) to avoid redundant compute.

### Phase 2: Pre-condition Check (SMAG)
* **Connectivity**: If the tool requires cloud syncing (e.g., Snyk), run `auth status --json` to verify the session.
* **Logic**:
    * **IF** tool is missing: **STOP**. Inform user: "Tool <X> is not installed. Would you like to try <Alternative>?"
    * **IF** target is too large (>1GB) without a `.ignore` file: **WARNING**. Ask the user if they wish to proceed or define exclusions.

### Phase 3: Execution (The Audit)
1.  **Run Scanner**: Execute the audit command with JSON output flags to ensure deterministic parsing.
    * *Semgrep*: `semgrep scan --json --config auto`
    * *Snyk*: `snyk test --json`
    * *Trivy*: `trivy fs --format json`
2.  **Data Extraction**: Parse the JSON output to categorize findings by:
    * Severity (Critical, High, Medium, Low)
    * CWE/CVE identifiers
    * File location (Line/Column)

### Phase 4: Reporting (UX Impeccable)
* Summarize findings using a **Markdown Table**.
* Use status emojis: 🔴 (Critical), 🟠 (High), 🟡 (Medium), 🔵 (Info).
* Provide a "Remediation Checklist" based on the highest-priority vulnerabilities.

## Output Format
| Severity | ID | Description | Location |
| :--- | :--- | :--- | :--- |
| 🔴 Critical | CVE-2023-XXXX | SQL Injection vulnerability | `src/db/query.js:42` |

- [ ] **Next Step**: Run `fix` command (if supported by tool).
- [ ] **Next Step**: Generate a detailed HTML report for stakeholders.

## Guardrails
* **No Data Exfiltration**: Do NOT upload code to external APIs unless the tool is explicitly configured for cloud-based scanning by the user.
* **Read-Only Integrity**: Do NOT use "auto-fix" flags unless specifically requested in a subsequent prompt.
* **Scope Limitation**: Do NOT audit directories outside of the identified project root (e.g., avoid `/etc` or system-level folders).
* **Performance**: If scanning a large `node_modules` or `vendor` folder, ensure the `--exclude` flag is applied to focus on first-party code.