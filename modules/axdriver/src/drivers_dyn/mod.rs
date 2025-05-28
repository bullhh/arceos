use alloc::boxed::Box;
use axdriver_base::{BaseDriverOps, DevError, DevResult, DeviceType};
use axdriver_block::BlockDriverOps;
pub use rdrive::dev_list;
use rdrive::{DeviceWeak, ErrorBase, block};

#[cfg(target_arch = "aarch64")]
mod rockchip;

pub struct Block(DeviceWeak<Box<dyn block::Interface + 'static>>);

impl BaseDriverOps for Block {
    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }
    fn device_name(&self) -> &str {
        self.0.descriptor.name
    }
}

impl BlockDriverOps for Block {
    fn num_blocks(&self) -> u64 {
        self.0.spin_try_borrow_by(0.into()).unwrap().num_blocks()
    }
    fn block_size(&self) -> usize {
        self.0.spin_try_borrow_by(0.into()).unwrap().block_size()
    }
    fn flush(&mut self) -> DevResult {
        self.0
            .spin_try_borrow_by(0.into())
            .unwrap()
            .flush()
            .map_err(convert_error)
    }

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult {
        self.0
            .spin_try_borrow_by(0.into())
            .unwrap()
            .read_block(block_id, buf)
            .map_err(convert_error)
    }

    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult {
        self.0
            .spin_try_borrow_by(0.into())
            .unwrap()
            .write_block(block_id, buf)
            .map_err(convert_error)
    }
}

impl From<DeviceWeak<Box<dyn block::Interface + 'static>>> for Block {
    fn from(base: DeviceWeak<Box<dyn block::Interface + 'static>>) -> Self {
        Self(base)
    }
}

fn convert_error(err: ErrorBase) -> DevError {
    match err {
        ErrorBase::Io => DevError::Io,
        ErrorBase::NoMem => DevError::NoMemory,
        ErrorBase::Again => DevError::Again,
        ErrorBase::Busy => DevError::ResourceBusy,
        ErrorBase::BadAddr(_) => DevError::BadState,
        ErrorBase::InvalidArg { .. } => DevError::InvalidParam,
    }
}
