use crate::config::Settings;
use crate::database::models::DhcpSubnet;
use crate::dhcp::lease_manager::LeaseManager;
use crate::dhcp::packet::{DhcpPacket, DhcpMessageType};
use crate::dhcp::packet::DhcpOption;
use crate::dhcp::options::{self, DhcpOptionsBuilder};
use anyhow::{Result, anyhow};
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use sqlx::PgPool;
use ipnet::Ipv4Net;

pub struct DhcpServer {
    socket: UdpSocket,
    lease_manager: Arc<LeaseManager>,
    settings: Arc<Settings>,
    server_ip: Ipv4Addr,
}

impl DhcpServer {
    pub async fn new(settings: Arc<Settings>, db: PgPool) -> Result<Self> {
        let bind_addr = format!("{}:{}", settings.dhcp.bind_address, settings.dhcp.port);
        let socket = UdpSocket::bind(&bind_addr).await?;

        // Enable broadcast
        socket.set_broadcast(true)?;

        info!("DHCP server listening on {}", bind_addr);

        let lease_manager = Arc::new(LeaseManager::new(db, Arc::clone(&settings)).await?);

        // Parse server IP from bind address
        let server_ip = settings.dhcp.bind_address.parse::<Ipv4Addr>()
            .unwrap_or(Ipv4Addr::new(0, 0, 0, 0));

        Ok(Self {
            socket,
            lease_manager,
            settings,
            server_ip,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut buf = vec![0u8; 1500];

        // Start cleanup task
        let cleanup_manager = Arc::clone(&self.lease_manager);
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(300));
            loop {
                cleanup_interval.tick().await;
                if let Err(e) = cleanup_manager.cleanup_expired_leases().await {
                    error!("Failed to cleanup expired leases: {}", e);
                }
            }
        });

        info!("DHCP server started successfully");

        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((size, src)) => {
                    let packet_data = &buf[..size];

                    match DhcpPacket::parse(packet_data) {
                        Ok(packet) => {
                            debug!("Received DHCP packet from {}: {:?}",
                                  src, packet.get_message_type());

                            if let Err(e) = self.handle_packet(packet, src).await {
                                error!("Error handling DHCP packet: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse DHCP packet from {}: {}", src, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Socket error: {}", e);
                }
            }
        }
    }

    async fn handle_packet(&self, packet: DhcpPacket, src: SocketAddr) -> Result<()> {
        let msg_type = packet.get_message_type()
            .ok_or_else(|| anyhow!("No message type in DHCP packet"))?;

        match msg_type {
            DhcpMessageType::Discover => self.handle_discover(packet, src).await,
            DhcpMessageType::Request => self.handle_request(packet, src).await,
            DhcpMessageType::Release => self.handle_release(packet).await,
            DhcpMessageType::Inform => self.handle_inform(packet, src).await,
            DhcpMessageType::Decline => self.handle_decline(packet).await,
            _ => {
                debug!("Ignoring DHCP message type: {:?}", msg_type);
                Ok(())
            }
        }
    }

    async fn handle_discover(&self, packet: DhcpPacket, src: SocketAddr) -> Result<()> {
        let mac = packet.get_client_mac();
        info!("DISCOVER from MAC: {}", format_mac(&mac));

        // Find subnet for client
        let subnet = self.lease_manager
            .find_subnet_for_client(src.ip().to_string().parse()?, packet.giaddr.into())
            .await;

        let subnet = match subnet {
            Some(s) => s,
            None => {
                warn!("No subnet found for client {}", src);
                return Ok(());
            }
        };

        // Find available IP
        let ip = match self.lease_manager.find_available_ip(subnet.id, &mac).await? {
            Some(ip) => ip,
            None => {
                warn!("No available IP addresses in subnet {}", subnet.name);
                return Ok(());
            }
        };

        // Create OFFER packet
        let mut reply = self.create_reply_packet(&packet, DhcpMessageType::Offer);
        reply.yiaddr = ip;

        // Add DHCP options
        let options = self.build_subnet_options(&subnet, ip)?;
        reply.options.extend(options);

        // Send OFFER
        self.send_reply(reply, packet.is_broadcast(), src).await?;
        info!("OFFER sent: MAC {} -> IP {}", format_mac(&mac), ip);

        Ok(())
    }

    async fn handle_request(&self, packet: DhcpPacket, src: SocketAddr) -> Result<()> {
        let mac = packet.get_client_mac();
        let requested_ip = packet.get_requested_ip()
            .or(Some(packet.ciaddr))
            .filter(|&ip| ip != Ipv4Addr::UNSPECIFIED);

        let requested_ip = match requested_ip {
            Some(ip) => ip,
            None => {
                warn!("REQUEST from {} with no requested IP", format_mac(&mac));
                return self.send_nak(packet, src).await;
            }
        };

        info!("REQUEST from MAC: {} for IP: {}", format_mac(&mac), requested_ip);

        // Try to renew existing lease
        if let Some(lease) = self.lease_manager.renew_lease(&mac, requested_ip).await? {
            // Send ACK
            let mut reply = self.create_reply_packet(&packet, DhcpMessageType::Ack);
            reply.yiaddr = lease.ip_address;

            // Get subnet for options
            if let Some(subnet) = self.lease_manager
                .find_subnet_for_client(requested_ip, packet.giaddr.into())
                .await {
                let options = self.build_subnet_options(&subnet, requested_ip)?;
                reply.options.extend(options);
            }

            self.send_reply(reply, packet.is_broadcast(), src).await?;
            info!("ACK sent (renewal): MAC {} -> IP {}", format_mac(&mac), requested_ip);
            return Ok(());
        }

        // Try to create new lease
        let subnet = match self.lease_manager
            .find_subnet_for_client(requested_ip, packet.giaddr.into())
            .await {
            Some(s) => s,
            None => {
                warn!("No subnet found for requested IP {}", requested_ip);
                return self.send_nak(packet, src).await;
            }
        };

        // Verify IP is available
        let available_ip = self.lease_manager.find_available_ip(subnet.id, &mac).await?;
        if available_ip != Some(requested_ip) {
            warn!("Requested IP {} not available for MAC {}",
                  requested_ip, format_mac(&mac));
            return self.send_nak(packet, src).await;
        }

        // Create lease
        let hostname = packet.get_hostname();
        let lease = self.lease_manager
            .create_lease(subnet.id, &mac, requested_ip, hostname)
            .await?;

        // Send ACK
        let mut reply = self.create_reply_packet(&packet, DhcpMessageType::Ack);
        reply.yiaddr = lease.ip_address;

        let options = self.build_subnet_options(&subnet, requested_ip)?;
        reply.options.extend(options);

        self.send_reply(reply, packet.is_broadcast(), src).await?;
        info!("ACK sent (new): MAC {} -> IP {}", format_mac(&mac), requested_ip);

        Ok(())
    }

    async fn handle_release(&self, packet: DhcpPacket) -> Result<()> {
        let mac = packet.get_client_mac();
        let ip = packet.ciaddr;

        if ip == Ipv4Addr::UNSPECIFIED {
            warn!("RELEASE with no IP address from MAC {}", format_mac(&mac));
            return Ok(());
        }

        info!("RELEASE from MAC: {} for IP: {}", format_mac(&mac), ip);

        if self.lease_manager.release_lease(&mac, ip).await? {
            info!("Lease released: MAC {} -> IP {}", format_mac(&mac), ip);
        }

        Ok(())
    }

    async fn handle_inform(&self, packet: DhcpPacket, src: SocketAddr) -> Result<()> {
        let mac = packet.get_client_mac();
        info!("INFORM from MAC: {}", format_mac(&mac));

        // Send ACK with configuration only (no IP assignment)
        let mut reply = self.create_reply_packet(&packet, DhcpMessageType::Ack);
        reply.yiaddr = Ipv4Addr::UNSPECIFIED;

        // Add configuration options if we can find the subnet
        if let Some(subnet) = self.lease_manager
            .find_subnet_for_client(packet.ciaddr, packet.giaddr.into())
            .await {
            let options = self.build_subnet_options(&subnet, packet.ciaddr)?;
            reply.options.extend(options);
        }

        self.send_reply(reply, packet.is_broadcast(), src).await?;

        Ok(())
    }

    async fn handle_decline(&self, packet: DhcpPacket) -> Result<()> {
        let mac = packet.get_client_mac();
        let ip = packet.get_requested_ip()
            .unwrap_or(Ipv4Addr::UNSPECIFIED);

        warn!("DECLINE from MAC: {} for IP: {}", format_mac(&mac), ip);

        // Mark IP as declined (could implement IP blacklist here)
        // For now, just release the lease
        if ip != Ipv4Addr::UNSPECIFIED {
            self.lease_manager.release_lease(&mac, ip).await?;
        }

        Ok(())
    }

    async fn send_nak(&self, packet: DhcpPacket, src: SocketAddr) -> Result<()> {
        let reply = self.create_reply_packet(&packet, DhcpMessageType::Nak);
        self.send_reply(reply, packet.is_broadcast(), src).await?;
        warn!("NAK sent to {}", format_mac(&packet.get_client_mac()));
        Ok(())
    }

    fn create_reply_packet(&self, request: &DhcpPacket, msg_type: DhcpMessageType) -> DhcpPacket {
        let mut reply = DhcpPacket::new();
        reply.op = 2; // BOOTREPLY
        reply.htype = request.htype;
        reply.hlen = request.hlen;
        reply.xid = request.xid;
        reply.flags = request.flags;
        reply.giaddr = request.giaddr;
        reply.chaddr = request.chaddr;
        reply.siaddr = self.server_ip;

        // Add message type
        reply.set_message_type(msg_type);

        // Add server identifier
        reply.set_server_id(self.server_ip);

        reply
    }

    fn build_subnet_options(&self, subnet: &DhcpSubnet, _ip: Ipv4Addr) -> Result<Vec<DhcpOption>> {
        // Convert ipnetwork to ipnet for compatibility
        let network_str = format!("{}/{}", subnet.network.ip(), subnet.network.prefix());
        let network: Ipv4Net = network_str.parse()?;

        let mut builder = DhcpOptionsBuilder::new();

        builder = builder
            .add_subnet_mask(options::calculate_subnet_mask(&network))
            .add_router(subnet.gateway)
            .add_broadcast(options::calculate_broadcast(&network))
            .add_lease_time(subnet.lease_duration as u32)
            .add_renewal_time((subnet.lease_duration / 2) as u32)
            .add_rebind_time((subnet.lease_duration * 7 / 8) as u32);

        if !subnet.dns_servers.is_empty() {
            builder = builder.add_dns_servers(subnet.dns_servers.clone());
        }

        if let Some(domain) = &subnet.domain_name {
            builder = builder.add_domain_name(domain);
        }

        Ok(builder.build())
    }

    async fn send_reply(&self, reply: DhcpPacket, broadcast: bool, _src: SocketAddr) -> Result<()> {
        let data = reply.to_bytes();

        let dest = if broadcast || reply.giaddr == Ipv4Addr::UNSPECIFIED {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), 68)
        } else {
            SocketAddr::new(IpAddr::V4(reply.giaddr), 67)
        };

        self.socket.send_to(&data, dest).await?;
        debug!("Sent DHCP reply to {}", dest);

        Ok(())
    }
}

pub async fn start(settings: Arc<Settings>, db: PgPool) -> Result<()> {
    let mut server = DhcpServer::new(settings, db).await?;
    server.run().await
}

fn format_mac(mac: &[u8]) -> String {
    mac.iter()
        .take(6)
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":")
}