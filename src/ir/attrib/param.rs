use std::fmt;

const PARAM_ATTRIB_DEFAULT_FLAG: u16 = 0x0;

pub enum ParamAttribFlag {
    Default,
}

impl From<ParamAttribFlag> for u16 {
    fn from(value: ParamAttribFlag) -> Self {
        match value {
            ParamAttribFlag::Default => PARAM_ATTRIB_DEFAULT_FLAG,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ParamAttrib {
    pub attrib: u16,
}

impl Default for ParamAttrib {
    fn default() -> Self {
        ParamAttrib { attrib: 0 }
    }
}

impl ParamAttrib {
    pub fn from(attrib: u16) -> ParamAttrib {
        ParamAttrib { attrib }
    }

    pub fn is(&self, flag: ParamAttribFlag) -> bool {
        self.attrib == u16::from(flag)
    }
}

impl fmt::Display for ParamAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")?;
        Ok(())
    }
}
