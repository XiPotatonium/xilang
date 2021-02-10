use std::fmt;

pub enum FlagTag {
    // Public = 0x0001,
    // Private = 0x0002,
    // Protected = 0x0004,
    Static = 0x0008,
    // Final = 0x0010,
    // Interface = 0x0200,
    // Abstract = 0x0400,
    // Synthetic = 0x1000,
    // Annotation = 0x2000,
    // Enum = 0x4000,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flag {
    pub flag: u16,
}

impl Default for Flag {
    fn default() -> Flag {
        Flag { flag: 0 }
    }
}

impl Flag {
    pub fn set(&mut self, tag: FlagTag) {
        self.flag |= tag as u16;
    }

    pub fn is(&self, tag: FlagTag) -> bool {
        self.flag & (tag as u16) != 0
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        let mut _i = 0;
        if self.is(FlagTag::Static) {
            s.push_str("static");
            _i += 1;
        }
        write!(f, "{}", s)
    }
}
