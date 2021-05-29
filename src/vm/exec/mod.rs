mod arr;
mod fld;
pub mod internal_calls;
mod op;

use super::data::{BuiltinType, MethodDesc, MethodILImpl, MethodImpl, MethodNativeImpl, REF_SIZE};
use super::heap::Heap;
use super::shared_mem::SharedMem;
use super::stack::{ActivationRecord, Args, EvalStack, ILocals, Locals, Slot, SlotTag};

use xir::attrib::MethodAttribFlag;
use xir::tok::{get_tok_tag, TokTag};

use std::ptr;

pub struct TExecutor<'m> {
    states: Vec<ActivationRecord<'m>>,
}

impl<'m> TExecutor<'m> {
    pub fn new(entry: *const MethodDesc) -> TExecutor<'m> {
        let mut ret = TExecutor { states: Vec::new() };
        // currently executor entry has no arguments
        let entry_ref = unsafe { entry.as_ref().unwrap() };
        ret.call(
            Args::new(entry_ref),
            ptr::null_mut(),
            entry_ref,
            entry_ref.method_impl.expect_il(),
        );
        ret
    }

    fn call(
        &mut self,
        args: Args<'m>,
        ret_addr: *mut Slot,
        method: &'m MethodDesc,
        il_impl: &'m MethodILImpl,
    ) {
        // Currently there is no verification of the arg type
        // TODO: Generate locals with type info set
        self.states.push(ActivationRecord {
            method,
            method_impl: il_impl,
            args,
            ret_addr,
            eval_stack: EvalStack::new(),
            locals: Locals::new(method),
            ip: 0,
        });
    }

    pub fn run(&mut self, mem: &'m mut SharedMem) -> isize {
        loop {
            let code = self.states.last_mut().unwrap().consume_u8();
            match code {
                // nop
                0x00 => {}
                // ldarg0
                0x02 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.args.load(0, &mut cur_state.eval_stack);
                }
                // ldarg1
                0x03 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.args.load(1, &mut cur_state.eval_stack);
                }
                // ldarg2
                0x04 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.args.load(2, &mut cur_state.eval_stack);
                }
                // ldarg3
                0x05 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.args.load(3, &mut cur_state.eval_stack);
                }
                // ldloc0
                0x06 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals.load(0, &mut cur_state.eval_stack);
                }
                // ldloc1
                0x07 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals.load(1, &mut cur_state.eval_stack);
                }
                // ldloc2
                0x08 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals.load(2, &mut cur_state.eval_stack);
                }
                // ldloc3
                0x09 => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals.load(3, &mut cur_state.eval_stack);
                }
                // stloc0
                0x0A => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals.store(0, &mut cur_state.eval_stack);
                }
                // stloc1
                0x0B => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals.store(1, &mut cur_state.eval_stack);
                }
                // stloc2
                0x0C => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals.store(2, &mut cur_state.eval_stack);
                }
                // stloc3
                0x0D => {
                    let cur_state = self.states.last_mut().unwrap();
                    cur_state.locals.store(3, &mut cur_state.eval_stack);
                }
                // ldarg.s
                0x0E => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state.args.load(idx as usize, &mut cur_state.eval_stack);
                }
                // ldarga.s
                0x0F => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state
                        .args
                        .loada(idx as usize, &mut cur_state.eval_stack);
                }
                // starg.s
                0x10 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state
                        .args
                        .store(idx as usize, &mut cur_state.eval_stack);
                }
                // ldloc.s
                0x11 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state
                        .locals
                        .load(idx as usize, &mut cur_state.eval_stack);
                }
                // ldloca.s
                0x12 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state
                        .locals
                        .loada(idx as usize, &mut cur_state.eval_stack);
                }
                // stloc.s
                0x13 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let idx = cur_state.consume_u8();
                    cur_state
                        .locals
                        .store(idx as usize, &mut cur_state.eval_stack);
                }
                // ldnull
                0x14 => {
                    self.states
                        .last_mut()
                        .unwrap()
                        .eval_stack
                        .push_slot(Slot::null());
                }
                // ldc.i4.m1
                0x15 => self.states.last_mut().unwrap().eval_stack.push_i32(-1),
                // ldc.i4.0
                0x16 => self.states.last_mut().unwrap().eval_stack.push_i32(0),
                // ldc.i4.1
                0x17 => self.states.last_mut().unwrap().eval_stack.push_i32(1),
                // ldc.i4.2
                0x18 => self.states.last_mut().unwrap().eval_stack.push_i32(2),
                // ldc.i4.3
                0x19 => self.states.last_mut().unwrap().eval_stack.push_i32(3),
                // ldc.i4.4
                0x1A => self.states.last_mut().unwrap().eval_stack.push_i32(4),
                // ldc.i4.5
                0x1B => self.states.last_mut().unwrap().eval_stack.push_i32(5),
                // ldc.i4.6
                0x1C => self.states.last_mut().unwrap().eval_stack.push_i32(6),
                // ldc.i4.7
                0x1D => self.states.last_mut().unwrap().eval_stack.push_i32(7),
                // ldc.i4.8
                0x1E => self.states.last_mut().unwrap().eval_stack.push_i32(8),
                // ldc.i4.s
                0x1F => {
                    let cur_state = self.states.last_mut().unwrap();
                    let v = cur_state.consume_i8();
                    cur_state.eval_stack.push_i32(v as i32);
                }
                // ldc.i4
                0x20 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let v = cur_state.consume_i32();
                    cur_state.eval_stack.push_i32(v);
                }
                // dup
                0x25 => self.states.last_mut().unwrap().eval_stack.dup(),
                // pop
                0x26 => self.states.last_mut().unwrap().eval_stack.pop(),
                // call
                0x28 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let ctx = unsafe { cur_state.method.ctx.as_ref().expect_il() };

                    let (tag, idx) = get_tok_tag(tok);

                    let callee = match tag {
                        TokTag::MethodDef => ctx.methods[idx as usize - 1].as_ref(),
                        TokTag::MemberRef => unsafe {
                            ctx.memberref[idx as usize - 1].expect_method().as_ref()
                        },
                        _ => unimplemented!(),
                    };

                    let mut args = Args::new(callee);
                    args.fill_args(&mut cur_state.eval_stack);
                    let ret_addr = cur_state.eval_stack.alloc_ret(&callee.ret.ty);
                    match &callee.method_impl {
                        MethodImpl::IL(il_impl) => {
                            self.call(args, ret_addr, callee, il_impl);
                        }
                        MethodImpl::Native(MethodNativeImpl { scope, .. }) => {
                            // currently there is no multi-slot user defined type
                            unsafe {
                                let callee_ctx = callee.ctx.as_ref().expect_il();
                                callee_ctx.modrefs[*scope].as_ref().expect_dll().call(
                                    &mem.str_pool[callee.name],
                                    args,
                                    ret_addr,
                                );
                            }
                        }
                        MethodImpl::Runtime(runtime_impl) => {
                            runtime_impl.func.call(args, ret_addr, mem);
                        }
                    }
                }
                // ret
                0x2A => {
                    let cur_state = self.states.last_mut().unwrap();
                    match cur_state.method.ret.ty {
                        BuiltinType::Void => {
                            self.states.pop();
                            if self.states.is_empty() {
                                return 0;
                            }
                        }
                        BuiltinType::Bool
                        | BuiltinType::Char
                        | BuiltinType::U1
                        | BuiltinType::I1
                        | BuiltinType::U4
                        | BuiltinType::I4
                        | BuiltinType::U8
                        | BuiltinType::I8
                        | BuiltinType::UNative
                        | BuiltinType::INative
                        | BuiltinType::R4
                        | BuiltinType::R8
                        | BuiltinType::String
                        | BuiltinType::ByRef(_)
                        | BuiltinType::SZArray(_) => {
                            let ret_v = cur_state.eval_stack.pop_with_slot();
                            let state = self.states.pop().unwrap();
                            if self.states.is_empty() {
                                return unsafe { ret_v.data.inative_ };
                            }
                            unsafe {
                                *state.ret_addr = ret_v;
                            }
                        }
                        BuiltinType::Class(ty) => {
                            if unsafe { ty.as_ref().ee_class.is_value } {
                                unimplemented!()
                            } else {
                                let ret_v = cur_state.eval_stack.pop_with_slot();
                                let state = self.states.pop().unwrap();
                                if self.states.is_empty() {
                                    return unsafe { ret_v.data.inative_ };
                                }
                                unsafe {
                                    *state.ret_addr = ret_v;
                                }
                            }
                        }
                        BuiltinType::Unk => unreachable!(),
                    }
                }
                // br
                0x38 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                }
                // brfalse
                0x39 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let v = cur_state.eval_stack.pop_with_slot();
                    unsafe {
                        v.expect(SlotTag::I32);
                        if v.data.i32_ == 0 {
                            // false
                            cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                        }
                    }
                }
                // brtrue
                0x3A => {
                    let cur_state = self.states.last_mut().unwrap();
                    let offset = cur_state.consume_i32();
                    let v = cur_state.eval_stack.pop_with_slot();
                    unsafe {
                        v.expect(SlotTag::I32);
                        if v.data.i32_ != 0 {
                            // true
                            cur_state.ip = (cur_state.ip as i32 + offset) as usize;
                        }
                    }
                }
                0x3B => op::exec_beq(self.states.last_mut().unwrap()),
                0x3C => op::exec_bge(self.states.last_mut().unwrap()),
                0x3D => op::exec_bgt(self.states.last_mut().unwrap()),
                0x3E => op::exec_ble(self.states.last_mut().unwrap()),
                0x3F => op::exec_blt(self.states.last_mut().unwrap()),
                0x58 => op::exec_add(self.states.last_mut().unwrap()),
                0x59 => op::exec_sub(self.states.last_mut().unwrap()),
                0x5A => op::exec_mul(self.states.last_mut().unwrap()),
                0x5B => op::exec_div(self.states.last_mut().unwrap()),
                0x5D => op::exec_rem(self.states.last_mut().unwrap()),
                0x65 => op::exec_neg(self.states.last_mut().unwrap()),
                // callvirt
                0x6F => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let ctx = unsafe { cur_state.method.ctx.as_ref().expect_il() };

                    let (tag, idx) = get_tok_tag(tok);

                    let mut callee = match tag {
                        TokTag::MethodDef => ctx.methods[idx as usize - 1].as_ref(),
                        TokTag::MemberRef => unsafe {
                            ctx.memberref[idx as usize - 1].expect_method().as_ref()
                        },
                        _ => unimplemented!(),
                    };

                    let mut args = Args::new(callee);
                    args.fill_args(&mut cur_state.eval_stack);
                    if let Some(self_ptr) = args.get_self() {
                        assert!(!self_ptr.is_null());

                        // If calle is virtual, use dynamic dispatching
                        if callee.attrib.is(MethodAttribFlag::Virtual) {
                            let callee_ptr = Heap::get_vtbl_ptr(self_ptr);
                            callee =
                                unsafe { callee_ptr.as_ref().unwrap().vtbl[callee.slot].as_ref() };
                        }
                    }
                    let ret_addr = cur_state.eval_stack.alloc_ret(&callee.ret.ty);

                    match &callee.method_impl {
                        MethodImpl::IL(il_impl) => {
                            self.call(args, ret_addr, callee, il_impl);
                        }
                        MethodImpl::Native(_) => panic!(),
                        MethodImpl::Runtime(runtime_impl) => {
                            runtime_impl.func.call(args, ret_addr, mem);
                        }
                    }
                }
                // cpobj
                0x70 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let ctx = unsafe { cur_state.method.ctx.as_ref().expect_il() };

                    let (tag, idx) = get_tok_tag(tok);

                    let ty = match tag {
                        TokTag::TypeDef => ctx.types[idx as usize - 1].as_ref(),
                        TokTag::TypeRef => unsafe { ctx.typerefs[idx as usize - 1].as_ref() },
                        TokTag::TypeSpec => unimplemented!(),
                        _ => unimplemented!(),
                    };

                    let src = unsafe { cur_state.eval_stack.pop_with_slot().expect_ptr() };
                    let dest = unsafe { cur_state.eval_stack.pop_with_slot().expect_ptr() };

                    unsafe {
                        // copy value if ty is value type, else copy ref
                        ptr::copy(
                            src,
                            dest,
                            if ty.ee_class.is_value {
                                ty.basic_instance_size
                            } else {
                                REF_SIZE
                            },
                        );
                    }
                }
                // ldstr
                0x72 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let literal_idx = cur_state.consume_u32() as usize;
                    let ctx = unsafe { cur_state.method.ctx.as_ref().expect_il() };

                    let str_ptr = unsafe { mem.new_str_from_str(ctx.usr_str_heap[literal_idx]) };
                    cur_state.eval_stack.push_ptr(str_ptr);
                }
                // newobj
                0x73 => {
                    let cur_state = self.states.last_mut().unwrap();
                    let tok = cur_state.consume_u32();
                    let ctx = unsafe { cur_state.method.ctx.as_ref().expect_il() };

                    let (tag, idx) = get_tok_tag(tok);

                    let callee = match tag {
                        TokTag::MethodDef => ctx.methods[idx as usize - 1].as_ref(),
                        TokTag::MemberRef => unsafe {
                            ctx.memberref[idx as usize - 1].expect_method().as_ref()
                        },
                        _ => unimplemented!(),
                    };
                    // TODO: make sure callee is .ctor

                    if callee.is_static() {
                        panic!(".ctor should be an instance method");
                    }
                    // TODO: more strict check

                    // Alloc space at heap
                    let mut args = Args::new(callee);
                    unsafe {
                        assert!(!callee.parent.is_null());
                        let instance_addr = mem.new_obj(callee.parent);
                        args.store_slot(0, Slot::new_ref(instance_addr));
                        args.fill_args_except_self(&mut cur_state.eval_stack);
                        cur_state.eval_stack.push_slot(Slot::new_ref(instance_addr));
                    }

                    self.call(
                        args,
                        ptr::null_mut(),
                        callee,
                        callee.method_impl.expect_il(),
                    );
                }
                0x7B => fld::exec_ldfld(self.states.last_mut().unwrap()),
                0x7C => fld::exec_ldflda(self.states.last_mut().unwrap()),
                0x7D => fld::exec_stfld(self.states.last_mut().unwrap()),
                0x7E => fld::exec_ldsfld(self.states.last_mut().unwrap()),
                0x7F => fld::exec_ldsflda(self.states.last_mut().unwrap()),
                0x80 => fld::exec_stsfld(self.states.last_mut().unwrap()),
                0x8D => arr::exec_newarr(self.states.last_mut().unwrap(), mem),
                0x8E => arr::exec_ldlen(self.states.last_mut().unwrap()),
                // ldelema
                0x8F => arr::exec_ldelema(self.states.last_mut().unwrap()),
                // ldelem.i4
                0x94 => unimplemented!(),
                // ldelem.ref
                0x9A => arr::exec_ldelem_ref(self.states.last_mut().unwrap()),
                // stelem.i4
                0x9E => unimplemented!(),
                // stelem.ref
                0xA2 => arr::exec_stelem_ref(self.states.last_mut().unwrap()),
                // ldelem
                0xA3 => unimplemented!(),
                // stelem
                0xA4 => unimplemented!(),

                0xFE => {
                    let inner_code = self.states.last_mut().unwrap().consume_u8();
                    match inner_code {
                        0x01 => op::exec_ceq(self.states.last_mut().unwrap()),
                        0x02 => op::exec_cgt(self.states.last_mut().unwrap()),
                        0x04 => op::exec_clt(self.states.last_mut().unwrap()),
                        // ldloc
                        0x0C => {
                            let cur_state = self.states.last_mut().unwrap();
                            let idx = cur_state.consume_u16();
                            cur_state
                                .locals
                                .load(idx as usize, &mut cur_state.eval_stack);
                        }
                        // ldloca
                        0x0D => {
                            let cur_state = self.states.last_mut().unwrap();
                            let idx = cur_state.consume_u16();
                            cur_state
                                .locals
                                .loada(idx as usize, &mut cur_state.eval_stack);
                        }
                        // stloc
                        0x0E => {
                            let cur_state = self.states.last_mut().unwrap();
                            let idx = cur_state.consume_u16();
                            cur_state
                                .locals
                                .store(idx as usize, &mut cur_state.eval_stack);
                        }
                        // initobj
                        0x15 => {
                            let cur_state = self.states.last_mut().unwrap();
                            let tok = cur_state.consume_u32();
                            let ctx = unsafe { cur_state.method.ctx.as_ref().expect_il() };

                            let (tag, idx) = get_tok_tag(tok);

                            let ty = match tag {
                                TokTag::TypeDef => ctx.types[idx as usize - 1].as_ref(),
                                TokTag::TypeRef => unsafe {
                                    ctx.typerefs[idx as usize - 1].as_ref()
                                },
                                TokTag::TypeSpec => unimplemented!(),
                                _ => unimplemented!(),
                            };
                            let dest = unsafe { cur_state.eval_stack.pop_with_slot().expect_ptr() };
                            unsafe {
                                // init value with all 0 if ty is value type else ldnull followed by stind.ref
                                ptr::write_bytes(
                                    dest,
                                    0,
                                    if ty.ee_class.is_value {
                                        ty.basic_instance_size
                                    } else {
                                        REF_SIZE
                                    },
                                )
                            }
                        }
                        _ => panic!("Unknown inst 0xFE{:X}", inner_code),
                    }
                }

                _ => panic!("Unknown inst: 0x{:X}", code),
            }
        }
    }
}
