use core::num::NonZeroUsize;
use pci_types::device_type::DeviceType;
use x86_64::PhysAddr;
use x86_64::structures::paging::PhysFrame;
use xhci::Registers;
use xhci::accessor::Mapper;

use crate::mem::convert_physical_to_virtual;
use crate::mem::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

use super::pcie::PCI_DEVICES;

#[derive(Clone)]
pub struct XHCIMapper;

impl Mapper for XHCIMapper {
    unsafe fn map(&mut self, physical_start: usize, length: usize) -> NonZeroUsize {
        let physical_address = PhysAddr::new(physical_start as u64);
        let virtual_address = convert_physical_to_virtual(physical_address);

        MemoryManager::map_range_to(
            virtual_address,
            PhysFrame::containing_address(physical_address),
            length as u64,
            MappingType::KernelData.flags(),
            &mut KERNEL_PAGE_TABLE.lock(),
        )
        .unwrap();

        NonZeroUsize::new(virtual_address.as_u64() as usize).unwrap()
    }

    fn unmap(&mut self, _virt_start: usize, _bytes: usize) {}
}

pub fn test_xhci() {
    for device in PCI_DEVICES.lock().iter() {
        if device.device_type == DeviceType::UsbController {
            let Some(bar) = device.bars[0] else {
                continue;
            };
            let (address, _size) = bar.unwrap_mem();

            let mut xhci = unsafe { Registers::new(address, XHCIMapper) };
            let operational = &mut xhci.operational;

            operational.usbcmd.update_volatile(|usb_command_register| {
                usb_command_register.set_run_stop();
            });
            while operational.usbsts.read_volatile().hc_halted() {}

            let hcsparams1 = xhci.capability.hcsparams1.read_volatile();
            log::info!("XHCI Ports: {}", hcsparams1.number_of_ports());

            operational.usbcmd.update_volatile(|usb_command_register| {
                usb_command_register.set_host_controller_reset();
            });
        }
    }
}
