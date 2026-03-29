from pathlib import Path

p = Path("crates/katana-ui/src/app/workspace.rs")
c = p.read_text()
# Ensure CacheFacade is imported
if "use katana_platform::CacheFacade;" not in c:
    c = "use katana_platform::CacheFacade;\n" + c

# Fix multi-line expanded_directories
c = c.replace("self\n            .state\n            .expanded_directories", "self.state.workspace.expanded_directories")
c = c.replace("self.state\n            .expanded_directories", "self.state.workspace.expanded_directories")
c = c.replace("self.state.expanded_directories", "self.state.workspace.expanded_directories")

p.write_text(c)
