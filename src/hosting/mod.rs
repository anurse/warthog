macro_rules! addr_type {
    ($name: ident) => {
        #[derive(Clone, Copy, PartialEq)]
        pub struct $name(::std::num::NonZeroUsize);
        
        impl $name {
            pub fn new(id: usize) -> Option<$name> {
                match ::std::num::NonZeroUsize::new(id) {
                    Some(id) => Some($name(id)),
                    None => None,
                }
            }
        
            pub fn val(&self) -> usize {
                self.0.get() - 1
            }
        }
        
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, concat!("[", stringify!($name), "]0x{:04X}"), self.0)
            }
        }

        impl ::std::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{:08x}", self.0)
            }
        }

        impl ::std::fmt::UpperHex for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{:08X}", self.0)
            }
        }
    };
}

mod export_inst;
mod func_inst;
mod host;
mod mem_inst;
mod module_inst;
mod external;
mod host_func;

pub use self::export_inst::{ExportInst, ExternVal};
pub use self::func_inst::{FuncAddr, FuncImpl, FuncInst};
pub use self::host::Host;
pub use self::mem_inst::{MemAddr, MemInst};
pub use self::module_inst::{ModuleAddr, ModuleInst};
pub use self::external::{ExternalModule, ExternalFunc, ExternalMemory};
pub use self::host_func::HostFunc;
