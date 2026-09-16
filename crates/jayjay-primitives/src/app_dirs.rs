use std::path::PathBuf;

use etcetera::{AppStrategy, AppStrategyArgs};

pub struct AppDirs {
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
}

impl AppDirs {
    pub fn new() -> Option<Self> {
        let strategy = etcetera::app_strategy::choose_native_strategy(AppStrategyArgs {
            top_level_domain: "dev".to_owned(),
            author: "hewig".to_owned(),
            app_name: "jayjay".to_owned(),
        })
        .ok()?;
        // macOS config lives in Application Support (etcetera's data dir), where existing stores already are; Preferences is for plists.
        let config = if cfg!(target_os = "macos") {
            strategy.data_dir()
        } else {
            strategy.config_dir()
        };
        Some(Self {
            config,
            data: strategy.data_dir(),
            cache: strategy.cache_dir(),
        })
    }
}

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod tests {
    use super::*;

    #[test]
    fn directories_match_the_layout_existing_stores_use() {
        let home = etcetera::home_dir().unwrap();
        let dirs = AppDirs::new().unwrap();
        if cfg!(target_os = "macos") {
            let support = home.join("Library/Application Support/dev.hewig.jayjay");
            assert_eq!(dirs.config, support);
            assert_eq!(dirs.data, support);
            assert_eq!(dirs.cache, home.join("Library/Caches/dev.hewig.jayjay"));
        } else {
            let xdg = |var: &str, default: &str| {
                std::env::var_os(var)
                    .map(PathBuf::from)
                    .filter(|path| path.is_absolute())
                    .unwrap_or_else(|| home.join(default))
                    .join("jayjay")
            };
            assert_eq!(dirs.config, xdg("XDG_CONFIG_HOME", ".config"));
            assert_eq!(dirs.data, xdg("XDG_DATA_HOME", ".local/share"));
            assert_eq!(dirs.cache, xdg("XDG_CACHE_HOME", ".cache"));
        }
    }
}
