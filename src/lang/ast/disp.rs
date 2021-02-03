use super::ast::{Type, AST};
use super::expr::Op;

use std::fmt;

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "({}, {})", self.x, self.y)
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Int => write!(f, "int"),
            Type::Double => write!(f, "double"),
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
                "{{\"name\":\"(func){}\",\"type\":{},\"ps\":{},\"body\":{}}}",
                id,
                ty,
                ASTChildrenWrapper(ps),
                body.as_ref()
            ),
            Self::Var(id, ty, flag, init) => write!(
                f,
                "{{\"name\":\"(var){}\",\"flag\": \"{}\", \"type\":{},\"init\":{}}}",
                id, flag, ty, *init
            ),
            Self::Block(children) => write!(
                f,
                "{{\"name\":\"(block)\",\"children\":{}}}",
                ASTChildrenWrapper(children)
            ),
            Self::If(cond, then, els) => write!(
                f,
                "{{\"name\":\"(if)\",\"cond\":{},\"then\":{},\"els\":{}}}",
                cond.as_ref(),
                then.as_ref(),
                els.as_ref()
            ),
            Self::Continue => write!(f, "{{\"name\":\"continue\"}}"),
            Self::Break => write!(f, "{{\"name\":\"break\"}}"),
            Self::Return(val) => write!(f, "{{\"name\":\"return\",\"val\":{}}}", *val),
            Self::For(pattern, iter, body) => write!(
                f,
                "{{\"name\":\"(for)\",\"var\":{},\"iter\":{},\"body\":{}}}",
                pattern.as_ref(),
                iter.as_ref(),
                body.as_ref()
            ),
            Self::While(cond, body) => write!(
                f,
                "{{\"name\":\"(while)\",\"cond\":{},\"body\":{}}}",
                cond.as_ref(),
                body.as_ref()
            ),
            Self::Loop(body) => write!(f, "{{\"name\":\"(loop)\",\"body\":{}}}", body.as_ref()),
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
            Self::Call(func, ps) => write!(
                f,
                "{{\"name\":\"(call)\",\"func\":{},\"args\":{}}}",
                func.as_ref(),
                ASTChildrenWrapper(ps)
            ),
            Self::New(ty) => write!(f, "{{\"name\":\"(new)\",\"type\":{}}}", *ty),
            Self::Cast(ty, val) => write!(
                f,
                "{{\"name\":\"(cast)\",\"type\":{},\"val\":{}}}",
                ty.as_ref(),
                val.as_ref()
            ),
            Self::Type(ty) => write!(f, "{{\"name\":\"(type)\",\"type\":\"{}\"}}", *ty),
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

struct ASTChildrenWrapper<'a>(&'a Vec<AST>);

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
            Op::AddAssign => write!(f, "+="),
            Op::SubAssign => write!(f, "-="),
            Op::MulAssign => write!(f, "*="),
            Op::DivAssign => write!(f, "/="),
            Op::ModAssign => write!(f, "%="),
            Op::StaticAccess => write!(f, "::"),
            Op::ClassAccess => write!(f, "."),
            Op::ArrayAccess => write!(f, "[]"),
        }
    }
}
