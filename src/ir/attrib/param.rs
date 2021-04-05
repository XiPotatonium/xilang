use std::fmt;

#[derive(Clone, Copy)]
pub struct ParamAttrib {
    pub attrib: u16,
}

impl ParamAttrib {
    pub fn from(attrib: u16) -> ParamAttrib {
        ParamAttrib { attrib }
    }
}

impl fmt::Display for ParamAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")?;
        Ok(())
    }
}
