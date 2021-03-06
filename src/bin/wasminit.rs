#![deny(warnings)]

extern crate warthog;

use std::{borrow::Cow, env, fs, path::Path, process};

use warthog::{
    hosting::{FuncImpl, Host, MemInst, ModuleAddr, ModuleInst},
    module::{Module, ModuleNames},
    reader::Reader,
    runtime,
};

fn main() {
    // Arg 0 is the executable name
    let arg0 = env::args().nth(0).unwrap();
    let args: Vec<_> = env::args().skip(1).collect();

    if args.len() > 0 {
        let file = &args[0];
        run(Path::new(file));
    } else {
        eprintln!("Usage: {} <wasm file>", arg0);
        process::exit(1);
    }
}

pub fn run(file: &Path) {
    // Create a host
    let mut host = Host::new();

    // Determine the module name
    let name = match file.file_stem() {
        Some(stem) => stem.to_string_lossy(),
        None => Cow::from("unnamed"),
    };

    // Load the module
    let module = {
        // Close the file once we're done loading
        let file = fs::File::open(file).unwrap();
        let reader = Reader::new(file);
        Module::load(reader).unwrap()
    };

    // Synthesize the 'env' module
    host.external(runtime::Env::new()).unwrap();

    // Instantiate the module
    let entry_point = host.instantiate(name, module).unwrap();

    // Dump the host
    println!("Host information:");
    dump_funcs(&host);
    dump_mems(&host);
    dump_instances(entry_point, &host);
}

fn dump_funcs(host: &Host) {
    println!("  Functions:");
    for (i, func_inst) in host.funcs().enumerate() {
        match func_inst.imp() {
            FuncImpl::Local(_, func_id) => {
                println!(
                    "  * {:04} {} {} {:04}",
                    i + 1,
                    func_inst.typ(),
                    func_inst.module(),
                    func_id
                );
            }
            FuncImpl::External(_) => println!("  * {:04} {} <extern>", i + 1, func_inst.typ()),
        }
    }
}

fn dump_mems(host: &Host) {
    println!("  Memories:");
    for (i, mem_inst) in host.mems().enumerate() {
        println!(
            "  * {:04} {} {}",
            i + 1,
            mem_inst.memory().len(),
            match mem_inst.memory().max_size() {
                Some(max) => format!("{}", max),
                None => "<unlimited>".to_owned(),
            }
        );

        println!("    Initialized Ranges:");
        dump_initialized_ranges(&mem_inst);
    }
}

fn dump_initialized_ranges(mem: &MemInst) {
    let mut range_start = None;
    let mem = mem.memory();
    unsafe {
        for (i, v) in mem.data().iter().enumerate() {
            match (v, range_start) {
                (0, Some(start)) => {
                    // End of a range
                    let end = i - 1;
                    println!(
                        "    * 0x{:08x} - 0x{:08x} (size: {})",
                        start,
                        end,
                        end - start
                    );
                    range_start = None;
                }
                (0, None) => { /* no-op */ }
                (_, None) => range_start = Some(i),
                _ => { /* no-op */ }
            }
        }
    }
}

fn dump_instances(entry_point: ModuleAddr, host: &Host) {
    for (i, module_inst) in host.modules().enumerate() {
        println!("{:04} Instance '{}':", i + 1, module_inst.name());
        if i == entry_point.val() {
            println!("  Entry Point");
        }
        dump_instance_funcs(&module_inst);
        dump_instance_mems(&module_inst);
        dump_instance_exports(&module_inst);

        if let Some(names) = module_inst.names() {
            dump_instance_names(names);
        }
    }
}

fn dump_instance_names(names: &ModuleNames) {
    println!("  Debug Names:");
    if let Some(module_name) = names.module_name() {
        println!("    Module: {}", module_name);
    }

    if names.funcs().len() > 0 {
        println!("    Functions:");
        for (idx, func_names) in names.funcs().iter() {
            if let Some(func_name) = func_names.func_name() {
                println!("    * {:04} {}", idx, func_name);
            } else {
                println!("    * {:04} <no name>", idx);
            }

            for (idx, local_name) in func_names.locals().iter() {
                println!("      * {:04} {}", idx, local_name);
            }
        }
    }
}

fn dump_instance_funcs(module_inst: &ModuleInst) {
    if module_inst.funcs().len() > 0 {
        println!("  Functions:");
        for (i, func_addr) in module_inst.funcs().iter().enumerate() {
            println!("  * {:04} {}", i, func_addr);
        }
    }
}

fn dump_instance_mems(module_inst: &ModuleInst) {
    if module_inst.mems().len() > 0 {
        println!("  Memories:");
        for (i, mem_addr) in module_inst.mems().iter().enumerate() {
            println!("  * {:04} {}", i, mem_addr);
        }
    }
}

fn dump_instance_exports(module_inst: &ModuleInst) {
    if module_inst.exports().len() > 0 {
        println!("  Exports:");
        for (i, export_inst) in module_inst.exports().iter().enumerate() {
            println!(
                "  * {:04} {} {:?}",
                i,
                export_inst.name(),
                export_inst.value()
            );
        }
    }
}
