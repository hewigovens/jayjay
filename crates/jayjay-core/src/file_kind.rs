pub(crate) const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "heic", "bmp", "tiff", "tif", "ico", "icns",
];

const LOCKFILE_NAMES: &[&str] = &[
    "package-lock.json",
    "npm-shrinkwrap.json",
    "pnpm-lock.yaml",
    "go.sum",
    "go.work.sum",
    "package.resolved",
    "cartfile.resolved",
    "packages.lock.json",
    "cabal.project.freeze",
];
const CONFIG_NAMES: &[&str] = &[
    "package.json",
    "tsconfig.json",
    "jsconfig.json",
    "deno.json",
    "go.mod",
    "go.work",
    "requirements.txt",
    "pipfile",
    "gemfile",
    "rakefile",
    "podfile",
    "brewfile",
    "composer.json",
    "pubspec.yaml",
    "mix.exs",
    "pom.xml",
    "cmakelists.txt",
    "meson.build",
    "dockerfile",
    "containerfile",
    "makefile",
    "gnumakefile",
    "justfile",
    "procfile",
    "jenkinsfile",
    "vagrantfile",
    "codeowners",
    "rust-toolchain",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileKind {
    Shell,
    Data,
    Config,
    Lockfile,
    Image,
    Table,
    Notebook,
    Archive,
    Other,
}

impl FileKind {
    pub(crate) fn for_path(path: &str) -> Self {
        let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
        let extension = name.rsplit_once('.').map_or("", |(_, extension)| extension);
        // Names win over extensions: `package-lock.json` is a lockfile, not data.
        if LOCKFILE_NAMES.contains(&name.as_str())
            || matches!(extension, "lock" | "lockb" | "lockfile")
        {
            return Self::Lockfile;
        }
        if CONFIG_NAMES.contains(&name.as_str()) {
            return Self::Config;
        }
        match extension {
            extension if extension == "svg" || IMAGE_EXTENSIONS.contains(&extension) => Self::Image,
            "csv" | "tsv" => Self::Table,
            "ipynb" => Self::Notebook,
            "zip" | "tar" | "gz" | "tgz" | "xz" | "bz2" | "7z" | "dmg" | "jar" => Self::Archive,
            "toml" | "ini" | "cfg" | "conf" | "properties" | "gradle" | "cabal" | "sbt" => {
                Self::Config
            }
            "json" | "yaml" | "yml" | "xml" | "plist" | "sarif" => Self::Data,
            "sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" | "bat" | "cmd" => Self::Shell,
            _ if name.starts_with('.') => Self::Config,
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FileKind;

    #[test]
    fn for_path_sorts_names_before_extensions() {
        for (path, kind) in [
            ("Makefile", FileKind::Config),
            ("scripts/setup.sh", FileKind::Shell),
            ("scripts/Install.PS1", FileKind::Shell),
            ("shell/mac/project.yml", FileKind::Data),
            ("results.sarif", FileKind::Data),
            ("web/package-lock.json", FileKind::Lockfile),
            ("Cargo.lock", FileKind::Lockfile),
            ("bun.lockb", FileKind::Lockfile),
            ("app/gradle.lockfile", FileKind::Lockfile),
            ("go.sum", FileKind::Lockfile),
            ("Package.resolved", FileKind::Lockfile),
            ("package.json", FileKind::Config),
            ("go.mod", FileKind::Config),
            ("Gemfile", FileKind::Config),
            ("CMakeLists.txt", FileKind::Config),
            ("app/build.gradle", FileKind::Config),
            (".npmrc", FileKind::Config),
            (".env.local", FileKind::Config),
            ("Cargo.toml", FileKind::Config),
            (".gitignore", FileKind::Config),
            ("assets/AppIcon.icns", FileKind::Image),
            ("docs/logo.svg", FileKind::Image),
            ("tests/fixtures/data.csv", FileKind::Table),
            ("analysis.ipynb", FileKind::Notebook),
            ("dist/release.tar.gz", FileKind::Archive),
            ("LICENSE", FileKind::Other),
            ("notes.unknown", FileKind::Other),
        ] {
            assert_eq!(FileKind::for_path(path), kind, "{path}");
        }
    }
}
