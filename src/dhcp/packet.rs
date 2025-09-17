use std::net::Ipv4Addr;
use anyhow::{anyhow, Result};
use bytes::{BytesMut, BufMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpMessageType {
    Discover = 1,
    Offer = 2,
    Request = 3,
    Decline = 4,
    Ack = 5,
    Nak = 6,
    Release = 7,
    Inform = 8,
}

impl TryFrom<u8> for DhcpMessageType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(DhcpMessageType::Discover),
            2 => Ok(DhcpMessageType::Offer),
            3 => Ok(DhcpMessageType::Request),
            4 => Ok(DhcpMessageType::Decline),
            5 => Ok(DhcpMessageType::Ack),
            6 => Ok(DhcpMessageType::Nak),
            7 => Ok(DhcpMessageType::Release),
            8 => Ok(DhcpMessageType::Inform),
            _ => Err(anyhow!("Invalid DHCP message type: {}", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DhcpPacket {
    pub op: u8,
    pub htype: u8,
    pub hlen: u8,
    pub hops: u8,
    pub xid: u32,
    pub secs: u16,
    pub flags: u16,
    pub ciaddr: Ipv4Addr,
    pub yiaddr: Ipv4Addr,
    pub siaddr: Ipv4Addr,
    pub giaddr: Ipv4Addr,
    pub chaddr: [u8; 16],
    pub sname: [u8; 64],
    pub file: [u8; 128],
    pub options: Vec<DhcpOption>,
}

#[derive(Debug, Clone)]
pub struct DhcpOption {
    pub code: u8,
    pub data: Vec<u8>,
}

impl DhcpPacket {
    const MAGIC_COOKIE: [u8; 4] = [0x63, 0x82, 0x53, 0x63];
    const MIN_PACKET_SIZE: usize = 236;

    pub fn new() -> Self {
        Self {
            op: 1,
            htype: 1,
            hlen: 6,
            hops: 0,
            xid: 0,
            secs: 0,
            flags: 0,
            ciaddr: Ipv4Addr::UNSPECIFIED,
            yiaddr: Ipv4Addr::UNSPECIFIED,
            siaddr: Ipv4Addr::UNSPECIFIED,
            giaddr: Ipv4Addr::UNSPECIFIED,
            chaddr: [0; 16],
            sname: [0; 64],
            file: [0; 128],
            options: Vec::new(),
        }
    }

    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < Self::MIN_PACKET_SIZE {
            return Err(anyhow!("DHCP packet too short: {} bytes", data.len()));
        }

        let mut packet = DhcpPacket::new();

        packet.op = data[0];
        packet.htype = data[1];
        packet.hlen = data[2];
        packet.hops = data[3];
        packet.xid = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        packet.secs = u16::from_be_bytes([data[8], data[9]]);
        packet.flags = u16::from_be_bytes([data[10], data[11]]);
        packet.ciaddr = Ipv4Addr::from([data[12], data[13], data[14], data[15]]);
        packet.yiaddr = Ipv4Addr::from([data[16], data[17], data[18], data[19]]);
        packet.siaddr = Ipv4Addr::from([data[20], data[21], data[22], data[23]]);
        packet.giaddr = Ipv4Addr::from([data[24], data[25], data[26], data[27]]);

        packet.chaddr.copy_from_slice(&data[28..44]);
        packet.sname.copy_from_slice(&data[44..108]);
        packet.file.copy_from_slice(&data[108..236]);

        // Parse options if present
        if data.len() > 240 && &data[236..240] == &Self::MAGIC_COOKIE {
            packet.options = Self::parse_options(&data[240..])?;
        }

        Ok(packet)
    }

    fn parse_options(data: &[u8]) -> Result<Vec<DhcpOption>> {
        let mut options = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let code = data[i];
            i += 1;

            if code == 255 {
                break;
            }

            if code == 0 {
                continue;
            }

            if i >= data.len() {
                break;
            }

            let len = data[i] as usize;
            i += 1;

            if i + len > data.len() {
                break;
            }

            let option_data = data[i..i + len].to_vec();
            i += len;

            options.push(DhcpOption {
                code,
                data: option_data,
            });
        }

        Ok(options)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(576);

        buffer.put_u8(self.op);
        buffer.put_u8(self.htype);
        buffer.put_u8(self.hlen);
        buffer.put_u8(self.hops);
        buffer.put_u32(self.xid);
        buffer.put_u16(self.secs);
        buffer.put_u16(self.flags);
        buffer.put_slice(&self.ciaddr.octets());
        buffer.put_slice(&self.yiaddr.octets());
        buffer.put_slice(&self.siaddr.octets());
        buffer.put_slice(&self.giaddr.octets());
        buffer.put_slice(&self.chaddr);
        buffer.put_slice(&self.sname);
        buffer.put_slice(&self.file);

        // Magic cookie
        buffer.put_slice(&Self::MAGIC_COOKIE);

        // Options
        for option in &self.options {
            buffer.put_u8(option.code);
            buffer.put_u8(option.data.len() as u8);
            buffer.put_slice(&option.data);
        }

        // End option
        buffer.put_u8(255);

        // Pad to minimum size
        while buffer.len() < 300 {
            buffer.put_u8(0);
        }

        buffer.to_vec()
    }

    pub fn get_message_type(&self) -> Option<DhcpMessageType> {
        self.get_option(53)
            .and_then(|opt| opt.data.first())
            .and_then(|&byte| DhcpMessageType::try_from(byte).ok())
    }

    pub fn set_message_type(&mut self, msg_type: DhcpMessageType) {
        self.set_option(53, vec![msg_type as u8]);
    }

    pub fn get_client_mac(&self) -> [u8; 6] {
        let mut mac = [0u8; 6];
        mac.copy_from_slice(&self.chaddr[..6]);
        mac
    }

    pub fn set_client_mac(&mut self, mac: &[u8; 6]) {
        self.chaddr[..6].copy_from_slice(mac);
    }

    pub fn get_requested_ip(&self) -> Option<Ipv4Addr> {
        self.get_option(50)
            .filter(|opt| opt.data.len() == 4)
            .map(|opt| Ipv4Addr::from([opt.data[0], opt.data[1], opt.data[2], opt.data[3]]))
    }

    pub fn set_requested_ip(&mut self, ip: Ipv4Addr) {
        self.set_option(50, ip.octets().to_vec());
    }

    pub fn get_server_id(&self) -> Option<Ipv4Addr> {
        self.get_option(54)
            .filter(|opt| opt.data.len() == 4)
            .map(|opt| Ipv4Addr::from([opt.data[0], opt.data[1], opt.data[2], opt.data[3]]))
    }

    pub fn set_server_id(&mut self, ip: Ipv4Addr) {
        self.set_option(54, ip.octets().to_vec());
    }

    pub fn get_hostname(&self) -> Option<String> {
        self.get_option(12)
            .and_then(|opt| String::from_utf8(opt.data.clone()).ok())
    }

    pub fn set_hostname(&mut self, hostname: &str) {
        self.set_option(12, hostname.as_bytes().to_vec());
    }

    pub fn get_lease_time(&self) -> Option<u32> {
        self.get_option(51)
            .filter(|opt| opt.data.len() == 4)
            .map(|opt| u32::from_be_bytes([opt.data[0], opt.data[1], opt.data[2], opt.data[3]]))
    }

    pub fn set_lease_time(&mut self, seconds: u32) {
        self.set_option(51, seconds.to_be_bytes().to_vec());
    }

    pub fn get_option(&self, code: u8) -> Option<&DhcpOption> {
        self.options.iter().find(|opt| opt.code == code)
    }

    pub fn set_option(&mut self, code: u8, data: Vec<u8>) {
        if let Some(opt) = self.options.iter_mut().find(|opt| opt.code == code) {
            opt.data = data;
        } else {
            self.options.push(DhcpOption { code, data });
        }
    }

    pub fn remove_option(&mut self, code: u8) {
        self.options.retain(|opt| opt.code != code);
    }

    pub fn is_broadcast(&self) -> bool {
        (self.flags & 0x8000) != 0
    }
}