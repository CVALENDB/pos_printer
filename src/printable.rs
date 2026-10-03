

use crate::builder::EscPosBuilder;
use crate::error::Result;

pub trait Printable {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()>;
}


impl Printable for str {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> {
        doc.line(self);
        Ok(())
    }
}

impl Printable for String {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> {
        self.as_str().render(doc)
    }
}

impl<T: Printable + ?Sized> Printable for &T {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> {
        (**self).render(doc)
    }
}

impl<T: Printable + ?Sized> Printable for Box<T> {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> {
        (**self).render(doc)
    }
}

impl<T: Printable> Printable for [T] {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> {
        for item in self {
            item.render(doc)?;
        }
        Ok(())
    }
}

impl<T: Printable> Printable for Vec<T> {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> {
        self.as_slice().render(doc)
    }
}


pub struct RawBytes(pub Vec<u8>);

impl Printable for RawBytes {
    fn render(&self, doc: &mut EscPosBuilder) -> Result<()> {
        doc.raw(&self.0);
        Ok(())
    }
}
