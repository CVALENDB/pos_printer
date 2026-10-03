use crate::error::{PrinterError, Result};
use crate::profile::{CutMode, PrinterProfile};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

impl Align {
    fn esc_pos(self) -> u8 {
        match self {
            Align::Left => 0,
            Align::Center => 1,
            Align::Right => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QrEcLevel {
    L,
    M,
    Q,
    H,
}

#[derive(Clone, Copy, Debug)]
pub struct QrOptions {
    pub module_size: u8,
    pub ec_level: QrEcLevel,
}

impl Default for QrOptions {
    fn default() -> Self {
        Self {
            module_size: 6,
            ec_level: QrEcLevel::M,
        }
    }
}

fn wrap(s: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut cur_len = 0usize;

    for word in s.split_whitespace() {
        let mut word: Vec<char> = word.chars().collect();
        while word.len() > width {
            if cur_len > 0 {
                out.push(std::mem::take(&mut cur));
                cur_len = 0;
            }
            let rest = word.split_off(width);
            out.push(word.iter().collect());
            word = rest;
        }
        let wl = word.len();
        if cur_len == 0 {
            cur = word.into_iter().collect();
            cur_len = wl;
        } else if cur_len + 1 + wl <= width {
            cur.push(' ');
            cur.extend(word);
            cur_len += 1 + wl;
        } else {
            out.push(std::mem::take(&mut cur));
            cur = word.into_iter().collect();
            cur_len = wl;
        }
    }
    out.push(cur);
    out
}

pub struct EscPosBuilder {
    buf: Vec<u8>,
    profile: PrinterProfile,
    width_mul: usize,
    align: Align,
}

impl EscPosBuilder {
    pub fn new(profile: PrinterProfile) -> Self {
        let mut b = Self {
            buf: Vec::new(),
            profile,
            width_mul: 1,
            align: Align::Left,
        };
        b.raw(&[0x1B, 0x40]); // ESC @
        let cp = b.profile.code_page.esc_pos_id();
        b.raw(&[0x1B, 0x74, cp]); // ESC t n
        b
    }

    pub fn profile(&self) -> &PrinterProfile {
        &self.profile
    }

    pub fn content_cols(&self) -> usize {
        self.profile
            .chars_per_line
            .saturating_sub(2 * self.profile.margin)
            .max(1)
    }

    pub fn width(&self) -> usize {
        (self.content_cols() / self.width_mul).max(1)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn raw(&mut self, bytes: &[u8]) -> &mut Self {
        self.buf.extend_from_slice(bytes);
        self
    }

    pub fn text(&mut self, s: &str) -> &mut Self {
        let cp = self.profile.code_page;
        self.buf.extend(s.chars().map(|c| cp.encode_char(c)));
        self
    }

    pub fn newline(&mut self) -> &mut Self {
        self.buf.push(b'\n');
        self
    }

    pub fn feed(&mut self, n: u8) -> &mut Self {
        self.raw(&[0x1B, 0x64, n])
    }

    pub fn align(&mut self, a: Align) -> &mut Self {
        self.align = a;
        self
    }

    pub fn bold(&mut self, on: bool) -> &mut Self {
        self.raw(&[0x1B, 0x45, on as u8])
    }

    pub fn underline(&mut self, on: bool) -> &mut Self {
        self.raw(&[0x1B, 0x2D, on as u8])
    }

    pub fn size(&mut self, width: u8, height: u8) -> &mut Self {
        let w = width.clamp(1, 8);
        let h = height.clamp(1, 8);
        self.width_mul = w as usize;
        self.raw(&[0x1D, 0x21, ((w - 1) << 4) | (h - 1)])
    }

  
    pub fn double(&mut self, on: bool) -> &mut Self {
        if on {
            self.size(2, 2)
        } else {
            self.size(1, 1)
        }
    }

    pub fn reset_style(&mut self) -> &mut Self {
        self.bold(false)
            .underline(false)
            .size(1, 1)
            .align(Align::Left)
    }


    fn spaces(&mut self, n: usize) {
        self.buf.extend(std::iter::repeat(b' ').take(n));
    }

    fn margin_line(&mut self, s: &str) {
        let n = self.profile.margin / self.width_mul;
        self.spaces(n);
        self.text(s);
        self.buf.push(b'\n');
    }
    
    fn aligned_line(&mut self, s: &str) {
        let w = self.width_mul;
        let content = self.content_cols();
        let text_cols = (s.chars().count() * w).min(content);
        let extra = match self.align {
            Align::Left => 0,
            Align::Center => (content - text_cols) / 2,
            Align::Right => content - text_cols,
        };
        let left_cols = self.profile.margin + extra;
        self.spaces(left_cols / w);
        self.text(s);
        self.buf.push(b'\n');
    }
    
    pub fn line(&mut self, s: &str) -> &mut Self {
        let width = self.width();
        for paragraph in s.split('\n') {
            if paragraph.chars().count() <= width {
                self.aligned_line(paragraph);
            } else {
                for l in wrap(paragraph, width) {
                    self.aligned_line(&l);
                }
            }
        }
        self
    }


    pub fn separator(&mut self, c: char) -> &mut Self {
        let s: String = std::iter::repeat(c).take(self.width()).collect();
        self.margin_line(&s);
        self
    }
    
    pub fn row(&mut self, left: &str, right: &str) -> &mut Self {
        let width = self.width();
        let n_right = right.chars().count();
        let max_left = width.saturating_sub(n_right + 1).max(1);

        let mut part = if left.chars().count() <= max_left {
            vec![left.to_string()]
        } else {
            wrap(left, max_left)
        };
        let end = part.pop().unwrap_or_default();
        for p in part {
            self.margin_line(&p);
        }

        let spaces = width
            .saturating_sub(end.chars().count() + n_right)
            .max(1);
        let linea = format!("{end}{}{right}", " ".repeat(spaces));
        self.margin_line(&linea);
        self
    }


    pub fn qr(&mut self, data: &str, opts: QrOptions) -> Result<&mut Self> {
        let bytes = data.as_bytes();
        if bytes.is_empty() || bytes.len() > 7089 {
            return Err(PrinterError::InvalidData(format!(
                "el QR admite entre 1 y 7089 bytes, se recibieron {}",
                bytes.len()
            )));
        }
        let ec = match opts.ec_level {
            QrEcLevel::L => 48,
            QrEcLevel::M => 49,
            QrEcLevel::Q => 50,
            QrEcLevel::H => 51,
        };
        let size = opts.module_size.clamp(1, 16);

        let a = self.align.esc_pos();
        
        self.raw(&[0x1B, 0x61, a]);
        
        self.raw(&[0x1D, 0x28, 0x6B, 0x04, 0x00, 0x31, 0x41, 0x32, 0x00]);
       
        self.raw(&[0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x43, size]);
        
        self.raw(&[0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x45, ec]);
   
        let len = bytes.len() + 3;
        self.raw(&[
            0x1D,
            0x28,
            0x6B,
            (len & 0xFF) as u8,
            ((len >> 8) & 0xFF) as u8,
            0x31,
            0x50,
            0x30,
        ]);
        self.raw(bytes);
   
        self.raw(&[0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x51, 0x30]);
        self.newline();
        self.raw(&[0x1B, 0x61, 0x00]);
        Ok(self)
    }

    
    pub fn barcode_code128(&mut self, data: &str) -> Result<&mut Self> {
        if data.is_empty() || !data.is_ascii() || data.len() > 40 {
            return Err(PrinterError::InvalidData(
                "CODE128: entre 1 y 40 caracteres ASCII".into(),
            ));
        }
        let a = self.align.esc_pos();
        self.raw(&[0x1B, 0x61, a]);
        self.raw(&[0x1D, 0x68, 80]); 
        self.raw(&[0x1D, 0x77, 2]); 
        self.raw(&[0x1D, 0x48, 2]); 
        let mut payload = b"{B".to_vec(); 
        payload.extend_from_slice(data.as_bytes());
        self.raw(&[0x1D, 0x6B, 73, payload.len() as u8]);
        self.raw(&payload);
        self.newline();
        self.raw(&[0x1B, 0x61, 0x00]);
        Ok(self)
    }


    pub fn open_cash_drawer(&mut self) -> &mut Self {
        self.raw(&[0x1B, 0x70, 0x00, 0x32, 0xFA])
    }
    
    pub(crate) fn finish(&mut self) {
        self.reset_style();
        self.feed(self.profile.feed_before_cut);
        match self.profile.cut {
            CutMode::None => {}
            CutMode::Full => {
                self.raw(&[0x1D, 0x56, 0x00]);
            }
            CutMode::Partial => {
                self.raw(&[0x1D, 0x56, 0x01]);
            }
        }
    }
}

