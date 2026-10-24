# LockSmith

Windows utility that lists files opened on this machine's network shares (`NetFileEnum`) and
force-closes selected handles (`NetFileClose`), with search, sorting, multi-select and a light or
dark theme. Pure Rust: one `eframe`/`egui` binary on the glow renderer, no WebView. Windows 10/11
only, and it needs administrator rights. Public repository `Swatto86/LockSmith`, MIT, single
branch `main`.

## Build, test, verify

- The crate is in `src/` (`src/Cargo.toml`, code in `src/src/main.rs`); run cargo from there.
  `build.bat` runs `cargo build --release` in `src/`.
- `cargo run`, `cargo build --release` (gives `src/target/release/locksmith.exe`), `cargo test`,
  `cargo bench` (criterion, `benches/benchmarks.rs`).
- Gate, as CI runs it (`ci.yml`, windows-latest, push and PR on `main`, `develop`):
  `cargo fmt -- --check`, `cargo clippy -- -D warnings`, then `cargo build`; a push to `main` also
  builds release. CI does not run `cargo test`, so run it yourself.
- Release: the version is in `src/Cargo.toml` (and the README badge). A pushed `v*` tag runs
  `release.yml`: `cargo build --release`, then it publishes `LockSmith_<tag version>_x64_portable.exe`
  and an NSIS installer `LockSmith_<tag version>_x64-setup.exe` built from `.github/installer.nsi`
  (installs to Program Files with an HKLM uninstall entry).

## Where things are

- `src/src/main.rs` holds everything: the `netapi32.dll` FFI declarations, `enumerate_files`,
  `release_lock`, `is_admin`, the error enums, `LockSmithApp` (the egui UI), icon loading with a
  generated fallback, and the unit tests (`mod tests`). It is commented heavily as teaching
  material for `book/`.
- `src/tests/integration_tests.rs` and `test_utils.rs` work on local copies of `FileLockInfo`;
  the crate has no library target, so they never exercise `main.rs`. Logic changes need unit
  tests in `main.rs`.
- `src/build.rs` embeds `icons/icon.ico` and the manifest through `winres`: `requireAdministrator`
  for release builds, `asInvoker` for debug so tests run unelevated.
- On non-Windows targets the API functions are stubs that return errors, so the UI compiles there.

## Constraints

- Releasing a lock can lose data or crash the owning program. Every release path (button, Delete,
  double-click) goes through the confirmation dialog; keep it that way.
- Only files opened over SMB shares hosted by this machine appear; local handles are out of scope
  (README "Known Limitations").
- No network access and no telemetry. The only saved state is the theme
  (`AppPersistence { dark_mode }`, default dark) through eframe's `persistence` feature under app
  id `LockSmith`.
- Keep `unsafe` inside the FFI wrappers in `main.rs`, and free every `NetFileEnum` buffer with
  `NetApiBufferFree`.

## Open questions

- The README's download and clone links point at `yourusername/LockSmith`, and `src/Cargo.toml`
  has `authors = ["LockSmith Contributors"]` and an empty `repository`; the repository is
  `Swatto86/LockSmith`.

## Where longer material lives

- `README.md`: usage, shortcuts, the safety warning, troubleshooting. `ICON.md`: icon sources and
  how to regenerate them.
- `book/`: a 15-chapter tutorial written around `main.rs` (`book/README.md`), including a plain
  English specification in `13-plain-english-specification.md`.
- There is no `ARCHITECTURE.md` or `CONTEXT.md` yet.
