# CraftMediaPreview contributor agreements

- Use `feature/`, `fix/`, `chore/` or `release/` branches; never `codex/`.
- Read `docs/architecture.md`, `docs/dependencies.md` and the active OpenSpec change before behavior changes.
- Keep project ownership, SQLite, storage paths, import/export, application catalogs and assistant commands in the host. Source file/URL strings are inert; decode only explicitly supplied immutable bytes or a bounded host capability.
- Never mutate original bytes. Validate length/hash and exact request identity; stale results cannot replace a new source or cross project lifecycles.
- Keep decoding off the GUI/audio callback. Bound source, pixel/frame/sample queues and work; cancellation ownership and no-device/error states must be explicit.
- Use plain Rust APIs. No WebView, bundled whole application, FFmpeg subprocess, ambient URL or implicit filesystem lookup.
- Audio callbacks perform no database access, decoder work or blocking allocation/locks. Playback position must follow actual consumed samples, with deliberate seek/end/segment behavior.
- Preserve supported source formats. An unimplemented codec is an explicit remaining requirement rather than silently advertised support.
- No production unwrap/expect/panic. Use typed actionable errors; no fake render or playback success.
- All public fixtures are synthetic. Never commit or upload private manuscripts, original reference archives, media or application state.
- English repository documentation and explanatory comments. Retain upstream licenses/assets notices.
- No local Cargo fetch/build/test in this task. Resolve and qualify on GitHub; direct local rustfmt/static inspection is allowed.
- Commit focused changes and exact qualification evidence. Library/model/egui-event checks do not qualify native OS playback, host UX, PDF fidelity or complete migration parity.
