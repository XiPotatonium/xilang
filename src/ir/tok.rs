use super::file::{IrFile, IrFmt};

use std::fmt;

pub const TOK_TAG_MASK: u32 = 0xFF;
pub const TOK_TAG_MASK_SIZE: u32 = 16;
pub const TOK_MOD_TAG: u32 = 0x00;
pub const TOK_MODREF_TAG: u32 = 0x1A;
pub const TOK_TYPEDEF_TAG: u32 = 0x02;
pub const TOK_TYPEREF_TAG: u32 = 0x01;
pub const TOK_FIELD_TAG: u32 = 0x04;
pub const TOK_METHOD_TAG: u32 = 0x06;
pub const TOK_MEMBERREF_TAG: u32 = 0x0A;
pub const TOK_IMPLMAP_TAG: u32 = 0x1C;

#[derive(Debug, PartialEq, Eq)]
pub enum TokTag {
    Mod = TOK_MOD_TAG as isize,
    ModRef = TOK_MODREF_TAG as isize,
    TypeDef = TOK_TYPEDEF_TAG as isize,
    TypeRef = TOK_TYPEREF_TAG as isize,
    Field = TOK_FIELD_TAG as isize,
    MethodDef = TOK_METHOD_TAG as isize,
    MemberRef = TOK_MEMBERREF_TAG as isize,
    ImplMap = TOK_IMPLMAP_TAG as isize,
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
            TOK_METHOD_TAG => TokTag::MethodDef,
            TOK_FIELD_TAG => TokTag::Field,
            TOK_MEMBERREF_TAG => TokTag::MemberRef,
            TOK_IMPLMAP_TAG => TokTag::ImplMap,
            _ => unreachable!(),
        },
        idx,
    )
}

pub fn to_tok(raw_idx: u32, tag: TokTag) -> u32 {
    (raw_idx << TOK_TAG_MASK_SIZE) | (tag as u32)
}

pub fn fmt_tok(tok: u32, f: &mut fmt::Formatter<'_>, ctx: &IrFile) -> fmt::Result {
    let (tok_tag, raw_idx) = get_tok_tag(tok);
    let idx = raw_idx as usize - 1;
    match tok_tag {
        TokTag::Mod => ctx.mod_tbl[idx].fmt(f, ctx),
        TokTag::ModRef => ctx.modref_tbl[idx].fmt(f, ctx),
        TokTag::TypeDef => ctx.typedef_tbl[idx].fmt(f, ctx),
        TokTag::TypeRef => ctx.typeref_tbl[idx].fmt(f, ctx),
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

                ctx.typedef_tbl[ty_idx - 1].fmt(f, ctx)?;
                write!(f, "::")?;
            }
            ctx.field_tbl[idx].fmt(f, ctx)
        }
        TokTag::MethodDef => {
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

                ctx.typedef_tbl[ty_idx - 1].fmt(f, ctx)?;
                write!(f, "::")?;
            }
            ctx.method_tbl[idx].fmt(f, ctx)
        }
        TokTag::MemberRef => ctx.memberref_tbl[idx].fmt(f, ctx),
        TokTag::ImplMap => unimplemented!(),
    }
}
