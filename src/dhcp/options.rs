use std::net::Ipv4Addr;
use crate::dhcp::packet::DhcpOption;

pub const OPTION_SUBNET_MASK: u8 = 1;
pub const OPTION_ROUTER: u8 = 3;
pub const OPTION_DNS_SERVERS: u8 = 6;
pub const OPTION_HOSTNAME: u8 = 12;
pub const OPTION_DOMAIN_NAME: u8 = 15;
pub const OPTION_BROADCAST: u8 = 28;
pub const OPTION_REQUESTED_IP: u8 = 50;
pub const OPTION_LEASE_TIME: u8 = 51;
pub const OPTION_MESSAGE_TYPE: u8 = 53;
pub const OPTION_SERVER_ID: u8 = 54;
pub const OPTION_PARAMETER_LIST: u8 = 55;
pub const OPTION_MESSAGE: u8 = 56;
pub const OPTION_MAX_MESSAGE_SIZE: u8 = 57;
pub const OPTION_RENEWAL_TIME: u8 = 58;
pub const OPTION_REBIND_TIME: u8 = 59;
pub const OPTION_VENDOR_CLASS: u8 = 60;
pub const OPTION_CLIENT_ID: u8 = 61;
pub const OPTION_USER_CLASS: u8 = 77;

pub struct DhcpOptionsBuilder {
    options: Vec<DhcpOption>,
}

impl DhcpOptionsBuilder {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
        }
    }

    pub fn add_subnet_mask(mut self, mask: Ipv4Addr) -> Self {
        self.add_option(OPTION_SUBNET_MASK, mask.octets().to_vec());
        self
    }

    pub fn add_router(mut self, router: Ipv4Addr) -> Self {
        self.add_option(OPTION_ROUTER, router.octets().to_vec());
        self
    }

    pub fn add_dns_servers(mut self, servers: Vec<Ipv4Addr>) -> Self {
        let mut data = Vec::new();
        for server in servers {
            data.extend_from_slice(&server.octets());
        }
        self.add_option(OPTION_DNS_SERVERS, data);
        self
    }

    pub fn add_domain_name(mut self, domain: &str) -> Self {
        self.add_option(OPTION_DOMAIN_NAME, domain.as_bytes().to_vec());
        self
    }

    pub fn add_broadcast(mut self, broadcast: Ipv4Addr) -> Self {
        self.add_option(OPTION_BROADCAST, broadcast.octets().to_vec());
        self
    }

    pub fn add_lease_time(mut self, seconds: u32) -> Self {
        self.add_option(OPTION_LEASE_TIME, seconds.to_be_bytes().to_vec());
        self
    }

    pub fn add_renewal_time(mut self, seconds: u32) -> Self {
        self.add_option(OPTION_RENEWAL_TIME, seconds.to_be_bytes().to_vec());
        self
    }

    pub fn add_rebind_time(mut self, seconds: u32) -> Self {
        self.add_option(OPTION_REBIND_TIME, seconds.to_be_bytes().to_vec());
        self
    }

    pub fn add_message(mut self, message: &str) -> Self {
        self.add_option(OPTION_MESSAGE, message.as_bytes().to_vec());
        self
    }

    fn add_option(&mut self, code: u8, data: Vec<u8>) {
        self.options.push(DhcpOption { code, data });
    }

    pub fn build(self) -> Vec<DhcpOption> {
        self.options
    }
}

pub fn parse_parameter_list(option: &DhcpOption) -> Vec<u8> {
    if option.code == OPTION_PARAMETER_LIST {
        option.data.clone()
    } else {
        Vec::new()
    }
}

pub fn calculate_subnet_mask(network: &ipnet::Ipv4Net) -> Ipv4Addr {
    network.netmask()
}

pub fn calculate_broadcast(network: &ipnet::Ipv4Net) -> Ipv4Addr {
    network.broadcast()
}