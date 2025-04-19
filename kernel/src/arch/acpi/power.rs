use acpi::AmlTable;
use alloc::boxed::Box;
use aml::{AmlContext, AmlError};
use aml::{AmlName, AmlValue, DebugVerbosity};
use core::slice::from_raw_parts;
use x86_64::PhysAddr;
use x86_64::instructions::port::Port;
use x86_64::structures::port::{PortRead, PortWrite};

use super::ACPI;
use crate::mem::convert_physical_to_virtual;

pub fn reboot() -> ! {
    loop {
        let reset = ACPI.fadt.reset_register().unwrap();
        unsafe { Port::new(reset.address as u16).write(ACPI.fadt.reset_value) };
    }
}

pub fn shutdown() -> ! {
    let slp_typa = find_slp_typa(&ACPI.aml_table).unwrap();
    loop {
        let pm1a = ACPI.fadt.pm1a_control_block().unwrap();
        unsafe { Port::new(pm1a.address as u16).write(slp_typa | (1 << 13)) };
    }
}

fn find_slp_typa(aml_table: &AmlTable) -> Result<u16, AmlError> {
    let handler = Box::new(AmlHandler);
    let mut dsdt = AmlContext::new(handler, DebugVerbosity::None);

    dsdt.parse_table(unsafe {
        let physical_address = PhysAddr::new(aml_table.address as u64);
        let virtual_address = convert_physical_to_virtual(physical_address);
        from_raw_parts(virtual_address.as_ptr(), aml_table.length as usize)
    })?;

    match dsdt.namespace.get_by_path(&AmlName::from_str("\\_S5")?)? {
        AmlValue::Package(values) => Ok(values[0].as_integer(&dsdt)? as u16),
        _ => panic!("Failed to find S5 as it's not a package"),
    }
}

pub(crate) struct AmlHandler;

impl AmlHandler {
    fn read<T>(&self, address: usize) -> T {
        let address = convert_physical_to_virtual(PhysAddr::new(address as u64));
        unsafe { address.as_ptr::<T>().read_volatile() }
    }

    fn write<T>(&mut self, address: usize, value: T) {
        let address = convert_physical_to_virtual(PhysAddr::new(address as u64));
        unsafe { address.as_mut_ptr::<T>().write_volatile(value) }
    }

    fn read_io<T: PortRead>(&self, port: u16) -> T {
        unsafe { Port::new(port).read() }
    }

    fn write_io<T: PortWrite>(&self, port: u16, value: T) {
        unsafe { Port::new(port).write(value) }
    }

    fn read_pci<T>(&self, _: u16, _: u8, _: u8, _: u8, _: u16) -> T {
        unimplemented!()
    }

    fn write_pci<T>(&self, _: u16, _: u8, _: u8, _: u8, _: u16, _: T) {
        unimplemented!()
    }
}

macro_rules! aml_io {
    ([$($mut:tt)?] $($op:ident)?, $size:ty, ($($v:tt: $t:ty),+)) => {
        pastey::paste! {
            fn [<read_ $($op _)? $size>](&self, $($v: $t),+) -> $size {
                self.[<read $(_ $op)?>]::<$size>($($v),+)
            }
            fn [<write_ $($op _)? $size>](&$($mut)? self, $($v: $t),+, value: $size) {
                self.[<write $(_ $op)?>]::<$size>($($v),+, value)
            }
        }
    };
}

impl aml::Handler for AmlHandler {
    aml_io!([mut], u8, (address: usize));
    aml_io!([mut], u16, (address: usize));
    aml_io!([mut], u32, (address: usize));
    aml_io!([mut], u64, (address: usize));
    aml_io!([] io, u8, (port: u16));
    aml_io!([] io, u16, (port: u16));
    aml_io!([] io, u32, (port: u16));
    aml_io!([] pci, u8, (segment: u16, bus: u8, device: u8, function: u8, offset: u16));
    aml_io!([] pci, u16, (segment: u16, bus: u8, device: u8, function: u8, offset: u16));
    aml_io!([] pci, u32, (segment: u16, bus: u8, device: u8, function: u8, offset: u16));
}
