---
name: improbable-recruiter
description: >
  Heuristically analyzes the codebase to identify a "cognitive or technical void." 
  Generates a job description for an atypical profile whose improbable expertise 
  would bring disruptive, high-leverage value to the project. 
  Use for strategic brainstorming or radical innovation cycles.
compatibility: Requires file system access and code analysis tools (grep, find, or cloc).
metadata:
  version: "1.0"
  author: "Principal Agentic Architect"
  generatedBy: "agent-architect-v1"
---

## Objective
To identify the "Black Swan" of talent: a profile whose intersection of skills seems foreign to the project but, through a logical pivot, would multiply its impact.

## Input Handling
1. **Directory Path**: Defaults to current working directory (`./`).
2. **Contextual Inference**: The agent analyzes the `README.md` and `package.json`/`requirements.txt` to determine the project's "Technical DNA."
3. **Ambiguity Resolution**: 
   - IF the codebase is empty or inaccessible: STOP. Call `AskUserQuestion` to request a valid path.
   - IF the project purpose is unclear: Call `AskUserQuestion` to ask: "What is the primary human impact of this software?"

## Steps

### Phase 1: Silent Discovery (The DNA Scan)
1. **Stack Identification**: Run `ls -R` and parse dependency files to categorize the project (e.g., "High-frequency trading," "Social networking," "Embedded firmware").
2. **Complexity Mapping**: Use `grep` to analyze comment density and `TODO` patterns to identify hidden technical debt or "tunnel vision."
3. **Semantic Extraction**: Read the `README.md` to identify the stated mission and contrast it with the actual implementation.

### Phase 2: State-Machine Augmented Generation (The Logic Gap)
- **Contrast Logic (SMAG)**:
    - **IF** Project == "Strictly Algorithmic/Data-Heavy" -> **THEN** Seek profile from "Human Sciences / Behavioral Psychology."
    - **IF** Project == "Pure UX/Frontend" -> **THEN** Seek profile from "System Safety / Logistics / Cryptography."
    - **IF** Project == "High Growth/Messy Code" -> **THEN** Seek profile from "Archival Science / Music Theory (Harmony)."
- **Validation**: Ensure the suggested profile is *improbable* but *defensible* through a specific technical bridge.

### Phase 3: Verbose Execution
1. **Profile Synthesis**: Generate an "Oxymoronic Job Title" (e.g., "Systemic Poetry Developer," "Chaos Orchestrator").
2. **Unexpected Value Proposition**: Explicitly define the technical or cultural "bridge" (e.g., "How an Ethnobotanist can solve your database scaling hierarchy").
3. **Actionable Missions**: Create 3 tasks for this new hire using the project's actual terminology.

## Output
The agent must present the findings using the following structure:

### 🕵️ The Improbable Profile: [Job Title]
**Why them?** [1-sentence logical pivot]

| Attribute | Value |
| :--- | :--- |
| **Missing Perspective** | [Identification of the team's blind spot] |
| **Disruptive Value** | [Description of the unexpected ROI] |
| **Improbability Score** | [8-10/10] |

**Immediate Impact Checklist:**
- [ ] Task 1: [Integration of their expertise with the current codebase]
- [ ] Task 2: [Strategic shift they would initiate]
- [ ] Task 3: [Metric they would improve that others ignored]

## Guardrails
- **No Hallucinations**: Do not invent tech stacks; only use what is detected in the `Phase 1` JSON/File scan.
- **Scope Limitation**: Do not modify any files. This is a read-only analytical skill.
- **Anti-Generic Filter**: If the suggested profile is just a "Senior Dev" or "Product Manager," the agent MUST discard and recalculate for a higher "Improbability Score."
