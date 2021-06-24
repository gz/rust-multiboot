extern crate core;
extern crate multiboot;

use core::convert::TryInto;
use core::mem;
use core::slice;
use multiboot::information::{
    ColorInfoType, MemoryManagement, MemoryType, Multiboot, PAddr, SymbolType,
};

const TEST_STR: [u8; 5] = [0x74, 0x65, 0x73, 0x74, 0x00]; // 'test'
const TEST_MOD: [u8; 16] = [
    0x78, 0x56, 0x34, 0x12, // start
    0x21, 0x43, 0x65, 0x87, // end
    0xaa, 0xaa, 0xaa, 0xaa, // TEST_STR
    0x00, 0x00, 0x00, 0x00, // reserved
];
const TEST_REGION: [u8; 24] = [
    0x20, 0x00, 0x00, 0x00, // size
    0x78, 0x56, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, // base_addr
    0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // length
    0x05, 0x00, 0x00, 0x00, // type
];

struct Mem;

impl MemoryManagement for Mem {
    unsafe fn paddr_to_slice(&self, addr: PAddr, size: usize) -> Option<&'static [u8]> {
        match (addr, size) {
            (0xaaaaaaaa, sz) => Some(&TEST_STR[0..sz]),
            (0xaaaaaaab, 1) => Some(&[TEST_STR[1]]),
            (0xaaaaaaac, 1) => Some(&[TEST_STR[2]]),
            (0xaaaaaaad, 1) => Some(&[TEST_STR[3]]),
            (0xaaaaaaae, 1) => Some(&[TEST_STR[4]]),
            (0xbbbbbbbb, 16) => Some(&TEST_MOD),
            (0xcccccccc, 24) => Some(&TEST_REGION),
            (p, sz) => {
                let ptr: usize = p.try_into().unwrap();
                let ptr = mem::transmute(ptr);
                Some(slice::from_raw_parts(ptr, sz))
            }
        }
    }

    unsafe fn allocate(&mut self, _length: usize) -> Option<(PAddr, &mut [u8])> {
        None
    }

    unsafe fn deallocate(&mut self, addr: PAddr) {
        if addr != 0 {
            unimplemented!()
        }
    }
}

static mut MEM: Mem = Mem {};

/// mboot_ptr is the initial pointer to the multiboot structure
/// provided in %ebx on start-up.
pub fn use_multiboot(mboot_ptr: PAddr) -> Option<Multiboot<'static, 'static>> {
    unsafe { Multiboot::from_ptr(mboot_ptr, &mut MEM) }
}

#[test]
fn null_ptr() {
    use_multiboot(0x0);
}

#[test]
/// Parse an almost empty information
fn empty() {
    let information: [u8; 120] = [
        0x00, 0x00, 0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // mem_lower
        0x00, 0x00, 0x00, 0x00, // mem_upper
        0x00, 0x00, 0x00, 0x00, // boot_device
        0x00, 0x00, 0x00, 0x00, // cmdline
        0x00, 0x00, 0x00, 0x00, // mods_count
        0x00, 0x00, 0x00, 0x00, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x00, 0x00, 0x00, 0x00, // mmap_length
        0x00, 0x00, 0x00, 0x00, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0x00, 0x00, 0x00, 0x00, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x00, 0x00, 0x00, 0x00, // framebuffer_pitch
        0x00, 0x00, 0x00, 0x00, // framebuffer_width
        0x00, 0x00, 0x00, 0x00, // framebuffer_height
        0x00, // framebuffer_bpp
        0x00, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(!parsed.has_memory_bounds());
    assert!(!parsed.has_boot_device());
    assert!(!parsed.has_cmdline());
    assert!(!parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(!parsed.has_elf_symbols());
    assert!(!parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(!parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(!parsed.has_framebuffer_table());
    assert!(parsed.lower_memory_bound().is_none());
    assert!(parsed.upper_memory_bound().is_none());
    assert!(parsed.boot_device().is_none());
    assert!(parsed.command_line().is_none());
    assert!(parsed.modules().is_none());
    assert!(parsed.symbols().is_none());
    assert!(parsed.memory_regions().is_none());
    assert!(parsed.framebuffer_table().is_none());
    assert_eq!(parsed.find_highest_address(), 0);
}

#[test]
/// Parse an almost empty information
fn memory_bounds() {
    let information: [u8; 120] = [
        0x01, 0x00, 0x00, 0x00, // flags
        0x80, 0x02, 0x00, 0x00, // mem_lower
        0x00, 0x20, 0x00, 0x00, // mem_upper
        0xff, 0x00, 0x00, 0x00, // boot_device
        0x00, 0x00, 0x00, 0x00, // cmdline
        0x00, 0x00, 0x00, 0x00, // mods_count
        0x00, 0x00, 0x00, 0x00, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x00, 0x00, 0x00, 0x00, // mmap_length
        0x00, 0x00, 0x00, 0x00, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0x00, 0x00, 0x00, 0x00, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x00, 0x00, 0x00, 0x00, // framebuffer_pitch
        0x00, 0x00, 0x00, 0x00, // framebuffer_width
        0x00, 0x00, 0x00, 0x00, // framebuffer_height
        0x00, // framebuffer_bpp
        0x00, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(parsed.has_memory_bounds());
    assert!(!parsed.has_boot_device());
    assert!(!parsed.has_cmdline());
    assert!(!parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(!parsed.has_elf_symbols());
    assert!(!parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(!parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(!parsed.has_framebuffer_table());
    assert_eq!(parsed.lower_memory_bound().unwrap(), 640);
    assert_eq!(parsed.upper_memory_bound().unwrap(), 8 * 1024);
    assert!(parsed.boot_device().is_none());
    assert!(parsed.command_line().is_none());
    assert!(parsed.modules().is_none());
    assert!(parsed.symbols().is_none());
    assert!(parsed.memory_regions().is_none());
    assert!(parsed.framebuffer_table().is_none());
    assert_eq!(parsed.find_highest_address(), 0);
}

#[test]
/// Parse an information containing a boot device
fn boot_device() {
    let information: [u8; 120] = [
        0x02, 0x00, 0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // mem_lower
        0x00, 0x00, 0x00, 0x00, // mem_upper
        0x80, 0x00, 0xff, 0xff, // boot_device
        0x00, 0x00, 0x00, 0x00, // cmdline
        0x00, 0x00, 0x00, 0x00, // mods_count
        0x00, 0x00, 0x00, 0x00, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x00, 0x00, 0x00, 0x00, // mmap_length
        0x00, 0x00, 0x00, 0x00, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0x00, 0x00, 0x00, 0x00, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x00, 0x00, 0x00, 0x00, // framebuffer_pitch
        0x00, 0x00, 0x00, 0x00, // framebuffer_width
        0x00, 0x00, 0x00, 0x00, // framebuffer_height
        0x00, // framebuffer_bpp
        0x00, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(!parsed.has_memory_bounds());
    assert!(parsed.has_boot_device());
    assert!(!parsed.has_cmdline());
    assert!(!parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(!parsed.has_elf_symbols());
    assert!(!parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(!parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(!parsed.has_framebuffer_table());
    assert!(parsed.lower_memory_bound().is_none());
    assert!(parsed.upper_memory_bound().is_none());
    let boot_device = parsed.boot_device().unwrap();
    assert_eq!(boot_device.drive, 0x80);
    assert!(boot_device.partition1_is_valid());
    assert_eq!(boot_device.partition1, 0);
    assert!(!boot_device.partition2_is_valid());
    assert!(!boot_device.partition3_is_valid());
    assert!(parsed.command_line().is_none());
    assert!(parsed.modules().is_none());
    assert!(parsed.symbols().is_none());
    assert!(parsed.memory_regions().is_none());
    assert!(parsed.framebuffer_table().is_none());
    assert_eq!(parsed.find_highest_address(), 0);
}

#[test]
/// Parse an information containing a command line
fn command_line() {
    let information: [u8; 120] = [
        0x04, 0x00, 0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // mem_lower
        0x00, 0x00, 0x00, 0x00, // mem_upper
        0x00, 0x00, 0x00, 0x00, // boot_device
        0xaa, 0xaa, 0xaa, 0xaa, // cmdline
        0x00, 0x00, 0x00, 0x00, // mods_count
        0x00, 0x00, 0x00, 0x00, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x00, 0x00, 0x00, 0x00, // mmap_length
        0x00, 0x00, 0x00, 0x00, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0x00, 0x00, 0x00, 0x00, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x00, 0x00, 0x00, 0x00, // framebuffer_pitch
        0x00, 0x00, 0x00, 0x00, // framebuffer_width
        0x00, 0x00, 0x00, 0x00, // framebuffer_height
        0x00, // framebuffer_bpp
        0x00, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(!parsed.has_memory_bounds());
    assert!(!parsed.has_boot_device());
    assert!(parsed.has_cmdline());
    assert!(!parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(!parsed.has_elf_symbols());
    assert!(!parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(!parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(!parsed.has_framebuffer_table());
    assert!(parsed.lower_memory_bound().is_none());
    assert!(parsed.upper_memory_bound().is_none());
    assert!(parsed.boot_device().is_none());
    assert_eq!(parsed.command_line().unwrap(), "test");
    assert!(parsed.modules().is_none());
    assert!(parsed.symbols().is_none());
    assert!(parsed.memory_regions().is_none());
    assert!(parsed.framebuffer_table().is_none());
    assert_eq!(parsed.find_highest_address(), 0xaaaab000);
}

#[test]
/// Parse an information containing a module
fn mods() {
    let information: [u8; 120] = [
        0x08, 0x00, 0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // mem_lower
        0x00, 0x00, 0x00, 0x00, // mem_upper
        0x00, 0x00, 0x00, 0x00, // boot_device
        0x00, 0x00, 0x00, 0x00, // cmdline
        0x01, 0x00, 0x00, 0x00, // mods_count
        0xbb, 0xbb, 0xbb, 0xbb, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x00, 0x00, 0x00, 0x00, // mmap_length
        0x00, 0x00, 0x00, 0x00, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0x00, 0x00, 0x00, 0x00, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x00, 0x00, 0x00, 0x00, // framebuffer_pitch
        0x00, 0x00, 0x00, 0x00, // framebuffer_width
        0x00, 0x00, 0x00, 0x00, // framebuffer_height
        0x00, // framebuffer_bpp
        0x00, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(!parsed.has_memory_bounds());
    assert!(!parsed.has_boot_device());
    assert!(!parsed.has_cmdline());
    assert!(parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(!parsed.has_elf_symbols());
    assert!(!parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(!parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(!parsed.has_framebuffer_table());
    assert!(parsed.lower_memory_bound().is_none());
    assert!(parsed.upper_memory_bound().is_none());
    assert!(parsed.boot_device().is_none());
    assert!(parsed.command_line().is_none());
    let mut module_iter = parsed.modules().unwrap();
    let module = module_iter.next().unwrap();
    assert_eq!(module.start, 0x12345678);
    assert_eq!(module.end, 0x87654321);
    assert_eq!(module.string.unwrap(), "test");
    assert!(module_iter.next().is_none());
    assert!(parsed.symbols().is_none());
    assert!(parsed.memory_regions().is_none());
    assert!(parsed.framebuffer_table().is_none());
    assert_eq!(parsed.find_highest_address(), 0xbbbbc000);
}

#[test]
/// Parse an information containing ELF symbols.
fn elf_symbols() {
    let information: [u8; 120] = [
        0x20, 0x00, 0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // mem_lower
        0x00, 0x00, 0x00, 0x00, // mem_upper
        0x00, 0x00, 0x00, 0x00, // boot_device
        0x00, 0x00, 0x00, 0x00, // cmdline
        0x00, 0x00, 0x00, 0x00, // mods_count
        0x00, 0x00, 0x00, 0x00, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x00, 0x00, 0x00, 0x00, // mmap_length
        0x00, 0x00, 0x00, 0x00, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0x00, 0x00, 0x00, 0x00, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x00, 0x00, 0x00, 0x00, // framebuffer_pitch
        0x00, 0x00, 0x00, 0x00, // framebuffer_width
        0x00, 0x00, 0x00, 0x00, // framebuffer_height
        0x00, // framebuffer_bpp
        0x00, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(!parsed.has_memory_bounds());
    assert!(!parsed.has_boot_device());
    assert!(!parsed.has_cmdline());
    assert!(!parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(parsed.has_elf_symbols());
    assert!(!parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(!parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(!parsed.has_framebuffer_table());
    assert!(parsed.lower_memory_bound().is_none());
    assert!(parsed.upper_memory_bound().is_none());
    assert!(parsed.boot_device().is_none());
    assert!(parsed.command_line().is_none());
    assert!(parsed.modules().is_none());
    match parsed.symbols().unwrap() {
        SymbolType::AOut(_) => panic!("wrong symbol type"),
        SymbolType::Elf(_) => (), // ok
    };
    assert!(parsed.memory_regions().is_none());
    assert!(parsed.framebuffer_table().is_none());
    assert_eq!(parsed.find_highest_address(), 0);
}

#[test]
/// Parse an information containing memory regions.
fn memory_regions() {
    let information: [u8; 120] = [
        0x40, 0x00, 0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // mem_lower
        0x00, 0x00, 0x00, 0x00, // mem_upper
        0x00, 0x00, 0x00, 0x00, // boot_device
        0x00, 0x00, 0x00, 0x00, // cmdline
        0x00, 0x00, 0x00, 0x00, // mods_count
        0x00, 0x00, 0x00, 0x00, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x01, 0x00, 0x00, 0x00, // mmap_length
        0xcc, 0xcc, 0xcc, 0xcc, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0x00, 0x00, 0x00, 0x00, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x00, 0x00, 0x00, 0x00, // framebuffer_pitch
        0x00, 0x00, 0x00, 0x00, // framebuffer_width
        0x00, 0x00, 0x00, 0x00, // framebuffer_height
        0x00, // framebuffer_bpp
        0x00, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(!parsed.has_memory_bounds());
    assert!(!parsed.has_boot_device());
    assert!(!parsed.has_cmdline());
    assert!(!parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(!parsed.has_elf_symbols());
    assert!(parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(!parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(!parsed.has_framebuffer_table());
    assert!(parsed.lower_memory_bound().is_none());
    assert!(parsed.upper_memory_bound().is_none());
    assert!(parsed.boot_device().is_none());
    assert!(parsed.command_line().is_none());
    assert!(parsed.modules().is_none());
    assert!(parsed.symbols().is_none());
    let mut memory_regions = parsed.memory_regions().unwrap();
    let region = memory_regions.next().unwrap();
    assert_eq!(region.base_address(), 0x12345678);
    assert_eq!(region.length(), 4096);
    assert_eq!(region.memory_type(), MemoryType::Defect);
    assert!(memory_regions.next().is_none());
    assert!(parsed.framebuffer_table().is_none());
    assert_eq!(parsed.find_highest_address(), 0xccccd000);
}

#[test]
/// Parse an information containing a boot loader name
fn boot_loader_name() {
    let information: [u8; 120] = [
        0x00, 0x02, 0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // mem_lower
        0x00, 0x00, 0x00, 0x00, // mem_upper
        0x00, 0x00, 0x00, 0x00, // boot_device
        0x00, 0x00, 0x00, 0x00, // cmdline
        0x00, 0x00, 0x00, 0x00, // mods_count
        0x00, 0x00, 0x00, 0x00, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x00, 0x00, 0x00, 0x00, // mmap_length
        0x00, 0x00, 0x00, 0x00, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0xaa, 0xaa, 0xaa, 0xaa, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x00, 0x00, 0x00, 0x00, // framebuffer_pitch
        0x00, 0x00, 0x00, 0x00, // framebuffer_width
        0x00, 0x00, 0x00, 0x00, // framebuffer_height
        0x00, // framebuffer_bpp
        0x00, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(!parsed.has_memory_bounds());
    assert!(!parsed.has_boot_device());
    assert!(!parsed.has_cmdline());
    assert!(!parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(!parsed.has_elf_symbols());
    assert!(!parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(!parsed.has_framebuffer_table());
    assert!(parsed.lower_memory_bound().is_none());
    assert!(parsed.upper_memory_bound().is_none());
    assert!(parsed.boot_device().is_none());
    assert_eq!(parsed.boot_loader_name().unwrap(), "test");
    assert!(parsed.modules().is_none());
    assert!(parsed.symbols().is_none());
    assert!(parsed.memory_regions().is_none());
    assert!(parsed.framebuffer_table().is_none());
    assert_eq!(parsed.find_highest_address(), 0xaaaab000);
}

#[test]
/// Parse an information containing a framebuffer table
fn framebuffer() {
    let information: [u8; 120] = [
        0x00, 0x10, 0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // mem_lower
        0x00, 0x00, 0x00, 0x00, // mem_upper
        0x00, 0x00, 0x00, 0x00, // boot_device
        0x00, 0x00, 0x00, 0x00, // cmdline
        0x00, 0x00, 0x00, 0x00, // mods_count
        0x00, 0x00, 0x00, 0x00, // mods_addr
        0x00, 0x00, 0x00, 0x00, // syms1
        0x00, 0x00, 0x00, 0x00, // syms2
        0x00, 0x00, 0x00, 0x00, // syms3
        0x00, 0x00, 0x00, 0x00, // syms4
        0x00, 0x00, 0x00, 0x00, // mmap_length
        0x00, 0x00, 0x00, 0x00, // mmap_addr
        0x00, 0x00, 0x00, 0x00, // drives_length
        0x00, 0x00, 0x00, 0x00, // drives_addr
        0x00, 0x00, 0x00, 0x00, // config_table
        0x00, 0x00, 0x00, 0x00, // boot_loader_name
        0x00, 0x00, 0x00, 0x00, // apm_table
        0x00, 0x00, 0x00, 0x00, // vbe_control_info
        0x00, 0x00, 0x00, 0x00, // vbe_mode_info
        0x00, 0x00, // vbe_mode
        0x00, 0x00, // vbe_interface_seg
        0x00, 0x00, // vbe_interface_off
        0x00, 0x00, // vbe_interface_len
        0x78, 0x56, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, // framebuffer_addr
        0x80, 0x0c, 0x00, 0x00, // framebuffer_pitch
        0x20, 0x03, 0x00, 0x00, // framebuffer_width
        0x58, 0x02, 0x00, 0x00, // framebuffer_height
        0x20, // framebuffer_bpp
        0x01, // framebuffer_type
        0x00, 0x00, // alignment
        0x00, 0x08, 0x08, 0x08, 0x10, 0x08, // color_info
        0x00, 0x00, // alignment
    ];
    let parsed = use_multiboot(information.as_ptr() as PAddr).unwrap();
    assert!(!parsed.has_memory_bounds());
    assert!(!parsed.has_boot_device());
    assert!(!parsed.has_cmdline());
    assert!(!parsed.has_modules());
    assert!(!parsed.has_aout_symbols());
    assert!(!parsed.has_elf_symbols());
    assert!(!parsed.has_memory_map());
    assert!(!parsed.has_drives());
    assert!(!parsed.has_config_table());
    assert!(!parsed.has_boot_loader_name());
    assert!(!parsed.has_apm_table());
    assert!(!parsed.has_vbe());
    assert!(parsed.has_framebuffer_table());
    assert!(parsed.lower_memory_bound().is_none());
    assert!(parsed.upper_memory_bound().is_none());
    assert!(parsed.boot_device().is_none());
    assert!(parsed.command_line().is_none());
    assert!(parsed.modules().is_none());
    assert!(parsed.symbols().is_none());
    assert!(parsed.memory_regions().is_none());
    let framebuffer_table = parsed.framebuffer_table().unwrap();
    assert_eq!(framebuffer_table.addr, 0x12345678);
    assert_eq!(framebuffer_table.pitch, 800 * 4);
    assert_eq!(framebuffer_table.width, 800);
    assert_eq!(framebuffer_table.height, 600);
    assert_eq!(framebuffer_table.bpp, 32);
    match framebuffer_table.color_info().unwrap() {
        ColorInfoType::Rgb(rgb) => {
            assert_eq!(rgb.red_field_position, 0);
            assert_eq!(rgb.red_mask_size, 8);
            assert_eq!(rgb.green_field_position, 8);
            assert_eq!(rgb.green_mask_size, 8);
            assert_eq!(rgb.blue_field_position, 16);
            assert_eq!(rgb.blue_mask_size, 8);
        }
        _ => panic!("wrong color info"),
    };
    assert_eq!(parsed.find_highest_address(), 0);
}
