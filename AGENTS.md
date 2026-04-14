# Apple Calendar Desktop

## Purpose
- Build a personal-use Windows 11 desktop client that provides fast access to Apple Calendar with a stable signed-in experience.
- Prioritize a wrapper-based desktop experience over rebuilding calendar functionality.

## Project Scope
- Target platform: Windows 11.
- Target user: single personal user.
- Preferred direction for v1: Tauri + WebView2.
- Accept system-browser-assisted sign-in if embedded sign-in is blocked or unreliable.

## v1 Goals
- Open Apple Calendar quickly from a desktop app.
- Preserve login state across launches when technically possible.
- Remember window size, position, and last window state.
- Support tray behavior.
- Support auto start.
- Enforce single instance behavior.
- Include Windows native notifications as a v1 target.
- Be easy to pin on desktop or taskbar.

## Non-Goals For Now
- Public distribution or multi-user support.
- Rebuilding Apple Calendar as a custom native UI.
- Deep iCloud reverse engineering beyond what is needed for a stable wrapper experience.

## Working Order
1. Read `memory/index.md`.
2. Read `memory/active-context.md`.
3. Open only the smallest additional memory file needed for the task.
4. Make the smallest viable change that moves the project forward.
5. Update memory when requirements, decisions, or active context change.

## Repository Conventions
- Use Chinese for user-facing chat.
- Use English for all repository files, filenames, headings, code comments, templates, and memory.
- Keep the repository structure minimal and practical.
- Prefer incremental decisions over speculative architecture.
- Record external platform constraints explicitly when they affect implementation.

## Memory Update Policy
- Update `memory/active-context.md` after each meaningful milestone or change in immediate focus.
- Update `memory/requirements.md` when scope, goals, or constraints change.
- Update `memory/decisions.md` when a decision affects architecture, tooling, UX direction, or delivery risk.
- Update `memory/index.md` only when the memory structure changes.

## Review Policy
- Check every meaningful change for scope fit, simplicity, and Windows 11 usability.
- Treat Apple login flow, embedded web compatibility, and session persistence as high-risk integration points.
- Before final delivery, verify the output against the current definition of done and note any platform-dependent uncertainty.

## Definition Of Done
- The requested output exists in the repository.
- Assumptions and external dependency risks are surfaced.
- Relevant decisions are recorded in memory.
- The result is consistent with the current v1 direction: a Windows desktop wrapper for Apple Calendar with persistent session behavior as a primary goal.
