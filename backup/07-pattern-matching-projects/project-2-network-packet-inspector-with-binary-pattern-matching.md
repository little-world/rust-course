## Project 2: Network Packet Inspector with Binary Pattern Matching

### Problem Statement

Build a network packet analyzer that:
- Parses binary network protocols (Ethernet, IP, TCP, UDP, HTTP)
- Uses pattern matching to destructure packet headers
- Implements protocol-aware filtering rules
- Supports deep packet inspection with nested destructuring
- Provides firewall rule engine using enum-driven architecture
- Extracts payload data with byte slice patterns
- Demonstrates range matching for port numbers and IP addresses
- Handles protocol variants (IPv4 vs IPv6, TCP vs UDP)

The tool must showcase pattern matching on binary data and protocol layers.

### Why It Matters

Network analysis is essential for:
- **Security**: Firewalls, intrusion detection, malware analysis
- **Debugging**: Protocol tracing, performance analysis
- **Monitoring**: Traffic analysis, bandwidth monitoring
- **Compliance**: Data loss prevention, audit logging

Pattern matching excels for packet parsing because:
- Protocol headers map directly to struct destructuring
- Enums represent protocol types naturally
- Match exhaustiveness ensures all protocols handled
- Range patterns perfect for port/address filtering
- Guards enable complex filtering rules

### Use Cases

1. **Firewalls**: Filter packets by IP, port, protocol
2. **IDS/IPS**: Detect malicious patterns in traffic
3. **Packet Capture**: tcpdump/Wireshark functionality
4. **Load Balancers**: Route packets by content
5. **VPN/Proxy**: Inspect and modify packets
6. **Network Monitoring**: Track bandwidth, connections
7. **Protocol Testing**: Validate protocol implementations

### Solution Outline

**Core Protocol Enums:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum EtherType {
    IPv4,
    IPv6,
    ARP,
    Other(u16),
}

#[derive(Debug, Clone)]
pub enum Packet {
    Ethernet {
        dst_mac: [u8; 6],
        src_mac: [u8; 6],
        ethertype: EtherType,
        payload: Box<Packet>,
    },
    IPv4 {
        version: u8,
        header_len: u8,
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        protocol: IpProtocol,
        payload: Box<Packet>,
    },
    TCP {
        src_port: u16,
        dst_port: u16,
        seq: u32,
        ack: u32,
        flags: TcpFlags,
        payload: Vec<u8>,
    },
    UDP {
        src_port: u16,
        dst_port: u16,
        payload: Vec<u8>,
    },
    HTTP {
        method: HttpMethod,
        path: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    },
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IpProtocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}
```

**Pattern Matching for Filtering:**
```rust
impl Packet {
    pub fn matches_filter(&self, filter: &PacketFilter) -> bool {
        match (self, filter) {
            // Match specific ports with range patterns
            (
                Packet::TCP { src_port, dst_port, .. },
                PacketFilter::Port(p @ 1..=1023) // Well-known ports
            ) => *src_port == *p || *dst_port == *p,

            // Deep destructuring for nested protocols
            (
                Packet::Ethernet {
                    payload: box Packet::IPv4 {
                        src_ip,
                        protocol: IpProtocol::TCP,
                        payload: box Packet::TCP { dst_port, .. },
                        ..
                    },
                    ..
                },
                PacketFilter::HttpTraffic
            ) if *dst_port == 80 || *dst_port == 443 => true,

            // Exhaustive protocol matching
            (packet, filter) => self.deep_match(packet, filter),
        }
    }
}
```

**Testing Hints:**
```rust
#[test]
fn test_tcp_packet_parsing() {
    let raw = &[/* TCP packet bytes */];
    let packet = Packet::parse(raw).unwrap();

    match packet {
        Packet::TCP { src_port: 80, dst_port, .. } => {
            assert!(dst_port > 1024);
        }
        _ => panic!("Expected TCP packet"),
    }
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Parse Ethernet Frames with Byte Slice Patterns

**Goal:** Parse Ethernet layer using slice destructuring.

**What to implement:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MacAddress([u8; 6]);

#[derive(Debug, Clone, PartialEq)]
pub enum EtherType {
    IPv4,     // 0x0800
    IPv6,     // 0x86DD
    ARP,      // 0x0806
    Unknown(u16),
}

#[derive(Debug, Clone)]
pub struct EthernetFrame {
    pub dst_mac: MacAddress,
    pub src_mac: MacAddress,
    pub ethertype: EtherType,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // Pattern match on slice length and destructure
        match data {
            // Minimum Ethernet frame: 14 bytes header + payload
            [dst @ .., src @ .., eth_type @ .., payload @ ..]
                if dst.len() == 6 && src.len() == 6 && eth_type.len() == 2 =>
            {
                let dst_mac = MacAddress([
                    dst[0], dst[1], dst[2], dst[3], dst[4], dst[5]
                ]);

                let src_mac = MacAddress([
                    src[0], src[1], src[2], src[3], src[4], src[5]
                ]);

                let ethertype_value = u16::from_be_bytes([eth_type[0], eth_type[1]]);

                let ethertype = match ethertype_value {
                    0x0800 => EtherType::IPv4,
                    0x86DD => EtherType::IPv6,
                    0x0806 => EtherType::ARP,
                    other => EtherType::Unknown(other),
                };

                Ok(EthernetFrame {
                    dst_mac,
                    src_mac,
                    ethertype,
                    payload: payload.to_vec(),
                })
            }

            // Better approach: explicit indexing
            data if data.len() >= 14 => {
                let dst_mac = MacAddress([
                    data[0], data[1], data[2], data[3], data[4], data[5]
                ]);

                let src_mac = MacAddress([
                    data[6], data[7], data[8], data[9], data[10], data[11]
                ]);

                let ethertype_value = u16::from_be_bytes([data[12], data[13]]);

                let ethertype = match ethertype_value {
                    0x0800 => EtherType::IPv4,
                    0x86DD => EtherType::IPv6,
                    0x0806 => EtherType::ARP,
                    other => EtherType::Unknown(other),
                };

                Ok(EthernetFrame {
                    dst_mac,
                    src_mac,
                    ethertype,
                    payload: data[14..].to_vec(),
                })
            }

            _ => Err(ParseError::TooShort {
                expected: 14,
                found: data.len(),
            }),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    TooShort { expected: usize, found: usize },
    InvalidProtocol(u8),
    Malformed(String),
}
```

**Check/Test:**
- Test parsing valid Ethernet frame
- Test different ethertypes
- Test too-short buffer returns error
- Test MAC address extraction

**Why this isn't enough:**
Only parses Ethernet layer. Real packet analysis needs IP, TCP, UDP layers. Pattern matching is basic—we're not demonstrating guards, nested destructuring, or complex filtering. We need to parse the protocol stack recursively and showcase deep pattern matching.

---

### Step 2: Add IPv4 Parsing with Nested Destructuring

**Goal:** Parse IP layer and demonstrate nested protocol destructuring.

**What to improve:**
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Address([a, b, c, d])
    }

    pub fn is_private(&self) -> bool {
        match self.0 {
            [10, _, _, _] => true,                    // 10.0.0.0/8
            [172, b, _, _] if (16..=31).contains(&b) => true,  // 172.16.0.0/12
            [192, 168, _, _] => true,                 // 192.168.0.0/16
            _ => false,
        }
    }

    pub fn is_loopback(&self) -> bool {
        matches!(self.0, [127, _, _, _])
    }

    pub fn is_multicast(&self) -> bool {
        matches!(self.0, [a, _, _, _] if (224..=239).contains(&a))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IpProtocol {
    ICMP,   // 1
    TCP,    // 6
    UDP,    // 17
    Unknown(u8),
}

#[derive(Debug, Clone)]
pub struct Ipv4Packet {
    pub version: u8,
    pub header_length: u8,
    pub total_length: u16,
    pub ttl: u8,
    pub protocol: IpProtocol,
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
    pub payload: Vec<u8>,
}

impl Ipv4Packet {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        match data {
            // IPv4 minimum header: 20 bytes
            [version_ihl, _tos, total_len @ .., _id @ .., _flags @ .., ttl, protocol, _checksum @ .., src @ .., dst @ .., rest @ ..]
                if data.len() >= 20 =>
            {
                let version = (version_ihl >> 4) & 0x0F;
                let header_length = (version_ihl & 0x0F) * 4;

                if version != 4 {
                    return Err(ParseError::InvalidProtocol(version));
                }

                // Better approach with explicit slicing
                let total_length = u16::from_be_bytes([data[2], data[3]]);
                let ttl = data[8];
                let protocol_num = data[9];

                let protocol = match protocol_num {
                    1 => IpProtocol::ICMP,
                    6 => IpProtocol::TCP,
                    17 => IpProtocol::UDP,
                    other => IpProtocol::Unknown(other),
                };

                let src_ip = Ipv4Address([data[12], data[13], data[14], data[15]]);
                let dst_ip = Ipv4Address([data[16], data[17], data[18], data[19]]);

                let header_len = header_length as usize;
                let payload = if data.len() > header_len {
                    data[header_len..].to_vec()
                } else {
                    vec![]
                };

                Ok(Ipv4Packet {
                    version,
                    header_length,
                    total_length,
                    ttl,
                    protocol,
                    src_ip,
                    dst_ip,
                    payload,
                })
            }

            _ => Err(ParseError::TooShort {
                expected: 20,
                found: data.len(),
            }),
        }
    }
}

// Layered packet representation
#[derive(Debug, Clone)]
pub enum Packet {
    Ethernet {
        frame: EthernetFrame,
        inner: Option<Box<Packet>>,
    },
    IPv4 {
        packet: Ipv4Packet,
        inner: Option<Box<Packet>>,
    },
    Raw(Vec<u8>),
}

impl Packet {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let ethernet = EthernetFrame::parse(data)?;

        let inner = match ethernet.ethertype {
            EtherType::IPv4 => {
                let ipv4 = Ipv4Packet::parse(&ethernet.payload)?;
                Some(Box::new(Packet::IPv4 {
                    packet: ipv4,
                    inner: None,
                }))
            }
            _ => None,
        };

        Ok(Packet::Ethernet {
            frame: ethernet,
            inner,
        })
    }

    // Deep destructuring to extract information
    pub fn extract_ips(&self) -> Option<(Ipv4Address, Ipv4Address)> {
        match self {
            // Nested destructuring
            Packet::Ethernet {
                inner: Some(box Packet::IPv4 { packet: Ipv4Packet { src_ip, dst_ip, .. }, .. }),
                ..
            } => Some((*src_ip, *dst_ip)),

            Packet::IPv4 { packet: Ipv4Packet { src_ip, dst_ip, .. }, .. } => {
                Some((*src_ip, *dst_ip))
            }

            _ => None,
        }
    }
}
```

**Pattern matching for IP classification:**
```rust
fn classify_traffic(packet: &Packet) -> TrafficType {
    match packet {
        // Local traffic
        Packet::IPv4 {
            packet: Ipv4Packet {
                src_ip: src @ Ipv4Address([10, _, _, _]),
                dst_ip: dst @ Ipv4Address([10, _, _, _]),
                ..
            },
            ..
        } => TrafficType::LocalPrivate,

        // Internet-bound traffic
        Packet::IPv4 {
            packet: Ipv4Packet {
                src_ip,
                dst_ip,
                ..
            },
            ..
        } if src_ip.is_private() && !dst_ip.is_private() => TrafficType::Outbound,

        // Multicast
        Packet::IPv4 {
            packet: Ipv4Packet {
                dst_ip,
                ..
            },
            ..
        } if dst_ip.is_multicast() => TrafficType::Multicast,

        _ => TrafficType::Other,
    }
}

#[derive(Debug, PartialEq)]
enum TrafficType {
    LocalPrivate,
    Outbound,
    Inbound,
    Multicast,
    Other,
}
```

**Check/Test:**
- Test IPv4 parsing
- Test private IP detection
- Test nested packet parsing (Ethernet → IPv4)
- Test deep destructuring for IP extraction
- Test traffic classification

**Why this isn't enough:**
We parse IP but not transport layer (TCP/UDP). Can't inspect port numbers or flags. Pattern matching for filtering is limited—we need complex filter rules. Real packet inspection requires TCP/UDP parsing with state tracking (connections, sessions).

---

### Step 3: Add TCP/UDP Parsing and Port Range Matching

**Goal:** Parse transport layer and demonstrate range patterns for port filtering.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub struct TcpPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub flags: TcpFlags,
    pub window_size: u16,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct TcpFlags {
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
}

impl TcpFlags {
    fn from_byte(byte: u8) -> Self {
        TcpFlags {
            fin: (byte & 0x01) != 0,
            syn: (byte & 0x02) != 0,
            rst: (byte & 0x04) != 0,
            psh: (byte & 0x08) != 0,
            ack: (byte & 0x10) != 0,
            urg: (byte & 0x20) != 0,
        }
    }
}

impl TcpPacket {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 20 {
            return Err(ParseError::TooShort {
                expected: 20,
                found: data.len(),
            });
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq_num = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let data_offset = (data[12] >> 4) * 4;
        let flags = TcpFlags::from_byte(data[13]);
        let window_size = u16::from_be_bytes([data[14], data[15]]);

        let header_len = data_offset as usize;
        let payload = if data.len() > header_len {
            data[header_len..].to_vec()
        } else {
            vec![]
        };

        Ok(TcpPacket {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            flags,
            window_size,
            payload,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UdpPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 8 {
            return Err(ParseError::TooShort {
                expected: 8,
                found: data.len(),
            });
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);

        let payload = data[8..].to_vec();

        Ok(UdpPacket {
            src_port,
            dst_port,
            length,
            payload,
        })
    }
}

// Enhanced packet enum
#[derive(Debug, Clone)]
pub enum Packet {
    Ethernet {
        frame: EthernetFrame,
        inner: Option<Box<Packet>>,
    },
    IPv4 {
        packet: Ipv4Packet,
        inner: Option<Box<Packet>>,
    },
    TCP(TcpPacket),
    UDP(UdpPacket),
    Raw(Vec<u8>),
}

// Pattern matching for port ranges
fn classify_port(port: u16) -> PortClass {
    match port {
        0 => PortClass::Reserved,
        1..=1023 => PortClass::WellKnown,
        1024..=49151 => PortClass::Registered,
        49152..=65535 => PortClass::Dynamic,
    }
}

#[derive(Debug, PartialEq)]
enum PortClass {
    Reserved,
    WellKnown,
    Registered,
    Dynamic,
}

// Service detection with range patterns
fn detect_service(packet: &Packet) -> Option<Service> {
    match packet {
        Packet::TCP(TcpPacket { dst_port: 80 | 8080 | 8000, .. }) => Some(Service::Http),
        Packet::TCP(TcpPacket { dst_port: 443 | 8443, .. }) => Some(Service::Https),
        Packet::TCP(TcpPacket { dst_port: 22, .. }) => Some(Service::SSH),
        Packet::TCP(TcpPacket { dst_port: 21 | 20, .. }) => Some(Service::FTP),
        Packet::TCP(TcpPacket { dst_port: 25, .. }) => Some(Service::SMTP),
        Packet::TCP(TcpPacket { dst_port: p @ 3306 | p @ 5432, .. }) => Some(Service::Database),
        Packet::UDP(UdpPacket { dst_port: 53, .. }) => Some(Service::DNS),
        Packet::UDP(UdpPacket { dst_port: 67 | 68, .. }) => Some(Service::DHCP),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
enum Service {
    Http,
    Https,
    SSH,
    FTP,
    SMTP,
    Database,
    DNS,
    DHCP,
}
```

**Check/Test:**
- Test TCP parsing with flags
- Test UDP parsing
- Test port classification
- Test service detection with or-patterns
- Test range patterns compile correctly

**Why this isn't enough:**
We parse packets but have no filtering engine. Real firewalls need complex rule matching with multiple criteria. We also don't handle HTTP payload inspection. The pattern matching showcases ranges and or-patterns but not guards, if-let chains, or exhaustive rule engines. Let's build a complete firewall rule system.

---

### Step 4: Add Firewall Rule Engine with Guards and Complex Patterns

**Goal:** Implement a firewall rule engine demonstrating guards, exhaustive matching, and complex filters.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub enum FirewallRule {
    AllowAll,
    DenyAll,
    AllowPort { port: u16 },
    DenyPort { port: u16 },
    AllowPortRange { start: u16, end: u16 },
    AllowIp { ip: Ipv4Address },
    DenyIp { ip: Ipv4Address },
    AllowSubnet { network: Ipv4Address, mask: u8 },
    AllowService(Service),
    DenyService(Service),
    Complex {
        src_ip: Option<Ipv4Address>,
        dst_ip: Option<Ipv4Address>,
        src_port: Option<u16>,
        dst_port: Option<u16>,
        protocol: Option<IpProtocol>,
        action: Action,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Allow,
    Deny,
    Log,
    LogAndAllow,
    LogAndDeny,
}

#[derive(Debug)]
pub struct Firewall {
    rules: Vec<FirewallRule>,
    default_action: Action,
}

impl Firewall {
    pub fn new(default_action: Action) -> Self {
        Firewall {
            rules: Vec::new(),
            default_action,
        }
    }

    pub fn add_rule(&mut self, rule: FirewallRule) {
        self.rules.push(rule);
    }

    pub fn check_packet(&self, packet: &Packet) -> Action {
        // Try each rule in order
        for rule in &self.rules {
            if let Some(action) = self.match_rule(rule, packet) {
                return action;
            }
        }

        self.default_action
    }

    fn match_rule(&self, rule: &FirewallRule, packet: &Packet) -> Option<Action> {
        match (rule, packet) {
            // Simple allow/deny all
            (FirewallRule::AllowAll, _) => Some(Action::Allow),
            (FirewallRule::DenyAll, _) => Some(Action::Deny),

            // Port-based rules with exhaustive protocol matching
            (FirewallRule::AllowPort { port }, Packet::TCP(tcp))
                if tcp.src_port == *port || tcp.dst_port == *port =>
            {
                Some(Action::Allow)
            }

            (FirewallRule::AllowPort { port }, Packet::UDP(udp))
                if udp.src_port == *port || udp.dst_port == *port =>
            {
                Some(Action::Allow)
            }

            // Port range with guards
            (
                FirewallRule::AllowPortRange { start, end },
                Packet::TCP(TcpPacket { dst_port, .. })
            ) if (*start..=*end).contains(dst_port) => Some(Action::Allow),

            (
                FirewallRule::AllowPortRange { start, end },
                Packet::UDP(UdpPacket { dst_port, .. })
            ) if (*start..=*end).contains(dst_port) => Some(Action::Allow),

            // Deep destructuring for nested packets with guards
            (
                FirewallRule::AllowIp { ip },
                Packet::Ethernet {
                    inner: Some(box Packet::IPv4 { packet, .. }),
                    ..
                }
            ) if packet.src_ip == *ip || packet.dst_ip == *ip => Some(Action::Allow),

            (
                FirewallRule::AllowIp { ip },
                Packet::IPv4 { packet, .. }
            ) if packet.src_ip == *ip || packet.dst_ip == *ip => Some(Action::Allow),

            // Subnet matching with guards
            (
                FirewallRule::AllowSubnet { network, mask },
                Packet::IPv4 { packet, .. }
            ) if Self::in_subnet(&packet.dst_ip, network, *mask) => Some(Action::Allow),

            // Service-based rules
            (FirewallRule::AllowService(service), packet)
                if detect_service(packet) == Some(*service) =>
            {
                Some(Action::Allow)
            }

            (FirewallRule::DenyService(service), packet)
                if detect_service(packet) == Some(*service) =>
            {
                Some(Action::Deny)
            }

            // Complex rule with multiple criteria
            (
                FirewallRule::Complex {
                    src_ip,
                    dst_ip,
                    src_port,
                    dst_port,
                    protocol,
                    action,
                },
                packet,
            ) => {
                // Extract packet details
                let packet_info = PacketInfo::extract(packet)?;

                // Check all criteria with if-let chains
                let src_ip_matches = src_ip
                    .map(|ip| packet_info.src_ip == Some(ip))
                    .unwrap_or(true);

                let dst_ip_matches = dst_ip
                    .map(|ip| packet_info.dst_ip == Some(ip))
                    .unwrap_or(true);

                let src_port_matches = src_port
                    .map(|port| packet_info.src_port == Some(port))
                    .unwrap_or(true);

                let dst_port_matches = dst_port
                    .map(|port| packet_info.dst_port == Some(port))
                    .unwrap_or(true);

                let protocol_matches = protocol
                    .map(|proto| packet_info.protocol == Some(proto))
                    .unwrap_or(true);

                if src_ip_matches
                    && dst_ip_matches
                    && src_port_matches
                    && dst_port_matches
                    && protocol_matches
                {
                    Some(*action)
                } else {
                    None
                }
            }

            // No match
            _ => None,
        }
    }

    fn in_subnet(ip: &Ipv4Address, network: &Ipv4Address, mask: u8) -> bool {
        let ip_bits = u32::from_be_bytes(ip.0);
        let net_bits = u32::from_be_bytes(network.0);
        let mask_bits = !0u32 << (32 - mask);

        (ip_bits & mask_bits) == (net_bits & mask_bits)
    }
}

// Helper to extract packet info
#[derive(Debug)]
struct PacketInfo {
    src_ip: Option<Ipv4Address>,
    dst_ip: Option<Ipv4Address>,
    src_port: Option<u16>,
    dst_port: Option<u16>,
    protocol: Option<IpProtocol>,
}

impl PacketInfo {
    fn extract(packet: &Packet) -> Option<Self> {
        match packet {
            // Deep destructuring for full packet info
            Packet::Ethernet {
                inner: Some(box Packet::IPv4 {
                    packet: ipv4,
                    inner: Some(box Packet::TCP(tcp)),
                }),
                ..
            } => Some(PacketInfo {
                src_ip: Some(ipv4.src_ip),
                dst_ip: Some(ipv4.dst_ip),
                src_port: Some(tcp.src_port),
                dst_port: Some(tcp.dst_port),
                protocol: Some(IpProtocol::TCP),
            }),

            Packet::Ethernet {
                inner: Some(box Packet::IPv4 {
                    packet: ipv4,
                    inner: Some(box Packet::UDP(udp)),
                }),
                ..
            } => Some(PacketInfo {
                src_ip: Some(ipv4.src_ip),
                dst_ip: Some(ipv4.dst_ip),
                src_port: Some(udp.src_port),
                dst_port: Some(udp.dst_port),
                protocol: Some(IpProtocol::UDP),
            }),

            Packet::IPv4 { packet, .. } => Some(PacketInfo {
                src_ip: Some(packet.src_ip),
                dst_ip: Some(packet.dst_ip),
                src_port: None,
                dst_port: None,
                protocol: Some(packet.protocol),
            }),

            Packet::TCP(tcp) => Some(PacketInfo {
                src_ip: None,
                dst_ip: None,
                src_port: Some(tcp.src_port),
                dst_port: Some(tcp.dst_port),
                protocol: Some(IpProtocol::TCP),
            }),

            Packet::UDP(udp) => Some(PacketInfo {
                src_ip: None,
                dst_ip: None,
                src_port: Some(udp.src_port),
                dst_port: Some(udp.dst_port),
                protocol: Some(IpProtocol::UDP),
            }),

            _ => None,
        }
    }
}
```

**Pattern matching utilities:**
```rust
// Using matches! macro for quick checks
pub fn is_tcp_syn(packet: &Packet) -> bool {
    matches!(
        packet,
        Packet::TCP(TcpPacket { flags: TcpFlags { syn: true, ack: false, .. }, .. })
    )
}

pub fn is_tcp_ack(packet: &Packet) -> bool {
    matches!(
        packet,
        Packet::TCP(TcpPacket { flags: TcpFlags { ack: true, .. }, .. })
    )
}

// Exhaustive pattern matching for TCP flags
fn classify_tcp_packet(flags: &TcpFlags) -> TcpPacketType {
    match (flags.syn, flags.ack, flags.fin, flags.rst) {
        (true, false, false, false) => TcpPacketType::Syn,
        (true, true, false, false) => TcpPacketType::SynAck,
        (false, true, false, false) => TcpPacketType::Ack,
        (false, true, true, false) => TcpPacketType::FinAck,
        (false, false, false, true) => TcpPacketType::Rst,
        (false, true, false, true) => TcpPacketType::RstAck,
        _ => TcpPacketType::Other,
    }
}

#[derive(Debug, PartialEq)]
enum TcpPacketType {
    Syn,
    SynAck,
    Ack,
    FinAck,
    Rst,
    RstAck,
    Other,
}
```

**Check/Test:**
- Test firewall allows/denies based on ports
- Test subnet matching works correctly
- Test complex rules with multiple criteria
- Test deep destructuring extracts all packet info
- Test exhaustive TCP flag matching
- Benchmark: rule evaluation performance

**Why this isn't enough:**
Firewall rules work but we don't inspect payload content. Real IDS/IPS systems need deep packet inspection—analyzing HTTP headers, extracting strings from payloads, detecting malicious patterns. We also don't handle stateful inspection (tracking TCP connections). Let's add HTTP parsing and payload analysis.

---

### Step 5: Deep Packet Inspection with HTTP Parsing and Pattern Guards

**Goal:** Parse HTTP over TCP and demonstrate payload inspection with pattern guards.

**What to improve:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // Convert to string for parsing
        let text = String::from_utf8_lossy(data);

        // Split headers and body
        let parts: Vec<&str> = text.splitn(2, "\r\n\r\n").collect();

        let header_section = parts[0];
        let body = parts.get(1).map(|s| s.as_bytes().to_vec()).unwrap_or_default();

        // Parse request line and headers
        let mut lines = header_section.lines();

        let Some(request_line) = lines.next() else {
            return Err(ParseError::Malformed("Missing request line".into()));
        };

        // Parse request line with pattern matching
        let request_parts: Vec<&str> = request_line.split_whitespace().collect();

        let (method_str, path, version) = match request_parts.as_slice() {
            [method, path, version] => (*method, *path, *version),
            _ => {
                return Err(ParseError::Malformed(
                    "Invalid request line format".into(),
                ))
            }
        };

        let method = match method_str {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            "PATCH" => HttpMethod::PATCH,
            other => HttpMethod::Other(other.to_string()),
        };

        // Parse headers
        let mut headers = Vec::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        Ok(HttpRequest {
            method,
            path: path.to_string(),
            version: version.to_string(),
            headers,
            body,
        })
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

// Enhanced packet enum with HTTP
#[derive(Debug, Clone)]
pub enum Packet {
    Ethernet {
        frame: EthernetFrame,
        inner: Option<Box<Packet>>,
    },
    IPv4 {
        packet: Ipv4Packet,
        inner: Option<Box<Packet>>,
    },
    TCP(TcpPacket),
    UDP(UdpPacket),
    Http(HttpRequest),
    Raw(Vec<u8>),
}

impl Packet {
    pub fn parse_full(data: &[u8]) -> Result<Self, ParseError> {
        let ethernet = EthernetFrame::parse(data)?;

        let inner = match ethernet.ethertype {
            EtherType::IPv4 => {
                let ipv4 = Ipv4Packet::parse(&ethernet.payload)?;

                let transport = match ipv4.protocol {
                    IpProtocol::TCP => {
                        let tcp = TcpPacket::parse(&ipv4.payload)?;

                        // Try to parse HTTP if it's HTTP ports
                        match tcp.dst_port {
                            80 | 8080 | 8000 if !tcp.payload.is_empty() => {
                                if let Ok(http) = HttpRequest::parse(&tcp.payload) {
                                    Packet::Http(http)
                                } else {
                                    Packet::TCP(tcp)
                                }
                            }
                            _ => Packet::TCP(tcp),
                        }
                    }

                    IpProtocol::UDP => Packet::UDP(UdpPacket::parse(&ipv4.payload)?),

                    _ => Packet::Raw(ipv4.payload.clone()),
                };

                Some(Box::new(Packet::IPv4 {
                    packet: ipv4,
                    inner: Some(Box::new(transport)),
                }))
            }
            _ => None,
        };

        Ok(Packet::Ethernet {
            frame: ethernet,
            inner,
        })
    }
}

// Deep packet inspection with pattern matching
pub fn inspect_http_request(packet: &Packet) -> Option<HttpInspectionReport> {
    match packet {
        // Deep destructuring through all layers
        Packet::Ethernet {
            inner: Some(box Packet::IPv4 {
                packet: ipv4,
                inner: Some(box Packet::Http(http)),
            }),
            ..
        } => {
            let mut report = HttpInspectionReport {
                src_ip: ipv4.src_ip,
                dst_ip: ipv4.dst_ip,
                method: http.method.clone(),
                path: http.path.clone(),
                user_agent: http.get_header("User-Agent").map(String::from),
                content_type: http.get_header("Content-Type").map(String::from),
                threats: Vec::new(),
            };

            // Detect threats using pattern guards
            report.threats.extend(detect_threats(http));

            Some(report)
        }

        _ => None,
    }
}

#[derive(Debug)]
pub struct HttpInspectionReport {
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
    pub method: HttpMethod,
    pub path: String,
    pub user_agent: Option<String>,
    pub content_type: Option<String>,
    pub threats: Vec<ThreatType>,
}

#[derive(Debug, PartialEq)]
pub enum ThreatType {
    SqlInjection,
    XssAttempt,
    PathTraversal,
    SuspiciousUserAgent,
    LargePayload,
    SuspiciousHeader,
}

// Threat detection with pattern matching and guards
fn detect_threats(http: &HttpRequest) -> Vec<ThreatType> {
    let mut threats = Vec::new();

    // SQL injection patterns
    let path_lower = http.path.to_lowercase();
    if path_lower.contains("' or ")
        || path_lower.contains("1=1")
        || path_lower.contains("union select")
        || path_lower.contains("drop table")
    {
        threats.push(ThreatType::SqlInjection);
    }

    // XSS patterns
    if path_lower.contains("<script")
        || path_lower.contains("javascript:")
        || path_lower.contains("onerror=")
    {
        threats.push(ThreatType::XssAttempt);
    }

    // Path traversal
    if http.path.contains("../") || http.path.contains("..\\") {
        threats.push(ThreatType::PathTraversal);
    }

    // User-Agent analysis with pattern matching
    if let Some(ua) = http.get_header("User-Agent") {
        match ua {
            // Suspicious tools
            ua if ua.contains("sqlmap")
                || ua.contains("nikto")
                || ua.contains("nmap")
                || ua.contains("masscan") =>
            {
                threats.push(ThreatType::SuspiciousUserAgent);
            }
            _ => {}
        }
    }

    // Large payload
    if http.body.len() > 1_000_000 {
        threats.push(ThreatType::LargePayload);
    }

    // Suspicious headers
    for (name, value) in &http.headers {
        match name.to_lowercase().as_str() {
            "x-forwarded-for" if value.split(',').count() > 10 => {
                threats.push(ThreatType::SuspiciousHeader);
            }
            "referer" if value.len() > 1000 => {
                threats.push(ThreatType::SuspiciousHeader);
            }
            _ => {}
        }
    }

    threats
}

// Pattern matching for HTTP analysis
pub fn classify_http_request(http: &HttpRequest) -> HttpRequestClass {
    match (&http.method, http.path.as_str(), http.get_header("Content-Type")) {
        // API requests
        (HttpMethod::GET, path, _) if path.starts_with("/api/") => {
            HttpRequestClass::ApiGet
        }

        (HttpMethod::POST, path, Some(ct))
            if path.starts_with("/api/") && ct.contains("json") =>
        {
            HttpRequestClass::ApiPost
        }

        // Form submission
        (HttpMethod::POST, _, Some(ct))
            if ct.contains("application/x-www-form-urlencoded") =>
        {
            HttpRequestClass::FormSubmission
        }

        // File upload
        (HttpMethod::POST | HttpMethod::PUT, _, Some(ct))
            if ct.contains("multipart/form-data") =>
        {
            HttpRequestClass::FileUpload
        }

        // Static resources
        (HttpMethod::GET, path, _)
            if path.ends_with(".css")
                || path.ends_with(".js")
                || path.ends_with(".png")
                || path.ends_with(".jpg") =>
        {
            HttpRequestClass::StaticResource
        }

        // Page request
        (HttpMethod::GET, _, _) => HttpRequestClass::PageRequest,

        _ => HttpRequestClass::Other,
    }
}

#[derive(Debug, PartialEq)]
enum HttpRequestClass {
    ApiGet,
    ApiPost,
    FormSubmission,
    FileUpload,
    StaticResource,
    PageRequest,
    Other,
}
```

**If-let chains for validation:**
```rust
// Use if-let chains to validate HTTP requests
fn validate_http_request(packet: &Packet) -> Result<(), ValidationError> {
    // Extract HTTP request using if-let chain
    if let Packet::Ethernet {
        inner: Some(box Packet::IPv4 {
            inner: Some(box Packet::Http(http)),
            ..
        }),
        ..
    } = packet
    {
        // Validate method
        if !matches!(http.method, HttpMethod::GET | HttpMethod::POST | HttpMethod::PUT | HttpMethod::DELETE) {
            return Err(ValidationError::InvalidMethod);
        }

        // Validate path
        if http.path.is_empty() || !http.path.starts_with('/') {
            return Err(ValidationError::InvalidPath);
        }

        // Validate headers
        if http.get_header("Host").is_none() {
            return Err(ValidationError::MissingHostHeader);
        }

        Ok(())
    } else {
        Err(ValidationError::NotHttpPacket)
    }
}

#[derive(Debug)]
enum ValidationError {
    InvalidMethod,
    InvalidPath,
    MissingHostHeader,
    NotHttpPacket,
}
```

**Check/Test:**
- Test HTTP parsing from TCP payload
- Test threat detection identifies SQL injection, XSS
- Test deep packet inspection through all layers
- Test HTTP request classification
- Test if-let chains for validation
- Verify pattern guards work correctly

**Why this isn't enough:**
We inspect individual packets but don't track connections or sessions. Real packet analyzers maintain state—tracking TCP handshakes, HTTP sessions, request/response pairs. Performance is also not optimal for high-throughput scenarios. Let's add connection tracking and optimize with memoization and parallel processing.

---

### Step 6: Connection Tracking, Statistics, and Performance Optimization

**Goal:** Add stateful packet inspection, connection tracking, statistics, and performance optimizations.

**What to improve:**
```rust
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

// Connection tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionKey {
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: IpProtocol,
}

impl ConnectionKey {
    fn from_packet(packet: &Packet) -> Option<Self> {
        let info = PacketInfo::extract(packet)?;

        Some(ConnectionKey {
            src_ip: info.src_ip?,
            dst_ip: info.dst_ip?,
            src_port: info.src_port?,
            dst_port: info.dst_port?,
            protocol: info.protocol?,
        })
    }

    // Bidirectional key (for tracking both directions)
    fn canonical(&self) -> Self {
        if self.src_ip.0 < self.dst_ip.0
            || (self.src_ip == self.dst_ip && self.src_port < self.dst_port)
        {
            *self
        } else {
            ConnectionKey {
                src_ip: self.dst_ip,
                dst_ip: self.src_ip,
                src_port: self.dst_port,
                dst_port: self.src_port,
                protocol: self.protocol,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub key: ConnectionKey,
    pub state: ConnectionState,
    pub packets: usize,
    pub bytes: usize,
    pub start_time: Instant,
    pub last_seen: Instant,
    pub http_requests: Vec<HttpRequest>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    TcpSynSent,
    TcpSynReceived,
    TcpEstablished,
    TcpFinWait,
    TcpClosed,
    UdpActive,
    Unknown,
}

pub struct PacketAnalyzer {
    connections: HashMap<ConnectionKey, Connection>,
    statistics: Statistics,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            connections: HashMap::new(),
            statistics: Statistics::default(),
        }
    }

    pub fn process_packet(&mut self, packet: &Packet) {
        self.statistics.total_packets += 1;

        // Update statistics based on packet type
        self.update_statistics(packet);

        // Track connection if applicable
        if let Some(key) = ConnectionKey::from_packet(packet) {
            self.track_connection(key.canonical(), packet);
        }
    }

    fn track_connection(&mut self, key: ConnectionKey, packet: &Packet) {
        let conn = self.connections.entry(key).or_insert_with(|| Connection {
            key,
            state: ConnectionState::Unknown,
            packets: 0,
            bytes: 0,
            start_time: Instant::now(),
            last_seen: Instant::now(),
            http_requests: Vec::new(),
        });

        conn.packets += 1;
        conn.last_seen = Instant::now();

        // Update connection state using pattern matching
        self.update_connection_state(conn, packet);

        // Extract HTTP if present
        if let Some(http) = self.extract_http(packet) {
            conn.http_requests.push(http);
        }
    }

    fn update_connection_state(&mut self, conn: &mut Connection, packet: &Packet) {
        match (packet, &conn.state) {
            // TCP state machine
            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { syn: true, ack: false, .. },
                    ..
                }),
                ConnectionState::Unknown,
            ) => {
                conn.state = ConnectionState::TcpSynSent;
            }

            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { syn: true, ack: true, .. },
                    ..
                }),
                ConnectionState::TcpSynSent,
            ) => {
                conn.state = ConnectionState::TcpSynReceived;
            }

            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { ack: true, syn: false, fin: false, .. },
                    ..
                }),
                ConnectionState::TcpSynReceived,
            ) => {
                conn.state = ConnectionState::TcpEstablished;
            }

            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { fin: true, .. },
                    ..
                }),
                ConnectionState::TcpEstablished,
            ) => {
                conn.state = ConnectionState::TcpFinWait;
            }

            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { rst: true, .. },
                    ..
                }),
                _,
            ) => {
                conn.state = ConnectionState::TcpClosed;
            }

            // UDP connections
            (Packet::UDP(_), _) => {
                conn.state = ConnectionState::UdpActive;
            }

            _ => {}
        }
    }

    fn extract_http(&self, packet: &Packet) -> Option<HttpRequest> {
        match packet {
            Packet::Ethernet {
                inner: Some(box Packet::IPv4 {
                    inner: Some(box Packet::Http(http)),
                    ..
                }),
                ..
            } => Some(http.clone()),

            _ => None,
        }
    }

    fn update_statistics(&mut self, packet: &Packet) {
        match packet {
            Packet::Ethernet { .. } => {
                self.statistics.ethernet_packets += 1;
            }

            Packet::IPv4 { packet, .. } => {
                self.statistics.ipv4_packets += 1;

                match packet.protocol {
                    IpProtocol::TCP => self.statistics.tcp_packets += 1,
                    IpProtocol::UDP => self.statistics.udp_packets += 1,
                    IpProtocol::ICMP => self.statistics.icmp_packets += 1,
                    _ => {}
                }
            }

            Packet::TCP(_) => self.statistics.tcp_packets += 1,
            Packet::UDP(_) => self.statistics.udp_packets += 1,
            Packet::Http(_) => self.statistics.http_requests += 1,
            Packet::Raw(_) => self.statistics.raw_packets += 1,
        }
    }

    pub fn get_active_connections(&self) -> Vec<&Connection> {
        let now = Instant::now();

        self.connections
            .values()
            .filter(|conn| {
                // Active if seen in last 60 seconds
                now.duration_since(conn.last_seen) < Duration::from_secs(60)
                    && !matches!(conn.state, ConnectionState::TcpClosed)
            })
            .collect()
    }

    pub fn cleanup_old_connections(&mut self, max_age: Duration) {
        let now = Instant::now();

        self.connections.retain(|_, conn| {
            now.duration_since(conn.last_seen) < max_age
                || !matches!(conn.state, ConnectionState::TcpClosed)
        });
    }
}

#[derive(Debug, Default)]
pub struct Statistics {
    pub total_packets: usize,
    pub ethernet_packets: usize,
    pub ipv4_packets: usize,
    pub tcp_packets: usize,
    pub udp_packets: usize,
    pub icmp_packets: usize,
    pub http_requests: usize,
    pub raw_packets: usize,
}

impl Statistics {
    pub fn report(&self) -> String {
        format!(
            "Total: {}, Ethernet: {}, IPv4: {}, TCP: {}, UDP: {}, HTTP: {}",
            self.total_packets,
            self.ethernet_packets,
            self.ipv4_packets,
            self.tcp_packets,
            self.udp_packets,
            self.http_requests
        )
    }
}
```

**Advanced pattern matching for analysis:**
```rust
// Exhaustive pattern matching for connection analysis
pub fn analyze_connection(conn: &Connection) -> ConnectionAnalysis {
    match (&conn.state, conn.packets, conn.http_requests.len()) {
        // Suspicious: many packets but no established connection
        (ConnectionState::TcpSynSent, p, _) if p > 100 => {
            ConnectionAnalysis::SynFlood
        }

        // Port scan: SYN sent but never established
        (ConnectionState::TcpSynSent, p, _) if p < 5 => {
            ConnectionAnalysis::PossiblePortScan
        }

        // Normal HTTP connection
        (ConnectionState::TcpEstablished, _, http_count) if http_count > 0 => {
            ConnectionAnalysis::NormalHttp
        }

        // Long-lived connection with many packets
        (ConnectionState::TcpEstablished, p, _) if p > 10000 => {
            ConnectionAnalysis::LongLived
        }

        // UDP without response
        (ConnectionState::UdpActive, p, _) if p < 3 => {
            ConnectionAnalysis::UdpQuery
        }

        // Closed properly
        (ConnectionState::TcpClosed, _, _) => {
            ConnectionAnalysis::Closed
        }

        _ => ConnectionAnalysis::Normal,
    }
}

#[derive(Debug, PartialEq)]
pub enum ConnectionAnalysis {
    Normal,
    NormalHttp,
    LongLived,
    SynFlood,
    PossiblePortScan,
    UdpQuery,
    Closed,
}

// While-let for processing packet stream
pub fn process_packet_stream<I>(analyzer: &mut PacketAnalyzer, mut packets: I)
where
    I: Iterator<Item = Packet>,
{
    while let Some(packet) = packets.next() {
        analyzer.process_packet(&packet);

        // Periodic cleanup
        if analyzer.statistics.total_packets % 1000 == 0 {
            analyzer.cleanup_old_connections(Duration::from_secs(300));
        }
    }
}

// Let-else for extracting connection info
pub fn get_connection_duration(conn: &Connection) -> Result<Duration, String> {
    let Some(duration) = conn.last_seen.checked_duration_since(conn.start_time) else {
        return Err("Invalid time range".into());
    };

    Ok(duration)
}
```

**Complete example:**
```rust
fn main() {
    let mut analyzer = PacketAnalyzer::new();
    let mut firewall = Firewall::new(Action::Allow);

    // Add firewall rules
    firewall.add_rule(FirewallRule::DenyPort { port: 23 }); // Block telnet
    firewall.add_rule(FirewallRule::AllowPortRange {
        start: 80,
        end: 443,
    }); // Allow HTTP/HTTPS

    // Process packets
    let packets = vec![/* ... packet data ... */];

    for packet_data in packets {
        let Ok(packet) = Packet::parse_full(&packet_data) else {
            continue;
        };

        // Check firewall
        let action = firewall.check_packet(&packet);

        match action {
            Action::Allow | Action::LogAndAllow => {
                analyzer.process_packet(&packet);
            }
            Action::Deny | Action::LogAndDeny => {
                eprintln!("Blocked packet: {:?}", packet);
                continue;
            }
            Action::Log => {
                println!("Logged packet: {:?}", packet);
            }
        }

        // Inspect HTTP if present
        if let Some(report) = inspect_http_request(&packet) {
            if !report.threats.is_empty() {
                eprintln!("Threats detected: {:?}", report.threats);
            }
        }
    }

    // Print statistics
    println!("{}", analyzer.statistics.report());

    // Analyze connections
    for conn in analyzer.get_active_connections() {
        let analysis = analyze_connection(conn);
        if !matches!(analysis, ConnectionAnalysis::Normal | ConnectionAnalysis::NormalHttp) {
            println!("Suspicious connection: {:?} - {:?}", conn.key, analysis);
        }
    }
}
```

**Check/Test:**
- Test connection tracking maintains state correctly
- Test TCP state machine transitions
- Test connection cleanup removes old entries
- Test statistics accumulate correctly
- Test pattern matching on connection state
- Test while-let processes packet streams
- Benchmark: throughput with connection tracking

**What this achieves:**
A complete network packet inspector demonstrating:
- **Exhaustive Pattern Matching**: All packet types and states handled
- **Deep Destructuring**: Extract data through protocol layers
- **Range Patterns**: Port and IP filtering
- **Guards**: Complex firewall rules
- **If-Let Chains**: HTTP validation
- **While-Let**: Stream processing
- **Let-Else**: Error handling
- **Matches! Macro**: Quick checks
- **Enum-Driven Architecture**: Protocol representation

**Extensions to explore:**
- IPv6 support
- More protocols (DNS, DHCP, SMTP, FTP)
- PCAP file format reading/writing
- TLS/SSL inspection
- Regex-based payload matching
- Distributed packet capture
- Real-time visualization
- Machine learning for anomaly detection

---
