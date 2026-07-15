import os

path = "generic_daw_gui/src/config_view.rs"
if not os.path.exists(path):
    print(f"Error: No se encuentra {path}")
    exit(1)

with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# 1. Corregimos el primer error (E0308): Convertir PathBuf a Arc<Path> usando .into()
# Buscamos la línea conflictiva y le agregamos .into()
old_push = "self.config.vst3_paths.push(path);"
new_push = "self.config.vst3_paths.push(path.into());"

if old_push in content:
    content = content.replace(old_push, new_push)
else:
    # Por si tiene espacios o sangrías distintas:
    content = content.replace("self.config.vst3_paths.push(path)", "self.config.vst3_paths.push(path.into())")

# 2. Corregimos el segundo error (E0283): Ayudar al compilador a inferir el tipo de Iced
# En el map(), en vez de usar .into() sobre la fila (row!), podemos tipar explícitamente 
# el elemento de retorno o cambiar el .into() por un casteo a Element.
# Vamos a buscar donde se construye el widget de la fila en el map de vst3_paths.
# Usualmente se ve algo como: .map(|...| row![...].into()) o similar.
# Reemplazamos la llamada genérica .into() de la fila por un casteo explícito de Iced.

# Buscamos la estructura del map en config_view.rs alrededor de la línea 378/380
# Para asegurar compatibilidad con iced, convertiremos la fila directamente usando Element::from(row![...])
# o especificando el tipo de retorno.
# Vamos a leer las líneas alrededor de la 375-385 para hacer un reemplazo seguro.

