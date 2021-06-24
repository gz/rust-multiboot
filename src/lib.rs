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
//! [`Multiboot`]: information/struct.Multiboot.html
//! [`Header`]: header/struct.Header.html

#![no_std]
#![crate_name = "multiboot"]
#![crate_type = "lib"]

macro_rules! round_up {
    ($num:expr, $s:expr) => {
        (($num + $s - 1) / $s) * $s
    };
}

macro_rules! flag {
    ($doc:meta, $fun:ident, $bit:expr) => {
        #[$doc]
        pub fn $fun(&self) -> bool {
            //assert!($bit <= 31);
            (self.header.flags & (1 << $bit)) > 0
        }

        paste::paste! {
            #[$doc]
            pub fn [< set_ $fun >] (&mut self, flag: bool) {
                //assert!($bit <= 31);
                self.header.flags = if flag {
                    self.header.flags | (1 << $bit)
                } else {
                    self.header.flags & !(1 << $bit)
                };
            }
        }
    };
}

pub mod header;
pub mod information;

#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}
