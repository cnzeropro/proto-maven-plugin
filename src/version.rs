//! Maven 版本号的校验、解析与比较工具。
//! 独立于 extism / proto_pdk，便于在宿主平台直接运行单元测试。

use std::cmp::Ordering;

/// 判断抓取到的目录名是否为合法的 Maven 版本号。
/// 规则：以数字开头、仅含数字/字母/点/连字符、至少含一个点。
/// 可覆盖 1.1、2.0.11、3.9.16、3.1.0-alpha-1、4.0.0-rc-6 等格式，
/// alpha/beta/rc 等预发布版本不做过滤，选择权交给用户。
pub fn is_valid_version(candidate: &str) -> bool {
    let mut has_dot = false;

    if !candidate.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }

    for ch in candidate.chars() {
        if !(ch.is_ascii_digit() || ch.is_ascii_alphabetic() || ch == '.' || ch == '-') {
            return false;
        }
        if ch == '.' {
            has_dot = true;
        }
    }

    has_dot
}

/// 版本号中的预发布标识：数字按数值比较，非数字按字典序比较
#[derive(Debug, PartialEq, Eq)]
enum PrereleasePart {
    Num(u64),
    Str(String),
}

/// 将版本号拆分为（数字主段，预发布标识列表）。
/// 例："3.10.0-rc-1" -> ([3, 10, 0], [Str("rc"), Num(1)])，"1.1" -> ([1, 1], [])。
fn split_version(version: &str) -> (Vec<u64>, Vec<PrereleasePart>) {
    let mut nums = Vec::new();
    let mut rest = version;

    // 解析点分数字段，遇到非数字字符即停止
    while let Some(first) = rest.chars().next() {
        if !first.is_ascii_digit() {
            break;
        }
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        nums.push(rest[..end].parse().unwrap_or(0));
        rest = &rest[end..];
        match rest.strip_prefix('.') {
            Some(remaining) => rest = remaining,
            None => break,
        }
    }

    // 余下部分作为预发布段，按 '-' 与 '.' 拆分标识
    let mut prerelease = Vec::new();
    for segment in rest.split(|c| c == '-' || c == '.') {
        if segment.is_empty() {
            continue;
        }
        if segment.chars().all(|c| c.is_ascii_digit()) {
            prerelease.push(PrereleasePart::Num(segment.parse().unwrap_or(0)));
        } else {
            prerelease.push(PrereleasePart::Str(segment.to_string()));
        }
    }

    (nums, prerelease)
}

/// 按 semver 规则比较版本号：先比数字主段；正式版大于任何预发布版本；
/// 预发布标识逐段比较，数字标识小于字母标识，数字按数值、字母按字典序。
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let (a_nums, a_pre) = split_version(a);
    let (b_nums, b_pre) = split_version(b);

    match a_nums.cmp(&b_nums) {
        Ordering::Equal => match (a_pre.is_empty(), b_pre.is_empty()) {
            (true, true) => Ordering::Equal,
            // 正式版 > 预发布版
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => {
                for (x, y) in a_pre.iter().zip(b_pre.iter()) {
                    let ord = match (x, y) {
                        (PrereleasePart::Num(xn), PrereleasePart::Num(yn)) => xn.cmp(yn),
                        (PrereleasePart::Num(_), PrereleasePart::Str(_)) => Ordering::Less,
                        (PrereleasePart::Str(_), PrereleasePart::Num(_)) => Ordering::Greater,
                        (PrereleasePart::Str(xs), PrereleasePart::Str(ys)) => xs.cmp(ys),
                    };
                    if ord != Ordering::Equal {
                        return ord;
                    }
                }
                a_pre.len().cmp(&b_pre.len())
            }
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_version() {
        // 正式版与两段式版本
        assert!(is_valid_version("1.1"));
        assert!(is_valid_version("2.0.11"));
        assert!(is_valid_version("3.9.16"));
        // 预发布版本
        assert!(is_valid_version("3.1.0-alpha-1"));
        assert!(is_valid_version("3.5.0-beta-1"));
        assert!(is_valid_version("3.10.0-rc-1"));
        assert!(is_valid_version("4.0.0-alpha-13"));
        assert!(is_valid_version("4.0.0-rc-6"));
        // 目录索引中的杂项
        assert!(!is_valid_version("binaries"));
        assert!(!is_valid_version("source"));
        assert!(!is_valid_version("HEADER.html"));
        assert!(!is_valid_version("?C=N;O=D"));
        assert!(!is_valid_version("maven-3"));
        assert!(!is_valid_version(""));
    }

    #[test]
    fn test_split_version() {
        assert_eq!(split_version("1.1"), (vec![1, 1], vec![]));
        assert_eq!(split_version("2.0.11"), (vec![2, 0, 11], vec![]));
        assert_eq!(
            split_version("3.10.0-rc-1"),
            (
                vec![3, 10, 0],
                vec![PrereleasePart::Str("rc".into()), PrereleasePart::Num(1)]
            )
        );
        assert_eq!(
            split_version("4.0.0-alpha-13"),
            (
                vec![4, 0, 0],
                vec![
                    PrereleasePart::Str("alpha".into()),
                    PrereleasePart::Num(13)
                ]
            )
        );
    }

    #[test]
    fn test_compare_versions() {
        // 自然数字序（而非字典序）
        assert_eq!(compare_versions("3.9.9", "3.9.10"), Ordering::Less);
        assert_eq!(compare_versions("2.0.11", "2.2.1"), Ordering::Less);
        // 跨大版本
        assert_eq!(compare_versions("1.1", "2.0.11"), Ordering::Less);
        assert_eq!(compare_versions("2.2.1", "3.0.4"), Ordering::Less);
        // 预发布 < 对应正式版
        assert_eq!(compare_versions("4.0.0-rc-6", "4.0.0"), Ordering::Less);
        assert_eq!(compare_versions("3.5.0-beta-1", "3.5.0"), Ordering::Less);
        // 预发布阶段排序：alpha < beta < rc
        assert_eq!(
            compare_versions("3.5.0-alpha-1", "3.5.0-beta-1"),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("3.5.0-beta-1", "3.10.0-rc-1"),
            Ordering::Less
        );
        // 预发布内部数字按数值比较
        assert_eq!(
            compare_versions("4.0.0-alpha-2", "4.0.0-alpha-10"),
            Ordering::Less
        );
        // 两段式版本
        assert_eq!(compare_versions("1.1", "1.1.1"), Ordering::Less);
        assert_eq!(compare_versions("3.9.16", "3.9.16"), Ordering::Equal);
    }
}
