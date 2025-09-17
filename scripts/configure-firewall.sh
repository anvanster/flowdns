#!/bin/bash

# Firewall configuration script for FlowDNS
# This script provides commands to open the required ports

set -e

echo "========================================="
echo "FlowDNS Firewall Configuration"
echo "========================================="
echo
echo "Both services are now listening on all interfaces:"
echo "  - Frontend: http://0.0.0.0:3000 (Vite dev server)"
echo "  - Backend API: http://0.0.0.0:8080"
echo
echo "Your server IP: 172.101.100.3"
echo
echo "To allow external access, you need to configure the firewall."
echo "Run the following commands with sudo:"
echo
echo "For UFW (Ubuntu/Debian):"
echo "  sudo ufw allow 3000/tcp"
echo "  sudo ufw allow 8080/tcp"
echo "  sudo ufw reload"
echo
echo "For firewalld (RHEL/CentOS/Fedora):"
echo "  sudo firewall-cmd --permanent --add-port=3000/tcp"
echo "  sudo firewall-cmd --permanent --add-port=8080/tcp"
echo "  sudo firewall-cmd --reload"
echo
echo "For iptables (generic):"
echo "  sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT"
echo "  sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT"
echo "  sudo iptables-save > /etc/iptables/rules.v4  # Debian/Ubuntu"
echo
echo "After configuring the firewall, you can access:"
echo "  - Frontend: http://172.101.100.3:3000"
echo "  - Backend API: http://172.101.100.3:8080"
echo "  - API Docs: http://172.101.100.3:8080/api/v1/docs"
echo
echo "========================================="