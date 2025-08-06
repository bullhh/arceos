//! Common traits and types for network device (NIC) drivers.

// #![no_std]
// #![feature(const_mut_refs)]
// #![feature(const_slice_from_raw_parts_mut)]

use core::convert::From;
use core::{mem::ManuallyDrop, ptr::NonNull};
use alloc::{collections::VecDeque, sync::Arc};
use axdriver_base::{BaseDriverOps, DevError, DevResult, DeviceType};
use crate::{IgbDevice, IgbError, IgbNetBuf, MemPool, NicDevice};
// pub use crate::{IgbHal, PhysAddr, INTEL_82576, INTEL_VEND};
pub use crate::IgbHal;
use axdriver_net::{ NetBuf, NetBufPtr, NetBufBox, NetBufPool, NetDriverOps, EthernetAddress};

// pub use crate::net_buf::{NetBuf, NetBufBox, NetBufPool};



const RECV_BATCH_SIZE: usize = 64;
const RX_BUFFER_SIZE: usize = 1024;
const MEM_POOL: usize = 4096;
const MEM_POOL_ENTRY_SIZE: usize = 2048;

/// The ixgbe NIC device driver.
///
/// `QS` is the ixgbe queue size, `QN` is the ixgbe queue num.
pub struct IgbNic<H: IgbHal, const QS: usize, const QN: u16> {
    inner: IgbDevice<H, QS>,
    mem_pool: Arc<MemPool>,
    rx_buffer_queue: VecDeque<NetBufPtr>,
}

unsafe impl<H: IgbHal, const QS: usize, const QN: u16> Sync for IgbNic<H, QS, QN> {}
unsafe impl<H: IgbHal, const QS: usize, const QN: u16> Send for IgbNic<H, QS, QN> {}

impl<H: IgbHal, const QS: usize, const QN: u16> IgbNic<H, QS, QN> {
    /// Creates a net ixgbe NIC instance and initialize, or returns a error if
    /// any step fails.
    pub fn init(base: usize, len: usize) -> DevResult<Self> {
        let mem_pool = MemPool::allocate::<H>(MEM_POOL, MEM_POOL_ENTRY_SIZE)
            .map_err(|_| DevError::NoMemory)?;
        let inner = IgbDevice::<H, QS>::init(base, len, QN, QN, &mem_pool).map_err(|err| {
            log::error!("Failed to initialize ixgbe device: {:?}", err);
            DevError::BadState
        })?;

        let rx_buffer_queue = VecDeque::with_capacity(RX_BUFFER_SIZE);
        Ok(Self {
            inner,
            mem_pool,
            rx_buffer_queue,
        })
    }
}

impl<H: IgbHal, const QS: usize, const QN: u16> BaseDriverOps for IgbNic<H, QS, QN> {
    fn device_name(&self) -> &str {
        self.inner.get_driver_name()
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Net
    }
}

impl<H: IgbHal, const QS: usize, const QN: u16> NetDriverOps for IgbNic<H, QS, QN> {
    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.inner.get_mac_addr())
    }

    fn rx_queue_size(&self) -> usize {
        QS
    }

    fn tx_queue_size(&self) -> usize {
        QS
    }

    fn can_receive(&self) -> bool {
        !self.rx_buffer_queue.is_empty() || self.inner.can_receive(0).unwrap()
    }

    fn can_transmit(&self) -> bool {
        // Default implementation is return true forever.
        self.inner.can_send(0).unwrap()
    }

    fn recycle_rx_buffer(&mut self, rx_buf: NetBufPtr) -> DevResult {
        let rx_buf = igb_ptr_to_buf(rx_buf, &self.mem_pool)?;
        drop(rx_buf);
        Ok(())
    }

    fn recycle_tx_buffers(&mut self) -> DevResult {
        self.inner
            .recycle_tx_buffers(0)
            .map_err(|_| DevError::BadState)?;
        Ok(())
    }

    fn receive(&mut self) -> DevResult<NetBufPtr> {
        if !self.can_receive() {
            return Err(DevError::Again);
        }
        if !self.rx_buffer_queue.is_empty() {
            // RX buffer have received packets.
            Ok(self.rx_buffer_queue.pop_front().unwrap())
        } else {
            let f = |rx_buf| {
                let rx_buf = NetBufPtr::from(rx_buf);
                self.rx_buffer_queue.push_back(rx_buf);
            };

            // RX queue is empty, receive from ixgbe NIC.
            match self.inner.receive_packets(0, RECV_BATCH_SIZE, f) {
                Ok(recv_nums) => {
                    if recv_nums == 0 {
                        // No packet is received, it is impossible things.
                        panic!("Error: No receive packets.")
                    } else {
                        Ok(self.rx_buffer_queue.pop_front().unwrap())
                    }
                }
                Err(e) => match e {
                    IgbError::NotReady => Err(DevError::Again),
                    _ => Err(DevError::BadState),
                },
            }
        }
    }

    fn transmit(&mut self, tx_buf: NetBufPtr) -> DevResult {
        let tx_buf = igb_ptr_to_buf(tx_buf, &self.mem_pool)?;
        match self.inner.send(0, tx_buf) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                IgbError::QueueFull => Err(DevError::Again),
                _ => panic!("Unexpected err: {:?}", err),
            },
        }
    }

    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<NetBufPtr> {
        let tx_buf = IgbNetBuf::alloc(&self.mem_pool, size).map_err(|_| DevError::NoMemory)?;
        Ok(NetBufPtr::from(tx_buf))
    }
}

impl From<IgbNetBuf> for NetBufPtr {
    fn from(buf: IgbNetBuf) -> Self {
        // Use `ManuallyDrop` to avoid drop `tx_buf`.
        let mut buf = ManuallyDrop::new(buf);
        // In ixgbe, `raw_ptr` is the pool entry, `buf_ptr` is the packet ptr, `len` is packet len
        // to avoid too many dynamic memory allocation.
        let buf_ptr = buf.packet_mut().as_mut_ptr();
        Self::new(
            NonNull::new(buf.pool_entry() as *mut u8).unwrap(),
            NonNull::new(buf_ptr).unwrap(),
            buf.packet_len(),
        )
    }
}

// Converts a `NetBufPtr` to `IxgbeNetBuf`.
fn igb_ptr_to_buf(ptr: NetBufPtr, pool: &Arc<MemPool>) -> DevResult<IgbNetBuf> {
    let pool_entry = ptr.raw_ptr::<u8>() as usize;
    IgbNetBuf::construct(pool_entry, pool, ptr.packet_len())
        .map_err(|_| DevError::BadState)
}



use alloc::vec::Vec;

use virtio_drivers::{device::net::VirtIONetRaw as InnerDev, transport::Transport, Hal};

/// The VirtIO network device driver.
///
/// `QS` is the VirtIO queue size.
pub struct VirtIoNetDev<H: Hal, T: Transport, const QS: usize> {
    rx_buffers: [Option<NetBufBox>; QS],
    tx_buffers: [Option<NetBufBox>; QS],
    free_tx_bufs: Vec<NetBufBox>,
    buf_pool: Arc<NetBufPool>,
    inner: InnerDev<H, T, QS>,
}

const NET_BUF_LEN: usize = 1526;

unsafe impl<H: Hal, T: Transport, const QS: usize> Send for VirtIoNetDev<H, T, QS> {}
unsafe impl<H: Hal, T: Transport, const QS: usize> Sync for VirtIoNetDev<H, T, QS> {}

impl<H: Hal, T: Transport, const QS: usize> VirtIoNetDev<H, T, QS> {
    /// Creates a new driver instance and initializes the device, or returns
    /// an error if any step fails.
    pub fn try_new(transport: T) -> DevResult<Self> {
        // 0. Create a new driver instance.
        const NONE_BUF: Option<NetBufBox> = None;
        let inner = InnerDev::new(transport).map_err(|_| DevError::BadState)?;
        let rx_buffers = [NONE_BUF; QS];
        let tx_buffers = [NONE_BUF; QS];
        let buf_pool = NetBufPool::new(2 * QS, NET_BUF_LEN)?;
        let free_tx_bufs = Vec::with_capacity(QS);

        let mut dev = Self {
            rx_buffers,
            inner,
            tx_buffers,
            free_tx_bufs,
            buf_pool,
        };

        // 1. Fill all rx buffers.
        for (i, rx_buf_place) in dev.rx_buffers.iter_mut().enumerate() {
            let mut rx_buf = dev.buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?;
            // Safe because the buffer lives as long as the queue.
            let token = unsafe {
                dev.inner
                    .receive_begin(rx_buf.raw_buf_mut())
                    .map_err(|_| DevError::BadState)?
            };
            assert_eq!(token, i as u16);
            *rx_buf_place = Some(rx_buf);
        }

        // 2. Allocate all tx buffers.
        for _ in 0..QS {
            let mut tx_buf = dev.buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?;
            // Fill header
            let hdr_len = dev
                .inner
                .fill_buffer_header(tx_buf.raw_buf_mut())
                .or(Err(DevError::InvalidParam))?;
            tx_buf.set_header_len(hdr_len);
            dev.free_tx_bufs.push(tx_buf);
        }

        // 3. Return the driver instance.
        Ok(dev)
    }
}

impl<H: Hal, T: Transport, const QS: usize> BaseDriverOps for VirtIoNetDev<H, T, QS> {
    fn device_name(&self) -> &str {
        "virtio-net"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Net
    }
}

impl<H: Hal, T: Transport, const QS: usize> NetDriverOps for VirtIoNetDev<H, T, QS> {
    #[inline]
    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.inner.mac_address())
    }

    #[inline]
    fn can_transmit(&self) -> bool {
        !self.free_tx_bufs.is_empty() && self.inner.can_send()
    }

    #[inline]
    fn can_receive(&self) -> bool {
        self.inner.poll_receive().is_some()
    }

    #[inline]
    fn rx_queue_size(&self) -> usize {
        QS
    }

    #[inline]
    fn tx_queue_size(&self) -> usize {
        QS
    }

    fn recycle_rx_buffer(&mut self, rx_buf: NetBufPtr) -> DevResult {
        let mut rx_buf = unsafe { NetBuf::from_buf_ptr(rx_buf) };
        // Safe because we take the ownership of `rx_buf` back to `rx_buffers`,
        // it lives as long as the queue.
        let new_token = unsafe {
            self.inner
                .receive_begin(rx_buf.raw_buf_mut())
                .map_err(|_| DevError::BadState)?
        };
        // `rx_buffers[new_token]` is expected to be `None` since it was taken
        // away at `Self::receive()` and has not been added back.
        if self.rx_buffers[new_token as usize].is_some() {
            return Err(DevError::BadState);
        }
        self.rx_buffers[new_token as usize] = Some(rx_buf);
        Ok(())
    }

    fn recycle_tx_buffers(&mut self) -> DevResult {
        while let Some(token) = self.inner.poll_transmit() {
            let tx_buf = self.tx_buffers[token as usize]
                .take()
                .ok_or(DevError::BadState)?;
            unsafe {
                self.inner
                    .transmit_complete(token, tx_buf.packet_with_header())
                    .map_err(|_| DevError::BadState)?;
            }
            // Recycle the buffer.
            self.free_tx_bufs.push(tx_buf);
        }
        Ok(())
    }

    fn transmit(&mut self, tx_buf: NetBufPtr) -> DevResult {
        // 0. prepare tx buffer.
        let tx_buf = unsafe { NetBuf::from_buf_ptr(tx_buf) };
        // 1. transmit packet.
        let token = unsafe {
            self.inner
                .transmit_begin(tx_buf.packet_with_header())
                .map_err(|_| DevError::BadState)?
        };
        self.tx_buffers[token as usize] = Some(tx_buf);
        Ok(())
    }

    fn receive(&mut self) -> DevResult<NetBufPtr> {
        if let Some(token) = self.inner.poll_receive() {
            let mut rx_buf = self.rx_buffers[token as usize]
                .take()
                .ok_or(DevError::BadState)?;
            // Safe because the buffer lives as long as the queue.
            let (hdr_len, pkt_len) = unsafe {
                self.inner
                    .receive_complete(token, rx_buf.raw_buf_mut())
                    .map_err(|_| DevError::BadState)?
            };
            rx_buf.set_header_len(hdr_len);
            rx_buf.set_packet_len(pkt_len);

            Ok(rx_buf.into_buf_ptr())
        } else {
            Err(DevError::Again)
        }
    }

    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<NetBufPtr> {
        // 0. Allocate a buffer from the queue.
        let mut net_buf = self.free_tx_bufs.pop().ok_or(DevError::NoMemory)?;
        let pkt_len = size;

        // 1. Check if the buffer is large enough.
        let hdr_len = net_buf.header_len();
        if hdr_len + pkt_len > net_buf.capacity() {
            return Err(DevError::InvalidParam);
        }
        net_buf.set_packet_len(pkt_len);

        // 2. Return the buffer.
        Ok(net_buf.into_buf_ptr())
    }
}