use acpi::aml::AmlError;
use acpi::{PciAddress, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::PhysAddr;
use x86_64::instructions::port::Port;
use x86_64::structures::paging::PhysFrame;
use x86_64::structures::port::{PortRead, PortWrite};

use crate::mem::convert_physical_to_virtual;
use crate::mem::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

#[derive(Clone)]
pub struct AcpiHandler;

impl AcpiHandler {
    fn read<T>(&self, address: usize) -> T {
        let address = convert_physical_to_virtual(PhysAddr::new(address as u64));
        unsafe { address.as_ptr::<T>().read_volatile() }
    }

    fn write<T>(&self, address: usize, value: T) {
        let address = convert_physical_to_virtual(PhysAddr::new(address as u64));
        unsafe { address.as_mut_ptr::<T>().write_volatile(value) }
    }

    fn read_io<T: PortRead>(&self, port: u16) -> T {
        unsafe { Port::new(port).read() }
    }

    fn write_io<T: PortWrite>(&self, port: u16, value: T) {
        unsafe { Port::new(port).write(value) }
    }

    fn read_pci<T>(&self, _address: PciAddress, _offset: u16) -> T {
        unimplemented!()
    }

    fn write_pci<T>(&self, _address: PciAddress, _offset: u16, _value: T) {
        unimplemented!()
    }
}

macro_rules! aml_io {
    ([$($op:ident)?], $size:ty, ($($v:tt: $t:ty),+)) => {
        pastey::paste! {
            fn [<read_ $($op _)? $size>](&self, $($v: $t),+) -> $size {
                self.[<read $(_ $op)?>]::<$size>($($v),+)
            }
            fn [<write_ $($op _)? $size>](&self, $($v: $t),+, value: $size) {
                self.[<write $(_ $op)?>]::<$size>($($v),+, value)
            }
        }
    };
}

impl acpi::Handler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let address = PhysAddr::new(physical_address as u64);
        let virtual_address = convert_physical_to_virtual(address);

        <MemoryManager>::map_range_to(
            virtual_address,
            PhysFrame::containing_address(address),
            size as u64,
            MappingType::KernelData.flags(),
            &mut KERNEL_PAGE_TABLE.lock(),
        )
        .unwrap();

        let virtual_start = NonNull::new_unchecked(virtual_address.as_mut_ptr());

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start,
            region_length: size,
            mapped_length: size,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}

    aml_io!([], u8, (address: usize));
    aml_io!([], u16, (address: usize));
    aml_io!([], u32, (address: usize));
    aml_io!([], u64, (address: usize));
    aml_io!([io], u8, (port: u16));
    aml_io!([io], u16, (port: u16));
    aml_io!([io], u32, (port: u16));
    aml_io!([pci], u8, (address: PciAddress, offset: u16));
    aml_io!([pci], u16, (address: PciAddress, offset: u16));
    aml_io!([pci], u32, (address: PciAddress, offset: u16));

    fn nanos_since_boot(&self) -> u64 {
        todo!()
    }

    fn stall(&self, _microseconds: u64) {
        todo!()
    }

    fn sleep(&self, _milliseconds: u64) {
        todo!()
    }

    fn create_mutex(&self) -> acpi::Handle {
        acpi::Handle(0)
    }

    fn acquire(&self, _mutex: acpi::Handle, _timeout: u16) -> Result<(), AmlError> {
        todo!()
    }

    fn release(&self, _mutex: acpi::Handle) {
        todo!()
    }
}
