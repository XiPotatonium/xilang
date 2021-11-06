use super::super::util::IItemPath;
use super::*;

use std::fmt;

// Restore escape chars
fn restore_escape(s: &str) -> String {
    let mut ret = String::new();

    for ch in s.chars() {
        match ch {
            '\n' => ret.push_str("\\n"),
            '"' => ret.push_str("\\\""),
            _ => ret.push(ch),
        }
    }

    ret
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "({}, {})", self.x, self.y)
        match self {
            Self::File(mods, classes) => {
                write!(
                    f,
                    "{{\"name\":\"(file)\",\"mods\":[{}],\"classes\":{}}}",
                    mods.iter()
                        .map(|m| format!("\"{}\"", m))
                        .collect::<Vec<String>>()
                        .join(","),
                    BoxASTVecWrapper(classes),
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
            Self::Struct(strukt) => strukt.ast_fmt(f),
            Self::Method(method) => method.fmt(f),
            Self::Field(id, flag, ty) => write!(
                f,
                "{{\"name\":\"(field){}\",\"flag\":\"{}\",\"type\":\"{}\"}}",
                id, flag, ty
            ),
            Self::Param(id, ty) => write!(f, "{{\"name\":\"(param){}\",\"type\":\"{}\"}}", id, ty),
            Self::Let(pattern, ty, init) => write!(
                f,
                "{{\"name\":\"(let)\",\"id\":{},\"type\":\"{}\",\"init\":{}}}",
                pattern, ty, init
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
                cond, then, els
            ),
            Self::Continue => write!(f, "{{\"name\":\"continue\"}}"),
            Self::Break(val) => write!(f, "{{\"name\":\"break\",\"val\":{}}}", val),
            Self::Return(val) => write!(f, "{{\"name\":\"return\",\"val\":{}}}", val),
            Self::Loop(body) => write!(f, "{{\"name\":\"(loop)\",\"body\":{}}}", body),
            Self::OpPos(o) => write!(f, "{{\"name\":\"+\",\"lhs\":{}}}", o),
            Self::OpNeg(o) => write!(f, "{{\"name\":\"-\",\"lhs\":{}}}", o),
            Self::OpAdd(o1, o2) => write!(f, "{{\"name\":\"+\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpSub(o1, o2) => write!(f, "{{\"name\":\"-\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpMul(o1, o2) => write!(f, "{{\"name\":\"*\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpDiv(o1, o2) => write!(f, "{{\"name\":\"/\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpMod(o1, o2) => write!(f, "{{\"name\":\"%\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpLogNot(o1) => write!(f, "{{\"name\":\"!\",\"lhs\":{}}}", o1),
            Self::OpLogAnd(o1, o2) => {
                write!(f, "{{\"name\":\"&&\",\"lhs\":{},\"rhs\":{}}}", o1, o2)
            }
            Self::OpLogOr(o1, o2) => write!(f, "{{\"name\":\"||\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpEq(o1, o2) => write!(f, "{{\"name\":\"==\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpNe(o1, o2) => write!(f, "{{\"name\":\"!=\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpGe(o1, o2) => write!(f, "{{\"name\":\">=\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpGt(o1, o2) => write!(f, "{{\"name\":\">\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpLe(o1, o2) => write!(f, "{{\"name\":\"<=\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpLt(o1, o2) => write!(f, "{{\"name\":\"<\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpAssign(o1, o2) => write!(f, "{{\"name\":\"=\",\"lhs\":{},\"rhs\":{}}}", o1, o2),
            Self::OpStaticAccess(o1, o2) => {
                write!(f, "{{\"name\":\"::\",\"lhs\":{},\"rhs\":\"{}\"}}", o1, o2)
            }
            Self::OpObjAccess(o1, o2) => {
                write!(f, "{{\"name\":\".\",\"lhs\":{},\"rhs\":\"{}\"}}", o1, o2)
            }
            Self::OpArrayAccess(o1, o2) => {
                write!(f, "{{\"name\":\"[]\",\"lhs\":{},\"rhs\":{}}}", o1, o2)
            }
            Self::OpCast(ty, expr) => {
                write!(
                    f,
                    "{{\"name\":\"(cast)\",\"ty\":\"{}\",\"val\":{}}}",
                    ty, expr
                )
            }
            Self::OpCall(func, ps) => write!(
                f,
                "{{\"name\":\"(call)\",\"func\":{},\"args\":{}}}",
                func,
                BoxASTVecWrapper(ps)
            ),
            Self::OpNewStruct(ty, fields) => write!(
                f,
                "{{\"name\":\"new\",\"type\":\"{}\",\"fields\":{}}}",
                ty,
                BoxASTVecWrapper(fields)
            ),
            Self::OpNewArr(ty, dim) => write!(
                f,
                "{{\"name\":\"newarr\",\"type\":\"{}\",\"dim\":{}}}",
                ty, dim,
            ),
            Self::Id(id) => write!(f, "{{\"name\":\"(id){}\"}}", id),
            Self::IdWithGenericParams(id_with_generic_ps) => {
                write!(f, "{{\"name\":\"(id){}\"}}", id_with_generic_ps)
            }
            Self::TuplePattern(p) => write!(
                f,
                "{{\"name\":\"(TuplePattern)\",\"children\":{}}}",
                BoxASTVecWrapper(p)
            ),
            Self::Type(ty) => write!(f, "{{\"name\":\"{}\"}}", ty),
            Self::Null => write!(f, "{{\"name\":\"null\"}}"),
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
        for (i, ast) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", ast)?;
        }
        write!(f, "]")
    }
}

impl<T: fmt::Display> fmt::Display for ASTVecWrapper<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, ast) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", ast)?;
        }
        write!(f, "]")
    }
}
