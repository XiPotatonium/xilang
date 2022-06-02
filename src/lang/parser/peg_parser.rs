use super::super::ast;
use core::util::ItemPathBuf;
use core::CCTOR_NAME;

use std::fs;
use std::path::Path;

use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

#[derive(Parser)]
#[grammar = "lang/parser/grammar.pest"]
struct LRParser;

pub fn parse(path: &Path) -> Result<ast::File, Error<Rule>> {
    let code = fs::read_to_string(path).unwrap();

    let file = LRParser::parse(Rule::File, &code)?.next().unwrap();

    let mut ret = ast::File::default();
    for sub in file.into_inner() {
        match sub.as_rule() {
            Rule::EOI => break,
            Rule::UseStmt => ret.uses.push(parse_use_stmt(sub)),
            Rule::Struct => ret.structs.push(parse_struct(sub)),
            Rule::Interface => ret.interfaces.push(parse_interface(sub)),
            Rule::Enum => ret.enums.push(parse_enum(sub)),
            Rule::Fn => ret.fns.push(parse_fn(sub)),
            Rule::Global => ret.globals.push(parse_global(sub)),
            _ => unreachable!(),
        };
    }

    Ok(ret)
}

fn parse_use_stmt(tree: Pair<Rule>) -> ast::Use {
    let mut iter = tree.into_inner();
    let path = parse_pathexpr(iter.next().unwrap());
    let as_clause = iter.next().unwrap();
    let as_id = match as_clause.as_rule() {
        Rule::Id => Some(parse_id(as_clause)),
        Rule::Semi => None,
        _ => unreachable!(),
    };

    ast::Use { path, id: as_id }
}

fn parse_global(tree: Pair<Rule>) -> ast::Global {
    let mut iter = tree.into_inner();

    let id = parse_id(iter.next().unwrap());
    let ty = parse_type(iter.next().unwrap());
    let value = parse_log_or_expr(iter.next().unwrap());

    ast::Global { id, ty, value }
}

fn parse_attribs(iter: &mut Pairs<Rule>) -> Vec<ast::CustomAttrib> {
    let mut ret = Vec::new();
    while let Rule::CustomAttribLst = iter.peek().unwrap().as_rule() {
        for attr in iter.next().unwrap().into_inner() {
            if let Rule::CustomAttrib = attr.as_rule() {
                let mut attr_iter = attr.into_inner();
                let attr_id = parse_id(attr_iter.next().unwrap());
                let attr_args = if let Some(args) = attr_iter.next() {
                    Some(attr_iter.map(parse_expr).collect())
                } else {
                    None
                };
                ret.push(ast::CustomAttrib {
                    id: attr_id,
                    args: attr_args,
                });
            } else {
                unreachable!();
            }
        }
    }
    ret
}

fn parse_generic_decl(tree: Pair<Rule>) -> Vec<String> {
    return tree.into_inner().map(parse_id).collect();
}

fn parse_enum(tree: Pair<Rule>) -> ast::Enum {
    let mut iter = tree.into_inner();
    let custom_attribs = parse_attribs(&mut iter);

    let id = parse_id(iter.next().unwrap());

    let mut generic_decl: Option<Vec<String>> = Option::None;
    if let Some(try_generic_decl) = iter.peek() {
        if let Rule::GenericDecl = try_generic_decl.as_rule() {
            generic_decl = Some(parse_generic_decl(iter.next().unwrap()));
        }
    }

    let mut fields = Vec::new();
    let mut fns = Vec::new();
    for item in iter {
        match item.as_rule() {
            Rule::EnumField => fields.push(parse_enum_field(item)),
            Rule::Fn => fns.push(parse_fn(item)),
            _ => unreachable!(),
        }
    }

    ast::Enum {
        id,
        custom_attribs,
        generic_decl,
        fns,
        fields,
    }
}

fn parse_enum_field(tree: Pair<Rule>) -> ast::EnumField {
    let mut iter = tree.into_inner();
    let id = parse_id(iter.next().unwrap());
    let ty = if let Some(ty) = iter.next() {
        Some(parse_type(iter.next().unwrap()))
    } else {
        None
    };

    ast::EnumField { id, ty }
}

fn parse_interface(tree: Pair<Rule>) -> ast::Interface {
    let mut iter = tree.into_inner();
    let custom_attribs = parse_attribs(&mut iter);

    let id = parse_id(iter.next().unwrap());

    let mut generic_decl: Option<Vec<String>> = Option::None;
    if let Some(try_generic_decl) = iter.peek() {
        if let Rule::GenericDecl = try_generic_decl.as_rule() {
            generic_decl = Some(parse_generic_decl(iter.next().unwrap()));
        }
    }

    let mut impls: Vec<ItemPathBuf> = Vec::new();
    if let Some(try_impls) = iter.peek() {
        if let Rule::Impls = try_impls.as_rule() {
            for class in iter.next().unwrap().into_inner() {
                impls.push(parse_pathexpr(class));
            }
        }
    }

    let fns = iter.map(parse_fn).collect();

    ast::Interface {
        id,
        custom_attribs,
        generic_decl,
        impls,
        fns,
    }
}

fn parse_struct(tree: Pair<Rule>) -> ast::Struct {
    let mut iter = tree.into_inner();
    let custom_attribs = parse_attribs(&mut iter);

    let id = parse_id(iter.next().unwrap());

    let mut generic_decl: Option<Vec<String>> = Option::None;
    if let Some(try_generic_decl) = iter.peek() {
        if let Rule::GenericDecl = try_generic_decl.as_rule() {
            generic_decl = Some(parse_generic_decl(iter.next().unwrap()));
        }
    }

    let mut impls: Vec<ItemPathBuf> = Vec::new();
    if let Some(try_impls) = iter.peek() {
        if let Rule::Impls = try_impls.as_rule() {
            for class in iter.next().unwrap().into_inner() {
                impls.push(parse_pathexpr(class));
            }
        }
    }

    let mut fields = Vec::new();
    let mut fns = Vec::new();
    for item in iter {
        match item.as_rule() {
            Rule::EnumField => fields.push(parse_field(item)),
            Rule::Fn => fns.push(parse_fn(item)),
            _ => unreachable!(),
        }
    }

    ast::Struct {
        id,
        custom_attribs,
        generic_decl,
        impls,
        fns,
        fields,
    }
}

fn parse_field(tree: Pair<Rule>) -> ast::Field {
    let mut iter = tree.into_inner();
    let id = parse_id(iter.next().unwrap());

    ast::Field {
        id,
        ty: parse_type(iter.next().unwrap()),
    }
}

fn parse_fn(tree: Pair<Rule>) -> ast::Fn {
    let mut iter = tree.into_inner();
    let custom_attribs = parse_attribs(&mut iter);

    // built-in attributes
    let mut attribs = FuncFlags::from(u16::from(FuncFlag::Public));

    let id = parse_id(iter.next().unwrap());

    let mut generic_decl: Option<Vec<String>> = Option::None;
    if let Some(try_generic_decl) = iter.peek() {
        if let Rule::GenericDecl = try_generic_decl.as_rule() {
            generic_decl = Some(parse_generic_decl(iter.next().unwrap()));
        }
    }

    let (ps, has_self) = parse_params(iter.next().unwrap());
    if !has_self {
        attribs.set(FuncFlag::Static);
    }

    let ty = if let Rule::Type = iter.peek().unwrap().as_rule() {
        parse_type(iter.next().unwrap())
    } else {
        ast::Type::None
    };

    let body = iter.next().unwrap();
    let body = match body.as_rule() {
        Rule::BlockExpr => Some(parse_block(body)),
        Rule::Semi => None,
        _ => unreachable!(),
    };

    ast::Fn {
        id,
        attribs,
        custom_attribs,
        generic_decl,
        ret: ty,
        ps,
        body,
    }
}

// Build parameters
fn parse_params(tree: Pair<Rule>) -> (Vec<ast::Param>, bool) {
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
                ps.push(ast::Param {
                    id: parse_id(p0),
                    ty: parse_type(p_iter.next().unwrap()),
                });
            }
            _ => unreachable!(),
        }
    } else {
        // no param
        return (vec![], false);
    }

    while let Some(p_id) = p_iter.next() {
        ps.push(ast::Param {
            id: parse_id(p_id),
            ty: parse_type(p_iter.next().unwrap()),
        });
    }
    (ps, has_self)
}

fn parse_pathexpr(tree: Pair<Rule>) -> ItemPathBuf {
    let mut ret = ItemPathBuf::default();
    for seg in tree.into_inner() {
        match seg.as_rule() {
            Rule::Id => ret.push(&parse_id(seg)),
            Rule::KwCrate => ret.push("crate"),
            Rule::KwSuper => ret.push("super"),
            Rule::KwLSelf => ret.push("self"),
            _ => unreachable!(),
        };
    }
    ret
}

fn parse_non_arr_type(tree: Pair<Rule>) -> Box<ast::Type> {
    Box::new(match tree.as_rule() {
        Rule::KwBool => ast::Type::Bool,
        Rule::KwChar => ast::Type::Char,
        Rule::KwI32 => ast::Type::I32,
        Rule::KwF64 => ast::Type::F64,
        Rule::KwISize => ast::Type::ISize,
        Rule::KwUSize => ast::Type::USize,
        Rule::KwStr => ast::Type::Str,
        Rule::KwUSelf => ast::Type::UsrType({
            let mut path = ItemPathBuf::default();
            path.push("Self");
            path
        }),
        Rule::Path => ast::Type::UsrType(parse_pathexpr(tree)),
        Rule::TupleType => ast::Type::Tuple(tree.into_inner().map(parse_type).collect()),
        _ => unreachable!(),
    })
}

/// tree: Type
fn parse_type(tree: Pair<Rule>) -> Box<ast::Type> {
    let mut iter = tree.into_inner();
    let mut ret = parse_non_arr_type(iter.next().unwrap());

    while let Some(_) = iter.next() {
        iter.next().unwrap(); // RBracket
        ret = Box::new(ast::Type::Arr(ret));
    }
    ret
}

fn parse_if(tree: Pair<Rule>) -> ast::If {
    let mut iter = tree.into_inner();
    let cond = parse_expr(iter.next().unwrap());
    let then = parse_block(iter.next().unwrap());
    let els = if let Some(els) = iter.next() {
        Some(Box::new(match els.as_rule() {
            Rule::IfExpr => ast::Expr::If(parse_if(els)),
            Rule::BlockExpr => ast::Expr::Block(parse_block(els)),
            _ => unreachable!(),
        }))
    } else {
        None
    };
    ast::If { cond, then, els }
}

fn parse_while(tree: Pair<Rule>) -> ast::While {
    let mut iter = tree.into_inner();
    let cond = parse_expr(iter.next().unwrap());
    let then = parse_block(iter.next().unwrap());
    ast::While { cond, then }
}

fn parse_loop(tree: Pair<Rule>) -> ast::Loop {
    ast::Loop {
        body: parse_block(tree.into_inner().next().unwrap()),
    }
}


fn parse_pattern(tree: Pair<Rule>) -> ast::Pattern {
    let tree = tree.into_inner().next().unwrap();
    match tree.as_rule() {
        Rule::Id => ast::Pattern::Id(parse_id(tree)),
        Rule::TuplePattern => {
            ast::Pattern::Tuple(tree.into_inner().map(|t| Box::new(parse_pattern(t))).collect())
        }
        _ => unreachable!(),
    }
}


fn parse_stmt(tree: Pair<Rule>) -> ast::Stmt {
    let mut iter = tree.into_inner();
    let clause = iter.next().unwrap();
    match clause.as_rule() {
        Rule::LetStmt => {
            let mut iter = clause.into_inner();
            let pattern = parse_pattern(iter.next().unwrap());
            let clause = iter.next().unwrap();
            ast::Stmt::Let(match clause.as_rule() {
                Rule::Type => {
                    // has type
                    let ty = Some(parse_type(clause));
                    let clause = iter.next().unwrap();
                    match clause.as_rule() {
                        // no init
                        Rule::Semi => ast::LetStmt {
                            pattern,
                            ty,
                            value: None,
                        },
                        Rule::Eq => ast::LetStmt {
                            pattern,
                            ty,
                            value: Some(parse_expr(iter.next().unwrap())),
                        },
                        _ => unreachable!(),
                    }
                }
                // no type but has init
                Rule::Eq => ast::LetStmt {
                    pattern,
                    ty: None,
                    value: Some(parse_expr(iter.next().unwrap())),
                },
                // no type and no init
                Rule::Semi => ast::LetStmt {
                    pattern,
                    ty: None,
                    value: None,
                },
                _ => unreachable!(),
            })
        }
        Rule::ContinueStmt => ast::Stmt::Continue,
        Rule::BreakStmt => ast::Stmt::Break(if let Some(ret_v) = tree.into_inner().next() {
                Some(parse_expr(ret_v))
            } else {
                None
            }),
        Rule::ReturnStmt => ast::Stmt::Return(
            if let Some(ret_v) = tree.into_inner().next() {
                Some(parse_expr(ret_v))
            } else {
                None
            },
        ),
        Rule::AssignStmt => {
            let mut iter = tree.into_inner();
            let lhs = parse_log_or_expr(iter.next().unwrap());
            iter.next(); // "="
            ast::Stmt::Assign(lhs, parse_log_or_expr(iter.next().unwrap()))
        },
        _ => {
            let sub = parse_expr(clause);
            if iter.next().is_none() {
                // no Semi
                // check if sub is a stmt
                sub
            }
            ast::Stmt::ExprStmt(sub)
        }
    }
}


fn parse_block(tree: Pair<Rule>) -> ast::Block {
    ast::Block {
        items: tree
            .into_inner()
            .map(|sub| match sub.as_rule() {
                Rule::Stmt => ast::BlockItem::Stmt(parse_stmt(sub)),
                _ => ast::BlockItem::Expr(parse_expr(sub)),
            })
            .collect(),
    }
}


fn parse_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    match tree.as_rule() {
        Rule::BlockExpr => Box::new(ast::Expr::Block(parse_block(tree))),
        Rule::LoopExpr => Box::new(ast::Expr::Loop(parse_loop(tree))),
        Rule::WhileExpr => Box::new(ast::Expr::While(parse_while(tree))),
        Rule::IfExpr => Box::new(ast::Expr::If(parse_if(tree))),
        Rule::LogOrExpr => parse_log_or_expr(tree),
        _ => unreachable!(),
    }
}


fn parse_log_or_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();
    let mut ret = parse_log_and_expr(iter.next().unwrap());

    for rhs in iter {
        // log or is left associative
        ret = Box::new(ast::Expr::OpLogOr(ret, parse_log_and_expr(rhs)));
    }
    ret
}

fn parse_log_and_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();
    let mut ret = parse_eq_expr(iter.next().unwrap());

    for rhs in iter {
        ret = Box::new(ast::Expr::OpLogAnd(ret, parse_eq_expr(rhs)));
    }
    ret
}

fn parse_eq_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();
    let mut ret = parse_comp_expr(iter.next().unwrap());

    while let Some(op) = iter.next() {
        ret = Box::new(match op.as_rule() {
            Rule::EqEq => ast::Expr::OpEq(ret, parse_comp_expr(iter.next().unwrap())),
            Rule::Ne => ast::Expr::OpNe(ret, parse_comp_expr(iter.next().unwrap())),
            _ => unreachable!(),
        });
    }
    ret
}

fn parse_comp_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();
    let mut ret = parse_add_expr(iter.next().unwrap());

    while let Some(op) = iter.next() {
        ret = Box::new(match op.as_rule() {
            Rule::Le => ast::Expr::OpLe(ret, parse_add_expr(iter.next().unwrap())),
            Rule::Lt => ast::Expr::OpLt(ret, parse_add_expr(iter.next().unwrap())),
            Rule::Ge => ast::Expr::OpGe(ret, parse_add_expr(iter.next().unwrap())),
            Rule::Gt => ast::Expr::OpGt(ret, parse_add_expr(iter.next().unwrap())),
            _ => unreachable!(),
        });
    }
    ret
}

fn parse_add_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();
    let mut ret = parse_mul_expr(iter.next().unwrap());

    while let Some(op) = iter.next() {
        ret = Box::new(match op.as_rule() {
            Rule::Plus => ast::Expr::OpAdd(ret, parse_mul_expr(iter.next().unwrap())),
            Rule::Minus => ast::Expr::OpSub(ret, parse_mul_expr(iter.next().unwrap())),
            _ => unreachable!(),
        });
    }
    ret
}

fn parse_mul_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();
    let mut ret = parse_cast_expr(iter.next().unwrap());

    while let Some(op) = iter.next() {
        ret = Box::new(match op.as_rule() {
            Rule::Star => ast::Expr::OpMul(ret, parse_cast_expr(iter.next().unwrap())),
            Rule::Slash => ast::Expr::OpDiv(ret, parse_cast_expr(iter.next().unwrap())),
            Rule::Percent => ast::Expr::OpMod(ret, parse_cast_expr(iter.next().unwrap())),
            _ => unreachable!(),
        });
    }
    ret
}

fn parse_cast_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();
    let mut ret = parse_unary_expr(iter.next().unwrap());

    for rhs in iter {
        ret = Box::new(ast::Expr::OpCast(parse_type(rhs), ret));
    }
    ret
}

fn parse_unary_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    // unary is right associative, iterate reversely
    let mut iter = tree.into_inner().rev();
    let mut ret = parse_call_expr(iter.next().unwrap());

    for op in iter {
        ret = Box::new(match op.as_rule() {
            Rule::Plus => ast::Expr::OpPos(ret),
            Rule::Not => ast::Expr::OpLogNot(ret),
            Rule::Minus => ast::Expr::OpNeg(ret),
            _ => unreachable!(),
        });
    }
    ret
}

fn parse_call_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();
    let mut ret = parse_primary_expr(iter.next().unwrap());

    for rhs in iter {
        ret = Box::new(match rhs.as_rule() {
            Rule::Args => ast::Expr::OpCall(ret, rhs.into_inner().map(parse_expr).collect()),
            Rule::ObjAccExpr => ast::Expr::OpObjAcc(ret, parse_id(rhs.into_inner().next().unwrap())),
            Rule::StaticAccExpr => {
                ast::Expr::OpStaticAcc(ret, parse_id(rhs.into_inner().next().unwrap()))
            }
            Rule::ArrAccExpr => ast::Expr::OpArrayAcc(ret, parse_expr(rhs.into_inner().next().unwrap())),
            _ => unreachable!(),
        });
    }
    ret
}

fn parse_primary_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let tree = tree.into_inner().next().unwrap();
    match tree.as_rule() {
        Rule::ParenExpr => parse_expr(tree.into_inner().next().unwrap()),
        Rule::LiteralExpr => Box::new(ast::Expr::Literal(parse_literal(tree))),
        Rule::KwLSelf => Box::new(ast::Expr::Id(String::from("self"))),
        Rule::Id => Box::new(ast::Expr::Id(parse_id(tree))),
        Rule::Type => Box::new(ast::Expr::Type(parse_type(tree))),
        Rule::NewExpr => parse_new_expr(tree),
        // Actually only expr with block
        _ => parse_expr(tree),
    }
}

fn parse_new_expr(tree: Pair<Rule>) -> Box<ast::Expr> {
    let mut iter = tree.into_inner();

    let ret = iter.next().unwrap();
    match ret.as_rule() {
        Rule::CallExpr => parse_call_expr(ret),
        Rule::Type => {
            let ty = parse_type(ret);
            let initializer = iter.next().unwrap();
            Box::new(match initializer.as_rule() {
                Rule::StructInitExpr => ast::Expr::OpNewStruct(
                    ty,
                    initializer
                        .into_inner()
                        .map(|field_init| {
                            let mut sub_iter = field_init.into_inner();
                            let field = parse_id(sub_iter.next().unwrap());
                            ast::FieldInit {
                                field,
                                value: if let Some(init_val) = sub_iter.next() {
                                    Some(parse_expr(init_val))
                                } else {
                                    None
                                },
                            }
                        })
                        .collect(),
                ),
                Rule::ArrAccExpr => {
                    let mut elem_ty = ty;
                    while let Some(_) = iter.next() {
                        iter.next().unwrap(); // RBracket
                        elem_ty = ast::Type::Arr(Box::new(elem_ty));
                    }
                    ast::Expr::OpNewArr(
                        elem_ty,
                        parse_expr(initializer.into_inner().next().unwrap()),
                    )
                }
                _ => unreachable!(),
            })
        }
        _ => unreachable!(),
    }
}

fn parse_literal(tree: Pair<Rule>) -> ast::Literal {
    let tree = tree.into_inner().next().unwrap();
    match tree.as_rule() {
        Rule::KwTrue => ast::Literal::Bool(true),
        Rule::KwFalse => ast::Literal::Bool(false),
        Rule::DecIntLiteral => ast::Literal::I32Int(
            tree.as_span()
                .as_str()
                .trim()
                .parse::<i32>()
                .unwrap_or_else(|_| {
                    panic!("Unable to parse \"{}\" as i32", tree.as_span().as_str())
                }),
        ),
        Rule::FloatLiteral => {
            ast::Literal::Float(tree.as_span().as_str().trim().parse::<f64>().unwrap())
        }
        Rule::StrLiteral => {
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
            ast::Literal::Str(s)
        }
        Rule::CharLiteral => {
            let mut chars = tree.as_span().as_str().trim().chars();
            chars.next(); // skip first '\''
            let ch = ast::Literal::Char(match chars.next().unwrap() {
                '\'' => panic!("Empty char literal"),
                '\\' => chars.next().unwrap(),
                c => c,
            });
            match chars.next().expect("Invalid char literal") {
                '\'' => (),
                _ => panic!("Too many chars in char literal"),
            }
            ch
        }
        _ => unreachable!(),
    }
}

fn parse_id(tree: Pair<Rule>) -> String {
    debug_assert_eq!(tree.as_rule(), Rule::Id);
    String::from(tree.as_span().as_str().trim())
}
