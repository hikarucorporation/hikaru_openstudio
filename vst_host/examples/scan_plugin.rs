// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2026 Hikaru Corporation - OpenStudio VST3 Host Example

use std::path::PathBuf;
use std::env;
use vst3_host::{Module, enumerate_audio_effects};

fn main() {
    // Tomamos la ruta del plugin desde los argumentos de la línea de comandos
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Uso: cargo run --example scan_plugin -- <RUTA_AL_BINARIO_VST3>");
        println!("Ejemplo: cargo run --example scan_plugin -- /usr/lib/vst3/Surge.vst3/Contents/x86_64-linux/Surge.so");
        return;
    }

    let plugin_path = PathBuf::from(&args[1]);
    println!("[-] Intentando cargar módulo VST3 desde: {:?}", plugin_path);

    // 1. Intentamos cargar el módulo dinámico
    match Module::load(&plugin_path) {
        Ok(module) => {
            println!("[✓] Módulo cargado con éxito.");
            println!("[-] Escaneando clases de la factoría...");

            // 2. Intentamos enumerar los efectos de la factoría
            let mut found_any = false;
            let scan_result = enumerate_audio_effects(&module, &mut |desc| {
                found_any = true;
                println!("\n========================================");
                println!("  Plugin Encontrado:");
                println!("  Nombre:    {}", desc.name);
                println!("  Proveedor: {}", desc.vendor);
                println!("  Categoría: {}", desc.category);
                println!("  Versión:   {}", desc.version);
                println!("  Class ID:  {:?}", desc.class_id);
                println!("========================================");
            });

            match scan_result {
                Ok(_) => {
                    if !found_any {
                        println!("[!] El módulo se cargó pero no expone ninguna clase compatible con 'Effect' o 'Synth'.");
                    } else {
                        println!("[✓] Escaneo finalizado con éxito.");
                    }
                }
                Err(e) => {
                    eprintln!("[✗] Error al enumerar las clases de la factoría: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[✗] Error fatal al cargar el módulo VST3: {:?}", e);
        }
    }
}
