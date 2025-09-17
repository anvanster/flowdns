use actix_web::{HttpResponse, web};
use serde_json::json;

pub async fn openapi_spec() -> HttpResponse {
    let spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "FlowDNS API",
            "version": "1.0.0",
            "description": "Multi-subnet DNS/DHCP server management API"
        },
        "servers": [
            {
                "url": "http://localhost:8080/api/v1",
                "description": "Local development server"
            }
        ],
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            },
            "schemas": {
                "LoginRequest": {
                    "type": "object",
                    "required": ["username", "password"],
                    "properties": {
                        "username": {"type": "string"},
                        "password": {"type": "string"}
                    }
                },
                "LoginResponse": {
                    "type": "object",
                    "properties": {
                        "token": {"type": "string"},
                        "expires_in": {"type": "integer"}
                    }
                },
                "Lease": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "format": "uuid"},
                        "subnet_id": {"type": "string", "format": "uuid"},
                        "mac_address": {"type": "string"},
                        "ip_address": {"type": "string", "format": "ipv4"},
                        "hostname": {"type": "string"},
                        "lease_start": {"type": "string", "format": "date-time"},
                        "lease_end": {"type": "string", "format": "date-time"},
                        "state": {"type": "string", "enum": ["active", "expired", "released"]}
                    }
                },
                "Subnet": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "format": "uuid"},
                        "name": {"type": "string"},
                        "network": {"type": "string"},
                        "start_ip": {"type": "string", "format": "ipv4"},
                        "end_ip": {"type": "string", "format": "ipv4"},
                        "gateway": {"type": "string", "format": "ipv4"},
                        "dns_servers": {"type": "array", "items": {"type": "string"}},
                        "domain_name": {"type": "string"},
                        "vlan_id": {"type": "integer"},
                        "enabled": {"type": "boolean"}
                    }
                },
                "DnsZone": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "format": "uuid"},
                        "name": {"type": "string"},
                        "type": {"type": "string", "enum": ["forward", "reverse"]},
                        "ttl": {"type": "integer"},
                        "soa_serial": {"type": "integer"},
                        "enabled": {"type": "boolean"}
                    }
                },
                "DnsRecord": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "format": "uuid"},
                        "zone_id": {"type": "string", "format": "uuid"},
                        "name": {"type": "string"},
                        "type": {"type": "string", "enum": ["A", "AAAA", "CNAME", "MX", "TXT", "PTR", "NS", "SOA"]},
                        "value": {"type": "string"},
                        "ttl": {"type": "integer"},
                        "priority": {"type": "integer"},
                        "is_dynamic": {"type": "boolean"}
                    }
                }
            }
        },
        "paths": {
            "/auth/login": {
                "post": {
                    "summary": "Login to get JWT token",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/LoginRequest"}
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Login successful",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/LoginResponse"}
                                }
                            }
                        }
                    }
                }
            },
            "/dhcp/leases": {
                "get": {
                    "summary": "List all DHCP leases",
                    "security": [{"bearerAuth": []}],
                    "parameters": [
                        {
                            "name": "state",
                            "in": "query",
                            "schema": {"type": "string", "enum": ["active", "expired", "released"]}
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of leases",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Lease"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create a new DHCP lease",
                    "security": [{"bearerAuth": []}],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/Lease"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Lease created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Lease"}
                                }
                            }
                        }
                    }
                }
            },
            "/dhcp/leases/{id}": {
                "get": {
                    "summary": "Get a specific lease",
                    "security": [{"bearerAuth": []}],
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": {"type": "string", "format": "uuid"}
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Lease details",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Lease"}
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "summary": "Release a DHCP lease",
                    "security": [{"bearerAuth": []}],
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": {"type": "string", "format": "uuid"}
                        }
                    ],
                    "responses": {
                        "204": {
                            "description": "Lease released"
                        }
                    }
                }
            },
            "/dhcp/subnets": {
                "get": {
                    "summary": "List all subnets",
                    "security": [{"bearerAuth": []}],
                    "responses": {
                        "200": {
                            "description": "List of subnets",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/Subnet"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create a new subnet",
                    "security": [{"bearerAuth": []}],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/Subnet"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Subnet created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Subnet"}
                                }
                            }
                        }
                    }
                }
            },
            "/dns/zones": {
                "get": {
                    "summary": "List all DNS zones",
                    "security": [{"bearerAuth": []}],
                    "responses": {
                        "200": {
                            "description": "List of zones",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/DnsZone"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create a new DNS zone",
                    "security": [{"bearerAuth": []}],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/DnsZone"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Zone created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/DnsZone"}
                                }
                            }
                        }
                    }
                }
            },
            "/dns/records": {
                "get": {
                    "summary": "List all DNS records",
                    "security": [{"bearerAuth": []}],
                    "parameters": [
                        {
                            "name": "zone_id",
                            "in": "query",
                            "schema": {"type": "string", "format": "uuid"}
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of records",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {"$ref": "#/components/schemas/DnsRecord"}
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create a new DNS record",
                    "security": [{"bearerAuth": []}],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/DnsRecord"}
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Record created",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/DnsRecord"}
                                }
                            }
                        }
                    }
                }
            },
            "/system/health": {
                "get": {
                    "summary": "Health check endpoint",
                    "responses": {
                        "200": {
                            "description": "System health status",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": {"type": "string"},
                                            "database": {"type": "string"},
                                            "dhcp_server": {"type": "string"},
                                            "dns_server": {"type": "string"},
                                            "api_server": {"type": "string"},
                                            "timestamp": {"type": "string", "format": "date-time"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/system/metrics": {
                "get": {
                    "summary": "System metrics",
                    "security": [{"bearerAuth": []}],
                    "responses": {
                        "200": {
                            "description": "System metrics",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "dhcp": {
                                                "type": "object",
                                                "properties": {
                                                    "total_subnets": {"type": "integer"},
                                                    "active_leases": {"type": "integer"},
                                                    "expired_leases": {"type": "integer"},
                                                    "reserved_addresses": {"type": "integer"},
                                                    "available_addresses": {"type": "integer"}
                                                }
                                            },
                                            "dns": {
                                                "type": "object",
                                                "properties": {
                                                    "total_zones": {"type": "integer"},
                                                    "total_records": {"type": "integer"},
                                                    "dynamic_records": {"type": "integer"}
                                                }
                                            },
                                            "system": {
                                                "type": "object",
                                                "properties": {
                                                    "uptime_seconds": {"type": "integer"},
                                                    "memory_usage_mb": {"type": "number"},
                                                    "cpu_usage_percent": {"type": "number"}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    HttpResponse::Ok()
        .content_type("application/json")
        .json(spec)
}

pub async fn swagger_ui() -> HttpResponse {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>FlowDNS API Documentation</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
    <style>
        html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin: 0; background: #fafafa; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            window.ui = SwaggerUIBundle({
                url: "/api/docs/openapi.json",
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>"#;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}