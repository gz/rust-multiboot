//! This modules contains the pieces for parsing and creating Multiboot information structures.
//!
//! If you don't know where to start, take a look at [`Multiboot`].
//!
//! [`Multiboot`]: struct.Multiboot.html

use core::cmp;
use core::convert::TryInto;
use core::fmt;
use core::fmt::Debug;
use core::mem::{size_of, transmute};
use core::slice;
use core::str;

/// Value found in %eax after multiboot jumps to our entry point.
pub const SIGNATURE_EAX: u32 = 0x2BADB002;

pub type PAddr = u64;

/// Implement this trait to be able to get or set fields containing a pointer.
///
/// Memory translation, allocation and deallocation happens here.
pub trait MemoryManagement {
    /// Translates physical addr + size into a kernel accessible slice.
    ///
    /// The simplest paddr_to_slice function would for example be just the
    /// identity function. But this may vary depending on how your page table
    /// layout looks like.
    ///
    /// If you only want to set fields, this can just always return `None`.
    ///
    /// # Safety
    /// Pretty unsafe. Translate a physical buffer in multiboot into something
    /// accessible in the current address space. Probably involves
    /// querying/knowledge about page-table setup. Also might want to verify
    /// that multiboot information is actually valid.
    unsafe fn paddr_to_slice(&self, addr: PAddr, length: usize) -> Option<&'static [u8]>;

    /// Allocates `length` bytes.
    ///
    /// The returned tuple consists of the physical address (that goes into the struct)
    /// and the slice which to use to write to.
    ///
    /// If you only want to read fields, this can just always return `None`.
    ///
    /// # Safety
    /// Lifetime of buffer should be >= self.
    unsafe fn allocate(&mut self, length: usize) -> Option<(PAddr, &mut [u8])>;

    /// Free the previously allocated memory.
    ///
    /// This should handle null pointers by doing nothing.
    ///
    /// If you only want to read fields, this can just always panic.
    ///
    /// # Safety
    /// TBD.
    unsafe fn deallocate(&mut self, addr: PAddr);
}

/// Multiboot struct clients mainly interact with
///
/// To create this use [`Multiboot::from_ptr`] or [`Multiboot::from_ref`].
///
/// [`Multiboot::from_ptr`]: struct.Multiboot.html#method.from_ptr
/// [`Multiboot::from_ref`]: struct.Multiboot.html#method.from_ref
pub struct Multiboot<'a, 'b> {
    header: &'a mut MultibootInfo,
    memory_management: &'b mut dyn MemoryManagement,
}

/// Representation of Multiboot Information according to specification.
///
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
/// 88      | framebuffer_addr  |    (present if flags[12] is set)
/// 96      | framebuffer_pitch |
/// 100     | framebuffer_width |
/// 104     | framebuffer_height|
/// 108     | framebuffer_bpp   |
/// 109     | framebuffer_type  |
/// 110-115 | color_info        |
///         +-------------------+
///</rawtext>
///
#[repr(C)]
#[derive(Default)]
pub struct MultibootInfo {
    flags: u32,

    mem_lower: u32,
    mem_upper: u32,

    boot_device: BootDevice,

    /// The command line is a normal C-style zero-terminated string.
    cmdline: u32,

    mods_count: u32,
    mods_addr: u32,

    symbols: Symbols,

    mmap_length: u32,
    mmap_addr: u32,

    drives_length: u32,
    drives_addr: u32,

    _config_table: u32,

    boot_loader_name: u32,

    _apm_table: u32,

    _vbe_control_info: u32,
    _vbe_mode_info: u32,
    _vbe_mode: u16,
    _vbe_interface_seg: u16,
    _vbe_interface_off: u16,
    _vbe_interface_len: u16,

    framebuffer_table: FramebufferTable,
}

/// Multiboot structure.
impl<'a, 'b> Multiboot<'a, 'b> {
    /// Initializes the multiboot structure from a passed address.
    ///
    /// This is the way to go, if you're writing a kernel.
    ///
    /// # Arguments
    ///
    ///  * `mboot_ptr` - The physical address of the multiboot header. On qemu for example
    ///                  this is typically at 0x9500.
    ///  * `memory_management` - Translation of the physical addresses into kernel addresses,
    ///                          allocation and deallocation of memory.
    ///                          See the [`MemoryManagement`] description for more details.
    ///
    /// # Safety
    /// The user must ensure that mboot_ptr holds the physical address of a valid
    /// Multiboot1 structure and that memory management provides correct translations.
    ///
    /// [`MemoryManagement`]: trait.MemoryManagement.html
    pub unsafe fn from_ptr(
        mboot_ptr: PAddr,
        memory_management: &'b mut dyn MemoryManagement,
    ) -> Option<Multiboot<'a, 'b>> {
        memory_management
            .paddr_to_slice(mboot_ptr, size_of::<MultibootInfo>())
            .map(move |inner| {
                let info = &mut *(inner.as_ptr() as *mut MultibootInfo);
                Multiboot {
                    header: info,
                    memory_management,
                }
            })
    }

    /// Initializes this struct from an already existing [`MultibootInfo`] reference.
    ///
    /// In combination with [`MultibootInfo::default`] this is useful for writing a bootloader.
    ///
    /// # Arguments
    ///
    ///  * `info` - The (mutable) reference to a [`MultibootInfo`] struct.
    ///  * `memory_management` - Translation of the physical addresses into kernel addresses,
    ///                          allocation and deallocation of memory.
    ///                          See the [`MemoryManagement`] description for more details.
    ///
    /// # Safety
    /// The user must ensure that the memory management can allocate memory.
    ///
    /// [`MultibootInfo`]: struct.MultibootInfo.html
    /// [`MultibootInfo::default`]: struct.MultibootInfo.html#impl-Default
    pub fn from_ref(
        info: &'a mut MultibootInfo,
        memory_management: &'b mut dyn MemoryManagement,
    ) -> Self {
        Self {
            header: info,
            memory_management,
        }
    }

    unsafe fn cast<T>(&self, addr: PAddr) -> Option<&T> {
        self.memory_management
            .paddr_to_slice(addr, size_of::<T>())
            .map(|inner| &*(inner.as_ptr() as *const T))
    }

    /// Convert a C string into a u8 slice and from there into a &str.
    /// This unsafe block builds on assumption that multiboot strings are sane.
    unsafe fn convert_c_string(&self, string: PAddr) -> Option<&'a str> {
        if string == 0 {
            return None;
        }
        let mut len = 0;
        let mut ptr = string;
        while let Some(byte) = self.memory_management.paddr_to_slice(ptr, 1) {
            if byte == [0] {
                break;
            }
            ptr += 1;
            len += 1;
        }
        self.memory_management
            .paddr_to_slice(string, len)
            .map(|slice| str::from_utf8_unchecked(slice))
    }

    /// Convert a &str into a u8 slice and from there into a C string.
    ///
    /// This unsafe block requires the possibility to allocate memory
    /// (and assumes that this memory can be addresses using an u32).
    unsafe fn convert_to_c_string(&mut self, string: Option<&str>) -> u32 {
        match string {
            Some(s) => {
                let bytes = s.bytes();
                let len = bytes.len();
                let (addr, slice) = self.memory_management.allocate(len + 1).unwrap();
                for (src, dst) in bytes.chain(core::iter::once(0)).zip(slice.iter_mut()) {
                    *dst = src;
                }
                addr.try_into().unwrap()
            }
            None => 0,
        }
    }

    flag!(
        doc = "If true, then the `mem_upper` and `mem_lower` fields are valid.",
        has_memory_bounds,
        0
    );
    flag!(
        doc = "If true, then the `boot_device` field is valid.",
        has_boot_device,
        1
    );
    flag!(
        doc = "If true, then the `cmdline` field is valid.",
        has_cmdline,
        2
    );
    flag!(
        doc = "If true, then the `mods_addr` and `mods_count` fields are valid.",
        has_modules,
        3
    );
    flag!(
        doc = "If true, then the `syms` field is valid and contains AOut symbols.",
        has_aout_symbols,
        4
    );
    flag!(
        doc = "If true, then the `syms` field is valid and containts ELF symbols.",
        has_elf_symbols,
        5
    );
    flag!(
        doc = "If true, then the `mmap_addr` and `mmap_length` fields are valid.",
        has_memory_map,
        6
    );
    flag!(
        doc = "If true, then the `drives_addr` and `drives_length` fields are valid.",
        has_drives,
        7
    );
    flag!(
        doc = "If true, then the `config_table` field is valid.",
        has_config_table,
        8
    );
    flag!(
        doc = "If true, then the `boot_loader_name` field is valid.",
        has_boot_loader_name,
        9
    );
    flag!(
        doc = "If true, then the `apm_table` field is valid.",
        has_apm_table,
        10
    );
    flag!(
        doc = "If true, then the `vbe_*` fields are valid.",
        has_vbe,
        11
    );
    flag!(
        doc = "If true, then the framebuffer table is valid.",
        has_framebuffer_table,
        12
    );

    /// Indicate the amount of lower memory in kilobytes.
    ///
    /// Lower memory starts at address 0. The maximum possible value for
    /// lower memory is 640 kilobytes.
    pub fn lower_memory_bound(&self) -> Option<u32> {
        match self.has_memory_bounds() {
            true => Some(self.header.mem_lower),
            false => None,
        }
    }

    /// Indicate the amount of upper memory in kilobytes.
    ///
    /// Upper memory starts at address 1 megabyte.
    /// The value returned for upper memory is maximally the address of
    /// the first upper memory hole minus 1 megabyte. It is not guaranteed
    /// to be this value.
    pub fn upper_memory_bound(&self) -> Option<u32> {
        match self.has_memory_bounds() {
            true => Some(self.header.mem_upper),
            false => None,
        }
    }

    /// Sets the memory bounds (lower, upper).
    ///
    /// This is one call because Multiboot requires both or none to be set.
    pub fn set_memory_bounds(&mut self, bounds: Option<(u32, u32)>) {
        self.set_has_memory_bounds(bounds.is_some());
        if let Some((lower, upper)) = bounds {
            self.header.mem_lower = lower;
            self.header.mem_upper = upper;
        }
    }

    /// Indicates which bios disk device the boot loader loaded the OS image from.
    ///
    /// If the OS image was not loaded from a bios disk, then this
    /// returns None.
    /// The operating system may use this field as a hint for determining its
    /// own root device, but is not required to.
    pub fn boot_device(&self) -> Option<BootDevice> {
        match self.has_boot_device() {
            true => Some(self.header.boot_device.clone()),
            false => None,
        }
    }

    /// Command line passed to the kernel.
    pub fn command_line(&self) -> Option<&'a str> {
        if self.has_cmdline() {
            unsafe { self.convert_c_string(self.header.cmdline as PAddr) }
        } else {
            None
        }
    }

    /// Command line to be passed to the kernel.
    ///
    /// The given string will be copied to newly allocated memory.
    pub fn set_command_line(&mut self, cmdline: Option<&str>) {
        // free the old string if it exists
        if self.has_cmdline() {
            unsafe {
                self.memory_management
                    .deallocate(self.header.cmdline.into())
            };
        }
        self.set_has_cmdline(cmdline.is_some());
        self.header.cmdline = unsafe { self.convert_to_c_string(cmdline) };
    }

    /// Get the name of the bootloader.
    pub fn boot_loader_name(&self) -> Option<&'a str> {
        if self.has_boot_loader_name() {
            unsafe { self.convert_c_string(self.header.boot_loader_name as PAddr) }
        } else {
            None
        }
    }

    /// Set the name of the bootloader.
    ///
    /// The given string will be copied to newly allocated memory.
    pub fn set_boot_loader_name(&mut self, name: Option<&str>) {
        // free the old string if it exists
        if self.has_boot_loader_name() {
            unsafe {
                self.memory_management
                    .deallocate(self.header.boot_loader_name.into())
            };
        }
        self.set_has_boot_loader_name(name.is_some());
        self.header.boot_loader_name = unsafe { self.convert_to_c_string(name) };
    }

    /// Discover all additional modules in multiboot.
    pub fn modules(&'a self) -> Option<ModuleIter<'a, 'b>> {
        if self.has_modules() {
            unsafe {
                self.memory_management
                    .paddr_to_slice(
                        self.header.mods_addr as PAddr,
                        self.header.mods_count as usize * size_of::<MBModule>(),
                    )
                    .map(|slice| {
                        let ptr = transmute(slice.as_ptr());
                        let mods = slice::from_raw_parts(ptr, self.header.mods_count as usize);
                        ModuleIter { mb: self, mods }
                    })
            }
        } else {
            None
        }
    }

    /// Publish modules to the kernel.
    ///
    /// This copies the given metadata into newly allocated memory.
    ///
    /// Note that the addresses in each [`Module`] must be and stay valid.
    ///
    /// [`Module`]: struct.Module.html
    pub fn set_modules(&mut self, modules: Option<&[Module]>) {
        // free the existing modules
        if self.has_modules() {
            unsafe {
                if let Some(mods) = self.memory_management.paddr_to_slice(
                    self.header.mods_addr.into(),
                    self.header.mods_count as usize * core::mem::size_of::<MBModule>(),
                ) {
                    let mods = slice::from_raw_parts(
                        mods.as_ptr().cast::<MBModule>(),
                        self.header.mods_count as usize,
                    );
                    for module in mods {
                        self.memory_management.deallocate(module.string.into());
                    }
                    self.memory_management
                        .deallocate(self.header.mods_addr.into());
                }
            }
        }
        self.set_has_modules(modules.is_some());
        if let Some(mods) = modules {
            let len = mods.len();
            self.header.mods_count = mods.len().try_into().unwrap();
            self.header.mods_addr = unsafe {
                let (addr, slice) = self
                    .memory_management
                    .allocate(len * core::mem::size_of::<MBModule>())
                    .unwrap();
                // change type
                let slice = slice::from_raw_parts_mut(slice.as_mut_ptr().cast::<MBModule>(), len);
                for (src, dst) in mods.iter().zip(slice.iter_mut()) {
                    *dst = MBModule {
                        start: src.start.try_into().unwrap(),
                        end: src.end.try_into().unwrap(),
                        string: self.convert_to_c_string(src.string),
                        reserved: 0,
                    }
                }
                addr.try_into().unwrap()
            };
        }
    }

    /// Get the symbols.
    pub fn symbols(&self) -> Option<SymbolType> {
        if self.has_elf_symbols() & self.has_aout_symbols() {
            // this is not supported
            return None;
        }
        if self.has_elf_symbols() {
            return Some(SymbolType::Elf(unsafe { self.header.symbols.elf }));
        }
        if self.has_aout_symbols() {
            return Some(SymbolType::AOut(unsafe { self.header.symbols.aout }));
        }
        None
    }

    /// Set the symbols.
    ///
    /// Note that the address in either [`AOutSymbols`] or [`ElfSymbols`] must stay valid.
    ///
    /// [`AOutSymbols`]: struct.AOutSymbols.html
    /// [`ElfSymbols`]: struct.ElfSymbols.html
    pub fn set_symbols(&mut self, symbols: Option<SymbolType>) {
        match symbols {
            None => {
                self.set_has_aout_symbols(false);
                self.set_has_elf_symbols(false);
            }
            Some(SymbolType::AOut(a)) => {
                self.set_has_aout_symbols(true);
                self.set_has_elf_symbols(false);
                self.header.symbols.aout = a;
            }
            Some(SymbolType::Elf(e)) => {
                self.set_has_aout_symbols(false);
                self.set_has_elf_symbols(true);
                self.header.symbols.elf = e;
            }
        }
    }

    /// Discover all memory regions in the multiboot memory map.
    pub fn memory_regions(&'a self) -> Option<MemoryMapIter<'a, 'b>> {
        match self.has_memory_map() {
            true => {
                let start = self.header.mmap_addr;
                let end = self.header.mmap_addr + self.header.mmap_length;
                Some(MemoryMapIter {
                    current: start,
                    end,
                    mb: self,
                })
            }
            false => None,
        }
    }

    /// Publish the memory regions to the kernel.
    ///
    /// The parameter is a pair of address and number of [`MemoryEntry`]s.
    ///
    /// Note that the underlying memory has to stay intact.
    ///
    /// [`MemoryEntry`]: struct.MemoryEntry.html
    pub fn set_memory_regions(&mut self, regions: Option<(PAddr, usize)>) {
        self.set_has_memory_map(regions.is_some());
        if let Some((addr, count)) = regions {
            self.header.mmap_addr = addr.try_into().unwrap();
            self.header.mmap_length = (count * core::mem::size_of::<MemoryEntry>())
                .try_into()
                .unwrap();
        }
    }

    /// Return end address of multiboot image.
    ///
    /// This function can be used to figure out a (hopefully) safe offset
    /// in the first region of memory to start using as free memory.
    pub fn find_highest_address(&self) -> PAddr {
        let end = cmp::max(
            self.header.cmdline as u64 + self.command_line().map_or(0, |f| f.len()) as u64,
            self.header.boot_loader_name as u64
                + self.boot_loader_name().map_or(0, |f| f.len()) as u64,
        )
        .max(match self.symbols() {
            Some(SymbolType::Elf(e)) => (e.addr + e.num * e.size) as u64,
            Some(SymbolType::AOut(a)) => {
                (a.addr + a.tabsize + a.strsize + 2 * core::mem::size_of::<u32>() as u32) as u64
            }
            None => 0,
        })
        .max((self.header.mmap_addr + self.header.mmap_length) as u64)
        .max((self.header.drives_addr + self.header.drives_length) as u64)
        .max(
            self.header.mods_addr as u64
                + self.header.mods_count as u64 * core::mem::size_of::<MBModule>() as u64,
        )
        .max(
            self.modules()
                .into_iter()
                .flatten()
                .map(|m| m.end)
                .max()
                .unwrap_or(0),
        );

        round_up!(end, 4096)
    }

    /// Return the framebuffer table, if it exists.
    pub fn framebuffer_table(&self) -> Option<&FramebufferTable> {
        if self.has_framebuffer_table() {
            Some(&self.header.framebuffer_table)
        } else {
            None
        }
    }

    /// Set the framebuffer table, if it exists.
    pub fn set_framebuffer_table(&mut self, table: Option<FramebufferTable>) {
        self.set_has_framebuffer_table(table.is_some());
        self.header.framebuffer_table = match table {
            Some(t) => t,
            None => FramebufferTable::default(),
        };
    }
}

/// The ‘boot_device’ field.
///
/// Partition numbers always start from zero. Unused partition
/// bytes must be set to 0xFF. For example, if the disk is partitioned
/// using a simple one-level DOS partitioning scheme, then
/// ‘part’ contains the DOS partition number, and ‘part2’ and ‘part3’
/// are both 0xFF. As another example, if a disk is partitioned first into
/// DOS partitions, and then one of those DOS partitions is subdivided
/// into several BSD partitions using BSD's disklabel strategy, then ‘part1’
/// contains the DOS partition number, ‘part2’ contains the BSD sub-partition
/// within that DOS partition, and ‘part3’ is 0xFF.
///
#[derive(Debug, Clone)]
#[repr(C)]
pub struct BootDevice {
    /// Contains the bios drive number as understood by
    /// the bios INT 0x13 low-level disk interface: e.g. 0x00 for the
    /// first floppy disk or 0x80 for the first hard disk.
    pub drive: u8,
    /// Specifies the top-level partition number.
    pub partition1: u8,
    /// Specifies a sub-partition in the top-level partition
    pub partition2: u8,
    /// Specifies a sub-partition in the 2nd-level partition
    pub partition3: u8,
}

impl BootDevice {
    /// Is partition1 a valid partition?
    pub fn partition1_is_valid(&self) -> bool {
        self.partition1 != 0xff
    }

    /// Is partition2 a valid partition?
    pub fn partition2_is_valid(&self) -> bool {
        self.partition2 != 0xff
    }

    /// Is partition3 a valid partition?
    pub fn partition3_is_valid(&self) -> bool {
        self.partition3 != 0xff
    }
}

impl Default for BootDevice {
    fn default() -> Self {
        Self {
            drive: 0xff,
            partition1: 0xff,
            partition2: 0xff,
            partition3: 0xff,
        }
    }
}

/// Types that define if the memory is usable or not.
#[derive(Debug, PartialEq, Eq)]
pub enum MemoryType {
    /// memory, available to OS
    Available = 1,
    /// reserved, not available (rom, mem map dev)
    Reserved = 2,
    /// ACPI Reclaim Memory
    ACPI = 3,
    /// ACPI NVS Memory
    NVS = 4,
    /// defective RAM modules
    Defect = 5,
}

/// Multiboot format of the MMAP buffer.
///
/// Note that size is defined to be at -4 bytes in multiboot.
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct MemoryEntry {
    size: u32,
    base_addr: u64,
    length: u64,
    mtype: u32,
}

impl Debug for MemoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let size = self.size;
        let base_addr = self.base_addr;
        let length = self.length;
        let mtype = self.mtype;
        write!(
            f,
            "MemoryEntry {{ size: {}, base_addr: {}, length: {}, mtype: {} }}",
            size, base_addr, length, mtype
        )
    }
}

impl Default for MemoryEntry {
    /// Get the "default" memory entry. (It's 0 bytes and reserved.)
    fn default() -> Self {
        Self::new(0, 0, MemoryType::Reserved)
    }
}

impl MemoryEntry {
    /// Create a new entry from the given data.
    ///
    /// Note that this will always create a struct that has a size of 20 bytes.
    pub fn new(base_addr: PAddr, length: PAddr, ty: MemoryType) -> Self {
        // the size field itself doesn't count
        let size = (core::mem::size_of::<MemoryEntry>() - core::mem::size_of::<u32>())
            .try_into()
            .unwrap();
        assert_eq!(size, 20);
        Self {
            size,
            base_addr,
            length,
            mtype: ty as u32,
        }
    }

    /// Get base of memory region.
    pub fn base_address(&self) -> PAddr {
        self.base_addr as PAddr
    }

    /// Get size of the memory region.
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Is the region type valid RAM?
    pub fn memory_type(&self) -> MemoryType {
        match self.mtype {
            1 => MemoryType::Available,
            3 => MemoryType::ACPI,
            4 => MemoryType::NVS,
            5 => MemoryType::Defect,
            _ => MemoryType::Reserved,
        }
    }
}

/// Used to iterate over all memory regions provided by multiboot.
pub struct MemoryMapIter<'a, 'b> {
    mb: &'a Multiboot<'a, 'b>,
    current: u32,
    end: u32,
}

impl<'a, 'b> Iterator for MemoryMapIter<'a, 'b> {
    type Item = &'a MemoryEntry;

    #[inline]
    fn next(&mut self) -> Option<&'a MemoryEntry> {
        if self.current < self.end {
            unsafe {
                self.mb
                    .cast(self.current as PAddr)
                    .map(|region: &'a MemoryEntry| {
                        self.current += region.size + 4;
                        region
                    })
            }
        } else {
            None
        }
    }
}

/// Multiboot format to information about module
#[repr(C)]
struct MBModule {
    /// Start address of module in memory.
    start: u32,

    /// End address of module in memory.
    end: u32,

    /// The `string` field provides an arbitrary string to be associated
    /// with that particular boot module.
    ///
    /// It is a zero-terminated ASCII string, just like the kernel command line.
    /// The `string` field may be 0 if there is no string associated with the module.
    /// Typically the string might be a command line (e.g. if the operating system
    /// treats boot modules as executable programs), or a pathname
    /// (e.g. if the operating system treats boot modules as files in a file system),
    /// but its exact use is specific to the operating system.
    string: u32,

    /// Must be zero.
    reserved: u32,
}

impl Debug for MBModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MBModule {{ start: {}, end: {}, string: {}, reserved: {} }}",
            self.start, self.end, self.string, self.reserved
        )
    }
}

/// Information about a module in multiboot.
#[derive(Debug)]
pub struct Module<'a> {
    /// Start address of module in physical memory.
    pub start: PAddr,
    /// End address of module in physic memory.
    pub end: PAddr,
    /// Name of the module.
    pub string: Option<&'a str>,
}

impl<'a> Module<'a> {
    pub fn new(start: PAddr, end: PAddr, name: Option<&'a str>) -> Module {
        Module {
            start,
            end,
            string: name,
        }
    }
}

/// Used to iterate over all modules in multiboot.
pub struct ModuleIter<'a, 'b> {
    mb: &'a Multiboot<'a, 'b>,
    mods: &'a [MBModule],
}

impl<'a, 'b> Iterator for ModuleIter<'a, 'b> {
    type Item = Module<'a>;

    #[inline]
    fn next(&mut self) -> Option<Module<'a>> {
        self.mods.split_first().map(|(first, rest)| {
            self.mods = rest;
            unsafe {
                Module::new(
                    first.start as PAddr,
                    first.end as PAddr,
                    self.mb.convert_c_string(first.string as PAddr),
                )
            }
        })
    }
}

/// Multiboot format for Symbols
#[repr(C)]
union Symbols {
    aout: AOutSymbols,
    elf: ElfSymbols,
    _bindgen_union_align: [u32; 4usize],
}

/// Safe wrapper for either [`AOutSymbols`] or [`ElfSymbols`]
///
/// [`AOutSymbols`]: struct.AOutSymbols.html
/// [`ElfSymbols`]: struct.ElfSymbols.html
#[derive(Debug, Copy, Clone)]
pub enum SymbolType {
    AOut(AOutSymbols),
    Elf(ElfSymbols),
}

impl Default for Symbols {
    fn default() -> Self {
        Self {
            elf: ElfSymbols::default(),
        }
    }
}

/// Multiboot format for AOut Symbols
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct AOutSymbols {
    tabsize: u32,
    strsize: u32,
    addr: u32,
    reserved: u32,
}

impl Debug for AOutSymbols {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AOutSymbols {{ tabsize: {}, strsize: {}, addr: {} }}",
            self.tabsize, self.strsize, self.addr
        )
    }
}

/// Multiboot format for ELF Symbols
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct ElfSymbols {
    num: u32,
    size: u32,
    addr: u32,
    shndx: u32,
}

impl Debug for ElfSymbols {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ElfSymbols {{ num: {}, size: {}, addr: {}, shndx: {} }}",
            self.num, self.size, self.addr, self.shndx
        )
    }
}

impl ElfSymbols {
    /// Uses a passed address for the symbols.
    ///
    /// Note that the underlying memory has to stay intact.
    ///
    /// Also, this doesn't check whether the supplied parameters are correct.
    pub fn from_addr(num: u32, size: u32, addr: PAddr, shndx: u32) -> Self {
        Self {
            num,
            size,
            shndx,
            addr: addr.try_into().unwrap(),
        }
    }
}

/// Contains the information about the framebuffer
#[repr(C)]
#[derive(Default)]
pub struct FramebufferTable {
    pub addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    ty: u8,
    color_info: ColorInfo,
}

impl fmt::Debug for FramebufferTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramebufferTable")
            .field("addr", &self.addr)
            .field("pitch", &self.pitch)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bpp", &self.bpp)
            .field("color_info", &self.color_info())
            .finish()
    }
}

impl FramebufferTable {
    /// Create this table from a color info.
    ///
    /// If the type is [`ColorInfoType::Text`], `bpp` has to be 16.
    ///
    /// [`ColorInfoType::Text`]: enum.ColorInfoType.html#variant.Text
    pub fn new(
        addr: u64,
        pitch: u32,
        width: u32,
        height: u32,
        bpp: u8,
        color_info_type: ColorInfoType,
    ) -> Self {
        let (ty, color_info) = match color_info_type {
            ColorInfoType::Palette(palette) => (0, ColorInfo { palette }),
            ColorInfoType::Rgb(rgb) => (1, ColorInfo { rgb }),
            ColorInfoType::Text => (2, ColorInfo::default()),
        };
        Self {
            addr,
            pitch,
            width,
            height,
            bpp,
            ty,
            color_info,
        }
    }

    /// Get the color info from this table.
    pub fn color_info(&self) -> Option<ColorInfoType> {
        unsafe {
            match self.ty {
                0 => Some(ColorInfoType::Palette(self.color_info.palette)),
                1 => Some(ColorInfoType::Rgb(self.color_info.rgb)),
                2 => Some(ColorInfoType::Text),
                _ => None,
            }
        }
    }
}

/// Safe wrapper for `ColorInfo`
#[derive(Debug)]
pub enum ColorInfoType {
    Palette(ColorInfoPalette),
    Rgb(ColorInfoRgb),
    Text,
}

/// Multiboot format for the frambuffer color info
///
/// According to the spec, if type == 0, it's indexed color and
///<rawtext>
///         +----------------------------------+
/// 110     | framebuffer_palette_addr         |
/// 114     | framebuffer_palette_num_colors   |
///         +----------------------------------+
///</rawtext>
/// The address points to an array of `ColorDescriptor`s.
/// If type == 1, it's RGB and
///<rawtext>
///        +----------------------------------+
///110     | framebuffer_red_field_position   |
///111     | framebuffer_red_mask_size        |
///112     | framebuffer_green_field_position |
///113     | framebuffer_green_mask_size      |
///114     | framebuffer_blue_field_position  |
///115     | framebuffer_blue_mask_size       |
///        +----------------------------------+
///</rawtext>
/// (If type == 2, it's just text.)
#[repr(C)]
union ColorInfo {
    palette: ColorInfoPalette,
    rgb: ColorInfoRgb,
    _union_align: [u32; 2usize],
}

// default type is 0, so indexed color
impl Default for ColorInfo {
    fn default() -> Self {
        Self {
            palette: ColorInfoPalette {
                palette_addr: 0,
                palette_num_colors: 0,
            },
        }
    }
}

/// Information for indexed color mode
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ColorInfoPalette {
    palette_addr: u32,
    palette_num_colors: u16,
}

/// Information for direct RGB color mode
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ColorInfoRgb {
    pub red_field_position: u8,
    pub red_mask_size: u8,
    pub green_field_position: u8,
    pub green_mask_size: u8,
    pub blue_field_position: u8,
    pub blue_mask_size: u8,
}
