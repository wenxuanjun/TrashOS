use limine::request::SmpRequest;

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

pub fn init() {
    let response = SMP_REQUEST.get_response().unwrap();

    for cpu in response.cpus() {
        log::debug!(
            "CPU id: {}, lapic id: {}, extra: {:?}",
            cpu.id,
            cpu.lapic_id,
            cpu.extra
        );
    }
}
