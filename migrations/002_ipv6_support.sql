-- IPv6 support migration for FlowDNS

-- IPv6 SLAAC addresses table
CREATE TABLE IF NOT EXISTS ipv6_slaac_addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    mac_address BYTEA NOT NULL,
    ipv6_address INET NOT NULL,
    prefix INET NOT NULL,
    prefix_length SMALLINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    hostname VARCHAR(255),
    UNIQUE(mac_address, ipv6_address)
);

-- IPv6 prefix pools for delegation
CREATE TABLE IF NOT EXISTS ipv6_prefix_pools (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) UNIQUE NOT NULL,
    prefix INET NOT NULL,
    prefix_length SMALLINT NOT NULL,
    delegation_length SMALLINT NOT NULL,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- IPv6 delegated prefixes
CREATE TABLE IF NOT EXISTS ipv6_delegated_prefixes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_duid BYTEA NOT NULL,
    iaid INTEGER NOT NULL,
    prefix INET NOT NULL,
    prefix_length SMALLINT NOT NULL,
    delegated_length SMALLINT NOT NULL,
    valid_lifetime INTEGER NOT NULL,
    preferred_lifetime INTEGER NOT NULL,
    lease_start TIMESTAMP WITH TIME ZONE NOT NULL,
    lease_end TIMESTAMP WITH TIME ZONE NOT NULL,
    state VARCHAR(20) NOT NULL DEFAULT 'available',
    UNIQUE(client_duid, iaid)
);

-- IPv6 neighbor cache
CREATE TABLE IF NOT EXISTS ipv6_neighbor_cache (
    ipv6_address INET PRIMARY KEY,
    mac_address BYTEA NOT NULL,
    interface VARCHAR(50),
    state VARCHAR(20) NOT NULL DEFAULT 'incomplete',
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- DHCPv6 leases
CREATE TABLE IF NOT EXISTS dhcpv6_leases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subnet_id UUID REFERENCES dhcp_subnets(id) ON DELETE CASCADE,
    duid BYTEA NOT NULL,
    iaid INTEGER NOT NULL,
    ipv6_address INET NOT NULL,
    hostname VARCHAR(255),
    lease_start TIMESTAMP WITH TIME ZONE NOT NULL,
    lease_end TIMESTAMP WITH TIME ZONE NOT NULL,
    preferred_lifetime INTEGER NOT NULL,
    valid_lifetime INTEGER NOT NULL,
    state VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(duid, iaid, ipv6_address)
);

-- Create indexes for IPv6 tables
CREATE INDEX idx_ipv6_slaac_mac ON ipv6_slaac_addresses(mac_address);
CREATE INDEX idx_ipv6_slaac_last_seen ON ipv6_slaac_addresses(last_seen);
CREATE INDEX idx_ipv6_delegated_duid ON ipv6_delegated_prefixes(client_duid);
CREATE INDEX idx_ipv6_delegated_state ON ipv6_delegated_prefixes(state);
CREATE INDEX idx_ipv6_neighbor_last_seen ON ipv6_neighbor_cache(last_seen);
CREATE INDEX idx_dhcpv6_leases_duid ON dhcpv6_leases(duid);
CREATE INDEX idx_dhcpv6_leases_state ON dhcpv6_leases(state);

-- Add IPv6 configuration columns to dhcp_subnets if not exists
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='dhcp_subnets' AND column_name='ipv6_enabled') THEN
        ALTER TABLE dhcp_subnets ADD COLUMN ipv6_enabled BOOLEAN DEFAULT FALSE;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='dhcp_subnets' AND column_name='ipv6_mode') THEN
        ALTER TABLE dhcp_subnets ADD COLUMN ipv6_mode VARCHAR(20) DEFAULT 'slaac';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='dhcp_subnets' AND column_name='ra_managed') THEN
        ALTER TABLE dhcp_subnets ADD COLUMN ra_managed BOOLEAN DEFAULT FALSE;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name='dhcp_subnets' AND column_name='ra_other_config') THEN
        ALTER TABLE dhcp_subnets ADD COLUMN ra_other_config BOOLEAN DEFAULT TRUE;
    END IF;
END $$;