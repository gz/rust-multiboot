//! Multiboot v1 library
//!
//! The main structs to interact with are [`Multiboot`] for the Multiboot information
//! passed from the bootloader to the kernel at runtime and [`Header`] for the static
//! information passed from the kernel to the bootloader in the kernel image.
//!
//!
//!
//! # Additional documentation
//!   * https://www.gnu.org/software/grub/manual/multiboot/multiboot.html
//!   * http://git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot.texi?h=multiboot
//!
//! [`Multiboot`]: struct.Multiboot.html
//! [`Header`]: struct.Header.html

#![no_std]
#![crate_name = "multiboot"]
#![crate_type = "lib"]

macro_rules! round_up {
    ($num:expr, $s:expr) => {
        (($num + $s - 1) / $s) * $s
    };
}

macro_rules! flag {
    ($doc:meta, $fun:ident, $bit:expr) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            //assert!($bit <= 31);
            (self.header.flags & (1 << $bit)) > 0
        }
        
        paste::paste! {
            #[$doc]
            fn [< set_ $fun >] (&mut self, flag: bool) {
                //assert!($bit <= 31);
                self.header.flags = if flag {
                    self.header.flags | (1 << $bit)
                } else {
                    self.header.flags & !(1 << $bit)
                };
            }
        }
    );
}

mod information;
pub use information::SIGNATURE_EAX;
pub use information::PAddr;
pub use information::Multiboot;
pub use information::MultibootInfo;
pub use information::BootDevice;
pub use information::MemoryType;
pub use information::MemoryEntry;
pub use information::MemoryManagement;
pub use information::MemoryMapIter;
pub use information::Module;
pub use information::ModuleIter;
pub use information::SymbolType;
pub use information::AOutSymbols;
pub use information::ElfSymbols;
pub use information::FramebufferTable;
pub use information::ColorInfoType;
pub use information::ColorInfoRgb;

mod header;
pub use header::Header;
pub use header::MultibootAddresses;
pub use header::MultibootVideoMode;
pub use header::VideoModeType;
