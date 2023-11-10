use anyhow::Result;
use anyhow::{anyhow, bail};
use itertools::Itertools;
use ra_ap_base_db::SourceDatabaseExt;
use ra_ap_hir::db::{DefDatabase, ExpandDatabase, HirDatabase};
use ra_ap_hir::{Adt, Crate, DefWithBody, HasAttrs, HirDisplay, ModuleDef, Name};
use ra_ap_hir_def::body::BodySourceMap;
use ra_ap_hir_def::hir::ExprId;
use ra_ap_hir_ty::{Interner, TyExt, TypeFlags};
use ra_ap_ide::{Analysis, LineCol, RootDatabase};
use ra_ap_ide_db::LineIndexDatabase;
use ra_ap_load_cargo::{load_workspace, LoadCargoConfig};
use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace};
use ra_ap_syntax::AstNode;
use ra_ap_vfs::{AbsPathBuf, Vfs, VfsPath};
use rhdl_bits::bits;
use rhdl_bits::{alias::*, Bits};
use std::collections::{BTreeMap, HashSet};
use std::env::args;
use std::path::Path;
use std::vec;
use std::{collections::HashMap, fmt::Display};

#[derive(PartialEq, Copy, Clone)]
#[must_use]
pub struct Foo {
    a: u8,
    b: u16,
    c: [u8; 3],
}

#[derive(PartialEq, Copy, Clone)]
#[must_use]
pub enum NooState {
    Init,
    Run(u8, u8, u8),
    Walk { foo: u8 },
    Boom,
}

mod ty_macro;

#[must_use]
fn do_stuff(mut a: Foo, mut s: NooState) -> u8 {
    let k = {
        bits::<12>(4);
        bits::<12>(6)
    };
    let mut a: Foo = a;
    let mut s: NooState = s;
    let q = if a.a > 0 { bits::<12>(3) } else { bits(0) };
    let y = bits::<12>(72);
    let t2 = (y, y);
    let q: u8 = 4;
    let z = a.c;
    let w = (a, a);
    a.c[1] = q + 3;
    a.c = [0; 3];
    a.c = [1, 2, 3];
    let q = (1, (0, 5), 6);
    let (q0, (q1, q1b), q2): (u8, (u8, u8), u16) = q; // Tuple destructuring
    a.a = 2 + 3 + q1;
    let z;
    if 1 > 3 {
        z = bits::<4>(2);
    } else {
        z = bits::<4>(5);
    }
    a.b = {
        7 + 9;
        5 + !8
    };
    a.a = if 1 > 3 {
        7
    } else {
        {
            a.b = 1;
            a.b = 4;
        }
        9
    };
    let g = 1 > 2;
    let h = 3 != 4;
    let mut i = g && h;
    if z == bits::<4>(3) {
        i = false;
    }
    let c = match z {
        Bits(1) => 2,
        Bits(2) => 3,
        Bits(3) => {
            a.a = 4;
            4
        }
        _ => 6,
    };
    let d = match s {
        NooState::Init => {
            a.a = 1;
            NooState::Run(1, 2, 3)
        }
        NooState::Run(x, _, y) => {
            a.a = x + y;
            NooState::Walk { foo: 7 }
        }
        NooState::Walk { foo: x } => {
            a.a = x;
            NooState::Boom
        }
        NooState::Boom => {
            a.a = a.a + 3;
            NooState::Init
        }
        _ => {
            a.a = 2;
            NooState::Boom
        }
    };
    3
}

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
        eprintln!("Full name of body -> {}", full_name());
        if let Some(only_name) = only.as_deref() {
            if name.display(db).to_string() != only_name && full_name() != only_name {
                continue;
            }
        }
        let (body, sm) = db.body_with_source_map(body_id.into());
        let inference_result = db.infer(body_id.into());
        eprintln!("Inference result -> {:?}", inference_result);

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
                } else {
                    eprintln!("unknown location: {}", ty.display(db));
                }
            }
        }
        /*
            let msg = move || {
                if verbosity.is_verbose() {
                    let source = match body_id {
                        DefWithBody::Function(it) => it.source(db).map(|it| it.syntax().cloned()),
                        DefWithBody::Static(it) => it.source(db).map(|it| it.syntax().cloned()),
                        DefWithBody::Const(it) => it.source(db).map(|it| it.syntax().cloned()),
                        DefWithBody::Variant(it) => it.source(db).map(|it| it.syntax().cloned()),
                        DefWithBody::InTypeConst(_) => unimplemented!(),
                    };
                    if let Some(src) = source {
                        let original_file = src.file_id.original_file(db);
                        let path = vfs.file_path(original_file);
                        let syntax_range = src.value.text_range();
                        format!("processing: {} ({} {:?})", full_name(), path, syntax_range)
                    } else {
                        format!("processing: {}", full_name())
                    }
                } else {
                    format!("processing: {}", full_name())
                }
            };
            if verbosity.is_spammy() {
                bar.println(msg());
            }
            bar.set_message(msg);
            let (body, sm) = db.body_with_source_map(body_id.into());
            let inference_result = db.infer(body_id.into());

            // region:expressions
            let (previous_exprs, previous_unknown, previous_partially_unknown) =
                (num_exprs, num_exprs_unknown, num_exprs_partially_unknown);
            for (expr_id, _) in body.exprs.iter() {
                let ty = &inference_result[expr_id];
                num_exprs += 1;
                let unknown_or_partial = if ty.is_unknown() {
                    num_exprs_unknown += 1;
                    if verbosity.is_spammy() {
                        if let Some((path, start, end)) = expr_syntax_range(db, vfs, &sm, expr_id) {
                            bar.println(format!(
                                "{} {}:{}-{}:{}: Unknown type",
                                path,
                                start.line + 1,
                                start.col,
                                end.line + 1,
                                end.col,
                            ));
                        } else {
                            bar.println(format!("{}: Unknown type", name.display(db)));
                        }
                    }
                    true
                } else {
                    let is_partially_unknown = ty.data(Interner).flags.contains(TypeFlags::HAS_ERROR);
                    if is_partially_unknown {
                        num_exprs_partially_unknown += 1;
                    }
                    is_partially_unknown
                };
                if self.only.is_some() && verbosity.is_spammy() {
                    // in super-verbose mode for just one function, we print every single expression
                    if let Some((_, start, end)) = expr_syntax_range(db, vfs, &sm, expr_id) {
                        bar.println(format!(
                            "{}:{}-{}:{}: {}",
                            start.line + 1,
                            start.col,
                            end.line + 1,
                            end.col,
                            ty.display(db)
                        ));
                    } else {
                        bar.println(format!("unknown location: {}", ty.display(db)));
                    }
                }
                if unknown_or_partial && self.output == Some(OutputFormat::Csv) {
                    println!(
                        r#"{},type,"{}""#,
                        location_csv_expr(db, vfs, &sm, expr_id),
                        ty.display(db)
                    );
                }
                if let Some(mismatch) = inference_result.type_mismatch_for_expr(expr_id) {
                    num_expr_type_mismatches += 1;
                    if verbosity.is_verbose() {
                        if let Some((path, start, end)) = expr_syntax_range(db, vfs, &sm, expr_id) {
                            bar.println(format!(
                                "{} {}:{}-{}:{}: Expected {}, got {}",
                                path,
                                start.line + 1,
                                start.col,
                                end.line + 1,
                                end.col,
                                mismatch.expected.display(db),
                                mismatch.actual.display(db)
                            ));
                        } else {
                            bar.println(format!(
                                "{}: Expected {}, got {}",
                                name.display(db),
                                mismatch.expected.display(db),
                                mismatch.actual.display(db)
                            ));
                        }
                    }
                    if self.output == Some(OutputFormat::Csv) {
                        println!(
                            r#"{},mismatch,"{}","{}""#,
                            location_csv_expr(db, vfs, &sm, expr_id),
                            mismatch.expected.display(db),
                            mismatch.actual.display(db)
                        );
                    }
                }
            }
            if verbosity.is_spammy() {
                bar.println(format!(
                    "In {}: {} exprs, {} unknown, {} partial",
                    full_name(),
                    num_exprs - previous_exprs,
                    num_exprs_unknown - previous_unknown,
                    num_exprs_partially_unknown - previous_partially_unknown
                ));
            }
            // endregion:expressions

            // region:patterns
            let (previous_pats, previous_unknown, previous_partially_unknown) =
                (num_pats, num_pats_unknown, num_pats_partially_unknown);
            for (pat_id, _) in body.pats.iter() {
                let ty = &inference_result[pat_id];
                num_pats += 1;
                let unknown_or_partial = if ty.is_unknown() {
                    num_pats_unknown += 1;
                    if verbosity.is_spammy() {
                        if let Some((path, start, end)) = pat_syntax_range(db, vfs, &sm, pat_id) {
                            bar.println(format!(
                                "{} {}:{}-{}:{}: Unknown type",
                                path,
                                start.line + 1,
                                start.col,
                                end.line + 1,
                                end.col,
                            ));
                        } else {
                            bar.println(format!("{}: Unknown type", name.display(db)));
                        }
                    }
                    true
                } else {
                    let is_partially_unknown = ty.data(Interner).flags.contains(TypeFlags::HAS_ERROR);
                    if is_partially_unknown {
                        num_pats_partially_unknown += 1;
                    }
                    is_partially_unknown
                };
                if self.only.is_some() && verbosity.is_spammy() {
                    // in super-verbose mode for just one function, we print every single pattern
                    if let Some((_, start, end)) = pat_syntax_range(db, vfs, &sm, pat_id) {
                        bar.println(format!(
                            "{}:{}-{}:{}: {}",
                            start.line + 1,
                            start.col,
                            end.line + 1,
                            end.col,
                            ty.display(db)
                        ));
                    } else {
                        bar.println(format!("unknown location: {}", ty.display(db)));
                    }
                }
                if unknown_or_partial && self.output == Some(OutputFormat::Csv) {
                    println!(
                        r#"{},type,"{}""#,
                        location_csv_pat(db, vfs, &sm, pat_id),
                        ty.display(db)
                    );
                }
                if let Some(mismatch) = inference_result.type_mismatch_for_pat(pat_id) {
                    num_pat_type_mismatches += 1;
                    if verbosity.is_verbose() {
                        if let Some((path, start, end)) = pat_syntax_range(db, vfs, &sm, pat_id) {
                            bar.println(format!(
                                "{} {}:{}-{}:{}: Expected {}, got {}",
                                path,
                                start.line + 1,
                                start.col,
                                end.line + 1,
                                end.col,
                                mismatch.expected.display(db),
                                mismatch.actual.display(db)
                            ));
                        } else {
                            bar.println(format!(
                                "{}: Expected {}, got {}",
                                name.display(db),
                                mismatch.expected.display(db),
                                mismatch.actual.display(db)
                            ));
                        }
                    }
                    if self.output == Some(OutputFormat::Csv) {
                        println!(
                            r#"{},mismatch,"{}","{}""#,
                            location_csv_pat(db, vfs, &sm, pat_id),
                            mismatch.expected.display(db),
                            mismatch.actual.display(db)
                        );
                    }
                }
            }
            if verbosity.is_spammy() {
                bar.println(format!(
                    "In {}: {} pats, {} unknown, {} partial",
                    full_name(),
                    num_pats - previous_pats,
                    num_pats_unknown - previous_unknown,
                    num_pats_partially_unknown - previous_partially_unknown
                ));
            }
            // endregion:patterns
            bar.inc(1);
        }

        bar.finish_and_clear();
        let inference_time = inference_sw.elapsed();
        eprintln!(
            "  exprs: {}, ??ty: {} ({}%), ?ty: {} ({}%), !ty: {}",
            num_exprs,
            num_exprs_unknown,
            percentage(num_exprs_unknown, num_exprs),
            num_exprs_partially_unknown,
            percentage(num_exprs_partially_unknown, num_exprs),
            num_expr_type_mismatches
        );
        eprintln!(
            "  pats: {}, ??ty: {} ({}%), ?ty: {} ({}%), !ty: {}",
            num_pats,
            num_pats_unknown,
            percentage(num_pats_unknown, num_pats),
            num_pats_partially_unknown,
            percentage(num_pats_partially_unknown, num_pats),
            num_pat_type_mismatches
        );
        eprintln!("{:<20} {}", "Inference:", inference_time);
        report_metric("unknown type", num_exprs_unknown, "#");
        report_metric("type mismatches", num_expr_type_mismatches, "#");
        report_metric("pattern unknown type", num_pats_unknown, "#");
        report_metric("pattern type mismatches", num_pat_type_mismatches, "#");
        report_metric(
            "inference time",
            inference_time.time.as_millis() as u64,
            "ms",
        );
        */
    }
}

// Try and reproduce batch analysis behavior from Rust Analyzer.
fn run_analysis_batch(path: &Path) -> anyhow::Result<()> {
    let mut cargo_config = CargoConfig::default();
    cargo_config.sysroot = None;
    let no_progress = &|_| ();
    let path = AbsPathBuf::assert(std::env::current_dir().unwrap().join(path));
    let manifest = ProjectManifest::discover_single(&path).unwrap();
    let mut workspace = ProjectWorkspace::load(manifest, &cargo_config, no_progress).unwrap();
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ra_ap_load_cargo::ProcMacroServerChoice::None,
        prefill_caches: false,
    };
    let (host, vfs, _proc_macro) =
        load_workspace(workspace, &cargo_config.extra_env, &load_config)?;
    let db = host.raw_database();
    eprintln!("db is {:?}", db);
    let mut krates = Crate::all(db);
    let mut visit_queue = krates
        .iter()
        .map(|krate| krate.root_module())
        .collect::<Vec<_>>();
    let mut visited_modules = HashSet::new();
    let mut bodies = Vec::new();
    let mut adts = Vec::new();
    let mut consts = Vec::new();
    while let Some(module) = visit_queue.pop() {
        eprintln!("Visit module: {:?}", module);
        if visited_modules.insert(module) {
            visit_queue.extend(module.children(db));
            for decl in module.declarations(db) {
                match decl {
                    ModuleDef::Function(f) => {
                        if f.attrs(db).by_key("must_use").exists() {
                            bodies.push(DefWithBody::from(f));
                        }
                    }
                    ModuleDef::Adt(a) => {
                        if a.attrs(db).by_key("must_use").exists() {
                            if let Adt::Enum(e) = a {
                                for v in e.variants(db) {
                                    bodies.push(DefWithBody::from(v));
                                }
                            }
                            adts.push(a);
                        }
                    }
                    ModuleDef::Const(c) => {
                        if c.attrs(db).by_key("must_use").exists() {
                            consts.push(c);
                            bodies.push(DefWithBody::from(c));
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    for body in &bodies {
        eprintln!("body is {:?} -> name {:?}", body, body.name(db));
    }
    for adt in adts {
        eprintln!("adt is {:?} -> name {:?}", adt, adt.name(db));
    }
    run_inference(db, &vfs, &bodies, Some("rhdl-x::do_stuff".into()));
    Ok(())
}

fn main() {
    println!("Hello, world!");
    do_stuff(
        Foo {
            a: 3,
            b: 7,
            c: [1, 2, 3],
        },
        NooState::Init,
    );
    let mut vfs = ra_ap_vfs::Vfs::default();
    let path = ra_ap_vfs::VfsPath::new_virtual_path("/root/foo.fs".into());
    vfs.set_file_contents(path.clone(), Some("Hello, world!".into()));
    // Get the change stream
    let changes = vfs.take_changes();
    for change in changes {
        println!("{:?}", change);
    }
    // Get the file ID
    let file_id = vfs.file_id(&path).unwrap();
    println!("file id is {:?}", file_id);
    // Get the file contents
    let contents = vfs.file_contents(file_id);
    println!("file contents is {:?}", contents);

    let file_contents = "
struct Foo<T> {
    a: T,
    b: u8,
}    
    
fn main() {
    let a = Foo { a: 3, b: 4 };
    let b = Foo { a: 3, b: 4 };
}
    ";

    let (analysis, file_id) = Analysis::from_single_file(file_contents.into());
    let structure = analysis.file_structure(file_id).unwrap();
    for s in structure {
        println!("{:?}", s);
    }
    let path = args().nth(1).unwrap();
    let path = Path::new(&path);
    run_analysis_batch(path).unwrap();
}
