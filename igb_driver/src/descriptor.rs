use bit_field::BitField;
use volatile::Volatile;

// Transmit descriptor bits
/// Tx Command: End of Packet
pub const TX_CMD_EOP: u8 = 1 << 0;
/// Tx Command: Insert MAC FCS
pub const TX_CMD_IFCS: u8 = 1 << 1;
/// Tx Command: Report Status
pub const TX_CMD_RS: u8 = 1 << 3;
/// Tx Command: Descriptor Extension (Advanced format)
pub const TX_CMD_DEXT: u8 = 1 << 5;
/// Tx Command: VLAN Packet Enable
pub const TX_CMD_VLE: u8 = 1 << 6;
/// Tx Command: TCP/UDP Segmentation Enable
pub const TX_CMD_TSE: u8 = 1 << 7;
/// Tx Status: descriptor Done
pub const TX_STATUS_DD: u8 = 1 << 0;
/// Tx Descriptor Type: advanced
pub const TX_DTYP_ADV: u8 = 0x3 << 4;
/// Tx Descriptor paylen shift
/// The paylen is located at bit 46 in the upper 64 bits of the advanced Tx descriptor.
/// Since we have divided the upper 64 bits into 4 parts (u16,u8,u8,u32),
/// the paylen is then located at bit 14 of the upper 32 bits of the descriptor.
pub const TX_PAYLEN_SHIFT: u8 = 46 - 32; //(actual offset - offset of variable)

// Receive descriptor bits
/// Rx Status: Descriptor Done
pub const RX_STATUS_DD: u8 = 1 << 0;
/// Rx Status: End of Packet
pub const RX_STATUS_EOP: u8 = 1 << 1;

/// refer: [Theseus](https://github.com/theseus-os/Theseus/blob/theseus_main/kernel/intel_ethernet/src/descriptors.rs#L218-L219)
/// Advanced Receive Descriptor used in the Ixgbe driver.
/// It has 2 modes: Read and Write Back, both of which use the whole 128 bits.
/// There is one receive descriptor per receive buffer that can be converted between these 2 modes.
/// Read contains the addresses that the driver writes.
/// Write Back contains information the hardware writes on receiving a packet.
/// More information can be found in the 82599 datasheet.
pub(crate) struct AdvancedRxDescriptor {
    /// Starting physical address of the receive bufffer for the packet.
    pub packet_buffer_address: Volatile<u64>,
    /// Starting physical address of the receive buffer for the header.
    /// This field will only be used if header splitting is enabled.
    pub header_buffer_address: Volatile<u64>,
}

impl AdvancedRxDescriptor {
    /// Initializes a receive descriptor by clearing its status
    /// and setting the descriptor's physical address.
    ///
    /// # Arguments
    /// * `packet_buffer_address`: starting physical address of the receive buffer.
    pub fn init(&mut self) {
        self.packet_buffer_address.write(0);
        self.header_buffer_address.write(0);
    }

    /// Updates the descriptor's physical address.
    ///
    /// # Arguments
    /// * `packet_buffer_address`: starting physical address of the receive buffer.
    pub fn set_packet_address(&mut self, packet_buffer_address: u64) {
        self.packet_buffer_address.write(packet_buffer_address);
    }

    /// Clears the status bits of the descriptor.
    pub fn reset_status(&mut self) {
        self.header_buffer_address.write(0);
    }

    /// Returns true if the descriptor has a received packet copied to its buffer.
    pub fn descriptor_done(&self) -> bool {
        (self.get_ext_status() & RX_STATUS_DD as u64) == RX_STATUS_DD as u64
    }

    /// Returns true if the descriptor's packet buffer is the last in a frame.
    pub fn end_of_packet(&self) -> bool {
        (self.get_ext_status() & RX_STATUS_EOP as u64) == RX_STATUS_EOP as u64
    }

    /// The length of the packet in the descriptor's packet buffer.
    pub fn length(&self) -> u64 {
        self.get_pkt_len()
    }

    /// Write Back mode function for the Advanced Receive Descriptor.
    /// Returns the packet type that was used for the Receive Side Scaling hash function.
    pub fn get_rss_type(&self) -> u64 {
        self.packet_buffer_address.read().get_bits(0..3)
    }

    /// Write Back mode function for the Advanced Receive Descriptor.
    /// Returns the packet type as identified by the hardware.
    pub fn get_packet_type(&self) -> u64 {
        self.packet_buffer_address.read().get_bits(4..16)
    }

    /// Write Back mode function for the Advanced Receive Descriptor.
    /// Returns the size of the packet header in bytes.
    pub fn get_hdr_len(&self) -> u64 {
        self.packet_buffer_address.read().get_bits(21..30)
    }

    /// Write Back mode function for the Advanced Receive Descriptor.
    /// Returns the Receive Side Scaling hash.
    pub fn get_rss_hash(&self) -> u64 {
        self.packet_buffer_address.read().get_bits(32..63)
    }

    /// Write Back mode function for the Advanced Receive Descriptor.
    /// Status information indicates whether a descriptor has been used
    /// and whether the buffer is the last one for a packet
    pub fn get_ext_status(&self) -> u64 {
        self.header_buffer_address.read().get_bits(0..19)
    }

    /// Write Back mode function for the Advanced Receive Descriptor.
    /// Returns errors reported by hardware for different packet types
    pub fn get_ext_error(&self) -> u64 {
        self.header_buffer_address.read().get_bits(20..31)
    }

    /// Write Back mode function for the Advanced Receive Descriptor.
    /// Returns the number of bytes posted to the packet buffer
    pub fn get_pkt_len(&self) -> u64 {
        self.header_buffer_address.read().get_bits(32..47)
    }

    /// Write Back mode function for the Advanced Receive Descriptor.
    /// If the vlan header is stripped from the packet, then the 16 bits of the VLAN tag are posted here
    pub fn get_vlan_tag(&self) -> u64 {
        self.header_buffer_address.read().get_bits(48..63)
    }
}

/// Advanced Transmit Descriptor used by the `ixgbe` NIC driver.
///
/// # Two usage modes
/// It has 2 modes: Read and Write Back, both of which use the whole 128 bits.
/// There is one transmit descriptor per transmit buffer; it can be converted between these 2 modes.
///
/// Read contains the addresses that the driver writes.
/// Write Back contains information the hardware writes on receiving a packet.
///
/// More information can be found in the 82599 datasheet.
#[repr(C)]
pub(crate) struct AdvancedTxDescriptor {
    /// Starting physical address of the receive buffer for the packet.
    pub packet_buffer_address: Volatile<u64>,
    /// Length of data buffer
    pub data_len: Volatile<u16>,
    /// A multi-part field:
    /// * `dtyp`: Descriptor Type, occupies bits `[7:4]`,
    /// * `mac`: options to apply LinkSec and time stamp, occupies bits `[3:2]`.
    pub dtyp_mac_rsv: Volatile<u8>,
    /// Command bits
    pub dcmd: Volatile<u8>,
    /// A multi-part field:
    /// * `paylen`: the size in bytes of the data buffer in host memory.
    ///   not including the fields that the hardware adds), occupies bits `[31:14]`.
    /// * `popts`: options to offload checksum calculation, occupies bits `[13:8]`.
    /// * `sta`: status of the descriptor (whether it's in use or not), occupies bits `[3:0]`.
    pub paylen_popts_cc_idx_sta: Volatile<u32>,
}

impl AdvancedTxDescriptor {
    /// Initializes a transmit descriptor by clearing all of its values.
    pub fn init(&mut self) {
        self.packet_buffer_address.write(0);
        self.paylen_popts_cc_idx_sta.write(0);
        self.dcmd.write(0);
        self.dtyp_mac_rsv.write(0);
        self.data_len.write(0);
    }

    /// Updates the transmit descriptor to send the packet.
    /// We assume that one transmit descriptor will be used to send one packet.
    ///
    /// # Arguments
    /// * `transmit_buffer_addr`: physical address of the transmit buffer.
    /// * `transmit_buffer_length`: length of packet we want to send.
    pub fn send(&mut self, transmit_buffer_addr: u64, transmit_buffer_length: u16) {
        self.packet_buffer_address.write(transmit_buffer_addr);
        self.data_len.write(transmit_buffer_length);
        self.dtyp_mac_rsv.write(TX_DTYP_ADV);
        self.paylen_popts_cc_idx_sta
            .write((transmit_buffer_length as u32) << TX_PAYLEN_SHIFT);
        self.dcmd
            .write(TX_CMD_DEXT | TX_CMD_RS | TX_CMD_IFCS | TX_CMD_EOP);
    }

    /// Polls the Descriptor Done bit until the packet has been sent.
    #[allow(clippy::while_immutable_condition)]
    pub fn wait_for_packet_tx(&self) {
        while (self.paylen_popts_cc_idx_sta.read() as u8 & TX_STATUS_DD) == 0 {}
    }
}
