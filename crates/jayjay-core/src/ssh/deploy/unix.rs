use crate::JayResult;
use crate::ssh::connection;
use crate::ssh::helper::Helper;

/// Joined into one line for `sh -c '<script>' sh <hash>`: end every statement, use no single quote.
const PRELUDE: &str = r#"
digest=$1;
base="$HOME/.local/share/jayjay/helpers/$digest";
verify() {
  if command -v sha256sum >/dev/null; then sum=$(sha256sum "$1"); else sum=$(shasum -a 256 "$1"); fi;
  [ "${sum%% *}" = "$digest" ];
};
"#;

const CHECK: &str = r#"[ -x "$base/jayjay-ssh" ] && verify "$base/jayjay-ssh" && { touch "$base" 2>/dev/null || :; printf installed; }"#;

/// A copy used within a day may belong to a build connecting right now.
const INSTALL: &str = r#"
set -eu;
umask 077;
mkdir -p "$base";
temporary="$base/.upload.$$";
trap "rm -f \"\$temporary\"" 0 1 2 15;
cat > "$temporary";
verify "$temporary";
chmod 700 "$temporary";
mv "$temporary" "$base/jayjay-ssh";
cd "$base/.." && find . ! -name . -prune -type d ! -name "$digest" -mtime +0 -exec rm -rf {} + || :
"#;

const RUN: &str = r#"exec "$base/jayjay-ssh""#;

impl Helper {
    pub(crate) fn deploy_unix(&self, machine: &str) -> JayResult<String> {
        let sh = |script: &str| {
            let script: Vec<&str> = [PRELUDE, script]
                .iter()
                .flat_map(|part| part.lines())
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect();
            format!("sh -c '{}' sh {}", script.join(" "), self.hash)
        };
        let installed = connection::run(machine, &sh(CHECK), b"").unwrap_or_default();
        if installed != "installed" {
            connection::run(machine, &sh(INSTALL), &self.bytes)?;
        }
        Ok(sh(RUN))
    }
}
