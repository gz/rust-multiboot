//! Multiboot v1 library
//!
//! # Additional documentation
//!   * https://www.gnu.org/software/grub/manual/multiboot/multiboot.html
//!   * http://git.savannah.gnu.org/cgit/grub.git/tree/doc/multiboot.texi?h=multiboot
//!

#![feature(no_std)]
#![feature(core)]
#![feature(raw)]
#![no_std]

#![crate_name = "multiboot"]
#![crate_type = "lib"]

extern crate core;

#[cfg(test)]
extern crate std;

use core::mem::{transmute};
use core::ops::FnMut;
use core::raw;
use core::str;

/// Value found in %rax after multiboot jumps to our entry point.
pub const SIGNATURE_RAX: u64 = 0x2BADB002;

/// Types that define if the memory is usable or not.
#[derive(Debug, PartialEq, Eq)]
pub enum MemType {
    RAM = 1,
    Unusable = 2,
}

pub type PAddr = u64;
pub type VAddr = usize;

/// Multiboot struct clients mainly interact with
/// To create this use Multiboot::new()
pub struct Multiboot<'a> {
    header: &'a MultibootInfo,
    paddr_to_vaddr: fn(PAddr) -> VAddr,
}

/// Representation of Multiboot header according to specification.
////
///<rawtext>
///         +-------------------+
/// 0       | flags             |    (required)
///         +-------------------+
/// 4       | mem_lower         |    (present if flags[0] is set)
/// 8       | mem_upper         |    (present if flags[0] is set)
///         +-------------------+
/// 12      | boot_device       |    (present if flags[1] is set)
///         +-------------------+
/// 16      | cmdline           |    (present if flags[2] is set)
///         +-------------------+
/// 20      | mods_count        |    (present if flags[3] is set)
/// 24      | mods_addr         |    (present if flags[3] is set)
///         +-------------------+
/// 28 - 40 | syms              |    (present if flags[4] or
///         |                   |                flags[5] is set)
///         +-------------------+
/// 44      | mmap_length       |    (present if flags[6] is set)
/// 48      | mmap_addr         |    (present if flags[6] is set)
///         +-------------------+
/// 52      | drives_length     |    (present if flags[7] is set)
/// 56      | drives_addr       |    (present if flags[7] is set)
///         +-------------------+
/// 60      | config_table      |    (present if flags[8] is set)
///         +-------------------+
/// 64      | boot_loader_name  |    (present if flags[9] is set)
///         +-------------------+
/// 68      | apm_table         |    (present if flags[10] is set)
///         +-------------------+
/// 72      | vbe_control_info  |    (present if flags[11] is set)
/// 76      | vbe_mode_info     |
/// 80      | vbe_mode          |
/// 82      | vbe_interface_seg |
/// 84      | vbe_interface_off |
/// 86      | vbe_interface_len |
///         +-------------------+
///</rawtext>
///
#[derive(Debug)]
#[repr(packed)]
struct MultibootInfo {
    flags: u32,

    /// Indicate the amount of lower memory in kilobytes.
    ///
    /// Lower memory starts at address 0. The maximum possible value for
    /// lower memory is 640 kilobytes.
    mem_lower: u32,

    /// Indicate the amount of upper memory in kilobytes.
    ///
    /// Upper memory starts at address 1 megabyte.
    /// The value returned for upper memory is maximally the address of
    /// the first upper memory hole minus 1 megabyte. It is not guaranteed
    /// to be this value.
    mem_upper: u32,

    boot_device: u32,
    cmdline: u32,

    mods_count: u32,
    mods_addr: u32,

    elf_symbols: ElfSymbols,

    mmap_length: u32,
    mmap_addr: u32,

    drives_length: u32,
    drives_addr: u32,

    config_table: u32,

    boot_loader_name: u32,

    apm_table: u32,

    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16
}

/// Multiboot format of the MMAP buffer.
///
/// Note that size is defined to be at -4 bytes in multiboot.
#[derive(Debug)]
#[repr(packed)]
struct MemEntry {
    size: u32,
    base_addr: u64,
    length: u64,
    mtype: u32
}

/// ELF Symbols
#[derive(Debug)]
#[repr(packed)]
struct ElfSymbols {
    num: u32,
    size: u32,
    addr: u32,
    shndx: u32,
}

/// Multiboot module representation
#[derive(Debug)]
#[repr(packed)]
struct Module {
    /// Start address of module in memory.
    start: u32,
    /// End address of module in memory.
    end: u32,
    /// Name of module.
    string: u32,
    /// Must be zero.
    reserved: u32
}

/// Convert a C string into a [u8 slice and from there into a &'static str.
/// This unsafe block builds on assumption that multiboot strings are sane.
fn convert_safe_c_string(cstring: *const u8) -> &'static str {
    unsafe {
        let mut iter = cstring;
        while *iter != 0 {
            iter = iter.offset(1);
        }

        let slice = raw::Slice { data: cstring, len: iter as usize - cstring as usize };
        let byte_array: &'static [u8] = transmute(slice);
        str::from_utf8_unchecked(byte_array)
    }
}

macro_rules! check_flag {
    ($doc:meta, $fun:ident, $bit:expr) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            //assert!($bit <= 31);
            (self.header.flags & (1 << $bit)) > 0
        }
    );

    // syms field is valid if bit 4 or 5 is set, wtf?
    ($doc:meta, $fun:ident, $bit1:expr, $bit2:expr) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            //assert!($bit1 <= 31);
            //assert!($bit2 <= 31);
            (self.header.flags & (1 << $bit1)) > 0 || (self.header.flags & (1 << $bit2)) > 0
        }
    );
}

impl<'a> Multiboot<'a> {

    /// Initializes the multiboot structure.
    ///
    /// # Arguments
    ///
    ///  * `mboot_ptr` - The physical address of the multiboot header. On qemu for example
    ///                  this is typically at 0x9500.
    ///  * `paddr_to_vaddr` - Translation of the physical addresses into kernel addresses.
    ///
    ///  `paddr_to_vaddr` translates physical it into a kernel accessible address.
    ///  The simplest paddr_to_vaddr function would for example be just the identity
    ///  function. But this may vary depending on how your page table layout looks like.
    ///
    pub fn new(mboot_ptr: u64, paddr_to_vaddr: fn(paddr: PAddr) -> VAddr) -> Multiboot<'a> {
        let header = paddr_to_vaddr(mboot_ptr);
        let mb: &MultibootInfo = unsafe { transmute::<VAddr, &MultibootInfo>(header) };

        Multiboot { header: mb, paddr_to_vaddr: paddr_to_vaddr }
    }

    check_flag!(doc = "If true, then the `mem_upper` and `mem_lower` fields are valid.",
               has_memory_bounds, 0);
    check_flag!(doc = "If true, then the `boot_device` field is valid.",
               has_boot_device, 1);
    check_flag!(doc = "If true, then the `cmdline` field is valid.",
               has_cmdline, 2);
    check_flag!(doc = "If true, then the `mods_addr` and `mods_count` fields are valid.",
               has_modules, 3);
    check_flag!(doc = "If true, then the `syms` field is valid.",
               has_symbols, 4, 5);
    check_flag!(doc = "If true, then the `mmap_addr` and `mmap_length` fields are valid.",
               has_memory_map, 6);
    check_flag!(doc = "If true, then the `drives_addr` and `drives_length` fields are valid.",
               has_drives, 7);
    check_flag!(doc = "If true, then the `config_table` field is valid.",
               has_config_table, 8);
    check_flag!(doc = "If true, then the `boot_loader_name` field is valid.",
               has_boot_loader_name, 9);
    check_flag!(doc = "If true, then the `apm_table` field is valid.",
               has_apm_table, 10);
    check_flag!(doc = "If true, then the `vbe_*` fields are valid.",
               has_vbe, 11);

    /// Discover all memory regions in the multiboot memory map.
    ///
    /// # Arguments
    ///
    ///  * `discovery_callback` - Function to notify your memory system about regions.
    ///
    pub fn find_memory<F: FnMut(u64, u64, MemType)>(&'a self, mut discovery_callback: F) {
        if !self.has_memory_map() {
            return
        }

        let paddr_to_vaddr = self.paddr_to_vaddr;

        let mut current = self.header.mmap_addr;
        let end = self.header.mmap_addr + self.header.mmap_length;
        while current < end
        {
            let memory_region: &MemEntry = unsafe { transmute::<VAddr, &MemEntry>(paddr_to_vaddr(current as u64)) };

            let mtype = match memory_region.mtype {
                1 => MemType::RAM,
                2 => MemType::Unusable,
                _ => MemType::Unusable
            };

            discovery_callback(memory_region.base_addr, memory_region.length, mtype);
            current += memory_region.size + 4;
        }
    }

    /// Discover all additional modules in multiboot.
    ///
    /// # Arguments
    ///
    ///  * `discovery_callback` - Function to notify your system about modules.
    ///
    pub fn find_modules<F: FnMut(&'static str, VAddr, VAddr)>(&'a self, mut discovery_callback: F) {
        if !self.has_modules() {
            return
        }

        let paddr_to_vaddr = self.paddr_to_vaddr;

        let module_start = paddr_to_vaddr(self.header.mods_addr as u64);
        let count: usize = self.header.mods_count as usize;
        for _ in 0..count {
            let current: &Module = unsafe { transmute::<VAddr, &Module>(module_start) };
            let path = unsafe { convert_safe_c_string(transmute::<VAddr, *const u8>(paddr_to_vaddr(current.string as u64))) };

            discovery_callback(path, paddr_to_vaddr(current.start as PAddr), paddr_to_vaddr(current.end as PAddr));
        }
    }
}
