
use crate::builder::{Align, EscPosBuilder, QrOptions};
use crate::error::Result;
use crate::printable::Printable;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub underline: bool,
    pub align: Option<Align>,
    pub size: (u8, u8),
}

impl Default for Style {
    fn default() -> Self {
        Self {
            bold: false,
            underline: false,
            align: None,
            size: (1, 1),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Element {
    Line { text: String, style: Style },
    Row { left: String, right: String, style: Style },
    Separator { ch: char, style: Style },
    Qr { data: String, opts: QrOptions, style: Style },
    Barcode { data: String, style: Style },
    Feed(u8),
    CashDrawer,
    Raw(Vec<u8>),
}

impl Element {
    fn style_mut(&mut self) -> Option<&mut Style> {
        match self {
            Element::Line { style, .. }
            | Element::Row { style, .. }
            | Element::Separator { style, .. }
            | Element::Qr { style, .. }
            | Element::Barcode { style, .. } => Some(style),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Document {
    elements: Vec<Element>,
    base_align: Align,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn default_align(mut self, a: Align) -> Self {
        self.base_align = a;
        self
    }

    fn push(mut self, e: Element) -> Self {
        self.elements.push(e);
        self
    }

    fn tweak(mut self, f: impl FnOnce(&mut Style)) -> Self {
        if let Some(s) = self.elements.last_mut().and_then(Element::style_mut) {
            f(s);
        }
        self
    }


    pub fn line(self, text: &str) -> Self {
        self.push(Element::Line {
            text: text.to_string(),
            style: Style::default(),
        })
    }

    pub fn blank(self) -> Self {
        self.line("")
    }


    pub fn row(self, left: &str, right: &str) -> Self {
        self.push(Element::Row {
            left: left.to_string(),
            right: right.to_string(),
            style: Style::default(),
        })
    }


    pub fn separator(self, ch: char) -> Self {
        self.push(Element::Separator {
            ch,
            style: Style::default(),
        })
    }
    
    pub fn feed(self, n: u8) -> Self {
        self.push(Element::Feed(n))
    }


    pub fn qr(self, data: &str) -> Self {
        self.qr_with(data, QrOptions::default())
    }

    pub fn qr_with(self, data: &str, opts: QrOptions) -> Self {
        self.push(Element::Qr {
            data: data.to_string(),
            opts,
            style: Style::default(),
        })
    }


    pub fn barcode(self, data: &str) -> Self {
        self.push(Element::Barcode {
            data: data.to_string(),
            style: Style::default(),
        })
    }

  
    pub fn cash_drawer(self) -> Self {
        self.push(Element::CashDrawer)
    }

  
    pub fn raw(self, bytes: &[u8]) -> Self {
        self.push(Element::Raw(bytes.to_vec()))
    }
    

    pub fn bold(self) -> Self {
        self.tweak(|s| s.bold = true)
    }

    pub fn underline(self) -> Self {
        self.tweak(|s| s.underline = true)
    }

    pub fn align(self, a: Align) -> Self {
        self.tweak(|s| s.align = Some(a))
    }

    pub fn left(self) -> Self {
        self.align(Align::Left)
    }

    pub fn center(self) -> Self {
        self.align(Align::Center)
    }

    pub fn right(self) -> Self {
        self.align(Align::Right)
    }
    
    pub fn size(self, width: u8, height: u8) -> Self {
        self.tweak(|s| s.size = (width, height))
    }
    
    pub fn double(self) -> Self {
        self.size(2, 2)
    }
    
    pub fn each<I, F>(self, items: I, mut f: F) -> Self
    where
        I: IntoIterator,
        F: FnMut(Document, I::Item) -> Document,
    {
        items.into_iter().fold(self, |d, it| f(d, it))
    }
    
    pub fn when(self, cond: bool, f: impl FnOnce(Document) -> Document) -> Self {
        if cond {
            f(self)
        } else {
            self
        }
    }
    
    pub fn append(mut self, other: Document) -> Self {
        self.elements.extend(other.elements);
        self
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    fn apply(&self, doc: &mut EscPosBuilder, style: &Style) {
        doc.bold(style.bold)
            .underline(style.underline)
            .size(style.size.0, style.size.1)
            .align(style.align.unwrap_or(self.base_align));
    }
}

impl Printable for Document {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> {
        for el in &self.elements {
            match el {
                Element::Line { text, style } => {
                    self.apply(doc, style);
                    doc.line(text);
                    doc.reset_style();
                }
                Element::Row { left, right, style } => {
                    self.apply(doc, style);
                    doc.row(left, right);
                    doc.reset_style();
                }
                Element::Separator { ch, style } => {
                    self.apply(doc, style);
                    doc.separator(*ch);
                    doc.reset_style();
                }
                Element::Qr { data, opts, style } => {
                    self.apply(doc, style);
                    doc.qr(data, *opts)?;
                    doc.reset_style();
                }
                Element::Barcode { data, style } => {
                    self.apply(doc, style);
                    doc.barcode_code128(data)?;
                    doc.reset_style();
                }
                Element::Feed(n) => {
                    doc.feed(*n);
                }
                Element::CashDrawer => {
                    doc.open_cash_drawer();
                }
                Element::Raw(bytes) => {
                    doc.raw(bytes);
                }
            }
        }
        Ok(())
    }
}
