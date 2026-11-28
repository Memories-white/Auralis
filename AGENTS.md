# Project implementation language

- User update: animation and presentation may use other languages/platform layers when they improve measured smoothness and efficiency. Do not claim lower energy or full frame rate without device evidence.
- New features and changes primarily use Rust. UI layout, interaction state and animation logic belong in `ui/`; protocol and shared data policies belong in `core/`.
- Adapt reference implementations to the chosen rendering layer. Rust remains preferred for shared state, but Kotlin/native or JavaScript presentation is allowed under the user update above when justified by measurements.
- Keep Android media, lifecycle and state ownership clear. Native presentation changes are allowed under the user update above; avoid duplicating playback or account state.
- CSS may define presentation and compositor properties; Rust owns animation state, progress and timing where custom motion is needed. JavaScript remains generated WASM bindings, bootstrap and test tooling.
- Preserve upstream per-file license notices and document adaptation scope. Do not assume every file shares a repository's root license.
- Validate animation trajectories against different frame intervals. Measure actual device callbacks before claiming a refresh rate or smoothness result; browser callbacks are not proof of presented GPU frames.

# Handoff after every change

- After each completed change, update `docs/HANDOFF.md` before delivery. Maintain a table with the request, behavior, exact source entry points, validation evidence, limitations and next action.
- Record source commit, APK version/hash, installation and publication separately. Never mark untested device behavior or pending work as complete.
- Preserve user data and existing playback/authentication when testing appearance. Restore temporary device appearance changes and do not publish personal screenshots.

# APK retention

- After the new APK has passed build and package validation, delete older versioned APKs and their matching checksum sidecars from this project's APKs/ delivery folder. Keep only the newest deliverable there. Do not delete unrelated debug evidence or files outside that folder.
