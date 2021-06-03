use super::super::ast::*;

use xir::attrib::*;
use xir::util::path::ModPath;

use std::fs;
use std::path::Path;

use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

#[derive(Parser)]
#[grammar = "lang/parser/grammar.pest"]
struct LRParser;

pub fn parse(path: &Path) -> Result<Box<AST>, Error<Rule>> {
    let code = fs::read_to_string(path).unwrap();

    let file = LRParser::parse(Rule::File, &code)?.next().unwrap();

    let mut uses: Vec<Box<AST>> = Vec::new();
    let mut mods: Vec<String> = Vec::new();
    let mut exts: Vec<String> = Vec::new();
    let mut classes: Vec<Box<AST>> = Vec::new();
    for sub in file.into_inner() {
        match sub.as_rule() {
            Rule::EOI => break,
            Rule::Class => classes.push(build_custom_type(sub)),
            Rule::Modules => mods.push(build_id(sub.into_inner().next().unwrap())),
            Rule::ExternMod => exts.push(build_id(sub.into_inner().next().unwrap())),
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

    Ok(Box::new(AST::File(mods, exts, uses, classes)))
}

fn build_attributes(iter: &mut Pairs<Rule>) -> Vec<Box<AST>> {
    let mut ret = Vec::new();
    while let Rule::AttributeLst = iter.peek().unwrap().as_rule() {
        for attr in iter.next().unwrap().into_inner() {
            if let Rule::Attribute = attr.as_rule() {
                let mut attr_iter = attr.into_inner();
                let attr_id = build_id(attr_iter.next().unwrap());
                let attr_args = attr_iter.map(|a| build_literal(a)).collect();
                ret.push(Box::new(AST::CustomAttrib(attr_id, attr_args)));
            } else {
                unreachable!();
            }
        }
    }
    ret
}

fn build_custom_type(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let custom_attribs = build_attributes(&mut iter);
    let sem = iter.next().unwrap().as_rule();
    let name = build_id(iter.next().unwrap());
    let mut extends_or_impls: Vec<ModPath> = Vec::new();

    if let Some(try_extends_or_impls) = iter.peek() {
        if let Rule::ExtendsOrImpls = try_extends_or_impls.as_rule() {
            let classes = iter.next().unwrap().into_inner();
            for class in classes {
                extends_or_impls.push(build_pathexpr(class));
            }
        }
    }

    let mut fields = Vec::new();
    let mut methods = Vec::new();
    let mut cctor: Option<Box<AST>> = None;
    let mut ctors = Vec::new();
    while let Some(_) = iter.peek() {
        let item_attrib = build_attributes(&mut iter);
        let class_item = iter.next().unwrap();
        match class_item.as_rule() {
            Rule::CCtor => {
                if let Some(_) = cctor {
                    panic!("Duplicated static init found in class {}", name);
                } else {
                    cctor = Some(build_block(class_item.into_inner().next().unwrap()));
                }
            }
            Rule::Ctor => ctors.push(build_ctor(class_item, item_attrib)),
            Rule::StaticField => fields.push(build_field(class_item, true, item_attrib)),
            Rule::NonStaticField => fields.push(build_field(class_item, false, item_attrib)),
            Rule::Method => methods.push(build_method(class_item, item_attrib)),
            _ => unreachable!(),
        }
    }

    let mut ret = ASTClass {
        name,
        attrib: TypeAttrib::new_class(TypeAttribVisFlag::Pub.into()),
        custom_attribs,
        extends_or_impls,
        methods,
        fields,
        cctor: if let Some(v) = cctor {
            v
        } else {
            Box::new(AST::None)
        },
        ctors,
    };
    Box::new(match sem {
        Rule::KwClass => AST::Class(ret),
        Rule::KwStruct => {
            // struct is default to be sealed
            ret.attrib.set(TypeAttribFlag::Sealed);
            AST::Struct(ret)
        }
        Rule::KwInterface => {
            unimplemented!()
        }
        _ => unreachable!(),
    })
}

fn build_field(tree: Pair<Rule>, is_static: bool, attr: Vec<Box<AST>>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let id = build_id(iter.next().unwrap());
    let mut flag = FieldAttrib::from(FieldAttribFlag::Pub.into());
    if is_static {
        flag.set(FieldAttribFlag::Static);
    }

    Box::new(AST::Field(id, flag, attr, build_type(iter.next().unwrap())))
}

fn build_ctor(tree: Pair<Rule>, custom_attribs: Vec<Box<AST>>) -> Box<AST> {
    let mut iter = tree.into_inner();

    let attrib = MethodAttrib::from(
        u16::from(MethodAttribFlag::Pub)
            | u16::from(MethodAttribFlag::SpecialName)
            | u16::from(MethodAttribFlag::RTSpecialName),
    );
    let ps = if let Rule::Params = iter.peek().unwrap().as_rule() {
        // Build parameters
        let (ps, has_self) = build_params(iter.next().unwrap());
        if !has_self {
            panic!("ctor must have \"self\" as its first param");
        }
        ps
    } else {
        // no param
        Vec::new()
    };

    let base_args = if let Rule::KwBase = iter.peek().unwrap().as_rule() {
        iter.next(); // skip base
        Some(
            iter.next()
                .unwrap()
                .into_inner()
                .map(|a| build_expr(a))
                .collect(),
        )
    } else {
        None
    };

    let body = iter.next().unwrap();
    let body = match body.as_rule() {
        Rule::BlockExpr => build_block(body),
        Rule::Semi => Box::new(AST::None),
        _ => unreachable!(),
    };

    Box::new(AST::Ctor(ASTCtor {
        attrib,
        custom_attribs,
        base_args,
        ps,
        body,
    }))
}

fn build_method(tree: Pair<Rule>, custom_attribs: Vec<Box<AST>>) -> Box<AST> {
    let mut iter = tree.into_inner();

    // built-in attributes
    let mut attrib = MethodAttrib::from(MethodAttribFlag::Pub.into());
    let mut ast_attrib = ASTMethodAttrib::default();
    loop {
        match iter.peek().unwrap().as_rule() {
            Rule::KwOverride => {
                iter.next();
                if ast_attrib.is(ASTMethodAttribFlag::Override) {
                    panic!("Duplicated override modifier");
                } else {
                    ast_attrib.set(ASTMethodAttribFlag::Override);
                }
            }
            Rule::KwVirtual => {
                iter.next();
                if attrib.is(MethodAttribFlag::Virtual) {
                    panic!("Duplicated virtual modifier");
                } else {
                    attrib.set(MethodAttribFlag::Virtual);
                }
            }
            _ => break,
        }
    }

    let name = build_id(iter.next().unwrap());

    let (ps, has_self) = build_params(iter.next().unwrap());
    if !has_self {
        attrib.set(MethodAttribFlag::Static);
    }

    let ty = if let Rule::Type = iter.peek().unwrap().as_rule() {
        build_type(iter.next().unwrap())
    } else {
        Box::new(ASTType::None)
    };

    let body = iter.next().unwrap();
    let body = match body.as_rule() {
        Rule::BlockExpr => build_block(body),
        Rule::Semi => Box::new(AST::None),
        _ => unreachable!(),
    };

    Box::new(AST::Method(ASTMethod {
        name,
        attrib,
        ast_attrib,
        custom_attribs,
        ret: ty,
        ps,
        body,
    }))
}

// Build parameters
fn build_params(tree: Pair<Rule>) -> (Vec<Box<AST>>, bool) {
    let mut ps = Vec::new();
    let mut has_self = false;
    let mut p_iter = tree.into_inner();
    if let Some(p0) = p_iter.next() {
        match p0.as_rule() {
            Rule::KwLSelf => {
                // non-static method
                has_self = true;
            }
            Rule::Id => {
                // static method
                ps.push(Box::new(AST::Param(
                    build_id(p0),
                    ParamAttrib::from(0),
                    build_type(p_iter.next().unwrap()),
                )));
            }
            _ => unreachable!(),
        }
    } else {
        // no param
        return (vec![], false);
    }

    loop {
        if let Some(p_id) = p_iter.next() {
            ps.push(Box::new(AST::Param(
                build_id(p_id),
                ParamAttrib::from(0),
                build_type(p_iter.next().unwrap()),
            )));
        } else {
            break;
        }
    }
    (ps, has_self)
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

fn _build_type(tree: Pair<Rule>) -> Box<ASTType> {
    Box::new(match tree.as_rule() {
        Rule::KwBool => ASTType::Bool,
        Rule::KwChar => ASTType::Char,
        Rule::KwI32 => ASTType::I32,
        Rule::KwF64 => ASTType::F64,
        Rule::KwString => ASTType::String,
        Rule::KwUSelf => ASTType::Class({
            let mut path = ModPath::new();
            path.push("Self");
            path
        }),
        Rule::PathExpr => ASTType::Class(build_pathexpr(tree)),
        Rule::TupleType => ASTType::Tuple(tree.into_inner().map(|ty| build_type(ty)).collect()),
        Rule::ArrType => {
            let inner = _build_type(tree.into_inner().next().unwrap());
            ASTType::Arr(inner)
        }
        _ => unreachable!(format!("Found {:?}", tree.as_rule())),
    })
}

/// tree: Type
fn build_type(tree: Pair<Rule>) -> Box<ASTType> {
    let tree = tree.into_inner().next().unwrap();
    _build_type(tree)
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
                            AST::Let(pattern, LocalAttrib::default(), ty, Box::new(AST::None))
                        }
                        Rule::Eq => AST::Let(
                            pattern,
                            LocalAttrib::default(),
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
                        LocalAttrib::default(),
                        Box::new(ASTType::None),
                        build_expr(iter.next().unwrap()),
                    )
                }
                Rule::Semi => {
                    // no type and no init
                    AST::Let(
                        pattern,
                        LocalAttrib::default(),
                        Box::new(ASTType::None),
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

    for rhs in iter {
        ret = Box::new(AST::OpCast(build_type(rhs), ret));
    }
    ret
}

fn build_unary_expr(tree: Pair<Rule>) -> Box<AST> {
    // unary is right associative, iterate reversely
    let mut iter = tree.into_inner().rev();
    let mut ret = build_new_expr(iter.next().unwrap());

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

    let ret = iter.next().unwrap();
    match ret.as_rule() {
        Rule::CallExpr => build_call_expr(ret),
        Rule::Type => {
            let ty = build_type(ret);
            let initializer = iter.next().unwrap();
            Box::new(match initializer.as_rule() {
                Rule::Args => AST::OpNew(
                    ty,
                    initializer
                        .into_inner()
                        .map(|sub| build_expr(sub))
                        .collect(),
                ),
                Rule::ArrAccessExpr => {
                    AST::OpNewArr(ty, build_expr(initializer.into_inner().next().unwrap()))
                }
                _ => unreachable!(),
            })
        }
        _ => unreachable!(),
    }
}

fn build_call_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_primary_expr(iter.next().unwrap());

    for rhs in iter {
        ret = Box::new(match rhs.as_rule() {
            Rule::Args => AST::OpCall(ret, rhs.into_inner().map(|sub| build_expr(sub)).collect()),
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
        Rule::GroupedExpr => build_expr(tree.into_inner().next().unwrap()),
        Rule::LiteralExpr => build_literal(tree),
        Rule::KwLSelf => Box::new(AST::Id(String::from("self"))),
        Rule::Id => Box::new(AST::Id(build_id(tree))),
        Rule::Type => Box::new(AST::Type(build_type(tree))),
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
                                    '"' => s.push('"'),
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
