-- Initial database schema for FlowDNS

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- DNS Zones table
CREATE TABLE IF NOT EXISTS dns_zones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    zone_type VARCHAR(20) NOT NULL DEFAULT 'master',
    serial_number BIGINT NOT NULL DEFAULT 1,
    refresh_interval INTEGER DEFAULT 3600,
    retry_interval INTEGER DEFAULT 900,
    expire_interval INTEGER DEFAULT 604800,
    minimum_ttl INTEGER DEFAULT 86400,
    primary_ns VARCHAR(255),
    admin_email VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- DNS Records table
CREATE TABLE IF NOT EXISTS dns_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    zone_id UUID REFERENCES dns_zones(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    record_type VARCHAR(10) NOT NULL,
    value TEXT NOT NULL,
    ttl INTEGER DEFAULT 3600,
    priority INTEGER DEFAULT NULL,
    weight INTEGER DEFAULT NULL,
    port INTEGER DEFAULT NULL,
    is_dynamic BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(zone_id, name, record_type, value)
);

-- DHCP Subnets table
CREATE TABLE IF NOT EXISTS dhcp_subnets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    network CIDR NOT NULL,
    start_ip INET NOT NULL,
    end_ip INET NOT NULL,
    gateway INET NOT NULL,
    dns_servers JSONB NOT NULL DEFAULT '[]',
    domain_name VARCHAR(255),
    lease_duration INTEGER DEFAULT 86400,
    vlan_id INTEGER DEFAULT NULL,
    ipv6_prefix CIDR DEFAULT NULL,
    enabled BOOLEAN DEFAULT TRUE,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- DHCP Static Reservations table
CREATE TABLE IF NOT EXISTS dhcp_reservations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subnet_id UUID REFERENCES dhcp_subnets(id) ON DELETE CASCADE,
    mac_address BYTEA NOT NULL,
    ip_address INET NOT NULL,
    hostname VARCHAR(255),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(mac_address),
    UNIQUE(subnet_id, ip_address)
);

-- DHCP Active Leases table
CREATE TABLE IF NOT EXISTS dhcp_leases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subnet_id UUID REFERENCES dhcp_subnets(id) ON DELETE CASCADE,
    mac_address BYTEA NOT NULL,
    ip_address INET NOT NULL,
    hostname VARCHAR(255),
    lease_start TIMESTAMP WITH TIME ZONE NOT NULL,
    lease_end TIMESTAMP WITH TIME ZONE NOT NULL,
    state VARCHAR(20) DEFAULT 'active',
    client_identifier TEXT,
    vendor_class TEXT,
    user_class TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(mac_address)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_dns_records_zone_name ON dns_records(zone_id, name);
CREATE INDEX IF NOT EXISTS idx_dns_records_type ON dns_records(record_type);
CREATE INDEX IF NOT EXISTS idx_dhcp_leases_subnet ON dhcp_leases(subnet_id);
CREATE INDEX IF NOT EXISTS idx_dhcp_leases_mac ON dhcp_leases(mac_address);
CREATE INDEX IF NOT EXISTS idx_dhcp_leases_ip ON dhcp_leases(ip_address);
CREATE INDEX IF NOT EXISTS idx_dhcp_leases_state ON dhcp_leases(state);
CREATE INDEX IF NOT EXISTS idx_dhcp_leases_expiry ON dhcp_leases(lease_end);

-- Update timestamp trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add update triggers
CREATE TRIGGER update_dns_zones_updated_at BEFORE UPDATE ON dns_zones
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dns_records_updated_at BEFORE UPDATE ON dns_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dhcp_subnets_updated_at BEFORE UPDATE ON dhcp_subnets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dhcp_leases_updated_at BEFORE UPDATE ON dhcp_leases
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();