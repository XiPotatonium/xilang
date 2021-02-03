use std::convert::TryFrom;

use std::fmt;

pub enum FlagTag {
    Static = 1,
}

impl TryFrom<u32> for FlagTag {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<FlagTag, Self::Error> {
        match value {
            1 => Ok(FlagTag::Static),
            _ => Err("Invalid flag"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flag {
    flag: u32,
}

impl Default for Flag {
    fn default() -> Flag {
        Flag { flag: 0 }
    }
}

impl Flag {
    pub fn set(&mut self, tag: FlagTag) {
        self.flag |= tag as u32;
    }

    pub fn is(&self, tag: FlagTag) -> bool {
        self.flag & (tag as u32) != 0
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
