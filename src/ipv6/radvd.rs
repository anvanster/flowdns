use std::process::{Command, Stdio};
use std::fs;
use std::path::Path;
use anyhow::Result;
use tracing::{info, error, debug};
use crate::config::Settings;
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
pub struct RadvdConfig {
    pub interfaces: Vec<InterfaceConfig>,
}

#[derive(Debug, Clone)]
pub struct InterfaceConfig {
    pub name: String,
    pub prefix: String,
    pub prefix_length: u8,
    pub send_advert: bool,
    pub managed_flag: bool,  // M flag - use DHCPv6 for addresses
    pub other_config_flag: bool,  // O flag - use DHCPv6 for other config
    pub min_rtr_adv_interval: u32,
    pub max_rtr_adv_interval: u32,
    pub default_lifetime: u32,
    pub prefix_valid_lifetime: u32,
    pub prefix_preferred_lifetime: u32,
    pub rdnss_servers: Vec<String>,
    pub dnssl_domains: Vec<String>,
}

pub struct RadvdManager {
    config_path: String,
    pid_path: String,
    settings: Arc<Settings>,
}

impl RadvdManager {
    pub fn new(settings: Arc<Settings>) -> Self {
        Self {
            config_path: "/etc/radvd.conf".to_string(),
            pid_path: "/var/run/radvd.pid".to_string(),
            settings,
        }
    }
    
    pub async fn configure(&self, config: RadvdConfig) -> Result<()> {
        // Generate radvd configuration
        let config_content = self.generate_config(&config)?;
        
        // Write configuration to file
        fs::write(&self.config_path, config_content)?;
        info!("Wrote radvd configuration to {}", self.config_path);
        
        // Reload or restart radvd
        self.reload_radvd().await?;
        
        Ok(())
    }
    
    fn generate_config(&self, config: &RadvdConfig) -> Result<String> {
        let mut content = String::new();
        
        for interface in &config.interfaces {
            content.push_str(&format!("interface {}\n{{\n", interface.name));
            
            // Interface options
            if interface.send_advert {
                content.push_str("    AdvSendAdvert on;\n");
            } else {
                content.push_str("    AdvSendAdvert off;\n");
            }
            
            if interface.managed_flag {
                content.push_str("    AdvManagedFlag on;\n");
            } else {
                content.push_str("    AdvManagedFlag off;\n");
            }
            
            if interface.other_config_flag {
                content.push_str("    AdvOtherConfigFlag on;\n");
            } else {
                content.push_str("    AdvOtherConfigFlag off;\n");
            }
            
            content.push_str(&format!(
                "    MinRtrAdvInterval {};\n",
                interface.min_rtr_adv_interval
            ));
            content.push_str(&format!(
                "    MaxRtrAdvInterval {};\n",
                interface.max_rtr_adv_interval
            ));
            content.push_str(&format!(
                "    AdvDefaultLifetime {};\n",
                interface.default_lifetime
            ));
            
            // Prefix configuration
            content.push_str(&format!(
                "\n    prefix {}/{}\n    {{\n",
                interface.prefix,
                interface.prefix_length
            ));
            content.push_str("        AdvOnLink on;\n");
            content.push_str("        AdvAutonomous on;\n");
            content.push_str(&format!(
                "        AdvValidLifetime {};\n",
                interface.prefix_valid_lifetime
            ));
            content.push_str(&format!(
                "        AdvPreferredLifetime {};\n",
                interface.prefix_preferred_lifetime
            ));
            content.push_str("    };\n");
            
            // RDNSS (Recursive DNS Server) configuration
            if !interface.rdnss_servers.is_empty() {
                content.push_str("\n    RDNSS ");
                content.push_str(&interface.rdnss_servers.join(" "));
                content.push_str("\n    {\n");
                content.push_str(&format!(
                    "        AdvRDNSSLifetime {};\n",
                    interface.default_lifetime
                ));
                content.push_str("    };\n");
            }
            
            // DNSSL (DNS Search List) configuration
            if !interface.dnssl_domains.is_empty() {
                content.push_str("\n    DNSSL ");
                content.push_str(&interface.dnssl_domains.join(" "));
                content.push_str("\n    {\n");
                content.push_str(&format!(
                    "        AdvDNSSLLifetime {};\n",
                    interface.default_lifetime
                ));
                content.push_str("    };\n");
            }
            
            content.push_str("};\n\n");
        }
        
        Ok(content)
    }
    
    async fn reload_radvd(&self) -> Result<()> {
        // Check if radvd is running
        if Path::new(&self.pid_path).exists() {
            // Reload configuration
            let output = Command::new("systemctl")
                .arg("reload")
                .arg("radvd")
                .output()?;
                
            if output.status.success() {
                info!("Successfully reloaded radvd");
            } else {
                // Try restart if reload fails
                self.restart_radvd().await?;
            }
        } else {
            // Start radvd
            self.start_radvd().await?;
        }
        
        Ok(())
    }
    
    async fn start_radvd(&self) -> Result<()> {
        let output = Command::new("systemctl")
            .arg("start")
            .arg("radvd")
            .output()?;
            
        if output.status.success() {
            info!("Successfully started radvd");
        } else {
            error!("Failed to start radvd: {:?}", output.stderr);
            return Err(anyhow::anyhow!("Failed to start radvd"));
        }
        
        Ok(())
    }
    
    async fn restart_radvd(&self) -> Result<()> {
        let output = Command::new("systemctl")
            .arg("restart")
            .arg("radvd")
            .output()?;
            
        if output.status.success() {
            info!("Successfully restarted radvd");
        } else {
            error!("Failed to restart radvd: {:?}", output.stderr);
            return Err(anyhow::anyhow!("Failed to restart radvd"));
        }
        
        Ok(())
    }
    
    pub async fn monitor(&self) -> Result<()> {
        let mut check_interval = interval(Duration::from_secs(30));
        
        loop {
            check_interval.tick().await;
            
            // Check if radvd is still running
            let output = Command::new("systemctl")
                .arg("is-active")
                .arg("radvd")
                .output()?;
                
            if !output.status.success() {
                error!("radvd is not running, attempting to restart");
                if let Err(e) = self.restart_radvd().await {
                    error!("Failed to restart radvd: {}", e);
                }
            } else {
                debug!("radvd is running normally");
            }
        }
    }
    
    pub fn generate_default_config(&self) -> RadvdConfig {
        RadvdConfig {
            interfaces: vec![
                InterfaceConfig {
                    name: "eth0".to_string(),
                    prefix: "2001:db8::".to_string(),
                    prefix_length: 64,
                    send_advert: true,
                    managed_flag: false,  // SLAAC by default
                    other_config_flag: true,  // Get DNS from DHCPv6
                    min_rtr_adv_interval: 3,
                    max_rtr_adv_interval: 10,
                    default_lifetime: 1800,
                    prefix_valid_lifetime: 86400,
                    prefix_preferred_lifetime: 14400,
                    rdnss_servers: vec![
                        "2001:4860:4860::8888".to_string(),
                        "2001:4860:4860::8844".to_string(),
                    ],
                    dnssl_domains: vec!["example.com".to_string()],
                },
            ],
        }
    }
}

// Helper function to install radvd if not present
pub async fn ensure_radvd_installed() -> Result<()> {
    let output = Command::new("which")
        .arg("radvd")
        .output()?;
        
    if !output.status.success() {
        info!("radvd not found, attempting to install");
        
        // Detect package manager and install
        if Path::new("/usr/bin/apt").exists() {
            Command::new("apt")
                .arg("install")
                .arg("-y")
                .arg("radvd")
                .status()?;
        } else if Path::new("/usr/bin/yum").exists() {
            Command::new("yum")
                .arg("install")
                .arg("-y")
                .arg("radvd")
                .status()?;
        } else if Path::new("/usr/bin/dnf").exists() {
            Command::new("dnf")
                .arg("install")
                .arg("-y")
                .arg("radvd")
                .status()?;
        }
    }
    
    Ok(())
}