# proto-maven-plugin

A [proto](https://moonrepo.dev/proto) WASM plugin for [Apache Maven](https://maven.apache.org/).

Versions are discovered from the [Apache archive](https://archive.apache.org/dist/maven/) and
include **final releases and prereleases (alpha/beta/rc) across all majors** — no filtering, the
choice is yours.

## Usage

Add the plugin to your proto configuration:

```shell
proto plugin add maven "github://cnzeropro/proto-maven-plugin"
```

Install a version (Maven requires a JDK; install one through the
[official Java plugin](https://moonrepo.dev/docs/proto/tools/java) first):

```shell
proto install maven            # latest stable
proto install maven 2.2.1      # a specific 2.x release
proto install maven 3.9.16     # a specific 3.x release
proto install maven 4.0.0-rc-6 # a prerelease
```

List all available versions:

```shell
proto versions maven
```

Both `mvn` and `maven` commands are registered (Maven 1.x only provides `maven`).

## Supported versions

| Major | Versions available on the archive | Notes |
| --- | --- | --- |
| 1.x | `1.1` | Executable is `maven`, no `apache-`/`-bin` archive naming |
| 2.x | `2.0.11`, `2.2.1` | Windows launcher is `mvn.bat` |
| 3.x | `3.0.4` – `3.9.16`, plus `alpha`/`beta`/`rc` | `mvn.bat` before 3.3.1, `mvn.cmd` from 3.3.1 ([MNG-5776](https://issues.apache.org/jira/browse/MNG-5776)) |
| 4.x | `4.0.0-alpha-2` – `4.0.0-rc-6` | `mvn.cmd` |

Checksum verification (`.sha512`) is enabled whenever the archive provides it; older releases only
ship `.md5`/`.sha1`, which proto does not support, so verification is skipped for those.

## Platforms

- Linux (x64, arm64)
- macOS (x64, arm64)
- Windows (x64)

## Development

This plugin is written in Rust with [proto_pdk](https://docs.rs/proto_pdk/).

```shell
# Run unit tests (version parsing/comparison logic)
cargo test --no-default-features

# Build the WASM plugin
cargo build --release --target wasm32-wasip1
```

On Windows, use the GNU toolchain: `cargo +stable-x86_64-pc-windows-gnu ...`.

## Publishing

Tag a release and push — GitHub Actions builds the plugin with
[moonrepo/build-wasm-plugin](https://github.com/moonrepo/build-wasm-plugin), generates the
`.sha256` checksum, and creates a GitHub release with both assets:

```shell
git tag v0.3.0
git push --tags
```

## License

MIT
