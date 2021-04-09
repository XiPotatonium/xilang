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
                    ASTChildrenWrapper(uses),
                    ASTChildrenWrapper(children)
                )
            }
            Self::Use(path, as_id) => write!(
                f,
                "{{\"name\":\"(use)\",\"path\":\"{}\",\"as\":\"{}\"}}",
                path.as_str(),
                if let Some(as_id) = as_id { as_id } else { "" }
            ),
            Self::CustomAttr(id, args) => write!(f, "{{\"name\":\"(Attr){}\",\"args\":{}}}", id, ASTChildrenWrapper(args)),
            Self::Class(id, flag, attr, funcs, fields, init) => write!(
                f,
                "{{\"name\":\"(class){}\",\"flag\":\"{}\",\"attr\":{},\"fields\":{},\"init\":{},\"funcs\":{}}}",
                id,
                flag,
                ASTChildrenWrapper(attr),
                ASTChildrenWrapper(fields),
                init,
                ASTChildrenWrapper(funcs)
            ),
            Self::Method(id, flag, attr, ty, ps, body) => write!(
                f,
                "{{\"name\":\"(method){}\",\"flag\":\"{}\",\"attr\":{},\"type\":{},\"ps\":{},\"body\":{}}}",
                id,
                flag,
                ASTChildrenWrapper(attr),
                ty,
                ASTChildrenWrapper(ps),
                body.as_ref()
            ),
            Self::Field(id, flag, attr, ty) => write!(
                f,
                "{{\"name\":\"(field){}\",\"flag\":\"{}\",\"attr\":{},\"type\":{}}}",
                id, flag, ASTChildrenWrapper(attr), ty
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
                ASTChildrenWrapper(ps)
            ),
            Self::StructExprField(id, expr) => write!(
                f,
                "{{\"name\":\"(field){}\",\"val\":{}}}",
                id,
                expr.as_ref()
            ),
            Self::OpNew(ty, struct_init) => write!(
                f,
                "{{\"name\":\"new\",\"type\":{},\"fields\":{}}}",
                ty.as_ref(),
                ASTChildrenWrapper(struct_init)
            ),
            Self::Id(id) => write!(f, "{{\"name\":\"(id){}\"}}", id),
            Self::TuplePattern(p) => write!(
                f,
                "{{\"name\":\"(TuplePattern)\",\"children\":{}}}",
                ASTChildrenWrapper(p)
            ),
            Self::TypeBool => write!(f, "{{\"name\":\"(bool)\"}}"),
            Self::TypeChar => write!(f, "{{\"name\":\"(char)\"}}"),
            Self::TypeI32 => write!(f, "{{\"name\":\"(i32)\"}}"),
            Self::TypeF64 => write!(f, "{{\"name\":\"(f64)\"}}"),
            Self::TypeTuple(v) => write!(
                f,
                "{{\"name\":\"(TupleType)\",\"children\":{}}}",
                ASTChildrenWrapper(v)
            ),
            Self::TypeArr(dtype, dim) => write!(
                f,
                "{{\"name\":\"(ArrType)\",\"dtype\":{},\"dim\":{}}}",
                dtype, dim
            ),
            Self::Path(names) => write!(f, "{{\"name\":\"(Path){}\"}}", names.as_str()),
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
