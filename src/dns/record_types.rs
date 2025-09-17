use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DnsRecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    PTR,
    NS,
    SOA,
    SRV,
}

impl FromStr for DnsRecordType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "A" => Ok(DnsRecordType::A),
            "AAAA" => Ok(DnsRecordType::AAAA),
            "CNAME" => Ok(DnsRecordType::CNAME),
            "MX" => Ok(DnsRecordType::MX),
            "TXT" => Ok(DnsRecordType::TXT),
            "PTR" => Ok(DnsRecordType::PTR),
            "NS" => Ok(DnsRecordType::NS),
            "SOA" => Ok(DnsRecordType::SOA),
            "SRV" => Ok(DnsRecordType::SRV),
            _ => Err(anyhow!("Unknown DNS record type: {}", s)),
        }
    }
}

impl ToString for DnsRecordType {
    fn to_string(&self) -> String {
        match self {
            DnsRecordType::A => "A",
            DnsRecordType::AAAA => "AAAA",
            DnsRecordType::CNAME => "CNAME",
            DnsRecordType::MX => "MX",
            DnsRecordType::TXT => "TXT",
            DnsRecordType::PTR => "PTR",
            DnsRecordType::NS => "NS",
            DnsRecordType::SOA => "SOA",
            DnsRecordType::SRV => "SRV",
        }.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: DnsRecordType,
    pub value: String,
    pub ttl: Option<u32>,
    pub priority: Option<u16>,  // For MX and SRV records
}

impl DnsRecord {
    pub fn new_a(name: String, ip: Ipv4Addr, ttl: Option<u32>) -> Self {
        Self {
            name,
            record_type: DnsRecordType::A,
            value: ip.to_string(),
            ttl,
            priority: None,
        }
    }

    pub fn new_aaaa(name: String, ip: Ipv6Addr, ttl: Option<u32>) -> Self {
        Self {
            name,
            record_type: DnsRecordType::AAAA,
            value: ip.to_string(),
            ttl,
            priority: None,
        }
    }

    pub fn new_cname(name: String, target: String, ttl: Option<u32>) -> Self {
        Self {
            name,
            record_type: DnsRecordType::CNAME,
            value: target,
            ttl,
            priority: None,
        }
    }

    pub fn new_mx(name: String, exchange: String, priority: u16, ttl: Option<u32>) -> Self {
        Self {
            name,
            record_type: DnsRecordType::MX,
            value: exchange,
            ttl,
            priority: Some(priority),
        }
    }

    pub fn new_txt(name: String, text: String, ttl: Option<u32>) -> Self {
        Self {
            name,
            record_type: DnsRecordType::TXT,
            value: text,
            ttl,
            priority: None,
        }
    }

    pub fn new_ptr(name: String, target: String, ttl: Option<u32>) -> Self {
        Self {
            name,
            record_type: DnsRecordType::PTR,
            value: target,
            ttl,
            priority: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self.record_type {
            DnsRecordType::A => {
                Ipv4Addr::from_str(&self.value)
                    .map_err(|_| anyhow!("Invalid IPv4 address for A record"))?;
            },
            DnsRecordType::AAAA => {
                Ipv6Addr::from_str(&self.value)
                    .map_err(|_| anyhow!("Invalid IPv6 address for AAAA record"))?;
            },
            DnsRecordType::MX => {
                if self.priority.is_none() {
                    return Err(anyhow!("MX record requires priority"));
                }
            },
            _ => {}
        }
        Ok(())
    }
}

/// Helper functions for PTR record generation
pub fn ipv4_to_ptr_name(ip: Ipv4Addr) -> String {
    let octets = ip.octets();
    format!("{}.{}.{}.{}.in-addr.arpa", octets[3], octets[2], octets[1], octets[0])
}

pub fn ipv6_to_ptr_name(ip: Ipv6Addr) -> String {
    let segments = ip.segments();
    let mut nibbles = Vec::new();

    for segment in segments.iter() {
        nibbles.push(format!("{:04x}", segment));
    }

    let full_hex: String = nibbles.join("");
    let reversed: String = full_hex.chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(1)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(".");

    format!("{}.ip6.arpa", reversed)
}

/// Helper to create reverse DNS zone name from network
pub fn network_to_reverse_zone(network: &ipnet::Ipv4Net) -> String {
    let prefix_len = network.prefix_len();
    let base = network.network();
    let octets = base.octets();

    match prefix_len {
        24 => format!("{}.{}.{}.in-addr.arpa", octets[2], octets[1], octets[0]),
        16 => format!("{}.{}.in-addr.arpa", octets[1], octets[0]),
        8 => format!("{}.in-addr.arpa", octets[0]),
        _ => {
            // For non-octet boundaries, use the /24 containing the network
            format!("{}.{}.{}.in-addr.arpa", octets[2], octets[1], octets[0])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_to_ptr() {
        let ip = Ipv4Addr::new(192, 168, 1, 100);
        assert_eq!(ipv4_to_ptr_name(ip), "100.1.168.192.in-addr.arpa");
    }

    #[test]
    fn test_record_validation() {
        let valid_a = DnsRecord::new_a("test".to_string(), Ipv4Addr::new(192, 168, 1, 1), None);
        assert!(valid_a.validate().is_ok());

        let invalid_a = DnsRecord {
            name: "test".to_string(),
            record_type: DnsRecordType::A,
            value: "not-an-ip".to_string(),
            ttl: None,
            priority: None,
        };
        assert!(invalid_a.validate().is_err());
    }
}