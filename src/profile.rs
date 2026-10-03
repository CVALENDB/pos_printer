
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodePage {
    Cp437,
    Cp858,
    Wpc1252,
}

impl CodePage {
 
    pub fn esc_pos_id(self) -> u8 {
        match self {
            CodePage::Cp437 => 0,
            CodePage::Cp858 => 19,
            CodePage::Wpc1252 => 16,
        }
    }


    pub fn encode_char(self, c: char) -> u8 {
        if c.is_ascii() {
            return c as u8;
        }
        match self {
            CodePage::Cp437 | CodePage::Cp858 => {
                let common = match c {
                    'ü' => Some(0x81),
                    'é' => Some(0x82),
                    'á' => Some(0xA0),
                    'í' => Some(0xA1),
                    'ó' => Some(0xA2),
                    'ú' => Some(0xA3),
                    'ñ' => Some(0xA4),
                    'Ñ' => Some(0xA5),
                    '¿' => Some(0xA8),
                    '¡' => Some(0xAD),
                    'É' => Some(0x90),
                    'Ü' => Some(0x9A),
                    _ => None,
                };
                if let Some(b) = common {
                    return b;
                }
                if self == CodePage::Cp858 {
                    match c {
                        'Á' => return 0xB5,
                        'Í' => return 0xD6,
                        'Ó' => return 0xE0,
                        'Ú' => return 0xE9,
                        '€' => return 0xD5,
                        _ => {}
                    }
                } else {
                    match c {
                        'Á' => return b'A',
                        'Í' => return b'I',
                        'Ó' => return b'O',
                        'Ú' => return b'U',
                        _ => {}
                    }
                }
                b'?'
            }
            CodePage::Wpc1252 => {
                let n = c as u32;
                if c == '€' {
                    0x80
                } else if (0xA0..=0xFF).contains(&n) {
                    n as u8
                } else {
                    b'?'
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutMode {
    None,
    Partial,
    Full,
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterProfile {
    pub chars_per_line: usize,
    pub code_page: CodePage,
    pub cut: CutMode,
    pub feed_before_cut: u8,
    pub margin: usize,
}

impl PrinterProfile {
    pub fn mm58() -> Self {
        Self {
            chars_per_line: 32,
            code_page: CodePage::Cp858,
            cut: CutMode::Full,
            feed_before_cut: 4,
            margin: 0,
        }
    }
    
    pub fn mm80() -> Self {
        Self {
            chars_per_line: 48,
            ..Self::mm58()
        }
    }
}


impl PrinterProfile {
    pub fn with_chars_per_line(mut self, n: usize) -> Self {
        self.chars_per_line = n.max(1);
        self
    }

    pub fn with_margin(mut self, columns: usize) -> Self {
        self.margin = columns;
        self
    }

    pub fn with_code_page(mut self, cp: CodePage) -> Self {
        self.code_page = cp;
        self
    }

    pub fn with_cut(mut self, cut: CutMode) -> Self {
        self.cut = cut;
        self
    }

    pub fn with_feed_before_cut(mut self, lines: u8) -> Self {
        self.feed_before_cut = lines;
        self
    }
}

impl Default for PrinterProfile {
    fn default() -> Self {
        Self::mm80()
    }
}
