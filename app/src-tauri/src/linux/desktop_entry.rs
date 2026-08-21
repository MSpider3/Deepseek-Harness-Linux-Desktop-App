use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct DesktopEntryManager;

impl DesktopEntryManager {
    /// Generates a standard freedesktop .desktop launcher in ~/.local/share/applications/
    pub fn install_desktop_entry(exec_path: &str, icon_path: &str) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let apps_dir = home.join(".local").join("share").join("applications");
        fs::create_dir_all(&apps_dir)?;

        let entry_content = format!(
            "[Desktop Entry]\n\
             Name=DeepSeek Harness Linux\n\
             Comment=Native Linux desktop wrapper for DeepSeek Harness\n\
             Exec=\"{}\"\n\
             Icon={}\n\
             Terminal=false\n\
             Type=Application\n\
             Categories=Development;IDE;Utility;\n\
             StartupWMClass=deepseek-harness-linux\n\
             Keywords=DeepSeek;AI;Harness;Coding;Agent;\n",
            exec_path, icon_path
        );

        let target_file = apps_dir.join("ai.deepseek.harness.linux.desktop");
        fs::write(&target_file, entry_content)?;
        Ok(target_file)
    }

    /// Toggles autostart on system boot via ~/.config/autostart/
    pub fn set_autostart(enable: bool, exec_path: &str) -> Result<()> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let autostart_dir = home.join(".config").join("autostart");
        let autostart_file = autostart_dir.join("ai.deepseek.harness.linux.desktop");

        if enable {
            fs::create_dir_all(&autostart_dir)?;
            let content = format!(
                "[Desktop Entry]\n\
                 Name=DeepSeek Harness Linux\n\
                 Exec=\"{}\" --minimized\n\
                 Terminal=false\n\
                 Type=Application\n\
                 X-GNOME-Autostart-enabled=true\n",
                exec_path
            );
            fs::write(&autostart_file, content)?;
        } else if autostart_file.exists() {
            let _ = fs::remove_file(&autostart_file);
        }

        Ok(())
    }
}
