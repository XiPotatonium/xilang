use std::convert::{self, TryFrom};

const GENERIC_PARAM_ATTRIB_VARIANCE_MASK: u16 = 0x0003;
const GENERIC_PARAM_ATTRIB_VARIANCE_NONE_FLAG: u16 = 0x0000;
const GENERIC_PARAM_ATTRIB_VARIANCE_COVARIANT_FLAG: u16 = 0x0001;
const GENERIC_PARAM_ATTRIB_VARIANCE_CONTRAVARIANT_FLAG: u16 = 0x0002;

/// II.23.1.7
#[derive(Clone, Copy)]
pub struct GenericParamAttrib {
    pub attirb: u16,
}

pub enum GenericParamVariance {
    None,
    Covariant,
    Contravariant,
}

impl From<GenericParamVariance> for u16 {
    fn from(value: GenericParamVariance) -> Self {
        match value {
            GenericParamVariance::None => GENERIC_PARAM_ATTRIB_VARIANCE_NONE_FLAG,
            GenericParamVariance::Covariant => GENERIC_PARAM_ATTRIB_VARIANCE_COVARIANT_FLAG,
            GenericParamVariance::Contravariant => GENERIC_PARAM_ATTRIB_VARIANCE_CONTRAVARIANT_FLAG,
        }
    }
}

impl convert::TryFrom<u16> for GenericParamVariance {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            GENERIC_PARAM_ATTRIB_VARIANCE_NONE_FLAG => Ok(Self::None),
            GENERIC_PARAM_ATTRIB_VARIANCE_COVARIANT_FLAG => Ok(Self::Covariant),
            GENERIC_PARAM_ATTRIB_VARIANCE_CONTRAVARIANT_FLAG => Ok(Self::Contravariant),
            _ => Err("Invalid value for GenericParamVariance"),
        }
    }
}

impl Default for GenericParamAttrib {
    fn default() -> Self {
        Self { attirb: 0 }
    }
}

impl GenericParamAttrib {
    pub fn set_variance(&mut self, variance: GenericParamVariance) {
        self.attirb = (self.attirb & !GENERIC_PARAM_ATTRIB_VARIANCE_MASK) | u16::from(variance);
    }

    pub fn get_variance(&self) -> GenericParamVariance {
        GenericParamVariance::try_from(self.attirb & GENERIC_PARAM_ATTRIB_VARIANCE_MASK).unwrap()
    }
}
