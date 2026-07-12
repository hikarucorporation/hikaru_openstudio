use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../icons.ttf");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("lucide_map.rs");
    let mut f = File::create(&dest_path).unwrap();

    writeln!(f, "pub static LUCIDE_MAP: ::once_cell::sync::Lazy<::std::collections::HashMap<char, u16>> = ::once_cell::sync::Lazy::new(|| {{").unwrap();
    writeln!(f, "    let mut m = ::std::collections::HashMap::new();").unwrap();

    let font_bytes = include_bytes!("../icons.ttf");
    
    if let Ok(font) = fontdue::Font::from_bytes(font_bytes as &[u8], fontdue::FontSettings::default()) {
        for char_code in 0xe000..0xf300 {
            if let Some(c) = std::char::from_u32(char_code) {
                let glyph_id = font.lookup_glyph_index(c);
                if glyph_id != 0 { 
                    writeln!(f, "    m.insert({:?}, {});", c, glyph_id).unwrap();
                }
            }
        }
    }

    writeln!(f, "    m").unwrap();
    writeln!(f, "}});").unwrap();
}
