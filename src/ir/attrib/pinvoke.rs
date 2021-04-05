use std::convert::TryFrom;
use std::fmt;

const PINVOKE_ATTRIB_NO_MANGLE_FLAG: u16 = 0x0001;

const PINVOKE_ATTRIB_CHARSET_MASK: u16 = 0x0006;

const PINVOKE_ATTRIB_ANSI_FLAG: u16 = 0x0002;

const PINVOKE_ATTRIB_CALL_CONV_MASK: u16 = 0x0700;

const PINVOKE_ATTRIB_CDECL_FLAG: u16 = 0x0200;

pub enum PInvokeAttribCharsetFlag {
    Ansi,
}

impl TryFrom<u16> for PInvokeAttribCharsetFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            PINVOKE_ATTRIB_ANSI_FLAG => Ok(Self::Ansi),
            _ => Err("Invalid value for PInvokeAttribCharSetFlag"),
        }
    }
}

impl From<PInvokeAttribCharsetFlag> for u16 {
    fn from(value: PInvokeAttribCharsetFlag) -> Self {
        match value {
            PInvokeAttribCharsetFlag::Ansi => PINVOKE_ATTRIB_ANSI_FLAG,
        }
    }
}

pub enum PInvokeAttribCallConvFlag {
    CDecl,
}

impl TryFrom<u16> for PInvokeAttribCallConvFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            PINVOKE_ATTRIB_CDECL_FLAG => Ok(Self::CDecl),
            _ => Err("Invalid value for PInvokeAttribCallConvFlag"),
        }
    }
}

impl From<PInvokeAttribCallConvFlag> for u16 {
    fn from(value: PInvokeAttribCallConvFlag) -> Self {
        match value {
            PInvokeAttribCallConvFlag::CDecl => PINVOKE_ATTRIB_CDECL_FLAG,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PInvokeAttrib {
    pub attrib: u16,
}

impl PInvokeAttrib {
    pub fn from(attrib: u16) -> PInvokeAttrib {
        PInvokeAttrib { attrib }
    }

    pub fn new(
        charset_flag: PInvokeAttribCharsetFlag,
        conv_flag: PInvokeAttribCallConvFlag,
    ) -> PInvokeAttrib {
        PInvokeAttrib {
            attrib: u16::from(charset_flag) | u16::from(conv_flag) | PINVOKE_ATTRIB_NO_MANGLE_FLAG,
        }
    }

    pub fn is_charset(&self, flag: PInvokeAttribCharsetFlag) -> bool {
        self.attrib & u16::from(flag) != 0
    }

    pub fn is_call_convention(&self, flag: PInvokeAttribCallConvFlag) -> bool {
        self.attrib & u16::from(flag) != 0
    }

    pub fn is_no_mangle(&self) -> bool {
        self.attrib & PINVOKE_ATTRIB_NO_MANGLE_FLAG != 0
    }
}

impl fmt::Display for PInvokeAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.attrib & PINVOKE_ATTRIB_CHARSET_MASK {
            PINVOKE_ATTRIB_ANSI_FLAG => write!(f, "ansi")?,
            _ => unreachable!(),
        }

        match self.attrib & PINVOKE_ATTRIB_CALL_CONV_MASK {
            PINVOKE_ATTRIB_CDECL_FLAG => write!(f, " cdecl")?,
            _ => unreachable!(),
        }

        Ok(())
    }
}
