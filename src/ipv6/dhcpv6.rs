use std::net::{Ipv6Addr, SocketAddrV6};
use tokio::net::UdpSocket;
use anyhow::Result;
use bytes::{Bytes, BytesMut, BufMut};
use tracing::{info, error, debug};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sqlx::PgPool;
use std::sync::Arc;
use crate::config::Settings;

#[derive(Debug, Clone)]
pub struct Dhcpv6Packet {
    pub msg_type: u8,
    pub transaction_id: [u8; 3],
    pub options: Vec<Dhcpv6Option>,
}

#[derive(Debug, Clone)]
pub struct Dhcpv6Option {
    pub code: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Dhcpv6Lease {
    pub id: Uuid,
    pub subnet_id: Uuid,
    pub duid: Vec<u8>,  // DHCP Unique Identifier
    pub iaid: u32,      // Identity Association Identifier
    pub ipv6_address: Ipv6Addr,
    pub prefix_length: u8,
    pub lease_start: DateTime<Utc>,
    pub lease_end: DateTime<Utc>,
    pub preferred_lifetime: u32,
    pub valid_lifetime: u32,
    pub hostname: Option<String>,
    pub state: String,
}

pub struct Dhcpv6Server {
    socket: Arc<UdpSocket>,
    db: PgPool,
    settings: Arc<Settings>,
}

const DHCPV6_SOLICIT: u8 = 1;
const DHCPV6_ADVERTISE: u8 = 2;
const DHCPV6_REQUEST: u8 = 3;
const DHCPV6_CONFIRM: u8 = 4;
const DHCPV6_RENEW: u8 = 5;
const DHCPV6_REBIND: u8 = 6;
const DHCPV6_REPLY: u8 = 7;
const DHCPV6_RELEASE: u8 = 8;
const DHCPV6_DECLINE: u8 = 9;
const DHCPV6_RECONFIGURE: u8 = 10;
const DHCPV6_INFO_REQUEST: u8 = 11;
const DHCPV6_RELAY_FORWARD: u8 = 12;
const DHCPV6_RELAY_REPLY: u8 = 13;

// DHCPv6 Option Codes
const OPT_CLIENTID: u16 = 1;
const OPT_SERVERID: u16 = 2;
const OPT_IA_NA: u16 = 3;     // Identity Association for Non-temporary Addresses
const OPT_IA_TA: u16 = 4;     // Identity Association for Temporary Addresses
const OPT_IAADDR: u16 = 5;    // IA Address
const OPT_ORO: u16 = 6;       // Option Request Option
const OPT_PREFERENCE: u16 = 7;
const OPT_ELAPSED_TIME: u16 = 8;
const OPT_RELAY_MSG: u16 = 9;
const OPT_STATUS_CODE: u16 = 13;
const OPT_RAPID_COMMIT: u16 = 14;
const OPT_USER_CLASS: u16 = 15;
const OPT_VENDOR_CLASS: u16 = 16;
const OPT_VENDOR_OPTS: u16 = 17;
const OPT_INTERFACE_ID: u16 = 18;
const OPT_RECONF_MSG: u16 = 19;
const OPT_RECONF_ACCEPT: u16 = 20;
const OPT_DNS_SERVERS: u16 = 23;
const OPT_DOMAIN_LIST: u16 = 24;
const OPT_IA_PD: u16 = 25;    // Prefix Delegation
const OPT_IAPREFIX: u16 = 26; // IA Prefix

impl Dhcpv6Server {
    pub async fn new(settings: Arc<Settings>, db: PgPool) -> Result<Self> {
        let addr = SocketAddrV6::new(
            Ipv6Addr::UNSPECIFIED,
            547,  // DHCPv6 server port
            0,
            0,
        );
        
        let socket = UdpSocket::bind(addr).await?;
        info!("DHCPv6 server listening on {}", addr);
        
        Ok(Self {
            socket: Arc::new(socket),
            db,
            settings,
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        let mut buf = vec![0u8; 1500];
        
        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    let packet_data = buf[..len].to_vec();
                    let socket = Arc::clone(&self.socket);
                    let db = self.db.clone();
                    let settings = Arc::clone(&self.settings);
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_packet(
                            packet_data,
                            src,
                            socket,
                            db,
                            settings,
                        ).await {
                            error!("Error handling DHCPv6 packet: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error receiving DHCPv6 packet: {}", e);
                }
            }
        }
    }
    
    async fn handle_packet(
        data: Vec<u8>,
        src: std::net::SocketAddr,
        socket: Arc<UdpSocket>,
        db: PgPool,
        settings: Arc<Settings>,
    ) -> Result<()> {
        let packet = Self::parse_packet(&data)?;
        debug!("Received DHCPv6 {} from {}", packet.msg_type, src);
        
        let response = match packet.msg_type {
            DHCPV6_SOLICIT => Self::handle_solicit(packet, db, settings).await?,
            DHCPV6_REQUEST | DHCPV6_CONFIRM | DHCPV6_RENEW | DHCPV6_REBIND => {
                Self::handle_request(packet, db, settings).await?
            }
            DHCPV6_RELEASE => {
                Self::handle_release(packet, db).await?;
                return Ok(());
            }
            DHCPV6_INFO_REQUEST => Self::handle_info_request(packet, settings).await?,
            _ => {
                debug!("Unhandled DHCPv6 message type: {}", packet.msg_type);
                return Ok(());
            }
        };
        
        if let Some(response_packet) = response {
            let response_data = Self::build_packet(response_packet);
            socket.send_to(&response_data, src).await?;
        }
        
        Ok(())
    }
    
    fn parse_packet(data: &[u8]) -> Result<Dhcpv6Packet> {
        if data.len() < 4 {
            return Err(anyhow::anyhow!("Packet too short"));
        }
        
        let msg_type = data[0];
        let transaction_id = [data[1], data[2], data[3]];
        let mut options = Vec::new();
        
        let mut offset = 4;
        while offset < data.len() {
            if offset + 4 > data.len() {
                break;
            }
            
            let opt_code = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let opt_len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
            
            if offset + 4 + opt_len > data.len() {
                break;
            }
            
            let opt_data = data[offset + 4..offset + 4 + opt_len].to_vec();
            options.push(Dhcpv6Option {
                code: opt_code,
                data: opt_data,
            });
            
            offset += 4 + opt_len;
        }
        
        Ok(Dhcpv6Packet {
            msg_type,
            transaction_id,
            options,
        })
    }
    
    fn build_packet(packet: Dhcpv6Packet) -> Vec<u8> {
        let mut buf = BytesMut::new();
        
        buf.put_u8(packet.msg_type);
        buf.put_slice(&packet.transaction_id);
        
        for option in packet.options {
            buf.put_u16(option.code);
            buf.put_u16(option.data.len() as u16);
            buf.put_slice(&option.data);
        }
        
        buf.to_vec()
    }
    
    async fn handle_solicit(
        packet: Dhcpv6Packet,
        db: PgPool,
        settings: Arc<Settings>,
    ) -> Result<Option<Dhcpv6Packet>> {
        // Extract client DUID
        let client_duid = packet.options.iter()
            .find(|opt| opt.code == OPT_CLIENTID)
            .map(|opt| opt.data.clone());
            
        if client_duid.is_none() {
            return Ok(None);
        }
        
        // Build ADVERTISE response
        let mut response = Dhcpv6Packet {
            msg_type: DHCPV6_ADVERTISE,
            transaction_id: packet.transaction_id,
            options: Vec::new(),
        };
        
        // Add server DUID
        let server_duid = Self::generate_server_duid();
        response.options.push(Dhcpv6Option {
            code: OPT_SERVERID,
            data: server_duid,
        });
        
        // Echo client DUID
        response.options.push(Dhcpv6Option {
            code: OPT_CLIENTID,
            data: client_duid.unwrap(),
        });
        
        // Add IA_NA with offered address
        // This is simplified - full implementation would check database for available addresses
        let ia_na = Self::build_ia_na_option(
            1,  // IAID
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x1),
            3600,  // preferred lifetime
            7200,  // valid lifetime
        );
        response.options.push(ia_na);
        
        // Add DNS servers
        if let Some(dns_servers) = Self::get_dns_servers(&settings) {
            response.options.push(Dhcpv6Option {
                code: OPT_DNS_SERVERS,
                data: dns_servers,
            });
        }
        
        Ok(Some(response))
    }
    
    async fn handle_request(
        packet: Dhcpv6Packet,
        db: PgPool,
        settings: Arc<Settings>,
    ) -> Result<Option<Dhcpv6Packet>> {
        // Similar to handle_solicit but commits the lease
        let mut response = Dhcpv6Packet {
            msg_type: DHCPV6_REPLY,
            transaction_id: packet.transaction_id,
            options: Vec::new(),
        };
        
        // Add server and client DUIDs
        let server_duid = Self::generate_server_duid();
        response.options.push(Dhcpv6Option {
            code: OPT_SERVERID,
            data: server_duid,
        });
        
        if let Some(client_duid) = packet.options.iter()
            .find(|opt| opt.code == OPT_CLIENTID)
            .map(|opt| opt.data.clone()) {
            response.options.push(Dhcpv6Option {
                code: OPT_CLIENTID,
                data: client_duid,
            });
        }
        
        // Add status code (success)
        response.options.push(Dhcpv6Option {
            code: OPT_STATUS_CODE,
            data: vec![0, 0],  // Success status
        });
        
        Ok(Some(response))
    }
    
    async fn handle_release(packet: Dhcpv6Packet, db: PgPool) -> Result<()> {
        // Extract client DUID and release the lease
        if let Some(client_duid) = packet.options.iter()
            .find(|opt| opt.code == OPT_CLIENTID)
            .map(|opt| &opt.data) {
            
            // Update database to release the lease
            info!("Releasing DHCPv6 lease for client DUID: {:?}", client_duid);
        }
        
        Ok(())
    }
    
    async fn handle_info_request(
        packet: Dhcpv6Packet,
        settings: Arc<Settings>,
    ) -> Result<Option<Dhcpv6Packet>> {
        let mut response = Dhcpv6Packet {
            msg_type: DHCPV6_REPLY,
            transaction_id: packet.transaction_id,
            options: Vec::new(),
        };
        
        // Add server DUID
        let server_duid = Self::generate_server_duid();
        response.options.push(Dhcpv6Option {
            code: OPT_SERVERID,
            data: server_duid,
        });
        
        // Echo client DUID if present
        if let Some(client_duid) = packet.options.iter()
            .find(|opt| opt.code == OPT_CLIENTID)
            .map(|opt| opt.data.clone()) {
            response.options.push(Dhcpv6Option {
                code: OPT_CLIENTID,
                data: client_duid,
            });
        }
        
        // Add DNS servers and domain list
        if let Some(dns_servers) = Self::get_dns_servers(&settings) {
            response.options.push(Dhcpv6Option {
                code: OPT_DNS_SERVERS,
                data: dns_servers,
            });
        }
        
        Ok(Some(response))
    }
    
    fn generate_server_duid() -> Vec<u8> {
        // DUID-LLT (Link-layer address plus time)
        // Type 1, hardware type 1 (Ethernet), time, MAC address
        let mut duid = Vec::new();
        duid.extend_from_slice(&[0, 1]);  // DUID-LLT
        duid.extend_from_slice(&[0, 1]);  // Hardware type (Ethernet)
        
        // Add timestamp (seconds since Jan 1, 2000)
        let timestamp = Utc::now().timestamp() - 946684800;
        duid.extend_from_slice(&(timestamp as u32).to_be_bytes());
        
        // Add MAC address (simplified - use actual interface MAC)
        duid.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        
        duid
    }
    
    fn build_ia_na_option(
        iaid: u32,
        addr: Ipv6Addr,
        preferred: u32,
        valid: u32,
    ) -> Dhcpv6Option {
        let mut data = BytesMut::new();
        
        // IAID
        data.put_u32(iaid);
        // T1 (renewal time)
        data.put_u32(preferred / 2);
        // T2 (rebinding time)
        data.put_u32(preferred * 3 / 4);
        
        // IA Address sub-option
        data.put_u16(OPT_IAADDR);
        data.put_u16(24);  // Option length
        data.put_slice(&addr.octets());
        data.put_u32(preferred);
        data.put_u32(valid);
        
        Dhcpv6Option {
            code: OPT_IA_NA,
            data: data.to_vec(),
        }
    }
    
    fn get_dns_servers(settings: &Settings) -> Option<Vec<u8>> {
        // Return IPv6 DNS servers if configured
        // This is simplified - would read from settings
        let mut data = Vec::new();
        
        // Example: 2001:4860:4860::8888 (Google DNS)
        let dns = Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888);
        data.extend_from_slice(&dns.octets());
        
        Some(data)
    }
}