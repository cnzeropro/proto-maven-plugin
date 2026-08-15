use extism_pdk::*;
use proto_pdk::*;
use rustc_hash::FxHashMap;

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
                    // Version looks like X.Y.Z (at least two dots, all digits and dots)
                    let dot_count = candidate.chars().filter(|c| *c == '.').count();
                    if dot_count >= 2
                        && candidate.chars().all(|c| c.is_ascii_digit() || c == '.')
                    {
                        let version = candidate.to_string();
                        if !all_versions.contains(&version) {
                            all_versions.push(version);
                        }
                    }
                }
            }
        }
    }

    // Sort using natural version order
    all_versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
        a_parts.cmp(&b_parts)
    });

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

    // Dynamically extract major version to support 2.x, 3.x, 4.x, 5.x, ...
    let major = version.split('.').next().unwrap_or("3");

    let download_file = match env.os {
        HostOS::Windows => format!("apache-maven-{version}-bin.zip"),
        _ => format!("apache-maven-{version}-bin.tar.gz"),
    };

    let archive_prefix = format!("apache-maven-{version}");
    let base = format!("https://archive.apache.org/dist/maven/maven-{major}/{version}/binaries");

    Ok(Json(DownloadPrebuiltOutput {
        archive_prefix: Some(archive_prefix),
        download_url: format!("{base}/{download_file}"),
        download_name: Some(download_file),
        // Checksums differ by major version (SHA1 for 2.x, SHA512 for 3.x),
        // so we skip verification to support all versions uniformly.
        checksum_url: None,
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

    // Maven's binary is named `mvn`, not `maven` — register it explicitly.
    // Windows script extension differs by major: 2.x ships mvn.bat, 3.x+ ships mvn.cmd.
    let exe_path = match env.os {
        HostOS::Windows if major == "2" => "bin/mvn.bat",
        HostOS::Windows => "bin/mvn.cmd",
        _ => "bin/mvn",
    };

    Ok(Json(LocateExecutablesOutput {
        // Register both names: `mvn` is the canonical binary (primary),
        // `maven` is an alias so `proto bin maven` / `maven --version` also work.
        exes: FxHashMap::from_iter([
            ("mvn".into(), ExecutableConfig::new_primary(exe_path.clone())),
            ("maven".into(), ExecutableConfig::new(exe_path)),
        ]),
        exes_dirs: vec!["bin".into()],
        ..LocateExecutablesOutput::default()
    }))
}
