//! This module contains the pieces for parsing Multiboot headers.
//!
//! If you don't know where to start, take a look at [`Header`].
//!
//! [`Header`]: struct.Header.html

use core::convert::TryInto;
use core::fmt;

pub const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BADB002;

/// Multiboot struct bootloaders mainly interact with
#[derive(Copy, Clone)]
pub struct Header {
    header: MultibootHeader,
    /// the index at which the header starts
    pub header_start: u32,
}

/// Representation of a Multiboot header according to specification.
///
/// <rawtext>
///          +-------------------+
/// 0        | magic             |    (required)
/// 4        | flags             |    (required)
/// 8        | checksum          |    (required)
///          +-------------------+
/// 12       | header_addr       |    (present if flags[16] is set)
/// 16       | load_addr         |    (present if flags[16] is set)
/// 20       | load_end_addr     |    (present if flags[16] is set)
/// 24       | bss_end_addr      |    (present if flags[16] is set)
/// 28       | entry_addr        |    (present if flags[16] is set)
///          +-------------------+
/// 32       | mode_type         |    (present if flags[2] is set)
/// 36       | width             |    (present if flags[2] is set)
/// 40       | height            |    (present if flags[2] is set)
/// 44       | depth             |    (present if flags[2] is set)
///          +-------------------+
/// </rawtext>
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct MultibootHeader {
    magic: u32,
    flags: u32,
    checksum: u32,
    addresses: MultibootAddresses,
    video_mode: MultibootVideoMode,
}

impl Header {
    /// Get the header by parsing it from a slice.
    ///
    /// The needed portion of the slice is copied.
    pub fn from_slice(buffer: &[u8]) -> Option<Self> {
        // first, find the header
        let (header, header_start) = Self::find_header(buffer)?;
        // then check that it's valid
        assert_eq!(header.magic, MULTIBOOT_HEADER_MAGIC);
        assert_eq!(
            header
                .magic
                .wrapping_add(header.flags)
                .wrapping_add(header.checksum),
            0
        );
        // finally, return it
        Some(Self {
            header,
            header_start,
        })
    }

    /// Find the header and copy it from a given slice.
    fn find_header(buffer: &[u8]) -> Option<(MultibootHeader, u32)> {
        // the magic is 32 bit aligned and inside the first 8192 bytes
        let magic_index = match buffer.chunks_exact(4).take(8192 / 4).position(|vals| {
            u32::from_le_bytes(vals.try_into().unwrap()) // yes, there's 4 bytes here
            == MULTIBOOT_HEADER_MAGIC
        }) {
            Some(idx) => idx * 4,
            None => return None,
        };
        const HEADER_SIZE: usize = core::mem::size_of::<MultibootHeader>();
        // TryInto only works for lengths <= 32, so let's copy the stuff :(
        let mut header_bytes: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        buffer
            .iter()
            .skip(magic_index)
            .zip(header_bytes.iter_mut())
            .for_each(|(&buf, arr)| {
                *arr = buf;
            });
        let header =
            unsafe { core::mem::transmute::<[u8; HEADER_SIZE], MultibootHeader>(header_bytes) };
        Some((header, magic_index as u32))
    }

    flag!(
        doc = "If true, then the modules have to be page aligned.",
        wants_modules_page_aligned,
        0
    );
    flag!(
        doc = "If true, memory information must be passed.",
        wants_memory_information,
        1
    );
    flag!(
        doc = "If true, then the `mode_type`, `width`, `height` and `depth` fields are valid and
        video information has to be passed.",
        has_video_mode,
        2
    );
    flag!(
        doc = "If true, then the `header_addr`, `load_addr`, `load_end_addr`, `bss_end_addr`
        and `entry_addr` fields are valid and must be used to load the kernel.",
        has_multiboot_addresses,
        16
    );

    /// Get the load addresses specified in the Multiboot header.
    ///
    /// If this function returns `None` the binary has to be loaded as an ELF instead.
    pub fn get_addresses(&self) -> Option<MultibootAddresses> {
        if self.has_multiboot_addresses() {
            assert!(self.header.addresses.load_address <= self.header.addresses.header_address);
            Some(self.header.addresses)
        } else {
            None
        }
    }

    /// Get the preferred video mode specified in the Multiboot header.
    pub fn get_preferred_video_mode(&self) -> Option<MultibootVideoMode> {
        if self.has_video_mode() {
            Some(self.header.video_mode)
        } else {
            None
        }
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field(
                "wants_modules_page_aligned",
                &self.wants_modules_page_aligned(),
            )
            .field("wants_memory_information", &self.wants_memory_information())
            .field("addresses", &self.get_addresses())
            .field("video_mode", &self.get_preferred_video_mode())
            .finish()
    }
}

/// Addresses specified in the Multiboot header
///
/// If present, they must be used to load the kernel (regardless, what the ELF header says).
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MultibootAddresses {
    pub header_address: u32,
    pub load_address: u32,
    pub load_end_address: u32,
    pub bss_end_address: u32,
    pub entry_address: u32,
}

impl MultibootAddresses {
    /// Compute the offset of the load address into the binary.
    ///
    /// Multiboot 0.6.96: section "3.1.3 The address fields of Multiboot header" says
    /// this is the "offset at which the header was found, minus (header_addr - load_addr)".
    pub fn compute_load_offset(&self, header_start: u32) -> u32 {
        header_start - (self.header_address - self.load_address)
    }
}

/// Preferred video mode specified in the Multiboot header
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MultibootVideoMode {
    mode_type: u32,
    pub width: u32,
    pub height: u32,
    depth: u32,
}

impl MultibootVideoMode {
    /// Get the preferred video mode type
    pub fn mode_type(&self) -> Option<VideoModeType> {
        match self.mode_type {
            0 => Some(VideoModeType::LinearGraphics),
            1 => Some(VideoModeType::TextMode),
            _ => None,
        }
    }

    /// Get the preferred depth, if possible.
    ///
    /// Only pixel-based modes have a depth, text modes do not.
    pub fn depth(&self) -> Option<u32> {
        if self.mode_type() == Some(VideoModeType::LinearGraphics) {
            Some(self.depth)
        } else {
            None
        }
    }
}

impl fmt::Debug for MultibootVideoMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultibootVideoMode")
            .field("mode_type", &self.mode_type())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("depth", &self.depth())
            .finish()
    }
}

/// Preferred video mode type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VideoModeType {
    LinearGraphics,
    TextMode,
}
