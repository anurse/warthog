use std::{sync::Arc, ops::Deref};

use crate::{
    module::{Export, Instruction, MemberDesc, Module},
    runtime::{
        ExportInst, ExternVal, FuncAddr, FuncInst, MemAddr, MemInst, ModuleAddr, ModuleInst,
    },
    synth::ModuleBuilder,
    Error,
};

pub struct Host {
    modules: Vec<Arc<ModuleInst>>,
    funcs: Vec<Arc<FuncInst>>,
    mems: Vec<Arc<MemInst>>,
}

// TODO: Consider if this type needs to be thread-safe
impl Host {
    pub fn new() -> Host {
        Host {
            modules: Vec::new(),
            funcs: Vec::new(),
            mems: Vec::new(),
        }
    }

    pub fn get_module(&self, addr: ModuleAddr) -> &ModuleInst {
        &self.modules[addr.val()]
    }

    pub fn get_func(&self, addr: FuncAddr) -> &FuncInst {
        &self.funcs[addr.val()]
    }

    pub fn modules<'a>(&'a self) -> impl 'a + Iterator<Item = Arc<ModuleInst>> {
        self.modules.iter().cloned()
    }

    pub fn funcs<'a>(&'a self) -> impl 'a + Iterator<Item = Arc<FuncInst>> {
        self.funcs.iter().cloned()
    }

    pub fn mems<'a>(&'a self) -> impl 'a + Iterator<Item = Arc<MemInst>> {
        self.mems.iter().cloned()
    }

    pub fn find_module(&self, name: &str) -> Option<ModuleAddr> {
        self.modules
            .iter()
            .position(|m| m.name() == name)
            .map(|a| ModuleAddr::new(a))
    }

    pub fn resolve_import(&self, module: ModuleAddr, name: &str) -> Result<&ExportInst, Error> {
        let module_inst = self.get_module(module);
        if let Some(export) = module_inst.find_export(name) {
            Ok(export)
        } else {
            Err(Error::ExportNotFound {
                module: module_inst.name().to_owned(),
                name: name.to_owned(),
            })
        }
    }

    /// Synthesizes a module from the provided [`ModuleBuilder`], consuming it in the process.
    pub fn synthesize(&mut self, module: ModuleBuilder) -> ModuleAddr {
        let module_addr = ModuleAddr::new(self.modules.len());

        let mut funcs = Vec::new();
        for func in module.funcs {
            // Allocate a func in the host
            let func_addr = FuncAddr::new(self.funcs.len());
            let func_inst = FuncInst::synthetic(func);
            self.funcs.push(Arc::new(func_inst));
            funcs.push(func_addr);
        }

        // Export the synthetic module
        let exports = self.export_module(&funcs, &module.exports);

        // Register the module and return
        self.modules
            .push(Arc::new(ModuleInst::new(module.name, funcs, Vec::new(), exports)));
        module_addr
    }

    /// Instantiates the provided [`Module`], consuming it in the process.
    pub fn instantiate(&mut self, module: Module) -> Result<ModuleAddr, Error> {
        let module_addr = ModuleAddr::new(self.modules.len());

        let mut funcs = Vec::new();
        let mut mems = Vec::new();

        self.resolve_imports(&module, &mut funcs, &mut mems)?;
        self.instantiate_funcs(module_addr, &module, &mut funcs);
        self.instantiate_data(&module, &mems)?;

        let exports = self.export_module(&funcs, module.exports());

        self.modules
            .push(Arc::new(ModuleInst::new(module.name(), funcs, mems, exports)));
        Ok(module_addr)
    }

    fn export_module(
        &mut self,
        funcs: &[FuncAddr],
        module_exports: &Vec<Export>,
    ) -> Vec<ExportInst> {
        let mut exports = Vec::new();
        for export in module_exports {
            match export.description {
                MemberDesc::Function(func_idx) => {
                    let func_addr = funcs[func_idx as usize];
                    let inst = ExportInst::func(export.name.as_str(), func_addr);
                    exports.push(inst);
                }
                MemberDesc::Memory(ref mem_type) => {
                    let mem_addr = MemAddr::new(self.mems.len());
                    self.mems.push(Arc::new(MemInst::from_type(mem_type)));
                    let inst = ExportInst::mem(export.name.as_str(), mem_addr);
                    exports.push(inst);
                }
                _ => { /* skip */ }
            }
        }
        exports
    }

    fn instantiate_funcs(
        &mut self,
        instance_addr: ModuleAddr,
        module: &Module,
        funcs: &mut Vec<FuncAddr>,
    ) {
        // Instantiate functions
        for (code_idx, type_id) in module.funcs().iter().enumerate() {
            // Assign an address
            let func_addr = FuncAddr::new(self.funcs.len());
            funcs.push(func_addr);

            // Get the function body and type
            let typ = module.types()[*type_id as usize].clone();
            let body = module.code()[code_idx].clone();

            // Create the instance and register it in the host
            self.funcs.push(Arc::new(FuncInst::local(typ, instance_addr, body)));
        }
    }

    fn resolve_imports(
        &mut self,
        module: &Module,
        funcs: &mut Vec<FuncAddr>,
        mems: &mut Vec<MemAddr>,
    ) -> Result<(), Error> {
        for import in module.imports() {
            if let Some(module_addr) = self.find_module(&import.module) {
                let export = self.resolve_import(module_addr, &import.name)?;
                match export.value() {
                    ExternVal::Func(func_addr) => funcs.push(func_addr.clone()),
                    ExternVal::Mem(mem_addr) => mems.push(mem_addr.clone()),
                }
            } else {
                return Err(Error::ModuleNotFound {
                    module: import.module.to_owned(),
                });
            }
        }
        Ok(())
    }

    fn instantiate_data(&mut self, module: &Module, mems: &Vec<MemAddr>) -> Result<(), Error> {
        for data in module.data() {
            // Offset must be a constant expression
            let offset = match data.expr.as_slice() {
                [Instruction::ConstI32(val)] => *val as usize,
                _ => return Err(Error::InvalidModule),
            };

            // Find an initialize the memory
            let mem_addr = mems[data.index as usize];
            let mem_inst = &mut self.mems[mem_addr.val()];
            let mut mem = mem_inst.memory_mut();

            // Bounds check
            let end = offset + data.init.len();
            if (offset + end) > mem.len() {
                return Err(Error::InvalidModule);
            }

            mem[offset..end].copy_from_slice(&data.init);
        }
        Ok(())
    }
}