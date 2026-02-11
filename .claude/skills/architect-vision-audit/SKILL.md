---
name: architect-vision-audit
description: >
  Acts as a High-Level Architect to audit the project's alignment with its global vision.
  Use when the user asks to "check alignment," "review architecture," "assess progress against goals," or "ensure we are on track."
  Performs a gap analysis between the defined Vision (documentation) and the Reality (codebase/git state).
compatibility: Requires 'git', 'find', and standard shell utilities.
metadata:
  version: "1.0"
  author: "Agent Architect"
  generatedBy: "agent-architect-v1"
---

# Project Vision & Alignment Audit

Orchestrate a high-level architectural review to ensure technical execution matches strategic intent.

## Input
- **Vision Source**: (Optional) Specific file defining the vision (e.g., `VISION.md`, `ARCHITECTURE.md`).
- **Focus Area**: (Optional) Specific directory or module to inspect.

## Steps

### Phase 1: Context Discovery (Silent Gathering)

1.  **Locate Vision Source of Truth**
    * Search for key documentation files using:
        ```bash
        find . -maxdepth 2 -name "*VISION*" -o -name "*ARCHITECTURE*" -o -name "*ROADMAP*" -not -path '*/.*'
        ```
    * **Logic (Ambiguity Resolution)**:
        * **IF** 0 files found: Call `AskUserQuestion`.
            * *Question*: "I cannot find a formal Vision or Architecture document. Please specify the filename or paste the core principles."
        * **IF** >1 files found (e.g., `VISION.md` and `docs/ARCHITECTURE.md`): Call `AskUserQuestion`.
            * *Question*: "Multiple architecture documents found. Which one represents the current source of truth?"
        * **IF** 1 file found: Read content using `cat <filename>`.

2.  **Snapshot Project Reality (JSON Determinism)**
    * To understand the "actual" direction of work, analyze recent commits and file structure.
    * Run `git log -n 10 --pretty=format:'{"hash":"%h", "author":"%an", "date":"%ad", "subject":"%s"}'` to get structured history.
    * Run `tree -L 2 -J .` (if `tree` is missing, use `find . -maxdepth 2`) to get the high-level topology of the codebase.

### Phase 2: Analysis & Gap Detection (Internal Processing)

3.  **Architectural Synthesis**
    * *Internal Thought Process*: Compare the **Vision Content** (Step 1) against the **Recent Activity/Structure** (Step 2).
    * **Check for**:
        * *Scope Creep*: Do recent commits (Step 2) align with the goals in Vision (Step 1)?
        * *Structural Drift*: Does the folder structure reflect the architectural patterns (e.g., Microservices vs Monolith) defined in the vision?
        * *Tech Debt*: Are there "fix" or "hack" patterns in the git log that contradict "quality" goals?

### Phase 3: Reporting (Action)

4.  **Generate Audit Report**
    * Present the findings using the output format below.

## Output

Display the analysis in the following format:

### 🏛️ High-Level Alignment Report

**Status**: [ ✅ On Track | ⚠️ Minor Deviations | 🚨 Critical Misalignment ]

| Vision Component | Current Reality (Evidence) | Verdict |
| :--- | :--- | :--- |
| *[e.g., "Use Modular Architecture"]* | *[e.g., "Commits show tight coupling in /src"]* | ⚠️ Drift |
| *[e.g., "TypeScript Only"]* | *[e.g., "New .js files detected in /lib"]* | 🚨 Violation |

**Strategic Recommendations:**
* [ ] Actionable step 1
* [ ] Actionable step 2

## Guardrails

* **Read-Only Operation**: This skill is strictly for *auditing*. Do NOT modify, delete, or refactor code during this execution.
* **No Hallucinations**: If the Vision document is vague, do NOT infer the architectural style. State "Undefined" in the report.
* **Privacy**: Do not output full source code content in the summary, only structural references and commit messages.