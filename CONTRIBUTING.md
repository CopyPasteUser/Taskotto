# Contributing to Taskotto

Taskotto is a small, Rust-only Linux process manager. Patches should preserve its unprivileged operation, standalone launch and optional RiceBowl integration.

1. Open an issue for substantial behavior changes; small fixes can go directly to a pull request.
2. Prefer narrow modules and standard Rust APIs over background daemons or extra dependencies.
3. Run the same checks as CI:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --locked --all-targets -- -D warnings
   cargo test --locked --all-targets
   cargo build --locked --release
   ```

4. Add regression tests for bugs. Use disposable child processes for signal tests; never target an unrelated system process.
5. Explain user-visible changes in the README and `RELEASE_NOTES.md`. Keep machine-specific GUI checks in `QA.md` until verified on that hardware.

By contributing, you agree your code is submitted under the project's MIT license.
