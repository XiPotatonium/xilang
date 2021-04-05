use std::fmt;

#[derive(Clone, Copy)]
pub struct LocalAttrib {
    pub attrib: u16,
}

impl LocalAttrib {
    pub fn from(attrib: u16) -> LocalAttrib {
        LocalAttrib { attrib }
    }
}

impl fmt::Display for LocalAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")?;
        Ok(())
    }
}
