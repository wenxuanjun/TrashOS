use acpi::aml::namespace::AmlName;
use acpi::aml::object::Object;
use acpi::aml::{AmlError, Interpreter};
use core::str::FromStr;
use x86_64::instructions::port::Port;

use super::ACPI;
use super::handler::AcpiHandler;

pub fn reboot() -> ! {
    loop {
        let reset = ACPI.fadt.reset_register().unwrap();
        unsafe { Port::new(reset.address as u16).write(ACPI.fadt.reset_value) };
    }
}

pub fn shutdown() -> ! {
    let slp_typa = find_slp_typa(&ACPI.aml_engine).unwrap();
    loop {
        let pm1a = ACPI.fadt.pm1a_control_block().unwrap();
        unsafe { Port::new(pm1a.address as u16).write(slp_typa | (1 << 13)) };
    }
}

fn find_slp_typa(aml_engine: &Interpreter<AcpiHandler>) -> Result<u16, AmlError> {
    let path = AmlName::from_str("\\_S5")?;
    match *aml_engine.namespace.lock().get(path)? {
        Object::Package(ref values) => Ok(values[0].as_integer()? as u16),
        _ => panic!("Failed to find S5 as it's not a package"),
    }
}
