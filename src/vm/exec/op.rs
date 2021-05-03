use super::super::stack::{ActivationRecord, SlotTag};

macro_rules! exec_numeric_op {
    ($op: tt, $lhs: ident, $rhs: ident) => {
        unsafe {
            match $lhs.tag {
                SlotTag::I32 => match $rhs.tag {
                    SlotTag::I32 => {
                        $lhs.data.i32_ = $lhs.data.i32_ $op $rhs.data.i32_;
                    }
                    SlotTag::I64 => panic!("Cannot add between i32 and i64"),
                    SlotTag::INative => {
                        $lhs.data.inative_ = $lhs.data.i32_ as isize $op $rhs.data.inative_;
                        $lhs.tag = SlotTag::INative;
                    }
                    SlotTag::F32 => panic!("Cannot add between float and int"),
                    SlotTag::F64 => panic!("Cannot add between float and int"),
                    SlotTag::Ref => panic!("Cannot add ref"),
                    SlotTag::Value => panic!("Cannot cmp value"),
                    SlotTag::Uninit => unreachable!(),
                },
                SlotTag::I64 => unimplemented!(),
                SlotTag::INative => unimplemented!(),
                SlotTag::F32 => unimplemented!(),
                SlotTag::F64 => unimplemented!(),
                SlotTag::Ref => panic!("Cannot add ref"),
                SlotTag::Value => panic!("Cannot add value"),
                SlotTag::Uninit => unreachable!(),
            }
        }
    };
}

macro_rules! exec_cmp_op {
    ($op: tt, $lhs: ident, $rhs: ident) => {
        unsafe {
            match $lhs.tag {
                SlotTag::I32 => match $rhs.tag {
                    SlotTag::I32 => {
                        $lhs.data.i32_ $op $rhs.data.i32_
                    }
                    SlotTag::I64 => panic!("Cannot cmp between i32 and i64"),
                    SlotTag::INative => {
                        ($lhs.data.i32_ as isize) $op $rhs.data.inative_
                    }
                    SlotTag::F32 => panic!("Cannot cmp between float and int"),
                    SlotTag::F64 => panic!("Cannot cmp between float and int"),
                    SlotTag::Ref => panic!("Cannot cmp ref"),
                    SlotTag::Value => panic!("Cannot cmp value"),
                    SlotTag::Uninit => unreachable!(),
                },
                SlotTag::I64 => unimplemented!(),
                SlotTag::INative => unimplemented!(),
                SlotTag::F32 => unimplemented!(),
                SlotTag::F64 => unimplemented!(),
                SlotTag::Ref => panic!("Cannot cmp ref"),
                SlotTag::Value => panic!("Cannot cmp value"),
                SlotTag::Uninit => unreachable!(),
            }
        }
    };
}

pub fn exec_beq(cur_state: &mut ActivationRecord) {
    let offset = cur_state.consume_i32();
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.pop_with_slot();
    let b = exec_cmp_op!(==, lhs, rhs);
    if b {
        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
    }
}

pub fn exec_bge(cur_state: &mut ActivationRecord) {
    let offset = cur_state.consume_i32();
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.pop_with_slot();
    let b = exec_cmp_op!(>=, lhs, rhs);
    if b {
        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
    }
}

pub fn exec_bgt(cur_state: &mut ActivationRecord) {
    let offset = cur_state.consume_i32();
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.pop_with_slot();
    let b = exec_cmp_op!(>, lhs, rhs);
    if b {
        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
    }
}

pub fn exec_ble(cur_state: &mut ActivationRecord) {
    let offset = cur_state.consume_i32();
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.pop_with_slot();
    let b = exec_cmp_op!(<=, lhs, rhs);
    if b {
        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
    }
}

pub fn exec_blt(cur_state: &mut ActivationRecord) {
    let offset = cur_state.consume_i32();
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.pop_with_slot();
    let b = exec_cmp_op!(<, lhs, rhs);
    if b {
        cur_state.ip = (cur_state.ip as i32 + offset) as usize;
    }
}

pub fn exec_ceq(cur_state: &mut ActivationRecord) {
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    let t = exec_cmp_op!(==, lhs, rhs);
    lhs.data.i32_ = if t { 1 } else { 0 };
    lhs.tag = SlotTag::I32
}

pub fn exec_cgt(cur_state: &mut ActivationRecord) {
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    let t = exec_cmp_op!(>, lhs, rhs);
    lhs.data.i32_ = if t { 1 } else { 0 };
    lhs.tag = SlotTag::I32;
}

pub fn exec_clt(cur_state: &mut ActivationRecord) {
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    let t = exec_cmp_op!(<, lhs, rhs);
    lhs.data.i32_ = if t { 1 } else { 0 };
    lhs.tag = SlotTag::I32;
}

pub fn exec_add(cur_state: &mut ActivationRecord) {
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    exec_numeric_op!(+, lhs, rhs);
}

pub fn exec_sub(cur_state: &mut ActivationRecord) {
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    exec_numeric_op!(-, lhs, rhs);
}

pub fn exec_mul(cur_state: &mut ActivationRecord) {
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    exec_numeric_op!(*, lhs, rhs);
}

pub fn exec_div(cur_state: &mut ActivationRecord) {
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    exec_numeric_op!(/, lhs, rhs);
}

pub fn exec_rem(cur_state: &mut ActivationRecord) {
    let rhs = cur_state.eval_stack.pop_with_slot();
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    exec_numeric_op!(%, lhs, rhs);
}

pub fn exec_neg(cur_state: &mut ActivationRecord) {
    let lhs = cur_state.eval_stack.peek_mut().unwrap();
    unsafe {
        match lhs.tag {
            SlotTag::I32 => {
                lhs.data.i32_ = -lhs.data.i32_;
            }
            SlotTag::I64 => {
                lhs.data.i64_ = -lhs.data.i64_;
            }
            SlotTag::INative => {
                lhs.data.inative_ = -lhs.data.inative_;
            }
            SlotTag::F32 => {
                lhs.data.f32_ = -lhs.data.f32_;
            }
            SlotTag::F64 => {
                lhs.data.f64_ = -lhs.data.f64_;
            }
            SlotTag::Value => unimplemented!("Neg value type is not implemented"),
            SlotTag::Ref => panic!("Cannot neg ref type"),
            SlotTag::Uninit => unreachable!(),
        }
    }
}
