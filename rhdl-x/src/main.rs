use anyhow::Result;
use anyhow::{anyhow, bail};
use itertools::Itertools;
use la_arena::Idx;
use ra_ap_base_db::SourceDatabaseExt;
use ra_ap_hir::db::{DefDatabase, ExpandDatabase, HirDatabase};
use ra_ap_hir::known::ge;
use ra_ap_hir::{
    Adt, AssocItem, Crate, DefWithBody, Function, HasAttrs, HirDisplay, Module, ModuleDef, Name,
    TypeRef,
};
use ra_ap_hir_def::body::{Body, BodySourceMap};
use ra_ap_hir_def::hir::{Binding, Expr, ExprId, Literal, Pat, Statement};
use ra_ap_hir_ty::{Interner, TyExt, TypeFlags};
use ra_ap_ide::{Analysis, LineCol, RootDatabase};
use ra_ap_ide_db::LineIndexDatabase;
use ra_ap_load_cargo::{load_workspace, LoadCargoConfig};
use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use ra_ap_syntax::ast::{BinaryOp, UnaryOp};
use ra_ap_syntax::AstNode;
use ra_ap_vfs::{AbsPathBuf, Vfs, VfsPath};
use std::collections::{BTreeMap, HashSet};
use std::env::args;
use std::path::Path;
use std::vec;
use std::{collections::HashMap, fmt::Display};
use syn::token::In;
use utils::IndentingFormatter;

pub mod utils;

fn expr_syntax_range(
    db: &RootDatabase,
    vfs: &Vfs,
    sm: &BodySourceMap,
    expr_id: ExprId,
) -> Option<(VfsPath, LineCol, LineCol)> {
    let src = sm.expr_syntax(expr_id);
    if let Ok(src) = src {
        let root = db.parse_or_expand(src.file_id);
        let node = src.map(|e| e.to_node(&root).syntax().clone());
        let original_range = node.as_ref().original_file_range(db);
        let path = vfs.file_path(original_range.file_id);
        let line_index = db.line_index(original_range.file_id);
        let text_range = original_range.range;
        let (start, end) = (
            line_index.line_col(text_range.start()),
            line_index.line_col(text_range.end()),
        );
        Some((path, start, end))
    } else {
        None
    }
}

fn full_module_name(db: &RootDatabase, module: &Module) -> String {
    let full_name = move || {
        module
            .krate()
            .display_name(db)
            .map(|it| it.canonical_name().to_string())
            .into_iter()
            .chain(
                module
                    .path_to_root(db)
                    .into_iter()
                    .filter_map(|it| it.name(db))
                    .rev()
                    .chain(Some(module.name(db).unwrap_or(Name::missing())))
                    .map(|it| it.display(db).to_string()),
            )
            .join("::")
    };
    full_name()
}

fn full_adt_name(db: &RootDatabase, adt: &Adt) -> String {
    let module = adt.module(db);
    let full_name = move || {
        module
            .krate()
            .display_name(db)
            .map(|it| it.canonical_name().to_string())
            .into_iter()
            .chain(
                module
                    .path_to_root(db)
                    .into_iter()
                    .filter_map(|it| it.name(db))
                    .rev()
                    .chain(Some(adt.name(db)))
                    .map(|it| it.display(db).to_string()),
            )
            .join("::")
    };
    full_name()
}

fn full_function_name(db: &RootDatabase, func: &Function) -> String {
    let module = func.module(db);
    let full_name = move || {
        module
            .krate()
            .display_name(db)
            .map(|it| it.canonical_name().to_string())
            .into_iter()
            .chain(
                module
                    .path_to_root(db)
                    .into_iter()
                    .filter_map(|it| it.name(db))
                    .rev()
                    .chain(Some(func.name(db)))
                    .map(|it| it.display(db).to_string()),
            )
            .join("::")
    };
    full_name()
}

fn run_inference(db: &RootDatabase, vfs: &Vfs, bodies: &[DefWithBody], only: Option<String>) {
    let mut num_exprs = 0;
    let mut num_exprs_unknown = 0;
    let mut num_exprs_partially_unknown = 0;
    let mut num_expr_type_mismatches = 0;
    let mut num_pats = 0;
    let mut num_pats_unknown = 0;
    let mut num_pats_partially_unknown = 0;
    let mut num_pat_type_mismatches = 0;
    for &body_id in bodies {
        let name = body_id.name(db).unwrap_or_else(Name::missing);
        let module = body_id.module(db);
        let full_name = move || {
            module
                .krate()
                .display_name(db)
                .map(|it| it.canonical_name().to_string())
                .into_iter()
                .chain(
                    module
                        .path_to_root(db)
                        .into_iter()
                        .filter_map(|it| it.name(db))
                        .rev()
                        .chain(Some(body_id.name(db).unwrap_or_else(Name::missing)))
                        .map(|it| it.display(db).to_string()),
                )
                .join("::")
        };
        let full_name = full_name();
        if let Some(only_name) = only.as_deref() {
            if !full_name.contains(only_name) {
                continue;
            }
        }
        eprintln!("Type inference for function : {full_name}");
        let (body, sm) = db.body_with_source_map(body_id.into());
        let inference_result = db.infer(body_id.into());

        /*
        for (expr_id, _) in body.exprs.iter() {
            let ty = &inference_result[expr_id];
            num_exprs += 1;
            let unknown_or_partial = if ty.is_unknown() {
                num_exprs_unknown += 1;
                if let Some((path, start, end)) = expr_syntax_range(db, vfs, &sm, expr_id) {
                    eprintln!(
                        "{} {}:{}-{}:{}: Unknown type",
                        path,
                        start.line + 1,
                        start.col,
                        end.line + 1,
                        end.col,
                    );
                } else {
                    eprintln!("{}: Unknown type", name.display(db));
                };
                true
            } else {
                let is_partially_unknown = ty.data(Interner).flags.contains(TypeFlags::HAS_ERROR);
                if is_partially_unknown {
                    num_exprs_partially_unknown += 1;
                }
                is_partially_unknown
            };
            if only.is_some() {
                // in super-verbose mode for just one function, we print every single expression
                if let Some((_, start, end)) = expr_syntax_range(db, vfs, &sm, expr_id) {
                    eprintln!(
                        "{}:{}-{}:{}: {}",
                        start.line + 1,
                        start.col,
                        end.line + 1,
                        end.col,
                        ty.display(db)
                    );
                    if let Some(adt) = ty.as_adt() {
                        eprintln!("adt is {:?}", adt);
                    }
                } else {
                    eprintln!("unknown location: {}", ty.display(db));
                }
            }
        }
            */
    }
}

// Try and reproduce batch analysis behavior from Rust Analyzer.
fn run_analysis_batch(path: &Path) -> anyhow::Result<()> {
    let cargo_config = CargoConfig {
        sysroot: Some(RustLibSource::Discover),
        ..CargoConfig::default()
    };
    let no_progress = &|_| ();
    let path = AbsPathBuf::assert(std::env::current_dir().unwrap().join(path));
    let manifest = ProjectManifest::discover_single(&path).unwrap();
    let mut workspace = ProjectWorkspace::load(manifest, &cargo_config, no_progress).unwrap();
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ra_ap_load_cargo::ProcMacroServerChoice::Sysroot,
        prefill_caches: false,
    };
    let bs = workspace.run_build_scripts(&cargo_config, no_progress)?;
    workspace.set_build_scripts(bs);
    let (host, vfs, _proc_macro) =
        load_workspace(workspace, &cargo_config.extra_env, &load_config)?;
    let db = host.raw_database();
    eprintln!("db is {:?}", db);
    let krates = Crate::all(db);
    let mut visit_queue = krates
        .iter()
        .map(|krate| krate.root_module())
        .collect::<Vec<_>>();
    let mut visited_modules = HashSet::new();
    let mut bodies = Vec::new();
    let mut adts = Vec::new();
    let mut funcs = Vec::new();
    let mut consts = Vec::new();
    while let Some(module) = visit_queue.pop() {
        if visited_modules.insert(module) {
            eprintln!("Visit module: {}", full_module_name(db, &module));
            visit_queue.extend(module.children(db));
            for decl in module.declarations(db) {
                match decl {
                    ModuleDef::Function(f) => {
                        funcs.push(f);
                        bodies.push(DefWithBody::from(f));
                    }
                    ModuleDef::Adt(a) => {
                        if let Adt::Enum(e) = a {
                            for v in e.variants(db) {
                                bodies.push(DefWithBody::from(v));
                            }
                        }
                        adts.push(a);
                    }
                    ModuleDef::Const(c) => {
                        consts.push(c);
                        bodies.push(DefWithBody::from(c));
                    }
                    ModuleDef::Static(s) => {
                        bodies.push(DefWithBody::from(s));
                    }
                    _ => (),
                }

                for impl_def in module.impl_defs(db) {
                    for item in impl_def.items(db) {
                        match item {
                            AssocItem::Function(f) => bodies.push(DefWithBody::from(f)),
                            AssocItem::Const(c) => {
                                bodies.push(DefWithBody::from(c));
                                consts.push(c);
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
    }
    /*    for body in &bodies {
        eprintln!("body is {:?} -> name {:?}", body, body.name(db));
    }*/
    /*
    for adt in &adts {
        let full_adt_name = full_adt_name(db, adt);
        if full_adt_name.contains("rhdl") {
            eprintln!("adt is {:?} -> name {}", adt, full_adt_name);
        }
    }*/
    //run_inference(db, &vfs, &bodies, None);
    // Get the function named "demo::do_stuff_a", and then walk it's AST.
    if let Some(test_function) = bodies
        .iter()
        .filter_map(|b| match b {
            DefWithBody::Function(f) => Some(f),
            _ => None,
        })
        .find(|f| full_function_name(db, f).contains("demo::do_stuff"))
    {
        eprintln!("test_function is {:?}", test_function);
        let fn_ty = test_function.ty(db);
        test_function.params_without_self(db).iter().for_each(|p| {
            eprintln!("param is {:?}", p);
        });
        let return_ty = test_function.ret_type(db);
        eprintln!("test function return type is {:?}", return_ty);
        // Get the body of the function.
        // This is probably backwards
        let def_with_body = DefWithBody::from(*test_function);
        let (body, sm) = db.body_with_source_map(def_with_body.into());
        let main_expr = &body.exprs[body.body_expr];
        pretty_print(db, &body, body.body_expr);
    }

    Ok(())
}

fn pretty_print(db: &RootDatabase, body: &Body, expr: Idx<Expr>) {
    let mut f = IndentingFormatter::default();
    print_expr(&mut f, db, body, expr);
    eprintln!("pretty print:\n{}", f.buffer());
}

fn print_binding(
    f: &mut IndentingFormatter,
    db: &RootDatabase,
    body: &Body,
    binding: Idx<Binding>,
) {
    let binding = &body.bindings[binding];
    let name = binding.name.as_str();
    if let Some(name) = name {
        f.write(&format!("{} ", name));
    }
}

fn print_pat(f: &mut IndentingFormatter, db: &RootDatabase, body: &Body, pat: Idx<Pat>) {
    match &body.pats[pat] {
        Pat::Bind { id, subpat } => {
            print_binding(f, db, body, *id);
            if let Some(subpat) = subpat {
                f.write(" @ ");
                print_pat(f, db, body, *subpat);
            }
        }
        Pat::Tuple { args, ellipsis } => {
            f.write("(");
            for pat in args.iter() {
                print_pat(f, db, body, *pat);
                f.write(",");
            }
            f.write(")");
        }
        _ => {}
    }
}

fn print_expr(f: &mut IndentingFormatter, db: &RootDatabase, body: &Body, expr: Idx<Expr>) {
    match &body.exprs[expr] {
        Expr::Field { expr, name } => {
            print_expr(f, db, body, *expr);
            f.write(&format!(".{}", name.as_str().unwrap()));
        }
        Expr::Literal(lit) => match lit {
            Literal::Bool(b) => {
                f.write(&format!("{}", b));
            }
            Literal::Int(i, ..) => {
                f.write(&format!("{}", i));
            }
            _ => {}
        },
        Expr::Path(path) => {
            f.write(&format!("{}", path.display(db)));
        }
        Expr::Tuple { exprs, .. } => {
            f.write("(");
            for expr in exprs.iter() {
                print_expr(f, db, body, *expr);
                f.write(",");
            }
            f.write(")");
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            f.write("if ");
            print_expr(f, db, body, *condition);
            f.write(" ");
            print_expr(f, db, body, *then_branch);
            if let Some(else_branch) = else_branch {
                f.write(" else ");
                print_expr(f, db, body, *else_branch);
            } else {
                f.write("\n");
            }
        }
        Expr::Block {
            id,
            statements,
            tail,
            label,
        } => {
            f.write("{\n");
            for stmt in statements.iter() {
                print_statement(f, db, body, stmt);
            }
            if let Some(tail) = tail {
                print_expr(f, db, body, *tail);
                f.write(" // <- block tail\n")
            }
            f.write("}");
        }
        Expr::BinaryOp { lhs, rhs, op } => {
            print_expr(f, db, body, *lhs);
            if let Some(op) = op {
                print_binary_op(f, db, body, op);
            }
            print_expr(f, db, body, *rhs);
        }
        Expr::Call {
            callee,
            args,
            is_assignee_expr,
        } => {
            print_expr(f, db, body, *callee);
            f.write("(");
            for arg in args.iter() {
                print_expr(f, db, body, *arg);
                f.write(",");
            }
            f.write(")");
        }
        Expr::Index { base, index } => {
            print_expr(f, db, body, *base);
            f.write("[");
            print_expr(f, db, body, *index);
            f.write("]");
        }
        Expr::UnaryOp { expr, op } => {
            match op {
                UnaryOp::Not => f.write("!"),
                UnaryOp::Neg => f.write("-"),
                _ => {}
            }
            print_expr(f, db, body, *expr);
        }
        Expr::MethodCall {
            receiver,
            method_name,
            args,
            generic_args,
        } => {
            print_expr(f, db, body, *receiver);
            f.write(&format!(".{}", method_name.as_str().unwrap()));
            if let Some(generic_args) = generic_args {
                f.write("<");
                for arg in generic_args.args.iter() {
                    f.write(&format!("{}", arg.display(db)));
                    f.write(",");
                }
                f.write(">");
            }
            f.write("(");
            for arg in args.iter() {
                print_expr(f, db, body, *arg);
                f.write(",");
            }
            f.write(")");
        }
        _ => {
            eprintln!("expr is {:?}", body.exprs[expr]);
        }
    }
}

fn print_binary_op(f: &mut IndentingFormatter, db: &RootDatabase, body: &Body, op: &BinaryOp) {
    match op {
        BinaryOp::ArithOp(op) => f.write(&format!(" {} ", op)),
        BinaryOp::CmpOp(op) => f.write(&format!(" {} ", op)),
        BinaryOp::LogicOp(op) => f.write(&format!(" {} ", op)),
        BinaryOp::Assignment { op: Some(op) } => f.write(&format!(" {}= ", op)),
        BinaryOp::Assignment { op: None } => f.write(" = "),
    }
}

fn print_type(f: &mut IndentingFormatter, db: &RootDatabase, body: &Body, type_ref: &TypeRef) {
    f.write(&format!("{}", type_ref.display(db)))
}

fn print_statement(f: &mut IndentingFormatter, db: &RootDatabase, body: &Body, stmt: &Statement) {
    match stmt {
        Statement::Let {
            pat,
            initializer,
            type_ref,
            ..
        } => {
            f.write("let ");
            print_pat(f, db, body, *pat);
            if let Some(type_ref) = type_ref {
                f.write(": ");
                print_type(f, db, body, type_ref);
            }
            if let Some(initializer) = initializer {
                f.write(" = ");
                print_expr(f, db, body, *initializer);
            }
            f.write(";\n");
        }
        Statement::Expr { expr, has_semi } => {
            print_expr(f, db, body, *expr);
            if *has_semi {
                f.write(";\n");
            } else {
                f.write("\n");
            }
        }
    }
}

// A couple of questions...
// 1. Need to walk the AST and get expressions/etc.
// 2. Assuming these have been type inferred, we should be able
//    to pretty print the code with the types included (as I had done before)

fn main() {
    let path = args().nth(1).unwrap();
    let path = Path::new(&path);
    run_analysis_batch(path).unwrap();
}
