use memory_addr::PhysAddr;
use somehal::mem::cpu_idx_to_id;

pub fn start_secondary_cpu(cpu_idx: usize, stack_top: PhysAddr) {
    unsafe {
        let cpu_id = cpu_idx_to_id(cpu_idx.into());
        somehal::mp::cpu_on(cpu_id);
    }
}
