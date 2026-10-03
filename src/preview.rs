use crate::builder::EscPosBuilder;
use crate::error::Result;
use crate::printable::Printable;
use crate::profile::{CodePage, PrinterProfile};

pub fn preview<T: Printable + ?Sized>(item: &T, profile: &PrinterProfile) -> Result<String> {
    let mut doc = EscPosBuilder::new(profile.clone());
    item.render(&mut doc)?;
    doc.finish();
    Ok(bytes_to_text(doc.as_bytes(), profile.code_page))
}

pub fn bytes_to_text(bytes: &[u8], cp: CodePage) -> String {
    let get = |i: usize| bytes.get(i).copied().unwrap_or(0);
    let mut out = String::new();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            0x1B => {
                i += match get(i + 1) {
                    b'@' => 2,
                    b'p' => 5,
                    _ => 3,
                };
            }
            0x1D => match get(i + 1) {
                b'(' => {
                    let len = get(i + 3) as usize + 256 * get(i + 4) as usize;
                    if get(i + 6) == 0x51 {
                        out.push_str("[QR]");
                    }
                    i += 5 + len;
                }
                b'k' => {
                    out.push_str("[CÓDIGO DE BARRAS]");
                    i += 4 + get(i + 3) as usize;
                }
                b'V' => {
                    out.push_str("✂ - - - - - - - - - - - - - -\n");
                    i += 3;
                }
                _ => i += 3,
            },
            b'\n' => {
                out.push('\n');
                i += 1;
            }
            b if b.is_ascii() => {
                out.push(b as char);
                i += 1;
            }
            b => {
                let c = "áéíóúñÑüÜÁÉÍÓÚ¿¡€"
                    .chars()
                    .find(|c| cp.encode_char(*c) == b)
                    .unwrap_or('?');
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

