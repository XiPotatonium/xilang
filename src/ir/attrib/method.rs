use std::convert::TryFrom;
use std::fmt;

const METHOD_ATTRIB_ACC_MASK: u16 = 0x0007;

const METHOD_ATTRIB_PRIV_FLAG: u16 = 0x0001;
const METHOD_ATTRIB_PUB_FLAG: u16 = 0x0006;
const METHOD_ATTRIB_STATIC_FLAG: u16 = 0x0010;
const METHOD_ATTRIB_VIRTUAL_FLAG: u16 = 0x0040;
const METHOD_ATTRIB_NEWSLOT_FLAG: u16 = 0x0100;
const METHOD_ATTRIB_SPECIAL_NAME_FLAG: u16 = 0x0800;
const METHOD_ATTRIB_PINVOKEIMPL_FLAG: u16 = 0x2000;
const METHOD_ATTRIB_RT_SPECIAL_NAME_FLAG: u16 = 0x1000;

pub enum MethodAttribFlag {
    Priv,
    Pub,
    Static,
    Virtual,
    /// for ReuseSlot, just unset NewSlot
    NewSlot,
    SpecialName,
    PInvokeImpl,
    RTSpecialName,
}

impl TryFrom<u16> for MethodAttribFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            METHOD_ATTRIB_PRIV_FLAG => Ok(Self::Priv),
            METHOD_ATTRIB_PUB_FLAG => Ok(Self::Pub),
            METHOD_ATTRIB_STATIC_FLAG => Ok(Self::Static),
            METHOD_ATTRIB_PINVOKEIMPL_FLAG => Ok(Self::PInvokeImpl),
            METHOD_ATTRIB_RT_SPECIAL_NAME_FLAG => Ok(Self::RTSpecialName),
            METHOD_ATTRIB_VIRTUAL_FLAG => Ok(Self::Virtual),
            METHOD_ATTRIB_SPECIAL_NAME_FLAG => Ok(Self::SpecialName),
            METHOD_ATTRIB_NEWSLOT_FLAG => Ok(Self::NewSlot),
            _ => Err("Invalid value for MethodFlagTag"),
        }
    }
}

impl From<MethodAttribFlag> for u16 {
    fn from(value: MethodAttribFlag) -> Self {
        match value {
            MethodAttribFlag::Priv => METHOD_ATTRIB_PRIV_FLAG,
            MethodAttribFlag::Pub => METHOD_ATTRIB_PUB_FLAG,
            MethodAttribFlag::Static => METHOD_ATTRIB_STATIC_FLAG,
            MethodAttribFlag::PInvokeImpl => METHOD_ATTRIB_PINVOKEIMPL_FLAG,
            MethodAttribFlag::RTSpecialName => METHOD_ATTRIB_RT_SPECIAL_NAME_FLAG,
            MethodAttribFlag::Virtual => METHOD_ATTRIB_VIRTUAL_FLAG,
            MethodAttribFlag::SpecialName => METHOD_ATTRIB_SPECIAL_NAME_FLAG,
            MethodAttribFlag::NewSlot => METHOD_ATTRIB_NEWSLOT_FLAG,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MethodAttrib {
    pub attrib: u16,
}

impl MethodAttrib {
    pub fn from(attrib: u16) -> MethodAttrib {
        MethodAttrib { attrib }
    }

    pub fn set(&mut self, flag: MethodAttribFlag) {
        match flag {
            MethodAttribFlag::Pub | MethodAttribFlag::Priv => {
                self.attrib = (self.attrib & !METHOD_ATTRIB_ACC_MASK) | u16::from(flag);
            }
            _ => self.attrib |= u16::from(flag),
        }
    }

    pub fn is(&self, flag: MethodAttribFlag) -> bool {
        match flag {
            MethodAttribFlag::Pub | MethodAttribFlag::Priv => {
                (self.attrib & METHOD_ATTRIB_ACC_MASK) == u16::from(flag)
            }
            _ => (self.attrib & u16::from(flag)) != 0,
        }
    }
}

impl fmt::Display for MethodAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.attrib & METHOD_ATTRIB_ACC_MASK {
            METHOD_ATTRIB_PRIV_FLAG => write!(f, "priv")?,
            METHOD_ATTRIB_PUB_FLAG => write!(f, "pub")?,
            _ => unreachable!(),
        }

        if self.is(MethodAttribFlag::Static) {
            write!(f, " static")?;
        }

        if self.is(MethodAttribFlag::Virtual) {
            write!(f, " virtual")?;
        }

        if self.is(MethodAttribFlag::NewSlot) {
            write!(f, " newslot")?;
        }

        if self.is(MethodAttribFlag::SpecialName) {
            write!(f, " specialname")?;
        }

        if self.is(MethodAttribFlag::RTSpecialName) {
            write!(f, " rtspecialname")?;
        }

        Ok(())
    }
}

const METHOD_IMPL_ATTRIB_CODE_TYPE_MASK: u16 = 0x0003;
const METHOD_IMPL_ATTRIB_IL_FLAG: u16 = 0x0000;
const METHOD_IMPL_ATTRIB_NATIVE_FLAG: u16 = 0x0001;
const METHOD_IMPL_ATTRIB_RUNTIME_FLAG: u16 = 0x0003;

const METHOD_IMPL_ATTRIB_MANAGED_MASK: u16 = 0x0004;
const METHOD_IMPL_ATTRIB_UNMANAGED_FLAG: u16 = 0x0004;
const METHOD_IMPL_ATTRIB_MANAGED_FLAG: u16 = 0x0000;

const METHOD_IMPL_ATTRIB_INTERNALCALL_FLAG: u16 = 0x1000;

pub enum MethodImplAttribCodeTypeFlag {
    IL,
    Native,
    Runtime,
}

pub enum MethodImplAttribManagedFlag {
    Unmanaged,
    Managed,
}

impl TryFrom<u16> for MethodImplAttribCodeTypeFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            METHOD_IMPL_ATTRIB_IL_FLAG => Ok(MethodImplAttribCodeTypeFlag::IL),
            METHOD_IMPL_ATTRIB_NATIVE_FLAG => Ok(MethodImplAttribCodeTypeFlag::Native),
            METHOD_IMPL_ATTRIB_RUNTIME_FLAG => Ok(MethodImplAttribCodeTypeFlag::Runtime),
            _ => Err("Invalid value for MethodImplAttribCodeTypeFlag"),
        }
    }
}

impl TryFrom<u16> for MethodImplAttribManagedFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            METHOD_IMPL_ATTRIB_MANAGED_FLAG => Ok(MethodImplAttribManagedFlag::Managed),
            METHOD_IMPL_ATTRIB_UNMANAGED_FLAG => Ok(MethodImplAttribManagedFlag::Unmanaged),
            _ => Err("Invalid value for MethodImplAttribManagedFlag"),
        }
    }
}

impl From<MethodImplAttribCodeTypeFlag> for u16 {
    fn from(value: MethodImplAttribCodeTypeFlag) -> Self {
        match value {
            MethodImplAttribCodeTypeFlag::IL => METHOD_IMPL_ATTRIB_IL_FLAG,
            MethodImplAttribCodeTypeFlag::Native => METHOD_IMPL_ATTRIB_NATIVE_FLAG,
            MethodImplAttribCodeTypeFlag::Runtime => METHOD_IMPL_ATTRIB_RUNTIME_FLAG,
        }
    }
}

impl From<MethodImplAttribManagedFlag> for u16 {
    fn from(value: MethodImplAttribManagedFlag) -> Self {
        match value {
            MethodImplAttribManagedFlag::Unmanaged => METHOD_IMPL_ATTRIB_UNMANAGED_FLAG,
            MethodImplAttribManagedFlag::Managed => METHOD_IMPL_ATTRIB_MANAGED_FLAG,
        }
    }
}

pub enum MethodImplInfoFlag {
    InternalCall,
}

impl TryFrom<u16> for MethodImplInfoFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            METHOD_IMPL_ATTRIB_INTERNALCALL_FLAG => Ok(MethodImplInfoFlag::InternalCall),
            _ => Err("Invalid value for MethodImplInfoFlag"),
        }
    }
}

impl From<MethodImplInfoFlag> for u16 {
    fn from(value: MethodImplInfoFlag) -> Self {
        match value {
            MethodImplInfoFlag::InternalCall => METHOD_IMPL_ATTRIB_INTERNALCALL_FLAG,
        }
    }
}

#[derive(Clone)]
pub struct MethodImplAttrib {
    pub attrib: u16,
}

impl MethodImplAttrib {
    pub fn from(attrib: u16) -> MethodImplAttrib {
        MethodImplAttrib { attrib }
    }

    pub fn new(
        code_ty: MethodImplAttribCodeTypeFlag,
        managed_flag: MethodImplAttribManagedFlag,
    ) -> MethodImplAttrib {
        MethodImplAttrib {
            attrib: u16::from(code_ty) | u16::from(managed_flag),
        }
    }

    pub fn is_impl_info(&self, info: MethodImplInfoFlag) -> bool {
        (self.attrib & u16::from(info)) != 0
    }

    pub fn set_impl_info(&mut self, info: MethodImplInfoFlag) {
        self.attrib = (self.attrib
            & (METHOD_IMPL_ATTRIB_CODE_TYPE_MASK | METHOD_IMPL_ATTRIB_MANAGED_MASK))
            | u16::from(info);
    }

    pub fn is_code_ty(&self, flag: MethodImplAttribCodeTypeFlag) -> bool {
        (self.attrib & METHOD_IMPL_ATTRIB_CODE_TYPE_MASK) == u16::from(flag)
    }

    pub fn set_code_ty(&mut self, flag: MethodImplAttribCodeTypeFlag) {
        self.attrib = (self.attrib & !METHOD_IMPL_ATTRIB_CODE_TYPE_MASK) | u16::from(flag)
    }

    pub fn code_ty(&self) -> MethodImplAttribCodeTypeFlag {
        MethodImplAttribCodeTypeFlag::try_from(self.attrib & METHOD_IMPL_ATTRIB_CODE_TYPE_MASK)
            .unwrap()
    }

    pub fn is_managed(&self, flag: MethodImplAttribManagedFlag) -> bool {
        (self.attrib & METHOD_IMPL_ATTRIB_MANAGED_MASK) == u16::from(flag)
    }

    pub fn set_managed(&mut self, flag: MethodImplAttribManagedFlag) {
        self.attrib = (self.attrib & !METHOD_IMPL_ATTRIB_MANAGED_MASK) | u16::from(flag)
    }

    pub fn managed(&self) -> MethodImplAttribManagedFlag {
        MethodImplAttribManagedFlag::try_from(self.attrib & METHOD_IMPL_ATTRIB_MANAGED_MASK)
            .unwrap()
    }
}

impl fmt::Display for MethodImplAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.managed() {
            MethodImplAttribManagedFlag::Unmanaged => write!(f, "unmanaged "),
            MethodImplAttribManagedFlag::Managed => write!(f, "managed "),
        }?;

        match self.code_ty() {
            MethodImplAttribCodeTypeFlag::IL => write!(f, "cil"),
            MethodImplAttribCodeTypeFlag::Native => write!(f, "native"),
            MethodImplAttribCodeTypeFlag::Runtime => write!(f, "runtime"),
        }
    }
}
