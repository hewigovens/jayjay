//! TOML config at `$HOME/.config/jayjay/config.toml`, owned by the GPUI shell; the SwiftUI shell uses its own `AppSettings`.

mod app_config;
pub mod appearance;
pub mod diff;
pub mod features;
pub mod layout;
pub mod onboarding;
pub mod store;
pub mod telemetry;
pub mod tools;
pub mod window;

pub use app_config::AppConfig;
pub use appearance::AppearanceMode;
pub use diff::DiffConfig;
pub use features::FeaturesConfig;
pub use layout::LayoutConfig;
pub use onboarding::OnboardingConfig;
pub use store::{AppConfigStore, current, update};
pub use telemetry::TelemetryConfig;
pub use tools::ToolsConfig;
pub use window::WindowState;
