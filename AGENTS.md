# Project implementation language

- New features and changes primarily use Rust. UI layout, interaction state and animation logic belong in `ui/`; protocol and shared data policies belong in `core/`.
- Reference implementations must also be adapted to Rust. Do not add Kotlin page layouts or JavaScript application animation controllers when following another repository.
- Keep Android code limited to necessary lifecycle, platform APIs, media-service integration and bridges to existing state. Existing platform code does not authorize expanding it into a second UI implementation.
- CSS may define presentation and compositor properties; Rust owns animation state, progress and timing where custom motion is needed. JavaScript remains generated WASM bindings, bootstrap and test tooling.
- Preserve upstream per-file license notices and document adaptation scope. Do not assume every file shares a repository's root license.
- Validate animation trajectories against different frame intervals. Measure actual device callbacks before claiming a refresh rate or smoothness result; browser callbacks are not proof of presented GPU frames.

# Handoff after every change

- After each completed change, update `docs/HANDOFF.md` before delivery. Maintain a table with the request, behavior, exact source entry points, validation evidence, limitations and next action.
- Record source commit, APK version/hash, installation and publication separately. Never mark untested device behavior or pending work as complete.
- Preserve user data and existing playback/authentication when testing appearance. Restore temporary device appearance changes and do not publish personal screenshots.
