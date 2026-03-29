from pathlib import Path

p = Path("crates/katana-ui/src/settings_window.rs")
c = p.read_text()
c = c.replace("state.active_settings_tab", "state.config.active_settings_tab")
c = c.replace("state.settings_tree_force_open", "state.config.settings_tree_force_open")
p.write_text(c)
