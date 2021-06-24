extern crate multiboot;

use multiboot::header::{Header, VideoModeType};

#[test]
/// Find an empty header and check that nothing is set.
fn empty() {
    let header = [
        0xff, 0xff, 0xff, 0xff, // some stuff before
        0x02, 0xb0, 0xad, 0x1b, // header magic
        0x00, 0x00, 0x00, 0x00, // flags
        0xfe, 0x4f, 0x52, 0xe4, // checksum
        0x00, 0x00, 0x00, 0x00, // header_addr
        0x00, 0x00, 0x00, 0x00, // load_addr
        0x00, 0x00, 0x00, 0x00, // load_end_addr
        0x00, 0x00, 0x00, 0x00, // bss_end_addr
        0x00, 0x00, 0x00, 0x00, // entry_addr
        0x00, 0x00, 0x00, 0x00, // mode_type
        0x00, 0x00, 0x00, 0x00, // width
        0x00, 0x00, 0x00, 0x00, // height
        0x00, 0x00, 0x00, 0x00, // depth
        0xff, 0xff, 0xff, 0xff, // some stuff afterwards
    ];
    let parsed = Header::from_slice(&header).unwrap();
    assert!(!parsed.wants_memory_information());
    assert!(!parsed.wants_modules_page_aligned());
    assert!(!parsed.has_video_mode());
    assert!(!parsed.has_multiboot_addresses());
    assert!(parsed.get_addresses().is_none());
    assert!(parsed.get_preferred_video_mode().is_none());
}

#[test]
/// Find a header that wants modules to be page-aligned.
fn page_aligned() {
    let header = [
        0xff, 0xff, 0xff, 0xff, // some stuff before
        0x02, 0xb0, 0xad, 0x1b, // header magic
        0x01, 0x00, 0x00, 0x00, // flags
        0xfd, 0x4f, 0x52, 0xe4, // checksum
        0x00, 0x00, 0x00, 0x00, // header_addr
        0x00, 0x00, 0x00, 0x00, // load_addr
        0x00, 0x00, 0x00, 0x00, // load_end_addr
        0x00, 0x00, 0x00, 0x00, // bss_end_addr
        0x00, 0x00, 0x00, 0x00, // entry_addr
        0x00, 0x00, 0x00, 0x00, // mode_type
        0x00, 0x00, 0x00, 0x00, // width
        0x00, 0x00, 0x00, 0x00, // height
        0x00, 0x00, 0x00, 0x00, // depth
        0xff, 0xff, 0xff, 0xff, // some stuff afterwards
    ];
    let parsed = Header::from_slice(&header).unwrap();
    assert!(!parsed.wants_memory_information());
    assert!(parsed.wants_modules_page_aligned());
    assert!(!parsed.has_video_mode());
    assert!(!parsed.has_multiboot_addresses());
    assert!(parsed.get_addresses().is_none());
    assert!(parsed.get_preferred_video_mode().is_none());
}

#[test]
/// Find a header that wants memory information.
fn memory_information() {
    let header = [
        0xff, 0xff, 0xff, 0xff, // some stuff before
        0x02, 0xb0, 0xad, 0x1b, // header magic
        0x02, 0x00, 0x00, 0x00, // flags
        0xfc, 0x4f, 0x52, 0xe4, // checksum
        0x00, 0x00, 0x00, 0x00, // header_addr
        0x00, 0x00, 0x00, 0x00, // load_addr
        0x00, 0x00, 0x00, 0x00, // load_end_addr
        0x00, 0x00, 0x00, 0x00, // bss_end_addr
        0x00, 0x00, 0x00, 0x00, // entry_addr
        0x00, 0x00, 0x00, 0x00, // mode_type
        0x00, 0x00, 0x00, 0x00, // width
        0x00, 0x00, 0x00, 0x00, // height
        0x00, 0x00, 0x00, 0x00, // depth
        0xff, 0xff, 0xff, 0xff, // some stuff afterwards
    ];
    let parsed = Header::from_slice(&header).unwrap();
    assert!(parsed.wants_memory_information());
    assert!(!parsed.wants_modules_page_aligned());
    assert!(!parsed.has_video_mode());
    assert!(!parsed.has_multiboot_addresses());
    assert!(parsed.get_addresses().is_none());
    assert!(parsed.get_preferred_video_mode().is_none());
}

#[test]
/// Find an header with video mode information.
fn video_mode_pixel() {
    let header = [
        0xff, 0xff, 0xff, 0xff, // some stuff before
        0x02, 0xb0, 0xad, 0x1b, // header magic
        0x04, 0x00, 0x00, 0x00, // flags
        0xfa, 0x4f, 0x52, 0xe4, // checksum
        0x00, 0x00, 0x00, 0x00, // header_addr
        0x00, 0x00, 0x00, 0x00, // load_addr
        0x00, 0x00, 0x00, 0x00, // load_end_addr
        0x00, 0x00, 0x00, 0x00, // bss_end_addr
        0x00, 0x00, 0x00, 0x00, // entry_addr
        0x00, 0x00, 0x00, 0x00, // mode_type
        0x20, 0x03, 0x00, 0x00, // width
        0x58, 0x02, 0x00, 0x00, // height
        0x20, 0x00, 0x00, 0x00, // depth
        0xff, 0xff, 0xff, 0xff, // some stuff afterwards
    ];
    let parsed = Header::from_slice(&header).unwrap();
    assert!(!parsed.wants_memory_information());
    assert!(!parsed.wants_modules_page_aligned());
    assert!(parsed.has_video_mode());
    assert!(!parsed.has_multiboot_addresses());
    assert!(parsed.get_addresses().is_none());
    let video_mode = parsed.get_preferred_video_mode().unwrap();
    assert_eq!(video_mode.depth().unwrap(), 32);
    assert_eq!(
        video_mode.mode_type().unwrap(),
        VideoModeType::LinearGraphics
    );
}

#[test]
/// Find an header with video mode information.
fn video_mode_text() {
    let header = [
        0xff, 0xff, 0xff, 0xff, // some stuff before
        0x02, 0xb0, 0xad, 0x1b, // header magic
        0x04, 0x00, 0x00, 0x00, // flags
        0xfa, 0x4f, 0x52, 0xe4, // checksum
        0x00, 0x00, 0x00, 0x00, // header_addr
        0x00, 0x00, 0x00, 0x00, // load_addr
        0x00, 0x00, 0x00, 0x00, // load_end_addr
        0x00, 0x00, 0x00, 0x00, // bss_end_addr
        0x00, 0x00, 0x00, 0x00, // entry_addr
        0x01, 0x00, 0x00, 0x00, // mode_type
        0x20, 0x03, 0x00, 0x00, // width
        0x58, 0x02, 0x00, 0x00, // height
        0x00, 0x00, 0x00, 0x00, // depth
        0xff, 0xff, 0xff, 0xff, // some stuff afterwards
    ];
    let parsed = Header::from_slice(&header).unwrap();
    assert!(!parsed.wants_memory_information());
    assert!(!parsed.wants_modules_page_aligned());
    assert!(parsed.has_video_mode());
    assert!(!parsed.has_multiboot_addresses());
    assert!(parsed.get_addresses().is_none());
    let video_mode = parsed.get_preferred_video_mode().unwrap();
    assert!(video_mode.depth().is_none());
    assert_eq!(video_mode.mode_type().unwrap(), VideoModeType::TextMode);
}

#[test]
/// Find a header containing addresses.
fn addresses() {
    let header = [
        0xff, 0xff, 0xff, 0xff, // some stuff before
        0x02, 0xb0, 0xad, 0x1b, // header magic
        0x00, 0x00, 0x01, 0x00, // flags
        0xfe, 0x4f, 0x51, 0xe4, // checksum
        0x68, 0x00, 0x00, 0x00, // header_addr
        0x64, 0x00, 0x00, 0x00, // load_addr
        0xc8, 0x00, 0x00, 0x00, // load_end_addr
        0x2c, 0x01, 0x00, 0x00, // bss_end_addr
        0x78, 0x00, 0x00, 0x00, // entry_addr
        0x00, 0x00, 0x00, 0x00, // mode_type
        0x00, 0x00, 0x00, 0x00, // width
        0x00, 0x00, 0x00, 0x00, // height
        0x00, 0x00, 0x00, 0x00, // depth
        0xff, 0xff, 0xff, 0xff, // some stuff afterwards
    ];
    let parsed = Header::from_slice(&header).unwrap();
    assert!(!parsed.wants_memory_information());
    assert!(!parsed.wants_modules_page_aligned());
    assert!(!parsed.has_video_mode());
    assert!(parsed.has_multiboot_addresses());
    let addresses = parsed.get_addresses().unwrap();
    assert_eq!(addresses.header_address, 104);
    assert_eq!(addresses.load_address, 100);
    assert_eq!(addresses.load_end_address, 200);
    assert_eq!(addresses.bss_end_address, 300);
    assert_eq!(addresses.entry_address, 120);
    assert_eq!(addresses.compute_load_offset(4), 0);
    assert!(parsed.get_preferred_video_mode().is_none());
}

#[test]
/// No header
fn no_header() {
    let header = [0xff];
    let parsed = Header::from_slice(&header);
    assert!(parsed.is_none());
}
