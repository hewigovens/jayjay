use serde_json::{Value, json};

use super::*;

const X64: &str = "0aac630d305e394e412fdfd44a7679a425a8054cf04c48c0e420c83348557311";
const ARM64: &str = "1bbd630d305e394e412fdfd44a7679a425a8054cf04c48c0e420c83348557311";

fn package() -> Package {
    Package::from_checksums(
        [
            ("windows/JayJay-0.3.9-windows-x64.zip.sha256", X64),
            ("windows/JayJay-0.3.23-windows-arm64.zip.sha256", ARM64),
            ("windows/JayJay-0.3.23-windows-x64.zip.sha256", X64),
            ("windows/JayJay-0.3.23-windows-x64.zip", X64),
            ("windows/JayJay-0.3.24-beta.1-windows-x64.zip.sha256", X64),
            ("windows/JayJay-0.3.24-windows-x86.zip.sha256", X64),
            ("windows/JayJay-0.3.25-windows-x64.zip.sha256", "not a hash"),
        ]
        .map(|(key, hash)| {
            (
                key.to_owned(),
                format!("{hash}  {}\n", key.rsplit('/').next().unwrap()),
            )
        }),
    )
}

fn search(request: Value) -> bool {
    serde_json::from_value::<SearchRequest>(request)
        .unwrap()
        .matches()
}

#[test]
fn releases_come_from_valid_stable_checksums_newest_first() {
    let package = package();
    assert_eq!(
        package.search_result().unwrap()["Versions"],
        json!([{ "PackageVersion": "0.3.23" }, { "PackageVersion": "0.3.9" }])
    );
    let manifest = package
        .manifest("hewig.jayjay", Some("0.3.23"), None)
        .unwrap();
    let installers: Vec<_> = manifest["Versions"][0]["Installers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|installer| {
            (
                installer["Architecture"].clone(),
                installer["InstallerUrl"].clone(),
                installer["InstallerSha256"].clone(),
            )
        })
        .collect();
    assert_eq!(
        installers,
        [
            (
                json!("x64"),
                json!("https://pkgs.hewig.dev/windows/JayJay-0.3.23-windows-x64.zip"),
                json!(X64.to_ascii_uppercase())
            ),
            (
                json!("arm64"),
                json!("https://pkgs.hewig.dev/windows/JayJay-0.3.23-windows-arm64.zip"),
                json!(ARM64.to_ascii_uppercase())
            ),
        ]
    );
    assert_eq!(
        package.manifest("hewig.JayJay", None, Some("")).unwrap()["Versions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        package
            .manifest("hewig.JayJay", Some("0.3.24"), None)
            .is_none()
    );
    assert!(
        package
            .manifest("hewig.JayJay", None, Some("beta"))
            .is_none()
    );
    assert!(package.manifest("other.Package", None, None).is_none());
    assert!(Package::from_checksums([]).search_result().is_none());
}

#[test]
fn search_widens_with_query_or_inclusions_and_narrows_with_filters() {
    let keyword =
        |key_word: &str, match_type: &str| json!({ "KeyWord": key_word, "MatchType": match_type });
    let field = |field: &str, key_word: &str, match_type: &str| json!({ "PackageMatchField": field, "RequestMatch": keyword(key_word, match_type) });
    for (request, expected) in [
        (json!({}), true),
        (json!({ "Query": keyword("JUJUTSU", "Substring") }), true),
        (json!({ "Query": keyword("jayjay-gpui", "Exact") }), true),
        (json!({ "Query": keyword("JayJay", "Exact") }), true),
        (json!({ "Query": keyword("jayjayx", "Fuzzy") }), false),
        (json!({ "Query": keyword("Jay*", "Wildcard") }), true),
        (
            json!({ "Query": keyword("hewig.?ayjay", "Wildcard") }),
            true,
        ),
        (json!({ "Query": keyword("jay*x", "Wildcard") }), false),
        (
            json!({ "Query": keyword(&format!("{}x", "*".repeat(64)), "Wildcard") }),
            false,
        ),
        (
            json!({ "Query": keyword("git", "Substring"), "Inclusions": [field("PackageIdentifier", "HEWIG.", "StartsWith")] }),
            true,
        ),
        (
            json!({ "Inclusions": [field("ProductCode", "hewig.JayJay", "Exact")] }),
            false,
        ),
        (
            json!({ "Filters": [field("PackageIdentifier", "hewig.jayjay", "CaseInsensitive")] }),
            true,
        ),
        (
            json!({ "Query": keyword("jayjay", "Substring"), "Filters": [field("PackageIdentifier", "hewig.jayjay", "Exact")] }),
            false,
        ),
    ] {
        assert_eq!(search(request.clone()), expected, "{request}");
    }
    assert!(
        serde_json::from_value::<SearchRequest>(json!({ "Query": keyword("jayjay", "Regex") }))
            .is_err()
    );
}
