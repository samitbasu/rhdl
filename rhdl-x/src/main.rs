use anyhow::Result;
use anyhow::{anyhow, bail};
use itertools::Itertools;
use ra_ap_base_db::SourceDatabaseExt;
use ra_ap_hir::{Adt, Crate, DefWithBody, HasAttrs, ModuleDef};
use ra_ap_ide::Analysis;
use ra_ap_load_cargo::{load_workspace, LoadCargoConfig};
use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace};
use ra_ap_vfs::AbsPathBuf;
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
    for body in bodies {
        eprintln!("body is {:?} -> name {:?}", body, body.name(db));
    }
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
