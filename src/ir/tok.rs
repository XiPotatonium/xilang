use super::file::IrFile;
use super::text_serde::IrFmt;

use std::fmt;

const TOK_TAG_MASK: u32 = 0xFF;
const TOK_TAG_MASK_SIZE: u32 = 16;
const TOK_MOD_TAG: u32 = 0x00;
const TOK_MODREF_TAG: u32 = 0x1A;
const TOK_TYPEDEF_TAG: u32 = 0x02;
const TOK_TYPEREF_TAG: u32 = 0x01;
const TOK_FIELD_TAG: u32 = 0x04;
const TOK_METHODDEF_TAG: u32 = 0x06;
const TOK_PARAM_TAG: u32 = 0x08;
const TOK_STANDALONESIG_TAG: u32 = 0x11;
const TOK_MEMBERREF_TAG: u32 = 0x0A;
const TOK_IMPLMAP_TAG: u32 = 0x1C;

#[derive(Debug, PartialEq, Eq)]
pub enum TokTag {
    Mod,
    ModRef,
    TypeDef,
    TypeRef,
    Field,
    MethodDef,
    MemberRef,
    Param,
    StandAloneSig,
    ImplMap,
}

impl From<TokTag> for u32 {
    fn from(value: TokTag) -> Self {
        match value {
            TokTag::Mod => TOK_MOD_TAG,
            TokTag::ModRef => TOK_MODREF_TAG,
            TokTag::TypeDef => TOK_TYPEDEF_TAG,
            TokTag::TypeRef => TOK_TYPEREF_TAG,
            TokTag::Field => TOK_FIELD_TAG,
            TokTag::MethodDef => TOK_METHODDEF_TAG,
            TokTag::MemberRef => TOK_MEMBERREF_TAG,
            TokTag::Param => TOK_PARAM_TAG,
            TokTag::StandAloneSig => TOK_STANDALONESIG_TAG,
            TokTag::ImplMap => TOK_IMPLMAP_TAG,
        }
    }
}

pub fn get_tok_tag(tok: u32) -> (TokTag, u32) {
    let tag = tok & TOK_TAG_MASK;
    let idx = tok >> TOK_TAG_MASK_SIZE;
    if idx == 0 {
        panic!();
    }

    (
        match tag {
            TOK_MOD_TAG => TokTag::Mod,
            TOK_MODREF_TAG => TokTag::ModRef,
            TOK_TYPEDEF_TAG => TokTag::TypeDef,
            TOK_TYPEREF_TAG => TokTag::TypeRef,
            TOK_METHODDEF_TAG => TokTag::MethodDef,
            TOK_FIELD_TAG => TokTag::Field,
            TOK_MEMBERREF_TAG => TokTag::MemberRef,
            TOK_PARAM_TAG => TokTag::Param,
            TOK_STANDALONESIG_TAG => TokTag::StandAloneSig,
            TOK_IMPLMAP_TAG => TokTag::ImplMap,
            _ => unreachable!(),
        },
        idx,
    )
}

pub fn to_tok(raw_idx: u32, tag: TokTag) -> u32 {
    (raw_idx << TOK_TAG_MASK_SIZE) | u32::from(tag)
}

pub fn fmt_tok(tok: u32, f: &mut fmt::Formatter<'_>, ctx: &IrFile) -> fmt::Result {
    let (tok_tag, raw_idx) = get_tok_tag(tok);
    let idx = raw_idx as usize - 1;
    match tok_tag {
        TokTag::Mod => ctx.mod_tbl[idx].fmt(f, ctx),
        TokTag::ModRef => ctx.modref_tbl[idx].fmt(f, ctx),
        TokTag::TypeDef => ctx.typedef_tbl[idx].fullname(f, ctx),
        TokTag::TypeRef => ctx.typeref_tbl[idx].fullname(f, ctx),
        TokTag::Field => {
            if ctx.typedef_tbl.len() == 0 || raw_idx < ctx.typedef_tbl[0].fields {
                // field has no parent
                write!(f, "{}::", ctx.mod_name())?;
            } else {
                let mut ty_idx = 0;
                while ty_idx < ctx.typedef_tbl.len() {
                    if ctx.typedef_tbl[ty_idx].fields > raw_idx {
                        break;
                    }
                    ty_idx += 1;
                }

                ctx.typedef_tbl[ty_idx - 1].fullname(f, ctx)?;
                write!(f, "::")?;
            }
            ctx.field_tbl[idx].fmt(f, ctx)
        }
        TokTag::MethodDef => {
            let method = &ctx.method_tbl[idx];

            if ctx.typedef_tbl.len() == 0 || raw_idx < ctx.typedef_tbl[0].methods {
                // method has no parent
                write!(f, "{}::", ctx.mod_name())?;
            } else {
                let mut ty_idx = 0;
                while ty_idx < ctx.typedef_tbl.len() {
                    if ctx.typedef_tbl[ty_idx].methods > raw_idx {
                        break;
                    }
                    ty_idx += 1;
                }

                ctx.typedef_tbl[ty_idx - 1].fullname(f, ctx)?;
                write!(f, "::")?;
            }
            method.fmt(f, ctx)
        }
        TokTag::MemberRef => ctx.memberref_tbl[idx].fmt(f, ctx),
        TokTag::Param => unimplemented!(),
        TokTag::StandAloneSig => unimplemented!(),
        TokTag::ImplMap => unimplemented!(),
    }
}
