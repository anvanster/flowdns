# FlowDNS - Multi-Subnet DNS/DHCP Server

A high-performance, production-ready DNS/DHCP server written in Rust with support for multiple IPv4 subnets, IPv6 autoconfiguration, and dynamic DNS updates.

## Features

### DHCP Server
- ✅ **Multi-Subnet Support**: Manage unlimited IPv4 subnets with independent configurations
- ✅ **DHCP Relay Agent Support**: Handle requests from different network segments
- ✅ **Static Reservations**: Assign fixed IPs based on MAC addresses
- ✅ **Dynamic Lease Management**: Automatic IP allocation with configurable lease times
- ✅ **VLAN Awareness**: Support for VLAN-tagged networks
- ✅ **Template-based Hostname Generation**: Auto-generate hostnames like `host-192-168-1-100`

### DNS Server (In Development)
- 🚧 Authoritative DNS server using Hickory DNS
- 🚧 Dynamic DNS updates from DHCP events
- 🚧 Forward and reverse zone management
- 🚧 DNS forwarding for external queries
- 🚧 DNSSEC support preparation

### Additional Features
- PostgreSQL backend for scalability
- REST API for management
- Real-time monitoring and statistics
- Audit logging for compliance
- IPv6 support with radvd integration

## Architecture

```
FlowDNS/
├── src/
│   ├── dhcp/           # DHCP server implementation
│   ├── dns/            # DNS server (planned)
│   ├── api/            # REST API (planned)
│   ├── database/       # Database models and queries
│   └── config/         # Configuration management
├── migrations/         # SQL migrations
└── config/            # Configuration files
```

## Prerequisites

- Rust 1.75 or higher
- PostgreSQL 14 or higher
- Linux operating system (for raw socket support)
- Root/sudo privileges (for binding to ports 67/53)

## Installation

### Quick Installation

```bash
# Automatic installation with all dependencies
./install.sh

# Quick start for testing (no PostgreSQL needed)
./quickstart.sh
```

### Manual Installation

#### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Proxy/Offline Environments

If you're behind a proxy or have restricted internet access:

```bash
# Configure proxy settings
./scripts/setup-proxy.sh

# Or set manually before installation
export HTTP_PROXY=http://your-proxy:8080
export HTTPS_PROXY=http://your-proxy:8080
./install.sh

# For offline installation
./scripts/install-offline.sh
```

### 2. Install PostgreSQL

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib

# Create database and user
sudo -u postgres psql
CREATE DATABASE flowdns;
CREATE USER flowdns WITH PASSWORD 'your-secure-password';
GRANT ALL PRIVILEGES ON DATABASE flowdns TO flowdns;
\q
```

### 3. Clone and Build

```bash
git clone <repository-url>
cd FlowDNS
cargo build --release
```

### 4. Configure

Edit `config/server.toml`:

```toml
[database]
url = "postgresql://flowdns:password@localhost/flowdns"

[dhcp]
enabled = true
bind_address = "0.0.0.0"
port = 67

[subnets.main]
network = "192.168.1.0/24"
start_ip = "192.168.1.100"
end_ip = "192.168.1.200"
gateway = "192.168.1.1"
dns_servers = ["192.168.1.1", "8.8.8.8"]
```

### 5. Run Migrations

```bash
cargo run -- --migrate
```

### 6. Start the Server

```bash
sudo cargo run --release
# or
sudo ./target/release/flowdns
```

## Usage

### Running as a Service

Create `/etc/systemd/system/flowdns.service`:

```ini
[Unit]
Description=FlowDNS DNS/DHCP Server
After=network.target postgresql.service

[Service]
Type=simple
User=root
WorkingDirectory=/opt/flowdns
ExecStart=/opt/flowdns/flowdns
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable flowdns
sudo systemctl start flowdns
```

### DHCP Client Configuration

Configure your network to use FlowDNS as the DHCP server:

1. **Disable existing DHCP servers** on your router/network
2. **Configure DHCP relay** if serving multiple VLANs
3. **Set static IP** for the FlowDNS server outside the DHCP range

### Database Management

View active leases:

```sql
psql -U flowdns -d flowdns
SELECT mac_address, ip_address, hostname, lease_end FROM dhcp_leases WHERE state = 'active';
```

Add static reservation:

```sql
INSERT INTO dhcp_reservations (subnet_id, mac_address, ip_address, hostname)
VALUES (
    (SELECT id FROM dhcp_subnets WHERE name = 'main'),
    E'\\xAABBCCDDEEFF',
    '192.168.1.50',
    'my-device'
);
```

## API Documentation

The API documentation is available via Swagger UI at:
```
http://localhost:8080/api/v1/docs
```

OpenAPI specification:
```
http://localhost:8080/api/v1/docs/openapi.json
```

### Authentication

Most API endpoints require JWT authentication. To get a token:

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin"}'
```

Use the token in subsequent requests:
```bash
curl http://localhost:8080/api/v1/dhcp/leases \
  -H "Authorization: Bearer <your-token>"
```

### API Endpoints

#### Authentication
- `POST /api/v1/auth/login` - Login and get JWT token
- `POST /api/v1/auth/refresh` - Refresh JWT token

#### DHCP Management
- `GET /api/v1/dhcp/leases` - List all DHCP leases
- `POST /api/v1/dhcp/leases` - Create manual lease
- `GET /api/v1/dhcp/leases/{id}` - Get specific lease
- `DELETE /api/v1/dhcp/leases/{id}` - Release lease
- `GET /api/v1/dhcp/subnets` - List all subnets
- `POST /api/v1/dhcp/subnets` - Create new subnet
- `GET /api/v1/dhcp/subnets/{id}` - Get subnet details
- `PUT /api/v1/dhcp/subnets/{id}` - Update subnet
- `DELETE /api/v1/dhcp/subnets/{id}` - Delete subnet
- `GET /api/v1/dhcp/reservations` - List reservations
- `POST /api/v1/dhcp/reservations` - Create reservation
- `DELETE /api/v1/dhcp/reservations/{id}` - Delete reservation
- `GET /api/v1/dhcp/stats` - Get DHCP statistics

#### DNS Management
- `GET /api/v1/dns/zones` - List all DNS zones
- `POST /api/v1/dns/zones` - Create new zone
- `GET /api/v1/dns/zones/{id}` - Get zone details
- `PUT /api/v1/dns/zones/{id}` - Update zone
- `DELETE /api/v1/dns/zones/{id}` - Delete zone
- `GET /api/v1/dns/zones/{zone_id}/records` - List records in zone
- `POST /api/v1/dns/zones/{zone_id}/records` - Create new record
- `PUT /api/v1/dns/records/{id}` - Update record
- `DELETE /api/v1/dns/records/{id}` - Delete record

#### System
- `GET /api/v1/system/health` - Health check (no auth required)
- `GET /api/v1/system/metrics` - System metrics
- `GET /api/v1/system/config` - Get server configuration

## Configuration Options

### DHCP Options

| Option | Description | Default |
|--------|------------|---------|
| `default_lease_time` | Default lease duration in seconds | 86400 (24h) |
| `max_lease_time` | Maximum allowed lease time | 604800 (7d) |
| `renewal_time` | When client should renew (T1) | 50% of lease |
| `rebind_time` | When client should rebind (T2) | 87.5% of lease |

### Subnet Configuration

Each subnet can have:
- IP range (start_ip, end_ip)
- Gateway address
- DNS servers
- Domain name
- VLAN ID
- Custom lease time
- IPv6 prefix (optional)

## Monitoring

### Logs

FlowDNS uses structured logging with tracing:

```bash
# Set log level
RUST_LOG=debug cargo run

# View logs
journalctl -u flowdns -f
```

### Metrics (Planned)

- Active leases per subnet
- IP pool utilization
- Request/response times
- Error rates

## Development

### Project Structure

```rust
src/
├── dhcp/
│   ├── packet.rs       // DHCP packet parsing/building
│   ├── server.rs       // Main DHCP server logic
│   ├── lease_manager.rs // Lease management
│   └── options.rs      // DHCP options handling
├── database/
│   ├── models.rs       // Database models
│   └── schema.rs       // SQL schema
└── config/
    └── settings.rs     // Configuration structures
```

### Running Tests

```bash
cargo test
cargo test --integration-tests
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Troubleshooting

### Common Issues

**Permission Denied (port 67)**
- Run with sudo or use capabilities:
```bash
sudo setcap 'cap_net_bind_service=+ep' ./target/release/flowdns
```

**Database Connection Failed**
- Check PostgreSQL is running: `sudo systemctl status postgresql`
- Verify credentials in config/server.toml
- Check firewall rules

**No DHCP Offers Received**
- Verify network interface is up
- Check firewall allows UDP port 67/68
- Ensure no other DHCP servers are active
- Check subnet configuration matches network

## Security Considerations

- Run with minimal privileges when possible
- Use strong database passwords
- Implement rate limiting for DHCP requests
- Regular security updates
- Monitor for unusual lease patterns

## License

MIT License - See LICENSE file for details

## Roadmap

### Phase 1 (Complete)
- ✅ Core DHCP server
- ✅ Multi-subnet support
- ✅ PostgreSQL integration
- ✅ Lease management

### Phase 2 (Completed)
- ✅ DNS server implementation (simplified version)
- ✅ Dynamic DNS updates framework
- ✅ REST API with JWT authentication
- ✅ Full API endpoints for DHCP and DNS management
- 🚧 Full Hickory DNS integration (Authority mutability issues being resolved)

### Phase 3 (Planned)
- ⏳ Web UI dashboard
- ⏳ IPv6 support
- ⏳ High availability
- ⏳ Prometheus metrics

### Phase 4 (Future)
- ⏳ DNSSEC support
- ⏳ DHCPv6
- ⏳ Kubernetes operator
- ⏳ Multi-master replication

## Support

  Main Installation Scripts:

  - install.sh - Complete production installation with PostgreSQL
  - quickstart.sh - Quick setup for testing without PostgreSQL
  - scripts/install-build-tools.sh - Installs essential build tools
  - scripts/dev-setup.sh - Development environment setup

  Docker Support:

  - docker/Dockerfile - Multi-stage Docker build
  - docker/docker-compose.yml - Complete stack with PostgreSQL
  - docker/init.sql - Database initialization

  Development Tools:

  - Makefile - Convenient commands for development
  - Git hooks setup
  - Test data insertion scripts

  3. Key Features of Installation

  The installation system provides:

  1. Multiple Setup Options:
    - Quick start (SQLite for testing)
    - Full installation (PostgreSQL)
    - Docker deployment
    - Development environment
  2. Automatic Detection:
    - OS detection (Ubuntu, Debian, Fedora, Arch)
    - Package manager selection
    - Dependency checking
  3. Database Setup:
    - Automatic PostgreSQL installation
    - Database and user creation
    - Secure password generation
    - Migration support
  4. Service Management:
    - Systemd service creation
    - Docker Compose orchestration
    - Health checks

  4. Quick Start Commands

  # Quick test without PostgreSQL
  ./quickstart.sh
  cargo run -- --config config/test.toml

  # Full installation
  ./install.sh

  # Docker deployment
  make docker-up

  # Development mode
  make dev

  # Show all commands
  make help

  5. Configuration Files

  Created multiple configuration options:
  - config/server.toml - Production configuration
  - config/dev.toml - Development configuration
  - config/test.toml - Testing without PostgreSQL

  6. Build Issue

  The project needs GCC/build-essential to compile. Users can install it with:
  ./scripts/install-build-tools.sh
  # or manually:
  sudo apt-get install build-essential  # Ubuntu/Debian
  sudo dnf group install "Development Tools"  # Fedora

  
For issues, questions, or contributions, please open an issue on GitHub.