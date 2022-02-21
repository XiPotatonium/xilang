use core::flags::FuncFlag;
use core::util::ItemPathBuf;
use std::ptr::NonNull;

use super::super::ast::{ASTFunc, ASTType, AST};
use super::super::sym::{Func, Param, RValType, TypeLinkContext};
use super::super::XiCfg;

pub struct FuncBuilder {
    pub sym: NonNull<Func>,
    pub ret_ast: Box<ASTType>,
    pub ps_ast: Vec<Box<AST>>,
}

impl FuncBuilder {
    pub fn load(
        path: ItemPathBuf,
        builders: &mut Vec<Box<FuncBuilder>>,
        ast: ASTFunc,
    ) -> Box<Func> {
        let ASTFunc {
            name: _,
            flags,
            custom_attribs,
            ret,
            ps,
            body,
        } = ast;

        let mut flags = flags;
        for custom_attrib in custom_attribs.iter() {
            if let AST::CustomAttrib(attrib, _) = custom_attrib.as_ref() {
                if attrib == "internal" {
                    flags.set(FuncFlag::Bridge);
                } else {
                    unimplemented!("CustomAttrib {} is not implemented", attrib);
                }
            } else {
                unreachable!()
            }
        }

        let method_sym = Box::new(Func {
            path,
            ret: RValType::UnInit,
            ps: Vec::new(),
            flags,
            body,
        });

        builders.push(Box::new(FuncBuilder {
            sym: NonNull::new(method_sym.as_ref() as *const Func as *mut Func).unwrap(),
            ret_ast: ret,
            ps_ast: ps,
        }));

        method_sym
    }

    pub fn link_type(&mut self, ctx: &TypeLinkContext) {
        let mut func_sym = unsafe { self.sym.as_mut() };
        func_sym.ret = ctx.resolve_rval_type(self.ret_ast.as_ref());
        for p_ast in self.ps_ast.iter() {
            if let AST::Param(name, ty) = p_ast.as_ref() {
                func_sym.ps.push(Param {
                    name: name.clone(),
                    ty: ctx.resolve_rval_type(ty.as_ref()),
                })
            } else {
                unreachable!()
            }
        }
    }

    pub fn code_gen(&mut self, _: &XiCfg) {
        unimplemented!()
    }
}
