# SkinStrike

Finish design prototype for layered color adjustments over a source PNG and PNG export. Valve-compatible asset round trips are not verified.

## Build and test

On Windows with a Rust 2024 toolchain:

```powershell
cargo test --locked
cargo build --locked --features native-ui --bins
```

The code is a local prototype. Native windows, installers, and full production workflows have not passed clean-machine acceptance. Project source in this private repository has no public license grant. Third-party crates retain their own licenses.
