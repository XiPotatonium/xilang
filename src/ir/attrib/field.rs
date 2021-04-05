use std::convert::TryFrom;
use std::fmt;

const FIELD_ATTRIB_ACC_MASK: u16 = 0x0007;

const FIELD_ATTRIB_PRIV_FLAG: u16 = 0x0001;
const FIELD_ATTRIB_PUB_FLAG: u16 = 0x0006;
const FIELD_ATTRIB_STATIC_FLAG: u16 = 0x0010;

pub enum FieldAttribFlag {
    Priv,
    Pub,
    Static,
}

impl TryFrom<u16> for FieldAttribFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            FIELD_ATTRIB_PRIV_FLAG => Ok(Self::Priv),
            FIELD_ATTRIB_PUB_FLAG => Ok(Self::Pub),
            FIELD_ATTRIB_STATIC_FLAG => Ok(Self::Static),
            _ => Err("Invalid value for MethodFlagTag"),
        }
    }
}

impl From<FieldAttribFlag> for u16 {
    fn from(value: FieldAttribFlag) -> Self {
        match value {
            FieldAttribFlag::Priv => FIELD_ATTRIB_PRIV_FLAG,
            FieldAttribFlag::Pub => FIELD_ATTRIB_PUB_FLAG,
            FieldAttribFlag::Static => FIELD_ATTRIB_STATIC_FLAG,
        }
    }
}

#[derive(Clone, Copy)]
pub struct FieldAttrib {
    pub attrib: u16,
}

impl FieldAttrib {
    pub fn from(attrib: u16) -> FieldAttrib {
        FieldAttrib { attrib }
    }

    pub fn set(&mut self, flag: FieldAttribFlag) {
        match flag {
            FieldAttribFlag::Pub | FieldAttribFlag::Priv => {
                self.attrib = (self.attrib & !FIELD_ATTRIB_ACC_MASK) | u16::from(flag);
            }
            _ => self.attrib |= u16::from(flag),
        }
    }

    pub fn is(&self, flag: FieldAttribFlag) -> bool {
        self.attrib & u16::from(flag) != 0
    }
}

impl fmt::Display for FieldAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.attrib & FIELD_ATTRIB_ACC_MASK {
            FIELD_ATTRIB_PRIV_FLAG => write!(f, "priv")?,
            FIELD_ATTRIB_PUB_FLAG => write!(f, "pub")?,
            _ => unreachable!(),
        }

        if self.is(FieldAttribFlag::Static) {
            write!(f, " static")?;
        }

        Ok(())
    }
}
