---
name: oss-community-catalyst
description: >
  Professional-grade audit and amplification module for Open Source projects. 
  Optimizes repository "Social Proof" (Markdown health) and generates 
  high-engagement technical communication for social platforms.
compatibility: Requires standard shell tools (grep, ls) and git.
metadata:
  version: "1.1.0"
  author: "Agent Architect"
  generatedBy: "agent-architect-v1"
---

# Skill: OSS Community Catalyst

## Input
* **Repo Path**: Target directory.
* **Objective**: (e.g., "Feature Launch", "Contributor Drive", "Project Polish").
* **Voice**: (e.g., "Academic", "Hype/Energetic", "Minimalist").

## Steps

### Phase 1: The "Onboarding Friction" Audit
1.  **Metadata Scrape**: 
    * Extract repository description and primary language.
    * Run `ls -F --json` to map project structure.
2.  **Friction Check**:
    * Search for keywords: "Installation", "Quick Start", "Usage".
    * Logic: 
        * IF "Installation" header is > 50 lines from top of `README.md`: **Flag as "High Friction"**.
3.  **Social Readiness**:
    * Check for `.github/` folder and issue templates.

### Phase 2: Cognitive Alignment (Disambiguation)
1.  **Audience Targeting**:
    * Call `AskUserQuestion`: "Who is the primary target for this wave? (1) Potential Users, (2) Future Contributors, or (3) Venture/Sponsorship partners?"
2.  **Context Extraction**:
    * IF objective is "Launch": Scan `CHANGELOG.md` or recent commit messages to summarize "What's New".

### Phase 3: Strategic Amplification (Execution)
1.  **README Refactoring**:
    * Suggested diff for the "Hero" section (First 20 lines).
    * Add "Community Badges" if missing (Discord/Slack, CI status).
2.  **The "Social Pack" Generation**:
    * **X/Twitter**: Thread structure for deep technical dives or single-post for hype.
    * **LinkedIn**: Narrative-driven (Problem -> Solution -> Community).
    * **Dev.to/Reddit**: High-value technical summary with markdown code blocks.

## Output
* **Project Vitality Score**: A summary of documentation health.
* **Refactoring Suggestions**: Markdown snippets for `README.md` and `CONTRIBUTING.md`.
* **The Launchpad**: 
    * [ ] Social Post: X
    * [ ] Social Post: LinkedIn
    * [ ] Social Post: Reddit/Niche
* **Final Action**: "Would you like me to apply these Markdown improvements to your files now?"

## Guardrails
* **NO DESTRUCTIVE EDITS**: Use `diff` format or temporary files before overwriting original documentation.
* **LINK INTEGRITY**: Never include a link in a social post that hasn't been verified via `curl -I`.
* **NO LLM FLUFF**: Avoid "In the rapidly evolving landscape of..."—use direct, developer-centric language.