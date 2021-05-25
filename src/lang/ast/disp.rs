use super::AST;
use xir::util::path::IModPath;

use std::fmt;

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
            Self::File(mods, ext, uses, children) => {
                write!(
                    f,
                    "{{\"name\":\"(file)\",\"mods\":[{}],\"extern\":[{}],\"uses\":{},\"classes\":{}}}",
                    mods.iter()
                        .map(|m| format!("\"{}\"", m))
                        .collect::<Vec<String>>()
                        .join(","),
                    ext.iter()
                        .map(|m| format!("\"{}\"", m))
                        .collect::<Vec<String>>()
                        .join(","),
                    BoxASTVecWrapper(uses),
                    BoxASTVecWrapper(children)
                )
            }
            Self::Use(path, as_id) => write!(
                f,
                "{{\"name\":\"(use)\",\"path\":\"{}\",\"as\":\"{}\"}}",
                path.as_str(),
                if let Some(as_id) = as_id { as_id } else { "" }
            ),
            Self::CustomAttrib(id, args) => write!(
                f,
                "{{\"name\":\"(Attr){}\",\"args\":{}}}",
                id,
                BoxASTVecWrapper(args)
            ),
            Self::Class(class) => class.ast_fmt(f, false),
            Self::Struct(class) => class.ast_fmt(f, true),
            Self::Ctor(ctor) => ctor.fmt(f),
            Self::Method(method) => method.fmt(f),
            Self::Field(id, flag, attr, ty) => write!(
                f,
                "{{\"name\":\"(field){}\",\"flag\":\"{}\",\"attr\":{},\"type\":{}}}",
                id,
                flag,
                BoxASTVecWrapper(attr),
                ty
            ),
            Self::Param(id, flag, ty) => write!(
                f,
                "{{\"name\":\"(param){}\",\"flag\":\"{}\",\"type\":{}}}",
                id, flag, ty
            ),
            Self::Let(pattern, flag, ty, init) => write!(
                f,
                "{{\"name\":\"(let)\", \"id\":{},\"flag\": \"{}\", \"type\":{},\"init\":{}}}",
                pattern,
                flag,
                ty,
                init.as_ref()
            ),
            Self::ExprStmt(stmt) => write!(f, "{{\"name\":\"(ExprStmt)\",\"stmt\":{}}}", stmt),
            Self::Block(children) => write!(
                f,
                "{{\"name\":\"(block)\",\"children\":{}}}",
                BoxASTVecWrapper(children)
            ),
            Self::If(cond, then, els) => write!(
                f,
                "{{\"name\":\"(if)\",\"cond\":{},\"then\":{},\"els\":{}}}",
                cond.as_ref(),
                then.as_ref(),
                els.as_ref()
            ),
            Self::Continue => write!(f, "{{\"name\":\"continue\"}}"),
            Self::Break(val) => write!(f, "{{\"name\":\"break\", \"val\": {}}}", val.as_ref()),
            Self::Return(val) => write!(f, "{{\"name\":\"return\",\"val\":{}}}", val.as_ref()),
            Self::Loop(body) => write!(f, "{{\"name\":\"(loop)\",\"body\":{}}}", body.as_ref()),
            Self::OpPos(o) => write!(f, "{{\"name\":\"+\",\"lhs\":{}}}", o.as_ref()),
            Self::OpNeg(o) => write!(f, "{{\"name\":\"-\",\"lhs\":{}}}", o.as_ref()),
            Self::OpAdd(o1, o2) => write!(
                f,
                "{{\"name\":\"+\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpSub(o1, o2) => write!(
                f,
                "{{\"name\":\"-\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpMul(o1, o2) => write!(
                f,
                "{{\"name\":\"*\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpDiv(o1, o2) => write!(
                f,
                "{{\"name\":\"/\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpMod(o1, o2) => write!(
                f,
                "{{\"name\":\"%\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpLogNot(o1) => write!(f, "{{\"name\":\"!\",\"lhs\":{}}}", o1.as_ref()),
            Self::OpLogAnd(o1, o2) => write!(
                f,
                "{{\"name\":\"&&\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpLogOr(o1, o2) => write!(
                f,
                "{{\"name\":\"||\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpEq(o1, o2) => write!(
                f,
                "{{\"name\":\"==\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpNe(o1, o2) => write!(
                f,
                "{{\"name\":\"!=\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpGe(o1, o2) => write!(
                f,
                "{{\"name\":\">=\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpGt(o1, o2) => write!(
                f,
                "{{\"name\":\">\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpLe(o1, o2) => write!(
                f,
                "{{\"name\":\"<=\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpLt(o1, o2) => write!(
                f,
                "{{\"name\":\"<\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpAssign(o1, o2) => write!(
                f,
                "{{\"name\":\"=\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpStaticAccess(o1, o2) => write!(
                f,
                "{{\"name\":\"::\",\"lhs\":{},\"rhs\":\"{}\"}}",
                o1.as_ref(),
                o2
            ),
            Self::OpObjAccess(o1, o2) => write!(
                f,
                "{{\"name\":\".\",\"lhs\":{},\"rhs\":\"{}\"}}",
                o1.as_ref(),
                o2
            ),
            Self::OpArrayAccess(o1, o2) => write!(
                f,
                "{{\"name\":\"[]\",\"lhs\":{},\"rhs\":{}}}",
                o1.as_ref(),
                o2.as_ref()
            ),
            Self::OpCast(ty, expr) => {
                write!(f, "{{\"name\":\"(cast)\",\"ty\":{},\"val\":{}}}", ty, expr)
            }
            Self::OpCall(func, ps) => write!(
                f,
                "{{\"name\":\"(call)\",\"func\":{},\"args\":{}}}",
                func.as_ref(),
                BoxASTVecWrapper(ps)
            ),
            Self::OpNew(ty, args) => write!(
                f,
                "{{\"name\":\"new\",\"type\":{},\"args\":{}}}",
                ty.as_ref(),
                BoxASTVecWrapper(args)
            ),
            Self::OpNewArr(ty, dim) => write!(
                f,
                "{{\"name\":\"newarr\",\"type\":{},\"dim\":{}}}",
                ty.as_ref(),
                dim,
            ),
            Self::Id(id) => write!(f, "{{\"name\":\"(id){}\"}}", id),
            Self::TuplePattern(p) => write!(
                f,
                "{{\"name\":\"(TuplePattern)\",\"children\":{}}}",
                BoxASTVecWrapper(p)
            ),
            Self::Type(ty) => ty.fmt(f),
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

pub struct BoxASTVecWrapper<'a, T: fmt::Display>(pub &'a Vec<Box<T>>);

pub struct ASTVecWrapper<'a, T: fmt::Display>(pub &'a Vec<T>);

impl<T: fmt::Display> fmt::Display for BoxASTVecWrapper<'_, T> {
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

impl<T: fmt::Display> fmt::Display for ASTVecWrapper<'_, T> {
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
