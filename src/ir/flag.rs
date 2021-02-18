use std::fmt;

static FIELD_ACC_MASK: u16 = 0x0007;

pub enum FieldFlagTag {
    Priv = 0x0001,
    Pub = 0x0006,
    Static = 0x0010,
}

#[derive(Clone, Copy)]
pub struct FieldFlag {
    pub flag: u16,
}

impl FieldFlag {
    pub fn new(flag: u16) -> FieldFlag {
        FieldFlag { flag }
    }

    pub fn set(&mut self, tag: FieldFlagTag) {
        match tag {
            FieldFlagTag::Pub | FieldFlagTag::Priv => {
                self.flag = (self.flag & !FIELD_ACC_MASK) | tag as u16;
            }
            _ => self.flag |= tag as u16,
        }
    }

    pub fn is(&self, tag: FieldFlagTag) -> bool {
        self.flag & (tag as u16) != 0
    }
}

impl Default for FieldFlag {
    fn default() -> FieldFlag {
        FieldFlag {
            flag: FieldFlagTag::Pub as u16,
        }
    }
}

impl fmt::Display for FieldFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.flag & FIELD_ACC_MASK {
            0x0001 => write!(f, "priv")?,
            0x0006 => write!(f, "pub")?,
            _ => unreachable!(),
        }

        if self.is(FieldFlagTag::Static) {
            write!(f, " static")?;
        }

        Ok(())
    }
}

static METHOD_ACC_MASK: u16 = 0x0007;

pub enum MethodFlagTag {
    Priv = 0x0001,
    Pub = 0x0006,
    Static = 0x0010,
}

#[derive(Clone, Copy)]
pub struct MethodFlag {
    pub flag: u16,
}

impl MethodFlag {
    pub fn new(flag: u16) -> MethodFlag {
        MethodFlag { flag }
    }
    pub fn set(&mut self, tag: MethodFlagTag) {
        match tag {
            MethodFlagTag::Pub | MethodFlagTag::Priv => {
                self.flag = (self.flag & !METHOD_ACC_MASK) | tag as u16;
            }
            _ => self.flag |= tag as u16,
        }
    }

    pub fn unset(&mut self, tag: MethodFlagTag) {
        match tag {
            MethodFlagTag::Pub | MethodFlagTag::Priv => {
                panic!("Cannot unset access tag. Use FieldFlag.set to set the correct tag")
            }
            _ => self.flag ^= tag as u16,
        }
    }

    pub fn is(&self, tag: MethodFlagTag) -> bool {
        self.flag & (tag as u16) != 0
    }
}

impl Default for MethodFlag {
    fn default() -> MethodFlag {
        MethodFlag {
            flag: MethodFlagTag::Pub as u16,
        }
    }
}

impl fmt::Display for MethodFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.flag & METHOD_ACC_MASK {
            0x0001 => write!(f, "priv")?,
            0x0006 => write!(f, "pub")?,
            _ => unreachable!(),
        }

        if self.is(MethodFlagTag::Static) {
            write!(f, " static")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct ParamFlag {
    pub flag: u16,
}

impl Default for ParamFlag {
    fn default() -> ParamFlag {
        ParamFlag { flag: 0 }
    }
}

impl fmt::Display for ParamFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "");
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct LocalFlag {
    pub flag: u16,
}

impl Default for LocalFlag {
    fn default() -> LocalFlag {
        LocalFlag { flag: 0 }
    }
}

impl fmt::Display for LocalFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "");
        Ok(())
    }
}

static TYPE_VIS_MASK: u32 = 0x00000007;

pub enum TypeVisTag {
    Priv = 0x00000000,
    Pub = 0x00000001,
}

static TYPE_SEM_MASK: u32 = 0x00000020;

pub enum TypeSemTag {
    Class = 0x00000000,
    Interface = 0x00000020,
}

#[derive(Clone, Copy)]
pub struct TypeFlag {
    pub flag: u32,
}

impl TypeFlag {
    pub fn new(flag: u32) -> TypeFlag {
        TypeFlag { flag }
    }

    pub fn set_vis(&mut self, tag: TypeVisTag) {
        self.flag = (self.flag & !TYPE_VIS_MASK) | tag as u32;
    }

    pub fn set_sem(&mut self, tag: TypeSemTag) {
        self.flag = (self.flag & !TYPE_SEM_MASK) | tag as u32;
    }

    pub fn is_vis(&self, tag: TypeVisTag) -> bool {
        self.flag & (tag as u32) != 0
    }

    pub fn is_sem(&self, tag: TypeSemTag) -> bool {
        self.flag & (tag as u32) != 0
    }
}

impl Default for TypeFlag {
    fn default() -> TypeFlag {
        TypeFlag {
            flag: TypeVisTag::Pub as u32,
        }
    }
}

impl fmt::Display for TypeFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.flag & TYPE_VIS_MASK {
            0x00000000 => write!(f, "priv")?,
            0x00000001 => write!(f, "pub")?,
            _ => unreachable!(),
        }

        match self.flag & TYPE_SEM_MASK {
            0x00000000 => write!(f, " class")?,
            0x00000020 => write!(f, " interface")?,
            _ => unreachable!(),
        }

        Ok(())
    }
}
