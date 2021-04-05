use std::convert::TryFrom;
use std::fmt;

const TYPE_ATTRIB_VIS_MASK: u32 = 0x00000007;

const TYPE_ATTRIB_PRIV_FLAG: u32 = 0x000000000;
const TYPE_ATTRIB_PUB_FLAG: u32 = 0x000000001;

const TYPE_ATTRIB_SEM_MASK: u32 = 0x00000020;

const TYPE_ATTRIB_CLASS_FLAG: u32 = 0x00000000;
const TYPE_ATTRIB_INTERFACE_FLAG: u32 = 0x00000020;

pub enum TypeAttribVisFlag {
    Priv,
    Pub,
}

impl TryFrom<u32> for TypeAttribVisFlag {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            TYPE_ATTRIB_PRIV_FLAG => Ok(Self::Priv),
            TYPE_ATTRIB_PUB_FLAG => Ok(Self::Pub),
            _ => Err("Invalid value for TypeAttribVisFlag"),
        }
    }
}

impl From<TypeAttribVisFlag> for u32 {
    fn from(value: TypeAttribVisFlag) -> Self {
        match value {
            TypeAttribVisFlag::Priv => TYPE_ATTRIB_PRIV_FLAG,
            TypeAttribVisFlag::Pub => TYPE_ATTRIB_PUB_FLAG,
        }
    }
}

pub enum TypeAttribSemFlag {
    Class,
    Interface,
}

impl TryFrom<u32> for TypeAttribSemFlag {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            TYPE_ATTRIB_CLASS_FLAG => Ok(Self::Class),
            TYPE_ATTRIB_INTERFACE_FLAG => Ok(Self::Interface),
            _ => Err("Invalid value for TypeAttribSemFlag"),
        }
    }
}

impl From<TypeAttribSemFlag> for u32 {
    fn from(value: TypeAttribSemFlag) -> Self {
        match value {
            TypeAttribSemFlag::Class => TYPE_ATTRIB_CLASS_FLAG,
            TypeAttribSemFlag::Interface => TYPE_ATTRIB_INTERFACE_FLAG,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TypeAttrib {
    pub attrib: u32,
}

impl TypeAttrib {
    pub fn from(attrib: u32) -> TypeAttrib {
        TypeAttrib { attrib }
    }

    pub fn new_class(attrib: u32) -> TypeAttrib {
        TypeAttrib {
            attrib: (attrib & !TYPE_ATTRIB_SEM_MASK) | TYPE_ATTRIB_CLASS_FLAG,
        }
    }

    pub fn set_vis(&mut self, flag: TypeAttribVisFlag) {
        self.attrib = (self.attrib & !TYPE_ATTRIB_VIS_MASK) | u32::from(flag);
    }

    pub fn set_sem(&mut self, flag: TypeAttribSemFlag) {
        self.attrib = (self.attrib & !TYPE_ATTRIB_SEM_MASK) | u32::from(flag);
    }

    pub fn is_vis(&self, flag: TypeAttribVisFlag) -> bool {
        self.attrib & u32::from(flag) != 0
    }

    pub fn is_sem(&self, flag: TypeAttribSemFlag) -> bool {
        self.attrib & u32::from(flag) != 0
    }
}

impl fmt::Display for TypeAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.attrib & TYPE_ATTRIB_VIS_MASK {
            TYPE_ATTRIB_PRIV_FLAG => write!(f, "priv")?,
            TYPE_ATTRIB_PUB_FLAG => write!(f, "pub")?,
            _ => unreachable!(),
        }

        match self.attrib & TYPE_ATTRIB_SEM_MASK {
            TYPE_ATTRIB_CLASS_FLAG => write!(f, " class")?,
            TYPE_ATTRIB_INTERFACE_FLAG => write!(f, " interface")?,
            _ => unreachable!(),
        }

        Ok(())
    }
}
