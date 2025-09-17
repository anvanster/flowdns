#!/bin/bash

# FlowDNS Proxy Configuration Helper
# This script helps configure proxy settings for installation

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_info() {
    echo -e "${YELLOW}➜${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

echo "================================================"
echo "        FlowDNS Proxy Configuration            "
echo "================================================"
echo ""

# Check current proxy settings
check_proxy() {
    print_info "Current proxy settings:"
    echo ""

    # Environment variables
    if [ -n "$HTTP_PROXY" ] || [ -n "$http_proxy" ]; then
        echo "  HTTP_PROXY: ${HTTP_PROXY:-$http_proxy}"
    else
        echo "  HTTP_PROXY: (not set)"
    fi

    if [ -n "$HTTPS_PROXY" ] || [ -n "$https_proxy" ]; then
        echo "  HTTPS_PROXY: ${HTTPS_PROXY:-$https_proxy}"
    else
        echo "  HTTPS_PROXY: (not set)"
    fi

    if [ -n "$NO_PROXY" ] || [ -n "$no_proxy" ]; then
        echo "  NO_PROXY: ${NO_PROXY:-$no_proxy}"
    else
        echo "  NO_PROXY: (not set)"
    fi

    echo ""

    # Check curl configuration
    if [ -f ~/.curlrc ]; then
        print_info "Curl configuration (~/.curlrc):"
        grep -E "^proxy|^noproxy" ~/.curlrc 2>/dev/null | sed 's/^/  /' || echo "  No proxy settings found"
    else
        print_info "No ~/.curlrc file found"
    fi

    echo ""

    # Check git configuration
    print_info "Git proxy configuration:"
    local git_http_proxy=$(git config --global --get http.proxy 2>/dev/null)
    local git_https_proxy=$(git config --global --get https.proxy 2>/dev/null)

    if [ -n "$git_http_proxy" ]; then
        echo "  http.proxy: $git_http_proxy"
    else
        echo "  http.proxy: (not set)"
    fi

    if [ -n "$git_https_proxy" ]; then
        echo "  https.proxy: $git_https_proxy"
    else
        echo "  https.proxy: (not set)"
    fi

    echo ""
}

# Set proxy configuration
set_proxy() {
    print_info "Enter proxy configuration"
    echo ""

    read -p "HTTP/HTTPS proxy (e.g., http://proxy.company.com:8080): " proxy_input
    read -p "SOCKS proxy (leave blank if not using SOCKS): " socks_proxy_input
    read -p "No proxy domains (e.g., localhost,127.0.0.1,.company.local): " no_proxy_input

    # Set default no_proxy if empty
    if [ -z "$no_proxy_input" ]; then
        no_proxy_input="localhost,127.0.0.1,::1,.local"
    fi

    echo ""
    print_info "Proxy configuration:"
    echo "  HTTP/HTTPS Proxy: $proxy_input"
    [ -n "$socks_proxy_input" ] && echo "  SOCKS Proxy: $socks_proxy_input"
    echo "  No Proxy: $no_proxy_input"
    echo ""

    read -p "Save this configuration? (y/n): " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # 1. Create/update ~/.curlrc
        print_info "Updating ~/.curlrc..."

        # Backup existing .curlrc if it exists
        if [ -f ~/.curlrc ]; then
            cp ~/.curlrc ~/.curlrc.backup
            print_success "Backed up existing ~/.curlrc to ~/.curlrc.backup"
        fi

        # Remove existing proxy settings from .curlrc
        if [ -f ~/.curlrc ]; then
            grep -v -E "^proxy|^noproxy|^socks5" ~/.curlrc > ~/.curlrc.tmp 2>/dev/null || true
            mv ~/.curlrc.tmp ~/.curlrc
        fi

        # Add new proxy settings to .curlrc
        {
            echo "# Proxy configuration added by FlowDNS setup"
            echo "proxy = $proxy_input"
            if [ -n "$socks_proxy_input" ]; then
                echo "socks5 = $socks_proxy_input"
            fi
            # Convert comma-separated no_proxy to curl format
            echo "noproxy = \"${no_proxy_input}\""
        } >> ~/.curlrc

        print_success "Updated ~/.curlrc"

        # 2. Create environment variable script
        cat > .proxy.env << EOF
#!/bin/bash
# FlowDNS Proxy Configuration
# Source this file to set proxy environment variables: source .proxy.env

# Standard proxy environment variables
export HTTP_PROXY="$proxy_input"
export HTTPS_PROXY="$proxy_input"
export http_proxy="$proxy_input"
export https_proxy="$proxy_input"
export NO_PROXY="$no_proxy_input"
export no_proxy="$no_proxy_input"

# Cargo (Rust) specific proxy settings
export CARGO_HTTP_PROXY="$proxy_input"

# All_proxy for tools that use it
export ALL_PROXY="$proxy_input"
export all_proxy="$proxy_input"

EOF

        if [ -n "$socks_proxy_input" ]; then
            echo "export SOCKS_PROXY=\"$socks_proxy_input\"" >> .proxy.env
            echo "export socks_proxy=\"$socks_proxy_input\"" >> .proxy.env
        fi

        chmod +x .proxy.env
        print_success "Created .proxy.env script"

        # 3. Configure Git
        print_info "Configuring Git proxy..."
        git config --global http.proxy "$proxy_input"
        git config --global https.proxy "$proxy_input"
        print_success "Git proxy configured"

        # 4. Create cargo config for Rust
        print_info "Configuring Cargo (Rust) proxy..."
        mkdir -p ~/.cargo

        # Check if cargo config exists
        if [ -f ~/.cargo/config ] || [ -f ~/.cargo/config.toml ]; then
            print_info "Backing up existing Cargo config..."
            [ -f ~/.cargo/config ] && cp ~/.cargo/config ~/.cargo/config.backup
            [ -f ~/.cargo/config.toml ] && cp ~/.cargo/config.toml ~/.cargo/config.toml.backup
        fi

        cat > ~/.cargo/config.toml << EOF
# Cargo proxy configuration added by FlowDNS setup
[http]
proxy = "$proxy_input"

[https]
proxy = "$proxy_input"

[net]
git-fetch-with-cli = true
EOF

        print_success "Cargo proxy configured"

        # 5. Apply settings for current session
        export HTTP_PROXY="$proxy_input"
        export HTTPS_PROXY="$proxy_input"
        export NO_PROXY="$no_proxy_input"
        export http_proxy="$proxy_input"
        export https_proxy="$proxy_input"
        export no_proxy="$no_proxy_input"

        print_success "Proxy settings applied to current session"

        echo ""
        print_success "Proxy configuration complete!"
        echo ""
        print_info "Configuration saved to:"
        echo "  - ~/.curlrc (curl proxy settings)"
        echo "  - ~/.cargo/config.toml (Rust/Cargo proxy)"
        echo "  - .proxy.env (environment variables)"
        echo ""
        print_info "To use these settings in a new terminal:"
        echo "  source .proxy.env"
        echo ""
        print_info "To install FlowDNS with proxy:"
        echo "  source .proxy.env && ./install.sh"
    fi
}

# Clear proxy settings
clear_proxy() {
    print_info "Clearing proxy settings..."

    # Clear environment variables
    unset HTTP_PROXY HTTPS_PROXY NO_PROXY http_proxy https_proxy no_proxy
    unset CARGO_HTTP_PROXY ALL_PROXY all_proxy SOCKS_PROXY socks_proxy

    # Clear curl configuration
    if [ -f ~/.curlrc ]; then
        print_info "Removing proxy settings from ~/.curlrc..."
        grep -v -E "^proxy|^noproxy|^socks5|# Proxy configuration added by FlowDNS" ~/.curlrc > ~/.curlrc.tmp 2>/dev/null || true

        if [ -s ~/.curlrc.tmp ]; then
            mv ~/.curlrc.tmp ~/.curlrc
            print_success "Cleaned ~/.curlrc"
        else
            rm -f ~/.curlrc.tmp ~/.curlrc
            print_success "Removed ~/.curlrc (was only proxy config)"
        fi

        # Restore backup if exists
        if [ -f ~/.curlrc.backup ]; then
            read -p "Restore original ~/.curlrc from backup? (y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                mv ~/.curlrc.backup ~/.curlrc
                print_success "Restored original ~/.curlrc"
            fi
        fi
    fi

    # Clear git configuration
    git config --global --unset http.proxy 2>/dev/null || true
    git config --global --unset https.proxy 2>/dev/null || true
    print_success "Cleared Git proxy settings"

    # Clear cargo configuration
    if [ -f ~/.cargo/config.toml ]; then
        if [ -f ~/.cargo/config.toml.backup ]; then
            read -p "Restore original Cargo config from backup? (y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                mv ~/.cargo/config.toml.backup ~/.cargo/config.toml
                print_success "Restored original Cargo config"
            else
                rm -f ~/.cargo/config.toml
                print_success "Removed Cargo proxy config"
            fi
        else
            rm -f ~/.cargo/config.toml
            print_success "Removed Cargo proxy config"
        fi
    fi

    # Remove .proxy.env
    if [ -f .proxy.env ]; then
        rm -f .proxy.env
        print_success "Removed .proxy.env"
    fi

    print_success "All proxy settings cleared"
}

# Test connectivity
test_connection() {
    print_info "Testing connectivity..."
    echo ""

    # Test DNS
    print_info "Testing DNS resolution..."
    if nslookup google.com > /dev/null 2>&1; then
        print_success "DNS working"
    else
        print_error "DNS failed"
    fi

    # Test with curl (will use ~/.curlrc if present)
    print_info "Testing HTTP with curl..."
    if curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 -L http://www.google.com 2>/dev/null | grep -q "200\|301\|302"; then
        print_success "HTTP working (curl)"
    else
        print_error "HTTP failed (curl)"
        print_info "Curl command used: curl -I http://www.google.com"
        if [ -f ~/.curlrc ]; then
            print_info "Check ~/.curlrc for proxy settings"
        fi
    fi

    # Test HTTPS
    print_info "Testing HTTPS with curl..."
    if curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 -L https://www.google.com 2>/dev/null | grep -q "200\|301\|302"; then
        print_success "HTTPS working (curl)"
    else
        print_error "HTTPS failed (curl)"
    fi

    # Test Rust installer download
    print_info "Testing rustup.rs download..."
    if curl --proto '=https' --tlsv1.2 -sSf --connect-timeout 5 --max-time 10 https://sh.rustup.rs > /dev/null 2>&1; then
        print_success "Can reach rustup.rs"
    else
        print_error "Cannot reach rustup.rs"
        print_info "This might need proxy configuration"
    fi

    # Test with wget if available
    if command -v wget > /dev/null 2>&1; then
        print_info "Testing with wget..."
        if wget -q --spider --timeout=5 http://www.google.com 2>/dev/null; then
            print_success "HTTP working (wget)"
        else
            print_error "HTTP failed (wget)"
        fi
    fi

    # Show current proxy settings being used
    echo ""
    print_info "Active proxy settings:"
    if [ -n "${http_proxy:-$HTTP_PROXY}" ]; then
        echo "  Environment: ${http_proxy:-$HTTP_PROXY}"
    fi
    if [ -f ~/.curlrc ] && grep -q "^proxy" ~/.curlrc 2>/dev/null; then
        echo "  Curl config: $(grep "^proxy" ~/.curlrc | head -1)"
    fi
}

# Show proxy configuration examples
show_examples() {
    echo ""
    print_info "Common proxy configurations:"
    echo ""
    echo "1. Corporate HTTP proxy:"
    echo "   http://proxy.company.com:8080"
    echo ""
    echo "2. Proxy with authentication:"
    echo "   http://username:password@proxy.company.com:8080"
    echo ""
    echo "3. SOCKS5 proxy:"
    echo "   socks5://proxy.company.com:1080"
    echo ""
    echo "4. Local proxy (like CNTLM):"
    echo "   http://127.0.0.1:3128"
    echo ""
    print_info "No-proxy typically includes:"
    echo "   localhost,127.0.0.1,::1,.local,.company.com,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16"
    echo ""
}

# Main menu
main() {
    while true; do
        echo ""
        echo "Select an option:"
        echo "1. Check current proxy settings"
        echo "2. Set proxy configuration"
        echo "3. Clear proxy settings"
        echo "4. Test connectivity"
        echo "5. Show proxy examples"
        echo "6. Exit"
        echo ""

        read -p "Choice [1-6]: " choice

        case $choice in
            1)
                check_proxy
                ;;
            2)
                set_proxy
                ;;
            3)
                clear_proxy
                ;;
            4)
                test_connection
                ;;
            5)
                show_examples
                ;;
            6)
                echo "Exiting..."
                exit 0
                ;;
            *)
                print_error "Invalid option"
                ;;
        esac
    done
}

# Show initial status
check_proxy

echo "This script helps configure proxy settings for FlowDNS installation"
echo ""
print_info "Proxy is needed for: curl, git, cargo (Rust), and general internet access"
echo ""

main