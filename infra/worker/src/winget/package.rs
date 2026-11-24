use serde_json::{Value, json};

const ID: &str = "hewig.JayJay";
const NAME: &str = "JayJay";
const PUBLISHER: &str = "hewig";
const MONIKER: &str = "jayjay";
const TAGS: &[&str] = &["jj", "jujutsu", "vcs"];
const COMMANDS: &[&str] = &["jayjay", "jayjay-gpui"];
const ARCHITECTURES: &[&str] = &["x64", "arm64"];
pub(crate) const CHECKSUM_PREFIX: &str = "windows/JayJay-";
pub(crate) const CHECKSUM_SUFFIX: &str = ".zip.sha256";
const PACKAGES_URL: &str = "https://pkgs.hewig.dev/";

/// Mirrors `dist/windows/winget/*.yaml`, with one release per stable portable ZIP on pkgs.hewig.dev.
pub(crate) struct Package {
    releases: Vec<Release>,
}

struct Release {
    version: String,
    installers: Vec<Installer>,
}

struct Installer {
    architecture: String,
    sha256: String,
}

impl Package {
    pub(crate) fn from_checksums(files: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut releases: Vec<Release> = Vec::new();
        for (key, contents) in files {
            let Some((version, installer)) = Installer::from_checksum(&key, &contents) else {
                continue;
            };
            match releases
                .iter_mut()
                .find(|release| release.version == version)
            {
                Some(release) => release.installers.push(installer),
                None => releases.push(Release {
                    version,
                    installers: vec![installer],
                }),
            }
        }
        for release in &mut releases {
            release.installers.sort_by_key(|installer| {
                ARCHITECTURES
                    .iter()
                    .position(|architecture| *architecture == installer.architecture)
            });
        }
        releases.sort_by_key(|release| std::cmp::Reverse(numeric_version(&release.version)));
        Self { releases }
    }

    pub(crate) fn field_values(field: &str) -> &'static [&'static str] {
        match field {
            "PackageIdentifier" => &[ID],
            "PackageName" => &[NAME],
            "Moniker" => &[MONIKER],
            "Tag" => TAGS,
            "Command" => COMMANDS,
            _ => &[],
        }
    }

    pub(crate) fn search_result(&self) -> Option<Value> {
        (!self.releases.is_empty()).then(|| {
            json!({
                "PackageIdentifier": ID,
                "PackageName": NAME,
                "Publisher": PUBLISHER,
                "Versions": self.releases.iter().map(|release| json!({ "PackageVersion": release.version })).collect::<Vec<_>>(),
            })
        })
    }

    pub(crate) fn manifest(
        &self,
        id: &str,
        version: Option<&str>,
        channel: Option<&str>,
    ) -> Option<Value> {
        let versions: Vec<_> = self
            .releases
            .iter()
            .filter(|release| version.is_none_or(|version| version == release.version))
            .map(Release::manifest)
            .collect();
        (id.eq_ignore_ascii_case(ID) && channel.is_none_or(str::is_empty) && !versions.is_empty())
            .then(|| json!({ "PackageIdentifier": ID, "Versions": versions }))
    }
}

impl Release {
    fn manifest(&self) -> Value {
        json!({
            "PackageVersion": self.version,
            "DefaultLocale": {
                "PackageLocale": "en-US",
                "Publisher": PUBLISHER,
                "PublisherUrl": "https://github.com/hewigovens",
                "PackageName": NAME,
                "PackageUrl": "https://jayjay.hewig.dev/",
                "License": "BUSL-1.1",
                "LicenseUrl": "https://github.com/hewigovens/jayjay/blob/main/LICENSE",
                "ShortDescription": "Native desktop client for Jujutsu version control",
                "Moniker": MONIKER,
                "Tags": TAGS,
            },
            "Locales": [],
            "Installers": self.installers.iter().map(|installer| installer.manifest(&self.version)).collect::<Vec<_>>(),
        })
    }
}

impl Installer {
    fn from_checksum(key: &str, contents: &str) -> Option<(String, Self)> {
        let name = key
            .strip_prefix(CHECKSUM_PREFIX)?
            .strip_suffix(CHECKSUM_SUFFIX)?;
        let (version, architecture) = name.rsplit_once("-windows-")?;
        let sha256 = contents.split_whitespace().next()?;
        (numeric_version(version).is_some()
            && ARCHITECTURES.contains(&architecture)
            && sha256.len() == 64
            && sha256.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| {
            (
                version.to_owned(),
                Self {
                    architecture: architecture.to_owned(),
                    sha256: sha256.to_ascii_uppercase(),
                },
            )
        })
    }

    fn manifest(&self, version: &str) -> Value {
        json!({
            "InstallerIdentifier": self.architecture,
            "Architecture": self.architecture,
            "InstallerType": "zip",
            "NestedInstallerType": "portable",
            "NestedInstallerFiles": COMMANDS.iter().map(|command| json!({
                "RelativeFilePath": format!("{command}.exe"),
                "PortableCommandAlias": command,
            })).collect::<Vec<_>>(),
            "InstallerUrl": format!("{PACKAGES_URL}{CHECKSUM_PREFIX}{version}-windows-{}.zip", self.architecture),
            "InstallerSha256": self.sha256,
            "Commands": COMMANDS,
            "MinimumOSVersion": "10.0.18362.0",
        })
    }
}

fn numeric_version(version: &str) -> Option<Vec<u64>> {
    version.split('.').map(|part| part.parse().ok()).collect()
}
