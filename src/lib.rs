#![feature(no_std)]
#![feature(core)]

#![crate_name = "multiboot"]
#![crate_type = "lib"]

extern crate core;

use core::mem::{transmute};

/// Value that is in rax after multiboot jumps to our entry point
pub const SIGNATURE_RAX: u64 = 0x2BADB002;

#[derive(Debug)]
pub enum MemType {
    RAM = 1,
    Unusable = 2,
}

pub struct Multiboot<'a> {
    header: &'a MultibootHeader,
    paddr_to_vaddr: fn(u64) -> u64,
}

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

impl<'a> Multiboot<'a> {

    /// Initializes the multiboot structure
    pub fn new(mboot_ptr: u64, paddr_to_vaddr: fn(paddr: u64) -> u64) -> Multiboot<'a> {
        let header = paddr_to_vaddr(mboot_ptr);
        let mb: &MultibootHeader = unsafe { transmute::<u64, &MultibootHeader>(header) };

        Multiboot { header: mb, paddr_to_vaddr: paddr_to_vaddr }
    }

    /// Discovers all memory region in the multiboot memory map
    pub fn find_memory(&'a self, discovery_callback: fn(base: u64, length: u64, MemType))
    {
        let paddr_to_vaddr = self.paddr_to_vaddr;

        let mut current = self.header.mmap_addr;
        let end = self.header.mmap_addr + self.header.mmap_length;
        while current < end
        {
            let memory_region: &MemEntry = unsafe { transmute::<u64, &MemEntry>(paddr_to_vaddr(current as u64)) };

            let mtype = match memory_region.mtype {
                1 => MemType::RAM,
                2 => MemType::Unusable,
                _ => MemType::Unusable
            };

            discovery_callback(memory_region.base_addr, memory_region.length, mtype);
            current += memory_region.size + 4;
        }
    }

}
