use extism_pdk::*;
use proto_pdk::*;
use rustc_hash::FxHashMap;

use crate::version::{compare_versions, is_valid_version, normalize_version};

static NAME: &str = "Maven";

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::CommandLine,
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        // Built against proto_pdk 0.34, which matches the proto 0.60.2 API
        minimum_proto_version: Version::parse("0.60.2").ok(),
        ..RegisterToolOutput::default()
    }))
}

/// Scrape the Apache archive directory listing to discover all Maven versions.
/// This queries https://archive.apache.org/dist/maven/ directly,
/// walking each maven-X/ subdirectory to extract version numbers.
#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let base_url = "https://archive.apache.org/dist/maven/";

    // Step 1: Fetch the top-level directory to find all maven-X directories
    let index_html = fetch_text(base_url)?;

    // Extract major version dirs like "maven-1", "maven-2", "maven-3", ...
    let mut major_dirs: Vec<String> = Vec::new();
    {
        let mut s = index_html.as_str();
        while let Some(pos) = s.find("href=\"maven-") {
            s = &s[pos + 6..]; // skip 'href="'
            if let Some(end) = s.find('/') {
                let dir = &s[..end]; // e.g. "maven-3"
                if !major_dirs.contains(&dir.to_string()) {
                    major_dirs.push(dir.to_string());
                }
            }
        }
    }

    // Step 2: For each major version directory, scrape version numbers
    let mut all_versions: Vec<String> = Vec::new();

    for major_dir in &major_dirs {
        let major_url = format!("{base_url}{major_dir}/");
        if let Ok(html) = fetch_text(&major_url) {
            let mut s = html.as_str();
            while let Some(pos) = s.find("href=\"") {
                s = &s[pos + 6..]; // skip 'href="'
                if let Some(end) = s.find('/') {
                    let candidate = &s[..end];
                    // 接受正式版与 alpha/beta/rc 等预发布版本，不做过滤；
                    // 两段式版本（如 1.1）补全为三段式 semver，proto 才能解析
                    if is_valid_version(candidate) {
                        let version = normalize_version(candidate);
                        if !all_versions.contains(&version) {
                            all_versions.push(version);
                        }
                    }
                }
            }
        }
    }

    // 按 semver 规则排序：自然数字序 + 预发布版本排在对应正式版之前
    all_versions.sort_by(|a, b| compare_versions(a, b));

    Ok(Json(LoadVersionsOutput::from(all_versions)?))
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let env = get_host_environment()?;

    check_supported_os_and_arch(
        NAME,
        &env,
        permutations! [
            HostOS::Linux => [HostArch::X64, HostArch::Arm64],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
            HostOS::Windows => [HostArch::X64],
        ],
    )?;

    let version = input.context.version.to_string();

    // Dynamically extract major version to support 1.x, 2.x, 3.x, 4.x, ...
    let major = version.split('.').next().unwrap_or("3");

    // 1.x 归档使用两段式版本号（目录 maven-1/1.1、文件 maven-1.1.zip），
    // 而 proto 传入的是补全后的三段式（1.1.0），需还原为 1.1
    let archive_version = if major == "1" {
        let mut parts = version.split('.');
        let first = parts.next().unwrap_or("1");
        let second = parts.next().unwrap_or("0");
        format!("{first}.{second}")
    } else {
        version.clone()
    };

    // 1.x 的发行包无 "apache-" 前缀与 "-bin" 后缀：maven-1.1.zip / maven-1.1.tar.gz
    let (download_file, archive_prefix) = match (env.os, major) {
        (HostOS::Windows, "1") => (
            format!("maven-{archive_version}.zip"),
            format!("maven-{archive_version}"),
        ),
        (_, "1") => (
            format!("maven-{archive_version}.tar.gz"),
            format!("maven-{archive_version}"),
        ),
        (HostOS::Windows, _) => (
            format!("apache-maven-{version}-bin.zip"),
            format!("apache-maven-{version}"),
        ),
        (_, _) => (
            format!("apache-maven-{version}-bin.tar.gz"),
            format!("apache-maven-{version}"),
        ),
    };

    let base = format!("https://archive.apache.org/dist/maven/maven-{major}/{archive_version}/binaries");

    let download_url = format!("{base}/{download_file}");

    // 校验和：proto 0.60 仅支持 sha256/sha512（及 minisign）算法。
    // 归档上 3.x+ 提供 .sha512 文件可用；1.x 仅有 .md5、2.x 仅有 .md5/.sha1，
    // 均不被 proto 支持，只能跳过校验。
    let checksum_url = match major {
        "1" | "2" => None,
        _ => Some(format!("{base}/{download_file}.sha512")),
    };

    Ok(Json(DownloadPrebuiltOutput {
        archive_prefix: Some(archive_prefix),
        download_url,
        download_name: Some(download_file),
        checksum_url,
        ..DownloadPrebuiltOutput::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(input): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;
    let version = input.context.version.to_string();
    let major = version.split('.').next().unwrap_or("3");

    // 可执行脚本随大版本不同：
    // 1.x 是 maven(.bat)，2.x 是 mvn(.bat)，3.x+ 是 mvn(.cmd)
    let exe_path = match (env.os, major) {
        (HostOS::Windows, "1") => "bin/maven.bat",
        (HostOS::Windows, "2") => "bin/mvn.bat",
        (HostOS::Windows, _) => "bin/mvn.cmd",
        (_, "1") => "bin/maven",
        (_, _) => "bin/mvn",
    };

    // 1.x 没有 mvn 脚本，只注册 maven（与 tool id 一致）；
    // 2.x+ 以 mvn 为主程序，maven 作为别名，使 `proto bin maven` 与 `maven --version` 也可用
    let exes = if major == "1" {
        FxHashMap::from_iter([(
            "maven".into(),
            ExecutableConfig::new_primary(exe_path),
        )])
    } else {
        FxHashMap::from_iter([
            ("mvn".into(), ExecutableConfig::new_primary(exe_path)),
            ("maven".into(), ExecutableConfig::new(exe_path)),
        ])
    };

    Ok(Json(LocateExecutablesOutput {
        exes,
        exes_dirs: vec!["bin".into()],
        ..LocateExecutablesOutput::default()
    }))
}
