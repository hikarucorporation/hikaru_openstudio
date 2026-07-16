import re

# Esto busca el bloque típico de Task::perform que causa el error y lo parchea
# Ajustá 'Message::ConfigUpdated' si tu enum tiene otro nombre
pattern = r"(Task::perform\(\s*async move \{.*?\}\s*,\s*move \|(.*?)\| \{)(.*?)(self\..*?)( \})"
replacement = r"\1 let config = self.config.clone(); \3 Message::ConfigUpdated(config, \2) \4"

with open("generic_daw_gui/src/config_view.rs", "r") as f:
    content = f.read()

new_content = re.sub(pattern, replacement, content, flags=re.DOTALL)

with open("generic_daw_gui/src/config_view.rs", "w") as f:
    f.write(new_content)
