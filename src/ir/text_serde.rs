use super::attrib::*;
use super::file::IrFile;
use super::inst::Inst;
use super::member::MemberForwarded;
use super::param::Param;
use super::sig::{IrSig, MethodSigFlagTag, ParamType, RetType};
use super::ty::TypeDefOrRef;

use std::fmt;

pub trait IrFmt {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result;
}

impl IrFile {
    pub fn write_field(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        field_i: usize,
    ) -> fmt::Result {
        let field = &self.field_tbl[field_i];
        let flag = FieldAttrib::from(field.flag);
        write!(
            f,
            "\n{}.field {} {} ",
            " ".repeat(indent * 4),
            flag,
            self.get_str(field.name),
        )?;
        self.blob_heap[field.sig as usize].fmt(f, self)?;
        write!(f, "\n")
    }

    fn write_named_param(
        &self,
        f: &mut fmt::Formatter<'_>,
        p: Option<&Param>,
        ty: &ParamType,
    ) -> fmt::Result {
        if let Some(p) = p {
            let name = self.get_str(p.name);
            let flag = ParamAttrib::from(p.flag);

            if name.len() != 0 {
                write!(f, "{}: ", name)?;
            }

            if !flag.is(ParamAttribFlag::Default) {
                // default flag will not display
                write!(f, "{} ", flag)?;
            }
        }

        ty.fmt(f, self)
    }

    /// who would name a return value?
    fn write_named_ret(
        &self,
        f: &mut fmt::Formatter<'_>,
        p: Option<&Param>,
        ty: &RetType,
    ) -> fmt::Result {
        if let Some(p) = p {
            let name = self.get_str(p.name);
            let flag = ParamAttrib::from(p.flag);

            if name.len() != 0 {
                write!(f, "{}: ", name)?;
            }

            if !flag.is(ParamAttribFlag::Default) {
                // default flag will not display
                write!(f, "{} ", flag)?;
            }
        }

        ty.fmt(f, self)
    }

    pub fn write_method(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        method_i: usize,
        is_entrypoint: bool,
    ) -> fmt::Result {
        let method = &self.method_tbl[method_i];
        let param = if method_i == self.method_tbl.len() - 1 {
            // last method
            &self.param_tbl[(method.param_list as usize - 1)..]
        } else {
            &self.param_tbl[(method.param_list as usize - 1)
                ..(self.method_tbl[method_i + 1].param_list as usize - 1)]
        };
        let flag = MethodAttrib::from(method.flag);
        let impl_flag = MethodImplAttrib::from(method.impl_flag);

        write!(f, "\n{}.method {} ", " ".repeat(indent * 4), flag)?;

        if method.body == 0 {
            // this is an external method
            // See ECMA-335 II.15.5.2

            // O(N) search, might need optimization
            let implmap_info = self.implmap_tbl.iter().find(|&info| {
                let (m_tag, m_idx) = info.get_member();
                MemberForwarded::MethodDef == m_tag && m_idx as usize == method_i + 1
            });

            if let Some(implmap_info) = implmap_info {
                let flag = PInvokeAttrib::from(implmap_info.flag);
                write!(
                    f,
                    "pinvokeimpl(\"{}\" {}) ",
                    self.get_str(self.modref_tbl[implmap_info.scope as usize - 1].name),
                    flag
                )?;
            } else {
                panic!("No implmap found for method {}", self.get_str(method.name));
            }
        }

        write!(f, "{} ", self.get_str(method.name))?;
        let mut param_iter = param.iter().peekable();
        if let Some(&p) = param_iter.peek() {
            if p.sequence == 0 {
                // return
                param_iter.next();
            }
        }
        if let IrSig::Method(flag, ps, ret) = &self.blob_heap[method.sig as usize] {
            // similar to IrSig::fmt
            if flag.has_flag(MethodSigFlagTag::HasThis) {
                write!(f, "instance ")?;
            }
            write!(f, "(")?;
            for (i, p_ty) in ps.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                // pattern binding after @ ?
                if let Some(&p) = param_iter.peek() {
                    if p.sequence - 1 == i as u16 {
                        self.write_named_param(f, Some(p), p_ty)?;
                        param_iter.next();
                    } else {
                        self.write_named_param(f, None, p_ty)?;
                    }
                } else {
                    self.write_named_param(f, None, p_ty)?;
                }
            }
            write!(f, ") -> ")?;
            if let Some(p) = param.first() {
                if p.sequence == 0 {
                    self.write_named_ret(f, Some(p), ret)?;
                } else {
                    self.write_named_ret(f, None, ret)?;
                }
            } else {
                self.write_named_ret(f, None, ret)?;
            }
        } else {
            unreachable!();
        }
        // TODO: display param name and flag
        write!(f, " {}", impl_flag)?;

        if method.body != 0 {
            // has body
            let body = &self.codes[method.body as usize - 1];

            write!(
                f,
                " {{\n{}.maxstacks\t{}",
                " ".repeat(indent * 8),
                body.max_stack
            )?;
            if body.locals != 0 {
                write!(f, "\n{}.locals\t", " ".repeat(indent * 8))?;
                self.blob_heap[self.stand_alone_sig_tbl[body.locals as usize - 1].sig as usize]
                    .fmt(f, self)?;
            }
            if is_entrypoint {
                write!(f, "\n{}.entrypoint", " ".repeat(indent * 8))?;
            }

            let code: Vec<Inst> = body.to_insts();

            let mut offset = 0;
            for inst in code.iter() {
                write!(f, "\n{}", " ".repeat(indent * 8))?;
                inst.fmt(f, self, offset)?;
                offset += inst.size();
            }
            write!(f, "\n{}}}\n", " ".repeat(indent * 4))
        } else {
            write!(f, " {{}}\n")
        }
    }
}

impl fmt::Display for IrFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".version {}.{}\n.mod {}",
            self.major_version,
            self.minor_version,
            self.get_str(self.mod_tbl[0].name)
        )?;

        let entrypoint = self.mod_tbl[0].entrypoint;

        let (mut field_i, mut method_i) = if let Some(c0) = self.typedef_tbl.first() {
            (c0.fields as usize - 1, c0.methods as usize - 1)
        } else {
            (self.field_tbl.len(), self.method_tbl.len())
        };

        for i in (0..field_i).into_iter() {
            self.write_field(f, 0, i)?;
        }

        for i in (0..method_i).into_iter() {
            self.write_method(f, 0, i, i as u32 + 1 == entrypoint)?;
        }

        for (typedef_i, typedef) in self.typedef_tbl.iter().enumerate() {
            let (field_lim, method_lim) = if typedef_i + 1 >= self.typedef_tbl.len() {
                // last class
                (self.field_tbl.len(), self.method_tbl.len())
            } else {
                let next_typedef = &self.typedef_tbl[typedef_i + 1];
                (
                    next_typedef.fields as usize - 1,
                    next_typedef.methods as usize - 1,
                )
            };

            let flag = TypeAttrib::from(typedef.flag);
            write!(f, "\n\n\n.class {} {}", flag, self.get_str(typedef.name))?;

            if let Some((extends_idx_tag, extends_idx)) = typedef.get_extends() {
                write!(f, " extends ")?;
                match extends_idx_tag {
                    TypeDefOrRef::TypeDef => {
                        self.typedef_tbl[extends_idx].fullname(f, self)?;
                    }
                    TypeDefOrRef::TypeRef => {
                        self.typeref_tbl[extends_idx].fullname(f, self)?;
                    }
                    TypeDefOrRef::TypeSpec => unimplemented!(),
                }
            }
            write!(f, " {{ ")?;

            while field_i < field_lim {
                self.write_field(f, 1, field_i)?;
                field_i += 1;
            }

            while method_i < method_lim {
                self.write_method(f, 1, method_i, method_i as u32 + 1 == entrypoint)?;
                method_i += 1;
            }
            write!(f, "}}")?;
        }

        Ok(())
    }
}
