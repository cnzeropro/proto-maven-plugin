# Proto Maven Plugin

[English](#english) | [中文](#chinese)

---

<a name="english"></a>

## English

A [WASM plugin](https://moonrepo.dev/docs/proto/wasm-plugin) for [proto](https://moonrepo.dev/docs/proto) that adds **Apache Maven** support.

Unlike the declarative [TOML plugin](https://moonrepo.dev/docs/proto/toml-plugin) approach, which cannot dynamically construct per-major-version download URLs (`{major}` is not a supported template variable), this WASM plugin implements the full plugin API in Rust, enabling:

- **Version discovery** directly from [Apache Archive](https://archive.apache.org/dist/maven/) directory listings — no external Git dependency.
- **Dynamic major-version routing** — extracts `major` from the version string at runtime, so `maven-1/`, `maven-2/`, `maven-3/`, `maven-4/`, and any future `maven-5/`, `maven-6/`, … all work automatically.
- **Cross-platform support** — Linux (x64, arm64), macOS (x64, arm64), and Windows (x64).

### Installation

Requires **proto 0.60.2+**.

Add this plugin to your `.prototools` file:

```toml
[plugins]
maven = "github://cnzeropro/proto-maven-plugin"
```

Then install Maven:

```shell
proto install maven 3.9.9
proto install maven latest
```

### Supported Versions

The plugin discovers all versions that have published binaries on [archive.apache.org](https://archive.apache.org/dist/maven/), covering:

| Major | Status | Example Versions |
|-------|--------|-----------------|
| 1.x | Not supported | Different structure & binary name |
| 2.x | Stable (archived) | 2.0.11, 2.2.1 |
| 3.x | Stable | 3.9.9, 3.9.16 |
| 4.x | Pre-release only | 4.0.0-rc-6 |
| 5.x+ | Future | Automatic |

Only stable releases (three-part numeric versions, `X.Y.Z`) are listed. Alpha/beta/RC releases are filtered out. Maven 1.x is excluded because it uses a completely different directory structure (`maven-1/1.1/binaries/`, two-part versions) and a different binary name (`maven` instead of `mvn`).

> **Note on running old versions:** Installing works across all major versions, but running Maven 2.x (and 3.0.x–3.3.x) requires a compatible JDK (Java 8 or earlier is recommended for Maven 2.x). Set `JAVA_HOME` accordingly. Maven 3.9+ works with modern JDKs.

### How It Works

**Version Discovery** (`load_versions`):

1. Fetches `https://archive.apache.org/dist/maven/` and extracts all `maven-X/` subdirectories.
2. For each major version directory, fetches the listing and extracts subdirectories matching `X.Y.Z` patterns.
3. Returns a deduplicated, numerically-sorted version list.

**Download** (`download_prebuilt`):

```
https://archive.apache.org/dist/maven/maven-{major}/{version}/binaries/apache-maven-{version}-bin.{ext}
```

The `{major}` is computed at runtime via `version.split('.').next()`, so URLs are always correct regardless of major version.

**Binary Detection** (`locate_executables`):

Sets `exes_dir = "bin"` so proto scans the `bin/` subdirectory. On Windows the primary executable is `mvn.cmd`; on Unix it is `mvn`.

### Configuration

This plugin does not expose additional configuration options. All behavior is automatic.

### Hooks

This plugin does not support pre/post-install or pre-run hooks.

### Building from Source

**Prerequisites:**

- Rust stable (1.80+)
- `wasm32-wasip1` target (formerly `wasm32-wasi`)

```shell
# Install the WASM target
rustup target add wasm32-wasip1

# Clone the repository
git clone https://github.com/cnzeropro/proto-maven-plugin.git
cd proto-maven-plugin

# Build
cargo build --release --target wasm32-wasip1

# The compiled WASM will be at:
# target/wasm32-wasip1/release/maven_plugin.wasm
```

> **Note for Windows users with MSVC toolchain:** If you encounter `link.exe` errors during build, install the GNU toolchain and build with it:
> ```shell
> rustup toolchain install stable-x86_64-pc-windows-gnu
> rustup target add wasm32-wasip1 --toolchain stable-x86_64-pc-windows-gnu
> cargo +stable-x86_64-pc-windows-gnu build --release --target wasm32-wasip1
> ```

### Why WASM Instead of TOML?

Proto's TOML plugin format supports only three template variables in `download-url`: `{version}`, `{download_file}`, and `{checksum_file}`. Apache Maven structures its download directories as `maven-{major}/{version}/binaries/`, which requires extracting the major version from the full version string — something inherently impossible with a declarative TOML template.

A WASM plugin, written in Rust and compiled to WebAssembly, can execute arbitrary logic at runtime and is the only way to support Maven's URL structure across all major versions.

### License

MIT

---

<a name="chinese"></a>

## 中文

[Proto](https://moonrepo.dev/docs/proto) 的 [WASM 插件](https://moonrepo.dev/docs/proto/wasm-plugin)，为 proto 添加 **Apache Maven** 支持。

[TOML 插件](https://moonrepo.dev/docs/proto/toml-plugin) 的 `download-url` 只支持 `{version}`、`{download_file}`、`{checksum_file}` 三个模板变量，无法动态提取大版本号。本 WASM 插件用 Rust 实现了完整的插件 API，从而解决这个问题：

- **从 [Apache Archive](https://archive.apache.org/dist/maven/) 直接发现版本** — 抓取目录列表，不依赖 Git。
- **动态大版本路由** — 运行时从版本号中提取 `major`，因此 `maven-1/`、`maven-2/`、`maven-3/`、`maven-4/`，以及未来的 `maven-5/`、`maven-6/`……全部自动支持。
- **跨平台** — Linux（x64、arm64）、macOS（x64、arm64）、Windows（x64）。

### 安装

需要 **proto 0.60.2 及以上版本**。

将本插件添加到 `.prototools`：

```toml
[plugins]
maven = "github://cnzeropro/proto-maven-plugin"
```

安装 Maven：

```shell
proto install maven 3.9.9
proto install maven latest
```

### 支持的版本

插件从 [archive.apache.org](https://archive.apache.org/dist/maven/) 自动发现所有已发布二进制的版本：

| 大版本 | 状态 | 示例版本 |
|--------|------|----------|
| 1.x | 不支持 | 目录结构与二进制名不同 |
| 2.x | 稳定（归档） | 2.0.11、2.2.1 |
| 3.x | 当前稳定 | 3.9.9、3.9.16 |
| 4.x | 仅有预发布 | 4.0.0-rc-6 |
| 5.x+ | 未来版本 | 自动支持 |

仅列出稳定版本（`X.Y.Z` 三段式数字），alpha/beta/rc 等预发布版本会被过滤。Maven 1.x 因目录结构完全不同（`maven-1/1.1/binaries/`、两段式版本号）且二进制名为 `maven` 而非 `mvn`，不支持。

> **旧版本运行提示：** 所有大版本均可安装，但运行 Maven 2.x（以及 3.0.x–3.3.x）需要兼容的 JDK（2.x 建议 Java 8 或更早）。请相应设置 `JAVA_HOME`。Maven 3.9+ 可在现代 JDK 上运行。

### 工作原理

**版本发现** (`load_versions`)：

1. 抓取 `https://archive.apache.org/dist/maven/`，提取所有 `maven-X/` 子目录。
2. 对每个大版本目录，抓取列表并提取匹配 `X.Y.Z` 模式的子目录。
3. 返回去重并按数值排序的版本列表。

**下载** (`download_prebuilt`)：

```
https://archive.apache.org/dist/maven/maven-{major}/{version}/binaries/apache-maven-{version}-bin.{ext}
```

`{major}` 在运行时通过 `version.split('.').next()` 动态计算，无论大版本如何变化都能正确构建 URL。

**可执行文件定位** (`locate_executables`)：

设置 `exes_dir = "bin"`，让 Proto 自动扫描 `bin/` 子目录。Windows 上主程序为 `mvn.cmd`，Unix 上为 `mvn`。

### 配置

本插件不提供额外的配置项，所有行为均为自动。

### Hooks

本插件不支持安装前后的 hook 和运行前 hook。

### 从源码构建

**前置条件：**

- Rust stable (1.80+)
- `wasm32-wasip1` target（原 `wasm32-wasi`）

```shell
# 安装 WASM 编译目标
rustup target add wasm32-wasip1

# 克隆仓库
git clone https://github.com/cnzeropro/proto-maven-plugin.git
cd proto-maven-plugin

# 构建
cargo build --release --target wasm32-wasip1

# 编译产物位于:
# target/wasm32-wasip1/release/maven_plugin.wasm
```

> **Windows MSVC 用户注意：** 如果构建时遇到 `link.exe` 报错，请安装 GNU 工具链后构建：
> ```shell
> rustup toolchain install stable-x86_64-pc-windows-gnu
> rustup target add wasm32-wasip1 --toolchain stable-x86_64-pc-windows-gnu
> cargo +stable-x86_64-pc-windows-gnu build --release --target wasm32-wasip1
> ```

### 为什么用 WASM 而不是 TOML？

Proto 的 TOML 插件格式在 `download-url` 中只支持三个模板变量：`{version}`、`{download_file}` 和 `{checksum_file}`。Apache Maven 的下载目录结构是 `maven-{major}/{version}/binaries/`，需要从完整版本号中提取主版本号——这对纯声明式的 TOML 模板是不可能的。

WASM 插件用 Rust 编写，编译为 WebAssembly，可以在运行时执行任意逻辑，是唯一能支持 Maven 跨大版本 URL 结构的方式。

### 许可证

MIT
