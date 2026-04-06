//! Platform-specific path catalogs for host integrations.

mod linux;
mod macos;
mod windows;

use crate::integrations::IntegrationSpec;
use crate::paths::{HostPlatform, PathEnvironment};
use std::path::PathBuf;

/// Build the integration catalog for the requested platform.
pub fn specs_for(platform: HostPlatform, env: &PathEnvironment) -> Vec<IntegrationSpec> {
    match platform {
        HostPlatform::Macos => macos::specs(env),
        HostPlatform::Linux | HostPlatform::Other => linux::specs(platform, env),
        HostPlatform::Windows => windows::specs(env),
    }
}

/// Platform paths needed to construct IntegrationSpecs.
#[derive(Debug, Clone)]
pub struct PlatformPaths {
    /// User home directory or fallback to current dir.
    pub home: PathBuf,
    /// Primary config root (XDG_CONFIG_HOME on Unix, APPDATA on Windows).
    pub config_root: PathBuf,
    /// GitHub Copilot CLI config root.
    pub copilot_root: PathBuf,
}

impl PlatformPaths {
    /// Build platform paths from the given environment and platform.
    pub fn new(env: &PathEnvironment, platform: HostPlatform) -> Self {
        let home = env
            .home_dir_for(platform)
            .or_else(|| env.current_dir.clone())
            .unwrap_or_else(|| PathBuf::from("."));

        let config_root = env
            .config_root_for(platform)
            .unwrap_or_else(|| home.join(".config"));

        let copilot_root = env
            .xdg_config_home
            .clone()
            .map(|root| root.join("copilot"))
            .unwrap_or_else(|| home.join(".copilot"));

        Self {
            home,
            config_root,
            copilot_root,
        }
    }
}
