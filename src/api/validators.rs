use regex::Regex;
use std::net::Ipv4Addr;
use std::str::FromStr;

pub fn validate_mac_address(mac: &str) -> bool {
    let re = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
    re.is_match(mac)
}

pub fn validate_hostname(hostname: &str) -> bool {
    if hostname.is_empty() || hostname.len() > 253 {
        return false;
    }

    let re = Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
    re.is_match(hostname)
}

pub fn validate_domain_name(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }

    let parts: Vec<&str> = domain.split('.').collect();
    if parts.is_empty() || parts.len() > 127 {
        return false;
    }

    for part in parts {
        if part.is_empty() || part.len() > 63 {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
        if part.starts_with('-') || part.ends_with('-') {
            return false;
        }
    }

    true
}

pub fn validate_ipv4_network(network: &str) -> bool {
    if let Ok(net) = ipnet::Ipv4Net::from_str(network) {
        net.prefix_len() >= 8 && net.prefix_len() <= 30
    } else {
        false
    }
}

pub fn validate_ip_in_range(ip: Ipv4Addr, start: Ipv4Addr, end: Ipv4Addr) -> bool {
    ip >= start && ip <= end
}

pub fn validate_dns_record_type(record_type: &str) -> bool {
    matches!(
        record_type.to_uppercase().as_str(),
        "A" | "AAAA" | "CNAME" | "MX" | "TXT" | "PTR" | "NS" | "SOA" | "SRV"
    )
}

pub fn validate_ttl(ttl: i32) -> bool {
    ttl >= 0 && ttl <= 2147483647  // Max signed 32-bit integer
}

pub fn mac_string_to_bytes(mac: &str) -> Option<Vec<u8>> {
    if !validate_mac_address(mac) {
        return None;
    }

    let cleaned = mac.replace([':', '-'], "");
    let mut bytes = Vec::with_capacity(6);

    for i in (0..12).step_by(2) {
        if let Ok(byte) = u8::from_str_radix(&cleaned[i..i + 2], 16) {
            bytes.push(byte);
        } else {
            return None;
        }
    }

    Some(bytes)
}

pub fn bytes_to_mac_string(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_mac_address() {
        assert!(validate_mac_address("00:11:22:33:44:55"));
        assert!(validate_mac_address("00-11-22-33-44-55"));
        assert!(validate_mac_address("aA:bB:cC:dD:eE:fF"));
        assert!(!validate_mac_address("00:11:22:33:44"));
        assert!(!validate_mac_address("00:11:22:33:44:55:66"));
        assert!(!validate_mac_address("00:11:22:33:44:GG"));
    }

    #[test]
    fn test_validate_hostname() {
        assert!(validate_hostname("example"));
        assert!(validate_hostname("example.com"));
        assert!(validate_hostname("sub.example.com"));
        assert!(validate_hostname("example-123"));
        assert!(!validate_hostname("-example"));
        assert!(!validate_hostname("example-"));
        assert!(!validate_hostname("example..com"));
        assert!(!validate_hostname(""));
    }

    #[test]
    fn test_validate_domain_name() {
        assert!(validate_domain_name("example.com"));
        assert!(validate_domain_name("sub.example.com"));
        assert!(validate_domain_name("example-123.com"));
        assert!(!validate_domain_name(""));
        assert!(!validate_domain_name("-example.com"));
        assert!(!validate_domain_name("example-.com"));
        assert!(!validate_domain_name("example..com"));
    }

    #[test]
    fn test_validate_ipv4_network() {
        assert!(validate_ipv4_network("192.168.1.0/24"));
        assert!(validate_ipv4_network("10.0.0.0/8"));
        assert!(!validate_ipv4_network("192.168.1.0/33"));
        assert!(!validate_ipv4_network("192.168.1.0/7"));
        assert!(!validate_ipv4_network("invalid"));
    }
}