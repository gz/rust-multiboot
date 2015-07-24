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

    mem_lower: u32,
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

    pub fn has_mmap(&'a self) -> bool {
        self.header.flags & 0x1 > 0
    }

    /// Discover all memory regions in the multiboot memory map.
    ///
    /// # Arguments
    ///
    ///  * `discovery_callback` - Function to notify your memory system about regions.
    ///
    pub fn find_memory<F: FnMut(u64, u64, MemType)>(&'a self, mut discovery_callback: F) {
        if !self.has_mmap() {
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

    pub fn has_modules(&'a self) -> bool {
        self.header.flags & (1<<3) > 0
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
