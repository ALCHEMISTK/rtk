//! Google Antigravity support: transparent command rewriting via PreToolUse lifecycle hook plugin.

use super::*;

pub const ANTIGRAVITY_PLUGIN_JSON: &str = r#"{
  "name": "rtk",
  "version": "1.0.0",
  "description": "Antigravity plugin for transparent command rewriting and token optimization using rtk"
}
"#;

pub const ANTIGRAVITY_HOOKS_JSON: &str = r#"{
  "rtk-rewrite": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "run_command",
        "hooks": [
          {
            "type": "command",
            "command": "rtk hook antigravity",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
"#;

pub fn run_antigravity_mode(global: bool, ctx: InitContext) -> Result<()> {
    if global {
        let home = dirs::home_dir().context("Could not determine user home directory")?;
        let base_dir = home.join(".gemini/config");
        run_antigravity_mode_at(&base_dir, true, ctx)
    } else {
        run_antigravity_mode_at(&std::env::current_dir()?, false, ctx)
    }
}

pub fn run_antigravity_mode_at(base_dir: &Path, global: bool, ctx: InitContext) -> Result<()> {
    let InitContext {
        verbose, dry_run, ..
    } = ctx;
    let plugin_dir = if global {
        base_dir.join("plugins/rtk")
    } else {
        base_dir.join(".agents/plugins/rtk")
    };

    let plugin_json_path = plugin_dir.join("plugin.json");
    let hooks_json_path = plugin_dir.join("hooks.json");

    if dry_run {
        println!(
            "[dry-run] would create plugin directory: {}",
            plugin_dir.display()
        );
        println!("[dry-run] would write {}", plugin_json_path.display());
        println!("[dry-run] would write {}", hooks_json_path.display());
        if verbose > 0 {
            println!(
                "[dry-run] plugin.json content:\n{}",
                ANTIGRAVITY_PLUGIN_JSON
            );
            println!("[dry-run] hooks.json content:\n{}", ANTIGRAVITY_HOOKS_JSON);
        }
        print_dry_run_footer();
    } else {
        fs::create_dir_all(&plugin_dir).context("Failed to create Antigravity plugin directory")?;
        fs::write(&plugin_json_path, ANTIGRAVITY_PLUGIN_JSON)
            .context("Failed to write Antigravity plugin.json")?;
        fs::write(&hooks_json_path, ANTIGRAVITY_HOOKS_JSON)
            .context("Failed to write Antigravity hooks.json")?;

        if verbose > 0 {
            eprintln!("Wrote {}", plugin_json_path.display());
            eprintln!("Wrote {}", hooks_json_path.display());
        }

        println!("\nRTK plugin configured for Google Antigravity.\n");
        println!("  Plugin: {} (installed)", plugin_dir.display());
        println!("  Hooks:  PreToolUse -> rtk hook antigravity");
        println!("  Antigravity will now transparently rewrite commands to rtk.");
        println!(
            "\n  Note: Antigravity checks permissions after hooks rewrite a command.\n  \
             If you use command allowlists, ensure `rtk` commands are permitted,\n  \
             e.g. `command(rtk git status)` or `command(rtk *)`.\n"
        );
    }

    Ok(())
}

pub fn uninstall_antigravity_mode(global: bool, ctx: InitContext) -> Result<Vec<String>> {
    if global {
        let home = dirs::home_dir().context("Could not determine user home directory")?;
        let base_dir = home.join(".gemini/config");
        uninstall_antigravity_mode_at(&base_dir, true, ctx)
    } else {
        uninstall_antigravity_mode_at(&std::env::current_dir()?, false, ctx)
    }
}

pub fn uninstall_antigravity_mode_at(
    base_dir: &Path,
    global: bool,
    ctx: InitContext,
) -> Result<Vec<String>> {
    let InitContext {
        verbose, dry_run, ..
    } = ctx;
    let mut removed = Vec::new();
    let plugin_dir = if global {
        base_dir.join("plugins/rtk")
    } else {
        base_dir.join(".agents/plugins/rtk")
    };

    if plugin_dir.exists() {
        if dry_run {
            println!(
                "[dry-run] would remove Antigravity plugin directory: {}",
                plugin_dir.display()
            );
        } else {
            // nosemgrep: filesystem-deletion -- uninstall intentionally removes only RTK's Antigravity plugin directory.
            fs::remove_dir_all(&plugin_dir).with_context(|| {
                format!(
                    "Failed to remove Antigravity plugin directory: {}",
                    plugin_dir.display()
                )
            })?;
            if verbose > 0 {
                eprintln!(
                    "Removed Antigravity plugin directory: {}",
                    plugin_dir.display()
                );
            }
        }
        removed.push(format!("Antigravity plugin: {}", plugin_dir.display()));
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_antigravity_mode_creates_plugin_files_local() {
        let temp = TempDir::new().unwrap();
        run_antigravity_mode_at(temp.path(), false, InitContext::default()).unwrap();

        let plugin_dir = temp.path().join(".agents/plugins/rtk");
        let manifest_path = plugin_dir.join("plugin.json");
        let hooks_path = plugin_dir.join("hooks.json");

        assert!(manifest_path.exists(), "plugin.json should exist");
        assert!(hooks_path.exists(), "hooks.json should exist");

        let manifest = fs::read_to_string(&manifest_path).unwrap();
        assert!(manifest.contains(r#""name": "rtk""#));

        let hooks = fs::read_to_string(&hooks_path).unwrap();
        assert!(hooks.contains(r#""rtk-rewrite""#));
        assert!(hooks.contains(r#""command": "rtk hook antigravity""#));
    }

    #[test]
    fn test_antigravity_mode_creates_plugin_files_global() {
        let temp = TempDir::new().unwrap();
        run_antigravity_mode_at(temp.path(), true, InitContext::default()).unwrap();

        let plugin_dir = temp.path().join("plugins/rtk");
        let manifest_path = plugin_dir.join("plugin.json");
        let hooks_path = plugin_dir.join("hooks.json");

        assert!(manifest_path.exists(), "global plugin.json should exist");
        assert!(hooks_path.exists(), "global hooks.json should exist");
    }

    #[test]
    fn test_antigravity_mode_dry_run_writes_nothing() {
        let temp = TempDir::new().unwrap();
        run_antigravity_mode_at(
            temp.path(),
            false,
            InitContext {
                dry_run: true,
                ..InitContext::default()
            },
        )
        .unwrap();

        let plugin_dir = temp.path().join(".agents/plugins/rtk");
        assert!(
            !plugin_dir.exists(),
            "Plugin dir must not exist after dry run"
        );
    }

    #[test]
    fn test_antigravity_mode_reinstall_idempotent() {
        let temp = TempDir::new().unwrap();
        run_antigravity_mode_at(temp.path(), false, InitContext::default()).unwrap();
        run_antigravity_mode_at(temp.path(), false, InitContext::default()).unwrap();

        let plugin_dir = temp.path().join(".agents/plugins/rtk");
        assert!(plugin_dir.join("plugin.json").exists());
        assert!(plugin_dir.join("hooks.json").exists());
    }

    #[test]
    fn test_antigravity_mode_uninstall_removes_plugin() {
        let temp = TempDir::new().unwrap();
        run_antigravity_mode_at(temp.path(), false, InitContext::default()).unwrap();

        let removed =
            uninstall_antigravity_mode_at(temp.path(), false, InitContext::default()).unwrap();
        assert_eq!(removed.len(), 1);

        let plugin_dir = temp.path().join(".agents/plugins/rtk");
        assert!(!plugin_dir.exists(), "Plugin dir should be removed");
    }
}
