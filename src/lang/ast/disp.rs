use super::ast::{Op, Type, AST};

use std::fmt;

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "({}, {})", self.x, self.y)
        match self {
            Type::Bool => write!(f, "bool"),
            Type::I32 => write!(f, "i32"),
            Type::F64 => write!(f, "f64"),
            Type::Tuple(v) => {
                let mut iter = v.iter();
                let mut s = String::from("(");
                if v.len() != 0 {
                    s.push_str(&format!("{}", iter.next().unwrap()));
                    for ty in iter {
                        s.push_str(&format!(",{}", ty));
                    }
                }
                s.push(')');
                write!(f, "{}", s)
            }
            Type::Arr(dtype, dim) => write!(f, "[{};{}]", dtype, dim),
            Type::Class(names) => {
                let mut iter = names.iter();
                let mut s = iter.next().unwrap().clone();
                for name in iter {
                    s.push('.');
                    s.push_str(name);
                }
                write!(f, "{}", s)
            }
            Type::Unk => write!(f, "unk"),
        }
    }
}

// Restore escape chars
fn restore_escape(s: &str) -> String {
    let mut ret = String::new();

    let mut char_it = s.chars();
    loop {
        match char_it.next() {
            Some(ch) => match ch {
                '\n' => ret.push_str("\\n"),
                _ => ret.push(ch),
            },
            None => break,
        }
    }

    ret
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "({}, {})", self.x, self.y)
        match self {
            Self::File(children) => write!(f, "{}", ASTChildrenWrapper(children)),
            Self::Class(id, funcs, fields) => write!(
                f,
                "{{\"name\":\"(class){}\",\"fields\":{},\"funcs\":{}}}",
                id,
                ASTChildrenWrapper(fields),
                ASTChildrenWrapper(funcs)
            ),
            Self::Func(id, ty, ps, body) => write!(
                f,
                "{{\"name\":\"(func){}\",\"type\":\"{}\",\"ps\":{},\"body\":{}}}",
                id,
                ty,
                ASTChildrenWrapper(ps),
                body.as_ref()
            ),
            Self::Field(id, ty, flag) => write!(
                f,
                "{{\"name\":\"(field){}\",\"flag\": \"{}\", \"type\":\"{}\"}}",
                id, flag, ty
            ),
            Self::Param(id, ty, flag) => write!(
                f,
                "{{\"name\":\"(field){}\",\"flag\": \"{}\", \"type\":\"{}\"}}",
                id, flag, ty
            ),
            Self::Var(pattern, ty, flag, init) => write!(
                f,
                "{{\"name\":\"(var)\", \"id\":{},\"flag\": \"{}\", \"type\":\"{}\",\"init\":{}}}",
                pattern, flag, ty, *init
            ),
            Self::Block(is_expr, children) => write!(
                f,
                "{{\"name\":\"(block)\", \"is_expr\": \"{}\",\"children\":{}}}",
                is_expr,
                ASTChildrenWrapper(children)
            ),
            Self::If(is_expr, cond, then, els) => write!(
                f,
                "{{\"name\":\"(if)\", \"is_expr\": \"{}\",\"cond\":{},\"then\":{},\"els\":{}}}",
                is_expr,
                cond.as_ref(),
                then.as_ref(),
                els.as_ref()
            ),
            Self::Continue => write!(f, "{{\"name\":\"continue\"}}"),
            Self::Break(val) => write!(f, "{{\"name\":\"break\", \"val\": {}}}", *val),
            Self::Return(val) => write!(f, "{{\"name\":\"return\",\"val\":{}}}", *val),
            Self::Loop(is_expr, body) => write!(
                f,
                "{{\"name\":\"(loop)\", \"is_expr\": \"{}\",\"body\":{}}}",
                is_expr,
                body.as_ref()
            ),
            Self::ExprStmt(expr) => write!(f, "{}", expr.as_ref()),
            Self::Unary(op, expr1) => {
                write!(f, "{{\"name\":\"{}\",\"operands\":[{}]}}", op, *expr1)
            }
            Self::Binary(op, expr1, expr2) => write!(
                f,
                "{{\"name\":\"{}\",\"operands\":[{},{}]}}",
                op,
                expr1.as_ref(),
                expr2.as_ref()
            ),
            Self::Cast(ty, expr) => write!(
                f, "{{\"name\": \"(cast)\", \"ty\": \"{}\", \"val\": {}}}",
                ty, expr
            ),
            Self::Call(func, ps) => write!(
                f,
                "{{\"name\":\"(call)\",\"func\":{},\"args\":{}}}",
                func.as_ref(),
                ASTChildrenWrapper(ps)
            ),
            Self::New(ty) => write!(f, "{{\"name\":\"(new)\",\"type\":\"{}\"}}", *ty),
            Self::Id(id) => write!(f, "{{\"name\":\"(id){}\"}}", id),
            Self::Null => write!(f, "{{\"name\":\"null\" }}"),
            Self::Bool(val) => write!(f, "{{\"name\":\"(bool){}\"}}", val),
            Self::Int(val) => write!(f, "{{\"name\":\"(int){}\"}}", val),
            Self::Float(val) => write!(f, "{{\"name\":\"(float){}\"}}", val),
            // TODO: escape chars
            Self::String(val) => write!(f, "{{\"name\":\"(string){}\"}}", restore_escape(val)),
            Self::Char(val) => write!(f, "{{\"name\":\"(char){}\"}}", val),
            Self::None => write!(f, "{{}}"),
        }
    }
}

struct ASTChildrenWrapper<'a>(&'a Vec<Box<AST>>);

impl fmt::Display for ASTChildrenWrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        let mut i = 0;
        for ast in self.0.iter() {
            if i != 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", ast)?;
            i += 1;
        }
        write!(f, "]")
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Neg => write!(f, "-"),
            Op::Add => write!(f, "+"),
            Op::Sub => write!(f, "-"),
            Op::Mul => write!(f, "*"),
            Op::Div => write!(f, "/"),
            Op::Mod => write!(f, "%"),
            Op::LogNot => write!(f, "!"),
            Op::LogAnd => write!(f, "&&"),
            Op::LogOr => write!(f, "||"),
            Op::Eq => write!(f, "=="),
            Op::Ne => write!(f, "!="),
            Op::Ge => write!(f, ">="),
            Op::Gt => write!(f, ">"),
            Op::Le => write!(f, "<="),
            Op::Lt => write!(f, "<"),
            Op::Assign => write!(f, "="),
            Op::StaticAccess => write!(f, "::"),
            Op::ObjAccess => write!(f, "."),
            Op::ArrayAccess => write!(f, "[]"),
        }
    }
}
