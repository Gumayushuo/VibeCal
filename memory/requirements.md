# Requirements

## Objective
- Build a personal Windows 11 desktop client that makes Apple Calendar feel like a native desktop app while still relying on Apple's web experience.

## Target User
- Single personal user.

## Product Direction
- Use a wrapper-style desktop client instead of rebuilding calendar functionality.
- Prefer Tauri + WebView2 for v1.
- Allow system-browser-assisted sign-in if that is more reliable than embedded login.

## v1 Scope
- Open directly into Apple Calendar from a desktop application.
- Keep the user signed in across launches when possible.
- Remember window geometry and window state.
- Provide tray integration.
- Support launching at system startup.
- Prevent multiple active instances.
- Support Windows native notifications.
- Fit normal Windows 11 usage patterns, including pinning to taskbar or desktop shortcuts.

## Constraints
- Windows 11 only.
- Personal-use quality bar, not public distribution quality.
- Generated repository content should remain in English.
- User-facing chat and explanations should remain in Chinese.
- External Apple login and web behavior may limit implementation details.

## Risks And Unknowns
- Embedded Apple sign-in may be blocked or unstable.
- Session persistence behavior may differ across WebView and browser-assisted flows.
- Notification support may depend on how the wrapper integrates with Windows APIs and the web content.
- Browser-assisted login should not be assumed to hydrate the app's WebView session automatically.

## Definition Of Done
- On Windows 11, the user can launch the app and reach Apple Calendar with a durable signed-in experience as the primary target.
- Core desktop features for v1 are represented in the implementation plan and later delivered incrementally.
