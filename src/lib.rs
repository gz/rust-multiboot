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

/// Value that is in rax after multiboot jumps to our entry point
pub const SIGNATURE_RAX: u64 = 0x2BADB002;

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
    header: &'a MultibootHeader,
    paddr_to_vaddr: fn(PAddr) -> VAddr,
}

/// Representation of Multiboot header according to specification.
#[derive(Debug)]
#[repr(packed)]
struct MultibootHeader {
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
}

/// Multiboot format of the MMAP buffer.
/// Note that size is defined to be at -4 bytes.
#[derive(Debug)]
#[repr(packed)]
struct MemEntry {
    size: u32,
    base_addr: u64,
    length: u64,
    mtype: u32
}

#[derive(Debug)]
#[repr(packed)]
struct ElfSymbols {
    num: u32,
    size: u32,
    addr: u32,
    shndx: u32,
}

/// Multiboot module structure
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
        let mb: &MultibootHeader = unsafe { transmute::<VAddr, &MultibootHeader>(header) };

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
