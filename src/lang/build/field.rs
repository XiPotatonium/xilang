use core::util::ItemPathBuf;
use std::ptr::NonNull;

use super::super::sym::{Field, RValType, TypeLinkContext};
use super::ClassBuilder;

pub struct FieldBuilder {
    pub sym: NonNull<Field>,
    pub ty_ast: Box<Type>,
}

impl FieldBuilder {
    pub fn load(path: ItemPathBuf, parent: &mut ClassBuilder, ast: Field) -> Box<Field> {
        let Field {
            name: _,
            flags,
            ty: ty_ast,
        } = ast;
        let method_sym = Box::new(Field {
            parent: parent.sym,
            path,
            flags,
            ty: RValType::UnInit,
        });

        parent.fields.push(Box::new(FieldBuilder {
            sym: NonNull::new(method_sym.as_ref() as *const Field as *mut Field).unwrap(),
            ty_ast,
        }));

        method_sym
    }

    pub fn link_type(&mut self, ctx: &TypeLinkContext) {
        let mut sym = unsafe { self.sym.as_mut() };
        sym.ty = ctx.resolve_rval_type(self.ty_ast.as_ref());
    }
}
