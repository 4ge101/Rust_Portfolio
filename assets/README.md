Put your portrait here as `profile.jpg` (or `.jpeg`/`.png`) before building.

If this file is present at build time, `build.rs` compiles it into the binary
so `portfolio` works offline right after installation. If it's absent, the
build still succeeds — the app just runs without a portrait until you pass
`--image PATH` or place `profile.jpg`/`.jpeg`/`.png` in the `--data` directory.
