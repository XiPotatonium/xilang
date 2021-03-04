use super::super::ast::AST;
use xir::flag::*;
use xir::path::ModPath;

use std::fs;
use std::path::Path;

use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "lang/parser/grammar.pest"]
struct LRParser;

pub fn parse(path: &Path) -> Result<Box<AST>, Error<Rule>> {
    let code = fs::read_to_string(path).unwrap();

    let file = LRParser::parse(Rule::File, &code)?.next().unwrap();

    let mut uses: Vec<Box<AST>> = Vec::new();
    let mut mods: Vec<String> = Vec::new();
    let mut classes: Vec<Box<AST>> = Vec::new();
    for sub in file.into_inner() {
        match sub.as_rule() {
            Rule::EOI => break,
            Rule::Class => classes.push(build_class(sub)),
            Rule::Modules => mods.push(build_id(sub.into_inner().next().unwrap())),
            Rule::UseDeclarations => {
                let mut iter = sub.into_inner();
                let path = build_pathexpr(iter.next().unwrap());
                let as_clause = iter.next().unwrap();
                let as_id = match as_clause.as_rule() {
                    Rule::Id => Some(build_id(as_clause)),
                    Rule::Semi => None,
                    _ => unreachable!(),
                };

                uses.push(Box::new(AST::Use(path, as_id)));
            }
            _ => unreachable!(),
        };
    }

    Ok(Box::new(AST::File(mods, uses, classes)))
}

fn build_class(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let id = build_id(iter.next().unwrap());
    let mut fields: Vec<Box<AST>> = Vec::new();
    let mut methods: Vec<Box<AST>> = Vec::new();
    let mut init: Option<Box<AST>> = None;
    for sub in iter {
        match sub.as_rule() {
            Rule::StaticInit => {
                if let Some(_) = init {
                    panic!("Duplicated static init found in class {}", id);
                } else {
                    init = Some(build_block(sub.into_inner().next().unwrap()));
                }
            }
            Rule::StaticField => fields.push(build_field(sub, true)),
            Rule::NonStaticField => fields.push(build_field(sub, false)),
            Rule::Func => methods.push(build_method(sub)),
            _ => unreachable!(),
        }
    }

    Box::new(AST::Class(
        id,
        TypeFlag::default(),
        methods,
        fields,
        if let Some(v) = init {
            v
        } else {
            Box::new(AST::None)
        },
    ))
}

fn build_field(tree: Pair<Rule>, is_static: bool) -> Box<AST> {
    let mut iter = tree.into_inner();
    let id = build_id(iter.next().unwrap());
    let mut flag = FieldFlag::default();
    if is_static {
        flag.set(FieldFlagTag::Static);
    }

    Box::new(AST::Field(id, flag, build_type(iter.next().unwrap())))
}

fn build_method(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let id = build_id(iter.next().unwrap());
    let mut flag = MethodFlag::default();
    flag.set(MethodFlagTag::Static);

    let mut ps: Vec<Box<AST>> = Vec::new();
    let mut sub = iter.next().unwrap();
    if let Rule::Params = sub.as_rule() {
        // Build parameters
        let mut p_iter = sub.into_inner();
        let p0 = p_iter.next().unwrap();
        match p0.as_rule() {
            Rule::KwLSelf => {
                // non-static method
                flag.unset(MethodFlagTag::Static);
            }
            Rule::Id => {
                // static method
                ps.push(Box::new(AST::Param(
                    build_id(p0),
                    ParamFlag::default(),
                    build_type(p_iter.next().unwrap()),
                )));
            }
            _ => unreachable!(),
        }

        loop {
            if let Some(p_id) = p_iter.next() {
                ps.push(Box::new(AST::Param(
                    build_id(p_id),
                    ParamFlag::default(),
                    build_type(p_iter.next().unwrap()),
                )));
            } else {
                break;
            }
        }
        sub = iter.next().unwrap();
    }

    let ret_type = if let Rule::BlockExpr = sub.as_rule() {
        Box::new(AST::None)
    } else {
        let ret_type = build_type(sub);
        sub = iter.next().unwrap();
        ret_type
    };

    Box::new(AST::Method(id, flag, ret_type, ps, build_block(sub)))
}

fn build_pathexpr(tree: Pair<Rule>) -> ModPath {
    let mut ret = ModPath::new();
    for seg in tree.into_inner() {
        ret.push(match seg.as_rule() {
            Rule::Id => seg.as_span().as_str().trim(),
            Rule::KwCrate => "crate",
            Rule::KwSuper => "super",
            _ => unreachable!(),
        });
    }
    ret
}

fn build_type(tree: Pair<Rule>) -> Box<AST> {
    let tree = tree.into_inner().next().unwrap();
    let ret = Box::new(match tree.as_rule() {
        Rule::KwBool => AST::TypeBool,
        Rule::KwChar => AST::TypeChar,
        Rule::KwI32 => AST::TypeI32,
        Rule::KwF64 => AST::TypeF64,
        Rule::KwUSelf => AST::Path({
            let mut path = ModPath::new();
            path.push("Self");
            path
        }),
        Rule::PathExpr => AST::Path(build_pathexpr(tree)),
        Rule::TupleType => AST::TypeTuple(tree.into_inner().map(|ty| build_type(ty)).collect()),
        Rule::ArrType => {
            let mut iter = tree.into_inner();
            let sub_ty = build_type(iter.next().unwrap());
            if let Some(expr) = iter.next() {
                AST::TypeArr(sub_ty, build_expr(expr))
            } else {
                AST::TypeArr(sub_ty, Box::new(AST::None))
            }
        }
        _ => unreachable!(format!("Found {:?}", tree.as_rule())),
    });
    ret
}

fn build_expr(tree: Pair<Rule>) -> Box<AST> {
    match tree.as_rule() {
        Rule::BlockExpr => build_block(tree),
        Rule::LoopExpr => Box::new(AST::Loop(build_block(tree.into_inner().next().unwrap()))),
        Rule::IfExpr => build_if(tree),
        Rule::ContinueExpr => Box::new(AST::Continue),
        Rule::BreakExpr => Box::new(AST::Break(if let Some(ret_v) = tree.into_inner().next() {
            build_expr(ret_v)
        } else {
            Box::new(AST::None)
        })),
        Rule::ReturnExpr => Box::new(AST::Return(if let Some(ret_v) = tree.into_inner().next() {
            build_expr(ret_v)
        } else {
            Box::new(AST::None)
        })),
        Rule::AssignExpr => build_assign(tree),
        Rule::OpExpr => {
            let tree = tree.into_inner().next().unwrap();
            assert_eq!(tree.as_rule(), Rule::LogOrExpr);
            build_log_or_expr(tree)
        }
        _ => unreachable!(format!("Found {:?}", tree.as_rule())),
    }
}

fn build_if(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let cond = build_expr(iter.next().unwrap());
    let then = build_block(iter.next().unwrap());
    let els = if let Some(els) = iter.next() {
        match els.as_rule() {
            Rule::IfExpr => build_if(els),
            Rule::BlockExpr => build_block(els),
            _ => unreachable!(),
        }
    } else {
        Box::new(AST::None)
    };
    Box::new(AST::If(cond, then, els))
}

fn build_stmt(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let clause = iter.next().unwrap();
    match clause.as_rule() {
        Rule::LetStmt => {
            let mut iter = clause.into_inner();
            let pattern = build_pattern(iter.next().unwrap());
            let clause = iter.next().unwrap();
            Box::new(match clause.as_rule() {
                Rule::Type => {
                    // has type
                    let ty = build_type(clause);
                    let clause = iter.next().unwrap();
                    match clause.as_rule() {
                        Rule::Semi => {
                            // no init
                            AST::Let(pattern, LocalFlag::default(), ty, Box::new(AST::None))
                        }
                        Rule::Eq => AST::Let(
                            pattern,
                            LocalFlag::default(),
                            ty,
                            build_expr(iter.next().unwrap()),
                        ),
                        _ => unreachable!(),
                    }
                }
                Rule::Eq => {
                    // no type but has init
                    AST::Let(
                        pattern,
                        LocalFlag::default(),
                        Box::new(AST::None),
                        build_expr(iter.next().unwrap()),
                    )
                }
                Rule::Semi => {
                    // no type and no init
                    AST::Let(
                        pattern,
                        LocalFlag::default(),
                        Box::new(AST::None),
                        Box::new(AST::None),
                    )
                }
                _ => unreachable!(),
            })
        }
        _ => {
            let sub = build_expr(clause);
            if let Some(_) = iter.next() {
                // Semi
                Box::new(AST::ExprStmt(sub))
            } else {
                sub
            }
        }
    }
}

fn build_block(tree: Pair<Rule>) -> Box<AST> {
    Box::new(AST::Block(
        tree.into_inner()
            .map(|sub| match sub.as_rule() {
                Rule::Stmt => build_stmt(sub),
                _ => build_expr(sub),
            })
            .collect(),
    ))
}

fn build_assign(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let lhs = build_log_or_expr(iter.next().unwrap());
    iter.next(); // "="
    Box::new(AST::OpAssign(lhs, build_log_or_expr(iter.next().unwrap())))
}

fn build_log_or_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_log_and_expr(iter.next().unwrap());

    for rhs in iter {
        // log or is left associative
        ret = Box::new(AST::OpLogOr(ret, build_log_and_expr(rhs)));
    }
    ret
}

fn build_log_and_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_eq_expr(iter.next().unwrap());

    for rhs in iter {
        ret = Box::new(AST::OpLogAnd(ret, build_eq_expr(rhs)));
    }
    ret
}

fn build_eq_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_comp_expr(iter.next().unwrap());

    loop {
        if let Some(op) = iter.next() {
            ret = Box::new(match op.as_rule() {
                Rule::EqEq => AST::OpEq(ret, build_comp_expr(iter.next().unwrap())),
                Rule::Ne => AST::OpNe(ret, build_comp_expr(iter.next().unwrap())),
                _ => unreachable!(),
            });
        } else {
            break;
        }
    }
    ret
}

fn build_comp_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_add_expr(iter.next().unwrap());

    loop {
        if let Some(op) = iter.next() {
            ret = Box::new(match op.as_rule() {
                Rule::Le => AST::OpLe(ret, build_add_expr(iter.next().unwrap())),
                Rule::Lt => AST::OpLt(ret, build_add_expr(iter.next().unwrap())),
                Rule::Ge => AST::OpGe(ret, build_add_expr(iter.next().unwrap())),
                Rule::Gt => AST::OpGt(ret, build_add_expr(iter.next().unwrap())),
                _ => unreachable!(),
            });
        } else {
            break;
        }
    }
    ret
}

fn build_add_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_mul_expr(iter.next().unwrap());

    loop {
        if let Some(op) = iter.next() {
            ret = Box::new(match op.as_rule() {
                Rule::Plus => AST::OpAdd(ret, build_mul_expr(iter.next().unwrap())),
                Rule::Minus => AST::OpSub(ret, build_mul_expr(iter.next().unwrap())),
                _ => unreachable!(),
            });
        } else {
            break;
        }
    }
    ret
}

fn build_mul_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_cast_expr(iter.next().unwrap());

    loop {
        if let Some(op) = iter.next() {
            ret = Box::new(match op.as_rule() {
                Rule::Star => AST::OpMul(ret, build_cast_expr(iter.next().unwrap())),
                Rule::Slash => AST::OpDiv(ret, build_cast_expr(iter.next().unwrap())),
                Rule::Percent => AST::OpMod(ret, build_cast_expr(iter.next().unwrap())),
                _ => unreachable!(),
            });
        } else {
            break;
        }
    }
    ret
}

fn build_cast_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_unary_expr(iter.next().unwrap());

    loop {
        if let Some(rhs) = iter.next() {
            ret = Box::new(AST::OpCast(build_type(rhs), ret));
        } else {
            break;
        }
    }
    ret
}

fn build_unary_expr(tree: Pair<Rule>) -> Box<AST> {
    // unary is right associative, iterate reversely
    let mut iter = tree.into_inner().rev();
    let ret = iter.next().unwrap();
    let mut ret = match ret.as_rule() {
        Rule::NewExpr => build_new_expr(ret),
        Rule::CallExpr => build_call_expr(ret),
        _ => unreachable!(),
    };

    for op in iter {
        ret = Box::new(match op.as_rule() {
            Rule::Plus => AST::OpPos(ret),
            Rule::Not => AST::OpLogNot(ret),
            Rule::Minus => AST::OpNeg(ret),
            _ => unreachable!(),
        });
    }
    ret
}

fn build_new_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let ty = build_type(iter.next().unwrap());
    let fields: Vec<Box<AST>> = iter
        .map(|sub| {
            let mut sub_iter = sub.into_inner();
            let id = build_id(sub_iter.next().unwrap());
            Box::new(AST::StructExprField(
                id,
                if let Some(val) = sub_iter.next() {
                    build_expr(val)
                } else {
                    Box::new(AST::None)
                },
            ))
        })
        .collect();

    Box::new(AST::OpNew(ty, fields))
}

fn build_call_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_primary_expr(iter.next().unwrap());

    for rhs in iter {
        ret = Box::new(match rhs.as_rule() {
            Rule::ArgsExpr => {
                AST::OpCall(ret, rhs.into_inner().map(|sub| build_expr(sub)).collect())
            }
            Rule::ObjAccessExpr => {
                AST::OpObjAccess(ret, build_id(rhs.into_inner().next().unwrap()))
            }
            Rule::PathAccessExpr => {
                AST::OpStaticAccess(ret, build_id(rhs.into_inner().next().unwrap()))
            }
            Rule::ArrAccessExpr => {
                AST::OpArrayAccess(ret, build_expr(rhs.into_inner().next().unwrap()))
            }
            _ => unreachable!(),
        });
    }
    ret
}

fn build_primary_expr(tree: Pair<Rule>) -> Box<AST> {
    let tree = tree.into_inner().next().unwrap();
    match tree.as_rule() {
        Rule::Type => build_type(tree),
        Rule::GroupedExpr => build_expr(tree.into_inner().next().unwrap()),
        Rule::LiteralExpr => build_literal(tree),
        Rule::KwLSelf => Box::new(AST::Id(String::from("self"))),
        // Actually only expr with block
        _ => build_expr(tree),
    }
}

fn build_literal(tree: Pair<Rule>) -> Box<AST> {
    let tree = tree.into_inner().next().unwrap();
    Box::new(match tree.as_rule() {
        Rule::KwTrue => AST::Bool(true),
        Rule::KwFalse => AST::Bool(false),
        Rule::KwNull => AST::Null,
        Rule::EmptyLiteral => AST::None,
        Rule::IntLiteral => AST::Int(tree.as_span().as_str().trim().parse::<i32>().expect(
            &format!("Unable to parse \"{}\" as i32", tree.as_span().as_str()),
        )),
        Rule::FloatLiteral => AST::Float(tree.as_span().as_str().trim().parse::<f64>().unwrap()),
        Rule::StringLiteral => {
            let mut chars = tree.as_span().as_str().trim().chars();
            chars.next(); // skip first '"'
            let mut s = String::new();
            loop {
                match chars.next() {
                    Some(ch) => {
                        match ch {
                            '\\' => {
                                // escape
                                match chars.next().expect("Invalid string literal") {
                                    'n' => s.push('\n'),
                                    _ => unimplemented!("Unsupported escape char"),
                                }
                            }
                            '"' => break,
                            _ => s.push(ch),
                        }
                    }
                    None => unreachable!(),
                }
            }
            AST::String(s)
        }
        Rule::CharLiteral => {
            let mut chars = tree.as_span().as_str().trim().chars();
            chars.next(); // skip first '\''
            let ch = AST::Char(match chars.next().unwrap() {
                '\'' => panic!("Empty char literal"),
                '\\' => chars.next().unwrap().into(),
                c => c.into(),
            });
            match chars.next().expect("Invalid char literal") {
                '\'' => (),
                _ => panic!("Too many chars in char literal"),
            }
            ch
        }
        _ => unreachable!(),
    })
}

fn build_pattern(tree: Pair<Rule>) -> Box<AST> {
    let tree = tree.into_inner().next().unwrap();
    Box::new(match tree.as_rule() {
        Rule::Id => AST::Id(build_id(tree)),
        Rule::TuplePattern => {
            AST::TuplePattern(tree.into_inner().map(|sub| build_pattern(sub)).collect())
        }
        _ => unreachable!(),
    })
}

fn build_id(tree: Pair<Rule>) -> String {
    assert_eq!(tree.as_rule(), Rule::Id);
    String::from(tree.as_span().as_str().trim())
}
