use super::super::ast::*;
use super::super::util::ItemPathBuf;

use ir::flags::*;

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

    let mut mods: Vec<String> = Vec::new();
    let mut classes: Vec<Box<AST>> = Vec::new();
    for sub in file.into_inner() {
        match sub.as_rule() {
            Rule::EOI => break,
            Rule::Class => classes.push(build_custom_type(sub)),
            Rule::Modules => mods.push(build_id(sub.into_inner().next().unwrap())),
            _ => unreachable!(),
        };
    }

    Ok(Box::new(AST::File(mods, classes)))
}

fn build_attributes(iter: &mut Pairs<Rule>) -> Vec<Box<AST>> {
    let mut ret = Vec::new();
    while let Rule::AttributeLst = iter.peek().unwrap().as_rule() {
        for attr in iter.next().unwrap().into_inner() {
            if let Rule::Attribute = attr.as_rule() {
                let mut attr_iter = attr.into_inner();
                let attr_id = build_id(attr_iter.next().unwrap());
                let attr_args = attr_iter.map(build_literal).collect();
                ret.push(Box::new(AST::CustomAttrib(attr_id, attr_args)));
            } else {
                unreachable!();
            }
        }
    }
    ret
}

fn build_generic_params_decl(tree: Pair<Rule>, decls: &mut Vec<ASTGenericParamDecl>) {
    for decl in tree.into_inner() {
        assert_eq!(Rule::GenericParamDecl, decl.as_rule());
        let mut decl_iter = decl.into_inner();
        let mut ast_decl = ASTGenericParamDecl {
            id: build_id(decl_iter.next().unwrap()),
            constraints: Vec::new(),
        };
        for constraint in decl_iter {
            ast_decl.constraints.push(build_pathexpr(constraint));
        }
        decls.push(ast_decl);
    }
}

fn build_custom_type(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let custom_attribs = build_attributes(&mut iter);
    let sem = iter.next().unwrap().as_rule();
    let name = build_id(iter.next().unwrap());
    let mut extends_or_impls: Vec<ItemPathBuf> = Vec::new();

    let mut generic_params = Vec::new();
    if let Some(try_generic) = iter.peek() {
        if let Rule::GenericParamsDecl = try_generic.as_rule() {
            build_generic_params_decl(iter.next().unwrap(), &mut generic_params);
        }
    }

    if let Some(try_impls) = iter.peek() {
        if let Rule::Impls = try_impls.as_rule() {
            for class in iter.next().unwrap().into_inner() {
                extends_or_impls.push(build_pathexpr(class));
            }
        }
    }

    let mut fields = Vec::new();
    let mut methods = Vec::new();
    let mut cctor: Option<Box<AST>> = None;
    for class_item in iter {
        match class_item.as_rule() {
            Rule::CCtor => {
                if cctor.is_some() {
                    panic!("Duplicated static init found in class {}", name);
                } else {
                    cctor = Some(build_block(class_item.into_inner().next().unwrap()));
                }
            }
            Rule::StaticField => fields.push(build_field(class_item, true)),
            Rule::NonStaticField => fields.push(build_field(class_item, false)),
            Rule::Method => methods.push(build_method(class_item)),
            _ => unreachable!(),
        }
    }

    let ret = ASTStruct {
        name,
        flags: ClassFlags::from(u16::from(ClassFlag::Public)),
        custom_attribs,
        impls: extends_or_impls,
        generic_params,
        methods,
        fields,
        cctor: if let Some(v) = cctor {
            v
        } else {
            Box::new(AST::None)
        },
    };
    Box::new(match sem {
        Rule::KwClass => {
            unimplemented!()
        }
        Rule::KwStruct => AST::Struct(ret),
        Rule::KwInterface => {
            unimplemented!()
        }
        _ => unreachable!(),
    })
}

fn build_field(tree: Pair<Rule>, is_static: bool) -> Box<AST> {
    let mut iter = tree.into_inner();
    let id = build_id(iter.next().unwrap());
    let mut flag = FieldFlags::from(u16::from(FieldFlag::Public));
    if is_static {
        flag.set(FieldFlag::Static);
    }

    Box::new(AST::Field(id, flag, build_type(iter.next().unwrap())))
}

fn build_method(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let custom_attribs = build_attributes(&mut iter);

    // built-in attributes
    let mut attrib = MethodFlags::from(u16::from(MethodFlag::Public));
    let mut ast_attrib = ASTMethodFlags::default();
    loop {
        match iter.peek().unwrap().as_rule() {
            Rule::KwOverride => {
                iter.next();
                if ast_attrib.is(ASTMethodFlag::Override) {
                    panic!("Duplicated override modifier");
                } else {
                    ast_attrib.set(ASTMethodFlag::Override);
                }
            }
            Rule::KwVirtual => {
                iter.next();
                if attrib.is(MethodFlag::Abstract) {
                    panic!("Duplicated virtual modifier");
                } else {
                    attrib.set(MethodFlag::Abstract);
                }
            }
            _ => break,
        }
    }

    let name = build_id(iter.next().unwrap());

    let mut generic_params = Vec::new();
    if let Some(try_generic) = iter.peek() {
        if let Rule::GenericParamsDecl = try_generic.as_rule() {
            build_generic_params_decl(iter.next().unwrap(), &mut generic_params);
        }
    }

    let (ps, has_self) = build_params(iter.next().unwrap());
    if !has_self {
        attrib.set(MethodFlag::Static);
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
        flags: attrib,
        ast_flags: ast_attrib,
        custom_attribs,
        generic_params,
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
                    build_type(p_iter.next().unwrap()),
                )));
            }
            _ => unreachable!(),
        }
    } else {
        // no param
        return (vec![], false);
    }

    while let Some(p_id) = p_iter.next() {
        ps.push(Box::new(AST::Param(
            build_id(p_id),
            build_type(p_iter.next().unwrap()),
        )));
    }
    (ps, has_self)
}

fn build_pathexpr(tree: Pair<Rule>) -> ItemPathBuf {
    let mut ret = ItemPathBuf::new();
    for seg in tree.into_inner() {
        match seg.as_rule() {
            Rule::IdWithGenericParams => {
                let ast_generic_ps = build_id_with_generic_params(seg);
                ret.push_id_with_generic(
                    &ast_generic_ps.id,
                    if ast_generic_ps.generic_params.is_empty() {
                        None
                    } else {
                        Some(ast_generic_ps.generic_params)
                    },
                );
            }
            Rule::KwCrate => ret.push("crate"),
            Rule::KwSuper => ret.push("super"),
            _ => unreachable!(),
        };
    }
    ret
}

fn build_non_arr_type(tree: Pair<Rule>) -> Box<ASTType> {
    Box::new(match tree.as_rule() {
        Rule::KwBool => ASTType::Bool,
        Rule::KwChar => ASTType::Char,
        Rule::KwI32 => ASTType::I32,
        Rule::KwF64 => ASTType::F64,
        Rule::KwString => ASTType::String,
        Rule::KwUSelf => ASTType::UsrType({
            let mut path = ItemPathBuf::new();
            path.push("Self");
            path
        }),
        Rule::PathExpr => ASTType::UsrType(build_pathexpr(tree)),
        Rule::TupleType => ASTType::Tuple(tree.into_inner().map(build_type).collect()),
        _ => unreachable!(format!("Found {:?}", tree.as_rule())),
    })
}

/// tree: Type
fn build_type(tree: Pair<Rule>) -> Box<ASTType> {
    let mut iter = tree.into_inner();
    let mut ret = build_non_arr_type(iter.next().unwrap());

    while let Some(_) = iter.next() {
        iter.next().unwrap(); // RBracket
        ret = Box::new(ASTType::Arr(ret));
    }
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
                            AST::Let(pattern, ty, Box::new(AST::None))
                        }
                        Rule::Eq => AST::Let(pattern, ty, build_expr(iter.next().unwrap())),
                        _ => unreachable!(),
                    }
                }
                Rule::Eq => {
                    // no type but has init
                    AST::Let(
                        pattern,
                        Box::new(ASTType::None),
                        build_expr(iter.next().unwrap()),
                    )
                }
                Rule::Semi => {
                    // no type and no init
                    AST::Let(pattern, Box::new(ASTType::None), Box::new(AST::None))
                }
                _ => unreachable!(),
            })
        }
        Rule::UseStmt => {
            let mut iter = clause.into_inner();
            let path = build_pathexpr(iter.next().unwrap());
            let as_clause = iter.next().unwrap();
            let as_id = match as_clause.as_rule() {
                Rule::Id => Some(build_id(as_clause)),
                Rule::Semi => None,
                _ => unreachable!(),
            };

            Box::new(AST::Use(path, as_id))
        }
        _ => {
            let sub = build_expr(clause);
            if iter.next().is_some() {
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

    while let Some(op) = iter.next() {
        ret = Box::new(match op.as_rule() {
            Rule::EqEq => AST::OpEq(ret, build_comp_expr(iter.next().unwrap())),
            Rule::Ne => AST::OpNe(ret, build_comp_expr(iter.next().unwrap())),
            _ => unreachable!(),
        });
    }
    ret
}

fn build_comp_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_add_expr(iter.next().unwrap());

    while let Some(op) = iter.next() {
        ret = Box::new(match op.as_rule() {
            Rule::Le => AST::OpLe(ret, build_add_expr(iter.next().unwrap())),
            Rule::Lt => AST::OpLt(ret, build_add_expr(iter.next().unwrap())),
            Rule::Ge => AST::OpGe(ret, build_add_expr(iter.next().unwrap())),
            Rule::Gt => AST::OpGt(ret, build_add_expr(iter.next().unwrap())),
            _ => unreachable!(),
        });
    }
    ret
}

fn build_add_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_mul_expr(iter.next().unwrap());

    while let Some(op) = iter.next() {
        ret = Box::new(match op.as_rule() {
            Rule::Plus => AST::OpAdd(ret, build_mul_expr(iter.next().unwrap())),
            Rule::Minus => AST::OpSub(ret, build_mul_expr(iter.next().unwrap())),
            _ => unreachable!(),
        });
    }
    ret
}

fn build_mul_expr(tree: Pair<Rule>) -> Box<AST> {
    let mut iter = tree.into_inner();
    let mut ret = build_cast_expr(iter.next().unwrap());

    while let Some(op) = iter.next() {
        ret = Box::new(match op.as_rule() {
            Rule::Star => AST::OpMul(ret, build_cast_expr(iter.next().unwrap())),
            Rule::Slash => AST::OpDiv(ret, build_cast_expr(iter.next().unwrap())),
            Rule::Percent => AST::OpMod(ret, build_cast_expr(iter.next().unwrap())),
            _ => unreachable!(),
        });
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
                Rule::StructFieldsInitExpr => AST::OpNewStruct(
                    ty,
                    initializer
                        .into_inner()
                        .map(|field_init| {
                            let mut sub_iter = field_init.into_inner();
                            let field = build_id(sub_iter.next().unwrap());
                            Box::new(ASTStructFieldInit {
                                field,
                                value: if let Some(init_val) = sub_iter.next() {
                                    build_expr(init_val)
                                } else {
                                    Box::new(AST::None)
                                },
                            })
                        })
                        .collect(),
                ),
                Rule::ArrAccessExpr => {
                    let mut elem_ty = ty;
                    while let Some(_) = iter.next() {
                        iter.next().unwrap(); // RBracket
                        elem_ty = Box::new(ASTType::Arr(elem_ty));
                    }
                    AST::OpNewArr(
                        elem_ty,
                        build_expr(initializer.into_inner().next().unwrap()),
                    )
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
            Rule::Args => AST::OpCall(ret, rhs.into_inner().map(build_expr).collect()),
            Rule::ObjAccessExpr => AST::OpObjAccess(
                ret,
                build_id_with_generic_params(rhs.into_inner().next().unwrap()),
            ),
            Rule::PathAccessExpr => AST::OpStaticAccess(
                ret,
                build_id_with_generic_params(rhs.into_inner().next().unwrap()),
            ),
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
        Rule::IdWithGenericParams => {
            Box::new(AST::IdWithGenericParams(build_id_with_generic_params(tree)))
        }
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
        Rule::IntLiteral => AST::Int(
            tree.as_span()
                .as_str()
                .trim()
                .parse::<i32>()
                .unwrap_or_else(|_| {
                    panic!("Unable to parse \"{}\" as i32", tree.as_span().as_str())
                }),
        ),
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
        Rule::TuplePattern => AST::TuplePattern(tree.into_inner().map(build_pattern).collect()),
        _ => unreachable!(),
    })
}

fn build_id(tree: Pair<Rule>) -> String {
    assert_eq!(tree.as_rule(), Rule::Id);
    String::from(tree.as_span().as_str().trim())
}

/// Same as GenericParamDecl
fn build_id_with_generic_params(tree: Pair<Rule>) -> ASTIdWithGenericParam {
    let mut param_iter = tree.into_inner();
    let mut ret = ASTIdWithGenericParam {
        id: build_id(param_iter.next().unwrap()),
        generic_params: Vec::new(),
    };
    for param in param_iter {
        ret.generic_params.push(build_type(param));
    }
    ret
}
