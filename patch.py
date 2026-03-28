import re

with open("crates/katana-platform/src/theme/builder.rs", "r") as f:
    text = f.read()

text = text.replace(".panel_background(Rgb", ".with_panel_bg(Rgb")
text = text.replace(".code_background(Rgb", ".with_code_bg(Rgb")
text = text.replace(".text_secondary(Rgb", ".with_text_sec(Rgb")
text = text.replace(".success(Rgb", ".with_success(Rgb")
text = text.replace(".warning(Rgb", ".with_warning(Rgb")
text = text.replace(".error(Rgb", ".with_error(Rgb")
text = text.replace(".border(Rgb", ".with_border(Rgb")
text = text.replace(".selection(Rgb", ".with_selection(Rgb")

with open("crates/katana-platform/src/theme/builder.rs", "w") as f:
    f.write(text)
