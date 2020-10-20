//! Multiboot v1 library
//!
//! # Additional documentation
//!   * https://www.gnu.org/software/grub/manual/multiboot/multiboot.html
//!   * http://git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot.texi?h=multiboot
//!

#![no_std]
#![crate_name = "multiboot"]
#![crate_type = "lib"]

macro_rules! round_up {
    ($num:expr, $s:expr) => {
        (($num + $s - 1) / $s) * $s
    };
}

macro_rules! check_flag {
    ($doc:meta, $fun:ident, $bit:expr) => (
        #[$doc]
        fn $fun(&self) -> bool {
            //assert!($bit <= 31);
            (self.header.flags & (1 << $bit)) > 0
        }
    );

    // syms field is valid if bit 4 or 5 is set, wtf?
    ($doc:meta, $fun:ident, $bit1:expr, $bit2:expr) => (
        #[$doc]
        fn $fun(&self) -> bool {
            //assert!($bit1 <= 31);
            //assert!($bit2 <= 31);
            (self.header.flags & (1 << $bit1)) > 0 || (self.header.flags & (1 << $bit2)) > 0
        }
    );
}

mod information;
pub use information::SIGNATURE_EAX;
pub use information::PAddr;
pub use information::Multiboot;
pub use information::BootDevice;
pub use information::MemoryType;
pub use information::MemoryEntry;
pub use information::MemoryMapIter;
pub use information::Module;
pub use information::ModuleIter;

mod header;
pub use header::Header;
pub use header::MultibootAddresses;
pub use header::MultibootVideoMode;
pub use header::VideoModeType;
