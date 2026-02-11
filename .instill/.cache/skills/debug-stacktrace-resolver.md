---
name: debug-stacktrace-resolver
description: >
  Analyze, research, and resolve complex software errors and stack traces. 
  Use when the user provides an error message, log dump, or stack trace. 
  This skill performs environment discovery, structured web research, and 
  proposes deterministic fixes after verifying context.
compatibility: Requires Internet access tool (e.g., Google Search/Tavily), File System access, and JSON-capable shell.
metadata:
  version: "1.0"
  author: "Agent Architect"
  generatedBy: "agent-architect-v1"
---

## Input Handling & Ambiguity
To ensure zero-hallucination, the agent must resolve inputs using the following hierarchy:
1. **Explicit Input:** The error message or file path provided in the prompt.
2. **Environmental Context:** If no error is provided, check the last 50 lines of active logs or the most recent build output.
3. **Missing Context:** If the language, framework version, or OS is not detectable from the stack trace, the agent **MUST** call `AskUserQuestion`.
   - *Example:* "I see a NullPointerException, but I cannot determine the Java version. Which JDK are you using?"

---

## Steps

### Phase 1: Discovery & State Analysis (Silent)
1.  **Log Extraction:** - If a file path is provided, read the file. 
    - If a process is failing, run the relevant command with `--json` or redirect `stderr` to a temporary file for parsing.
2.  **Fingerprinting:** Parse the stack trace to extract:
    - `Error_Type`: (e.g., RuntimeError, Segment Fault, SyntaxError).
    - `Faulty_File`: Identify the specific line in the local workspace.
    - `Dependencies`: Run `npm list --json`, `pip list --json`, or equivalent to get exact versions of involved packages.
3.  **State Verification:** - Check if the code was recently modified (Git diff).
    - Check if environmental variables (env) match expected patterns for the framework.

### Phase 2: Knowledge Acquisition (Web Search)
1.  **Structured Querying:** Do not search for "how to fix error". Use specific tokens:
    - Search 1: `"<Error_Type>" + "<Faulty_Package>" + "version <Version>"`
    - Search 2: `"<Top_Stack_Frame_Line>"`
    - Search 3: `"site:stackoverflow.com <Error_Message_Snippet>"`
2.  **Synthesis:** Compare search results against the local state. Identify the "Root Cause Candidate".

### Phase 3: Action & Resolution (Verbose)
1.  **Understanding Check:** Before proposing a fix, present a summary to the user:
    - "I have identified the error as [X] caused by [Y]. Does this align with your recent changes?"
2.  **Solution Generation:** Provide a step-by-step resolution plan.
    - **Step A:** Configuration changes or dependency updates.
    - **Step B:** Code modification (using `sed` or file rewrite).
3.  **Validation:** After the fix is applied, the agent **MUST** attempt to reproduce the trigger (e.g., run the build/test command) to verify the state transition from *Error* to *Success*.

---

## Output Formatting
The agent must present the findings using the following structure:

### 🔍 Error Analysis
| Component | Value |
| :--- | :--- |
| **Error Type** | `Name of the error` |
| **Location** | `path/to/file.ext:line` |
| **Detected Version** | `vX.Y.Z` |

### 🛠 Proposed Fix
- [ ] **Step 1:** Description of the change.
- [ ] **Step 2:** Command to run or code to insert.

> **Note:** Use ⚠ for breaking changes (e.g., major version upgrades).

---

## Guardrails
- **No Blind Fixes:** Never modify `node_modules`, `venv`, or system-level binary files directly unless it's a package manager command.
- **Scope Limitation:** Do not attempt to fix errors in files not mentioned in the stack trace unless they are direct configuration dependencies.
- **Search Hygiene:** Filter out results older than 2 years if the framework moves fast (e.g., React, Next.js), unless no other data exists.
- **Confirmation:** Always ask "Should I apply this fix now?" before writing to the filesystem.