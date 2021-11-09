use std::{convert::TryFrom, fmt};

/// Table 4.1-B
pub enum ClassFlag {
    /// 0x0001
    Public,
    /// 0x0010
    Final,
    /// 0x0020, Treat superclass methods specially when invoked by the invokespecial instruction.
    Super,
    /// 0x0200
    Interface,
    /// 0x0400
    Abstract,
    /// 0x1000, Generated by compiler
    Synthetic,
    /// 0x2000
    Annotation,
    /// 0x4000
    Enum,
    /// 0x8000, This is a module not a class
    Module,
}

const CLASS_ACC_PUBLIC: u16 = 0x0001;
const CLASS_ACC_FINAL: u16 = 0x0010;
const CLASS_ACC_SUPER: u16 = 0x0020;
const CLASS_ACC_INTERFACE: u16 = 0x0200;
const CLASS_ACC_ABSTRACT: u16 = 0x0400;
const CLASS_ACC_SYNTHETIC: u16 = 0x1000;
const CLASS_ACC_ANNOTATION: u16 = 0x2000;
const CLASS_ACC_ENUM: u16 = 0x4000;
const CLASS_ACC_MODULE: u16 = 0x8000;

impl TryFrom<u16> for ClassFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            CLASS_ACC_PUBLIC => Ok(ClassFlag::Public),
            CLASS_ACC_FINAL => Ok(ClassFlag::Final),
            CLASS_ACC_SUPER => Ok(ClassFlag::Super),
            CLASS_ACC_INTERFACE => Ok(ClassFlag::Interface),
            CLASS_ACC_ABSTRACT => Ok(ClassFlag::Abstract),
            CLASS_ACC_SYNTHETIC => Ok(ClassFlag::Synthetic),
            CLASS_ACC_ANNOTATION => Ok(ClassFlag::Annotation),
            CLASS_ACC_ENUM => Ok(ClassFlag::Enum),
            CLASS_ACC_MODULE => Ok(ClassFlag::Module),
            _ => Err("Invalid value for ClassFlag"),
        }
    }
}

impl From<ClassFlag> for u16 {
    fn from(value: ClassFlag) -> Self {
        match value {
            ClassFlag::Public => CLASS_ACC_PUBLIC,
            ClassFlag::Final => CLASS_ACC_FINAL,
            ClassFlag::Super => CLASS_ACC_SUPER,
            ClassFlag::Interface => CLASS_ACC_INTERFACE,
            ClassFlag::Abstract => CLASS_ACC_ABSTRACT,
            ClassFlag::Synthetic => CLASS_ACC_SYNTHETIC,
            ClassFlag::Annotation => CLASS_ACC_ANNOTATION,
            ClassFlag::Enum => CLASS_ACC_ENUM,
            ClassFlag::Module => CLASS_ACC_MODULE,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ClassFlags(pub u16);

impl From<u16> for ClassFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl ClassFlags {
    pub fn is(&self, flag: ClassFlag) -> bool {
        (self.0 & u16::from(flag)) != 0
    }

    /// Compliance will not be checked here
    pub fn set(&mut self, flag: ClassFlag) {
        self.0 |= u16::from(flag);
    }
}

impl fmt::Display for ClassFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is(ClassFlag::Public) {
            write!(f, "public")?;
        } else {
            write!(f, "internal")?;
        }

        if self.is(ClassFlag::Final) {
            write!(f, " final")?;
        }

        if self.is(ClassFlag::Super) {
            write!(f, " super")?;
        }

        if self.is(ClassFlag::Interface) {
            write!(f, " interface")?;
        }

        if self.is(ClassFlag::Abstract) {
            write!(f, " abstract")?;
        }

        if self.is(ClassFlag::Synthetic) {
            write!(f, " synthetic")?;
        }

        if self.is(ClassFlag::Annotation) {
            write!(f, " annotation")?;
        }

        if self.is(ClassFlag::Enum) {
            write!(f, " enum")?;
        }

        if self.is(ClassFlag::Module) {
            write!(f, " module")?;
        }

        Ok(())
    }
}

/// JVMs 4.5 Fields
pub enum FieldFlag {
    /// 0x0001
    Public,
    /// 0x0002
    Private,
    /// 0x0004
    Protected,
    /// 0x0008
    Static,
    /// 0x0010
    Final,
    /// 0x0040
    Volatile,
    /// 0x0080
    Transient,
    /// 0x1000
    Synthetic,
    /// 0x4000
    Enum,
}

const FIELD_ACC_PUBLIC: u16 = 0x0001;
const FIELD_ACC_PRIVATE: u16 = 0x0002;
const FIELD_ACC_PROTECTED: u16 = 0x0004;
const FIELD_ACC_STATIC: u16 = 0x0008;
const FIELD_ACC_FINAL: u16 = 0x0010;
const FIELD_ACC_VOLATILE: u16 = 0x0040;
const FIELD_ACC_TRANSIENT: u16 = 0x0080;
const FIELD_ACC_SYNTHETIC: u16 = 0x1000;
const FIELD_ACC_ENUM: u16 = 0x4000;

impl TryFrom<u16> for FieldFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            FIELD_ACC_PUBLIC => Ok(FieldFlag::Public),
            FIELD_ACC_PRIVATE => Ok(FieldFlag::Private),
            FIELD_ACC_PROTECTED => Ok(FieldFlag::Protected),
            FIELD_ACC_STATIC => Ok(FieldFlag::Static),
            FIELD_ACC_FINAL => Ok(FieldFlag::Final),
            FIELD_ACC_VOLATILE => Ok(FieldFlag::Volatile),
            FIELD_ACC_TRANSIENT => Ok(FieldFlag::Transient),
            FIELD_ACC_SYNTHETIC => Ok(FieldFlag::Synthetic),
            FIELD_ACC_ENUM => Ok(FieldFlag::Enum),
            _ => Err("Invalid value for FieldFlag"),
        }
    }
}

impl From<FieldFlag> for u16 {
    fn from(value: FieldFlag) -> Self {
        match value {
            FieldFlag::Public => FIELD_ACC_PUBLIC,
            FieldFlag::Private => FIELD_ACC_PRIVATE,
            FieldFlag::Protected => FIELD_ACC_PROTECTED,
            FieldFlag::Static => FIELD_ACC_STATIC,
            FieldFlag::Final => FIELD_ACC_FINAL,
            FieldFlag::Volatile => FIELD_ACC_VOLATILE,
            FieldFlag::Transient => FIELD_ACC_TRANSIENT,
            FieldFlag::Synthetic => FIELD_ACC_SYNTHETIC,
            FieldFlag::Enum => FIELD_ACC_ENUM,
        }
    }
}

#[derive(Clone, Copy)]
pub struct FieldFlags(pub u16);

impl From<u16> for FieldFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl FieldFlags {
    pub fn is(&self, flag: FieldFlag) -> bool {
        (self.0 & u16::from(flag)) != 0
    }

    pub fn set(&mut self, flag: FieldFlag) {
        self.0 |= u16::from(flag);
    }
}

impl fmt::Display for FieldFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is(FieldFlag::Public) {
            write!(f, "public")?;
        } else if self.is(FieldFlag::Private) {
            write!(f, "private")?;
        } else if self.is(FieldFlag::Protected) {
            write!(f, "protected")?;
        } else {
            unreachable!();
        }

        if self.is(FieldFlag::Static) {
            write!(f, " static")?;
        }

        if self.is(FieldFlag::Final) {
            write!(f, " final")?;
        }

        if self.is(FieldFlag::Volatile) {
            write!(f, " volatile")?;
        }

        if self.is(FieldFlag::Transient) {
            write!(f, " transient")?;
        }

        if self.is(FieldFlag::Synthetic) {
            write!(f, " synthetic")?;
        }

        if self.is(FieldFlag::Enum) {
            write!(f, " enum")?;
        }

        Ok(())
    }
}

/// JVMs 4.6 Methods
pub enum MethodFlag {
    /// 0x0001
    Public,
    /// 0x0002
    Private,
    /// 0x0004
    Protected,
    /// 0x0008
    Static,
    /// 0x0010
    Final,
    /// 0x0020
    Synchronized,
    /// 0x0040, a bridge method generated by compiler
    Bridge,
    /// 0x0080
    VarArgs,
    /// 0x0100
    Native,
    /// 0x0400
    Abstract,
    /// 0x0800, useless for our project
    Strict,
    /// 0x1000
    Synthetic,
}

const METHOD_ACC_PUBLIC: u16 = 0x0001;
const METHOD_ACC_PRIVATE: u16 = 0x0002;
const METHOD_ACC_PROTECTED: u16 = 0x0004;
const METHOD_ACC_STATIC: u16 = 0x0008;
const METHOD_ACC_FINAL: u16 = 0x0010;
const METHOD_ACC_SYNCHRONIZED: u16 = 0x0020;
const METHOD_ACC_BRIDGE: u16 = 0x0040;
const METHOD_ACC_VARARGS: u16 = 0x0080;
const METHOD_ACC_NATIVE: u16 = 0x0100;
const METHOD_ACC_ABSTRACT: u16 = 0x0400;
const METHOD_ACC_STRICT: u16 = 0x0800;
const METHOD_ACC_SYNTHETIC: u16 = 0x1000;

impl TryFrom<u16> for MethodFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            METHOD_ACC_PUBLIC => Ok(MethodFlag::Public),
            METHOD_ACC_PRIVATE => Ok(MethodFlag::Private),
            METHOD_ACC_PROTECTED => Ok(MethodFlag::Protected),
            METHOD_ACC_STATIC => Ok(MethodFlag::Static),
            METHOD_ACC_FINAL => Ok(MethodFlag::Final),
            METHOD_ACC_SYNCHRONIZED => Ok(MethodFlag::Synchronized),
            METHOD_ACC_BRIDGE => Ok(MethodFlag::Bridge),
            METHOD_ACC_VARARGS => Ok(MethodFlag::VarArgs),
            METHOD_ACC_NATIVE => Ok(MethodFlag::Native),
            METHOD_ACC_ABSTRACT => Ok(MethodFlag::Abstract),
            METHOD_ACC_STRICT => Ok(MethodFlag::Strict),
            METHOD_ACC_SYNTHETIC => Ok(MethodFlag::Synthetic),
            _ => Err("Invalid value for MethodFlag"),
        }
    }
}

impl From<MethodFlag> for u16 {
    fn from(value: MethodFlag) -> Self {
        match value {
            MethodFlag::Public => METHOD_ACC_PUBLIC,
            MethodFlag::Private => METHOD_ACC_PRIVATE,
            MethodFlag::Protected => METHOD_ACC_PROTECTED,
            MethodFlag::Static => METHOD_ACC_STATIC,
            MethodFlag::Final => METHOD_ACC_FINAL,
            MethodFlag::Synchronized => METHOD_ACC_SYNCHRONIZED,
            MethodFlag::Bridge => METHOD_ACC_BRIDGE,
            MethodFlag::VarArgs => METHOD_ACC_VARARGS,
            MethodFlag::Native => METHOD_ACC_NATIVE,
            MethodFlag::Abstract => METHOD_ACC_ABSTRACT,
            MethodFlag::Strict => METHOD_ACC_STRICT,
            MethodFlag::Synthetic => METHOD_ACC_SYNTHETIC,
        }
    }
}

#[derive(Clone, Copy)]
pub struct MethodFlags(pub u16);

impl From<u16> for MethodFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl MethodFlags {
    pub fn is(&self, flag: MethodFlag) -> bool {
        (self.0 & u16::from(flag)) != 0
    }

    pub fn set(&mut self, flag: MethodFlag) {
        self.0 |= u16::from(flag);
    }
}

impl fmt::Display for MethodFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is(MethodFlag::Public) {
            write!(f, "public")?;
        } else if self.is(MethodFlag::Private) {
            write!(f, "private")?;
        } else if self.is(MethodFlag::Protected) {
            write!(f, "protected")?;
        } else {
            unreachable!();
        }

        if self.is(MethodFlag::Static) {
            write!(f, " static")?;
        }

        if self.is(MethodFlag::Final) {
            write!(f, " final")?;
        }

        if self.is(MethodFlag::Synchronized) {
            write!(f, " sync")?;
        }

        if self.is(MethodFlag::Bridge) {
            write!(f, " bridge")?;
        }

        if self.is(MethodFlag::VarArgs) {
            write!(f, " varargs")?;
        }

        if self.is(MethodFlag::Native) {
            write!(f, " native")?;
        }

        if self.is(MethodFlag::Abstract) {
            write!(f, " abstract")?;
        }

        if self.is(MethodFlag::Strict) {
            write!(f, " strict")?;
        }

        if self.is(MethodFlag::Synthetic) {
            write!(f, " synthetic")?;
        }

        Ok(())
    }
}