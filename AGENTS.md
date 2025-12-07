# Agent Guide

This repository uses a prompts-driven workflow to understand the current architecture and plan new features. When working in this repo, follow the flow below when asked to "run the prompts workflow" or similar.

## Prompts Workflow

1) Architecture snapshot

- Read `prompts/understand_architecture.md` carefully as the framing input.
- Explore the codebase as needed to build an accurate, up-to-date view.
- Write a concise but thorough snapshot to `prompts/current_architecture.md`.
  - Overwrite existing content (this file is a refreshable snapshot).
  - Use short, scannable sections and include file references like `src/path.rs:42` when helpful.

2) Feature planning

- Read the feature description in `prompts/features/<feature>` (a Markdown/plaintext file).
- Read `prompts/plan_feature.md` for instructions/spec on how to structure the plan.
- Produce the plan in `prompts/new_feature.md`.
  - Overwrite existing content (this file is the latest plan draft).
  - The plan should be implementation-ready, but do not modify code unless explicitly asked.

## Conventions

- Keep edits narrowly scoped to the prompt files above unless explicitly asked to implement.
- When generating or updating plan content, prefer:
  - Clear headings, short lists, explicit file paths, and small code blocks as needed.
  - Call out assumptions and open questions at the end.
- If the specified feature file doesn’t exist, ask the user which feature to plan (list available files in `prompts/features/`).

## Examples

- "Run the prompts workflow for feature: groups"
  - Reads: `prompts/understand_architecture.md`, `prompts/features/groups`, and `prompts/plan_feature.md`
  - Writes: `prompts/current_architecture.md` and `prompts/new_feature.md`

