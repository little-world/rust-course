# Project 2: Network Packet Inspector with Binary Pattern Matching

## Learning Objectives

By completing this project, you will:

- Master **byte slice pattern matching** for binary protocol parsing
- Use **deep destructuring** through nested protocol layers (Ethernet → IP → TCP → HTTP)
- Apply **range patterns** for port and IP address filtering
- Leverage **pattern guards** for complex firewall rules
- Practice **or-patterns** for service detection and protocol variants
- Use **if-let chains** for payload validation
- Apply **exhaustive matching** on protocol enums
- Understand **stateful pattern matching** for connection tracking
- Build a complete packet analyzer demonstrating network security concepts

## Problem Statement

Build a network packet inspector that:
- Parses binary network protocols (Ethernet, IPv4, TCP, UDP, HTTP)
- Uses pattern matching to destructure packet headers from byte arrays
- Implements a firewall rule engine with complex filtering
- Supports deep packet inspection through all protocol layers
- Detects security threats (SQL injection, XSS, suspicious patterns)
- Tracks TCP connection state using pattern matching
- Provides statistics and connection monitoring
- Demonstrates ALL binary pattern matching techniques

## Why It Matters

Network packet analysis is essential for:
- **Security**: Firewalls, intrusion detection systems (IDS), malware analysis
- **Debugging**: Protocol tracing, network troubleshooting
- **Monitoring**: Traffic analysis, bandwidth tracking
- **Compliance**: Data loss prevention, audit logging
- **Performance**: Identifying bottlenecks, optimizing traffic

Pattern matching excels for packet parsing because:
- Protocol headers map naturally to struct destructuring
- Enums represent protocol variants (IPv4/IPv6, TCP/UDP)
- Exhaustive matching ensures all protocol cases are handled
- Range patterns perfect for port/address filtering
- Guards enable sophisticated firewall rules

## Use Cases

1. **Firewalls**: Filter packets by IP address, port, protocol
2. **IDS/IPS**: Detect malicious patterns in network traffic
3. **Packet Capture**: tcpdump/Wireshark-like functionality
4. **Load Balancers**: Route packets based on content
5. **VPN/Proxy**: Inspect and modify network traffic
6. **Network Monitoring**: Track connections, bandwidth usage
7. **Security Research**: Analyze attack patterns and vulnerabilities

---

## Milestone 1: Ethernet Frame Parsing with Byte Slice Patterns

**Goal:** Parse Ethernet layer using byte slice destructuring.

**Concepts:**
- Byte array indexing and slicing
- Enum matching for protocol types
- Big-endian byte order conversion
- Error handling with Result

### Implementation Steps

#### Step 1.1: Define Ethernet Types

```rust
// TODO: Define MAC address wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub fn new(bytes: [u8; 6]) -> Self {
        // TODO: Create MacAddress from byte array
        // Hint: MacAddress(bytes)
        todo!()
    }

    // TODO: Check for broadcast address (FF:FF:FF:FF:FF:FF)
    pub fn is_broadcast(&self) -> bool {
        // Pseudocode:
        // return self.0 == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        todo!()
    }

    // TODO: Check for multicast (first byte's LSB is 1)
    pub fn is_multicast(&self) -> bool {
        // Pseudocode:
        // return (self.0[0] & 0x01) != 0
        todo!()
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Format as XX:XX:XX:XX:XX:XX
        // Pseudocode:
        // write!(f, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        //     self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5])
        todo!()
    }
}

// TODO: Define EtherType enum for protocol identification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtherType {
    // TODO: Add variants
    // Hint: IPv4,        // 0x0800
    // Hint: IPv6,        // 0x86DD
    // Hint: ARP,         // 0x0806
    // Hint: Unknown(u16),
}

impl EtherType {
    // TODO: Parse from 2-byte big-endian value using pattern matching
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        // Pseudocode:
        // value = u16::from_be_bytes(bytes)
        // match value:
        //     0x0800 => EtherType::IPv4
        //     0x86DD => EtherType::IPv6
        //     0x0806 => EtherType::ARP
        //     other => EtherType::Unknown(other)
        todo!()
    }
}

// TODO: Define Ethernet frame structure
#[derive(Debug, Clone)]
pub struct EthernetFrame {
    // TODO: Add fields
    // Hint: pub dst_mac: MacAddress,
    // Hint: pub src_mac: MacAddress,
    // Hint: pub ethertype: EtherType,
    // Hint: pub payload: Vec<u8>,
}
```

#### Step 1.2: Implement Ethernet Parsing

```rust
// TODO: Define parse errors
#[derive(Debug, PartialEq)]
pub enum ParseError {
    // TODO: Add error variants
    // Hint: TooShort { expected: usize, found: usize },
    // Hint: InvalidProtocol(u8),
    // Hint: Malformed(String),
}

impl EthernetFrame {
    // TODO: Parse Ethernet frame from byte slice
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // TODO: Validate minimum length (14 bytes: 6 dst + 6 src + 2 type)
        // Pseudocode:
        // if data.len() < 14:
        //     return Err(ParseError::TooShort { expected: 14, found: data.len() })
        //
        // Extract fields using array indexing:
        // dst_mac = MacAddress([data[0], data[1], data[2], data[3], data[4], data[5]])
        // src_mac = MacAddress([data[6], data[7], data[8], data[9], data[10], data[11]])
        // ethertype = EtherType::from_bytes([data[12], data[13]])
        // payload = data[14..].to_vec()
        //
        // Ok(EthernetFrame { dst_mac, src_mac, ethertype, payload })
        todo!()
    }

    // TODO: Helper to display frame info
    pub fn summary(&self) -> String {
        // Pseudocode:
        // format!("{} -> {} ({:?})", self.src_mac, self.dst_mac, self.ethertype)
        todo!()
    }
}
```

#### Step 1.3: Pattern Matching on EtherType

```rust
// TODO: Classify traffic based on EtherType using exhaustive matching
pub fn classify_ethernet(frame: &EthernetFrame) -> &'static str {
    // Pseudocode:
    // match frame.ethertype:
    //     EtherType::IPv4 => "IPv4 traffic"
    //     EtherType::IPv6 => "IPv6 traffic"
    //     EtherType::ARP => "ARP request/reply"
    //     EtherType::Unknown(_) => "Unknown protocol"
    todo!()
}

// TODO: Check if frame is interesting for analysis
pub fn is_interesting(frame: &EthernetFrame) -> bool {
    // TODO: Use matches! macro for quick checks
    // Pseudocode:
    // matches!(frame.ethertype, EtherType::IPv4 | EtherType::IPv6)
    //     && !frame.dst_mac.is_broadcast()
    todo!()
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_address() {
        let mac = MacAddress([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(mac.is_broadcast());

        let multicast = MacAddress([0x01, 0x00, 0x5E, 0x00, 0x00, 0x01]);
        assert!(multicast.is_multicast());
    }

    #[test]
    fn test_ethernet_parsing() {
        let data = vec![
            // Destination MAC
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
            // Source MAC
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
            // EtherType (IPv4 = 0x0800)
            0x08, 0x00,
            // Payload
            0x45, 0x00, 0x00, 0x3C,
        ];

        let frame = EthernetFrame::parse(&data).unwrap();
        assert_eq!(frame.dst_mac, MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]));
        assert_eq!(frame.src_mac, MacAddress([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]));
        assert_eq!(frame.ethertype, EtherType::IPv4);
        assert_eq!(frame.payload.len(), 4);
    }

    #[test]
    fn test_too_short() {
        let data = vec![0x00, 0x11, 0x22];
        let result = EthernetFrame::parse(&data);
        assert_eq!(result, Err(ParseError::TooShort { expected: 14, found: 3 }));
    }
}
```

### Check Your Understanding

1. Why do we use big-endian byte order for network protocols?
2. How does exhaustive matching on EtherType prevent bugs when adding new protocols?
3. What would happen if we tried to parse a truncated Ethernet frame?
4. How would you extend this to support VLAN tags (802.1Q)?

---

## Milestone 2: IPv4 Parsing with Nested Destructuring

**Goal:** Parse IP layer and demonstrate pattern matching on IP addresses.

**Concepts:**
- Array pattern matching for IP addresses
- Range patterns for IP classification
- Pattern guards for validation
- Nested protocol parsing

### Implementation Steps

#### Step 2.1: Define IPv4 Types

```rust
// TODO: Define IPv4 address wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        // TODO: Create Ipv4Address from octets
        // Hint: Ipv4Address([a, b, c, d])
        todo!()
    }

    // TODO: Check if IP is in private range using pattern matching
    pub fn is_private(&self) -> bool {
        // Pseudocode:
        // match self.0:
        //     [10, _, _, _] => true  // 10.0.0.0/8
        //     [172, b, _, _] if (16..=31).contains(&b) => true  // 172.16.0.0/12
        //     [192, 168, _, _] => true  // 192.168.0.0/16
        //     _ => false
        todo!()
    }

    // TODO: Check for loopback (127.0.0.0/8)
    pub fn is_loopback(&self) -> bool {
        // Pseudocode:
        // matches!(self.0, [127, _, _, _])
        todo!()
    }

    // TODO: Check for multicast (224.0.0.0 to 239.255.255.255)
    pub fn is_multicast(&self) -> bool {
        // Pseudocode:
        // matches!(self.0, [a, _, _, _] if (224..=239).contains(&a))
        todo!()
    }

    // TODO: Check for link-local (169.254.0.0/16)
    pub fn is_link_local(&self) -> bool {
        // Pseudocode:
        // matches!(self.0, [169, 254, _, _])
        todo!()
    }
}

impl std::fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Format as A.B.C.D
        // Pseudocode:
        // write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
        todo!()
    }
}

// TODO: Define IP protocol types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IpProtocol {
    // TODO: Add variants
    // Hint: ICMP,   // 1
    // Hint: TCP,    // 6
    // Hint: UDP,    // 17
    // Hint: Unknown(u8),
}

impl IpProtocol {
    pub fn from_u8(value: u8) -> Self {
        // Pseudocode:
        // match value:
        //     1 => IpProtocol::ICMP
        //     6 => IpProtocol::TCP
        //     17 => IpProtocol::UDP
        //     other => IpProtocol::Unknown(other)
        todo!()
    }
}

// TODO: Define IPv4 packet structure
#[derive(Debug, Clone)]
pub struct Ipv4Packet {
    // TODO: Add fields
    // Hint: pub version: u8,
    // Hint: pub header_length: u8,
    // Hint: pub total_length: u16,
    // Hint: pub ttl: u8,
    // Hint: pub protocol: IpProtocol,
    // Hint: pub src_ip: Ipv4Address,
    // Hint: pub dst_ip: Ipv4Address,
    // Hint: pub payload: Vec<u8>,
}
```

#### Step 2.2: Implement IPv4 Parsing

```rust
impl Ipv4Packet {
    // TODO: Parse IPv4 packet from bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // TODO: Validate minimum header size (20 bytes)
        // Pseudocode:
        // if data.len() < 20:
        //     return Err(ParseError::TooShort { expected: 20, found: data.len() })
        //
        // Extract version and header length from first byte:
        // version_ihl = data[0]
        // version = (version_ihl >> 4) & 0x0F
        // ihl = version_ihl & 0x0F
        // header_length = ihl * 4
        //
        // Validate IPv4 version:
        // if version != 4:
        //     return Err(ParseError::InvalidProtocol(version))
        //
        // Parse remaining fields:
        // total_length = u16::from_be_bytes([data[2], data[3]])
        // ttl = data[8]
        // protocol = IpProtocol::from_u8(data[9])
        // src_ip = Ipv4Address([data[12], data[13], data[14], data[15]])
        // dst_ip = Ipv4Address([data[16], data[17], data[18], data[19]])
        //
        // Extract payload (skip header):
        // header_len = header_length as usize
        // payload = if data.len() > header_len { data[header_len..].to_vec() } else { vec![] }
        //
        // Ok(Ipv4Packet { ... })
        todo!()
    }
}
```

#### Step 2.3: Traffic Classification with Pattern Matching

```rust
// TODO: Define traffic types
#[derive(Debug, PartialEq)]
pub enum TrafficType {
    // TODO: Add variants
    // Hint: LocalPrivate,
    // Hint: Outbound,
    // Hint: Inbound,
    // Hint: Loopback,
    // Hint: Multicast,
    // Hint: InternetRouted,
    // Hint: Other,
}

// TODO: Classify traffic based on IP addresses
pub fn classify_traffic(packet: &Ipv4Packet) -> TrafficType {
    // TODO: Use pattern matching on source and destination properties
    // Pseudocode:
    // match (&packet.src_ip, &packet.dst_ip):
    //     (src, _) if src.is_loopback() => TrafficType::Loopback
    //     (_, dst) if dst.is_loopback() => TrafficType::Loopback
    //     (_, dst) if dst.is_multicast() => TrafficType::Multicast
    //     (src, dst) if src.is_private() && dst.is_private() => TrafficType::LocalPrivate
    //     (src, dst) if src.is_private() && !dst.is_private() => TrafficType::Outbound
    //     (src, dst) if !src.is_private() && dst.is_private() => TrafficType::Inbound
    //     (src, dst) if !src.is_private() && !dst.is_private() => TrafficType::InternetRouted
    //     _ => TrafficType::Other
    todo!()
}
```

#### Step 2.4: Layered Packet Enum

```rust
// TODO: Define layered packet representation
#[derive(Debug, Clone)]
pub enum Packet {
    // TODO: Add variants
    // Hint: Ethernet {
    //     frame: EthernetFrame,
    //     inner: Option<Box<Packet>>,
    // },
    // Hint: IPv4 {
    //     packet: Ipv4Packet,
    //     inner: Option<Box<Packet>>,
    // },
    // Hint: Raw(Vec<u8>),
}

impl Packet {
    // TODO: Parse from Ethernet layer
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // Pseudocode:
        // ethernet = EthernetFrame::parse(data)?
        //
        // Try to parse inner protocol based on EtherType:
        // inner = match ethernet.ethertype:
        //     EtherType::IPv4 =>
        //         match Ipv4Packet::parse(&ethernet.payload):
        //             Ok(ipv4) => Some(Box::new(Packet::IPv4 { packet: ipv4, inner: None }))
        //             Err(_) => None
        //     _ => None
        //
        // Ok(Packet::Ethernet { frame: ethernet, inner })
        todo!()
    }

    // TODO: Extract IP addresses using deep destructuring
    pub fn extract_ips(&self) -> Option<(Ipv4Address, Ipv4Address)> {
        // Pseudocode:
        // match self:
        //     Packet::Ethernet {
        //         inner: Some(box Packet::IPv4 { packet, .. }),
        //         ..
        //     } => Some((packet.src_ip, packet.dst_ip))
        //
        //     Packet::IPv4 { packet, .. } => Some((packet.src_ip, packet.dst_ip))
        //
        //     _ => None
        todo!()
    }
}
```

### Checkpoint Tests

```rust
#[test]
fn test_ipv4_address_classification() {
    let private = Ipv4Address::new(192, 168, 1, 1);
    assert!(private.is_private());

    let public = Ipv4Address::new(8, 8, 8, 8);
    assert!(!public.is_private());

    let loopback = Ipv4Address::new(127, 0, 0, 1);
    assert!(loopback.is_loopback());

    let multicast = Ipv4Address::new(224, 0, 0, 1);
    assert!(multicast.is_multicast());
}

#[test]
fn test_traffic_classification() {
    let local = Ipv4Packet {
        version: 4,
        header_length: 20,
        total_length: 60,
        ttl: 64,
        protocol: IpProtocol::TCP,
        src_ip: Ipv4Address::new(192, 168, 1, 1),
        dst_ip: Ipv4Address::new(192, 168, 1, 2),
        payload: vec![],
    };
    assert_eq!(classify_traffic(&local), TrafficType::LocalPrivate);
}
```

### Check Your Understanding

1. How do array patterns simplify IP address classification?
2. Why is pattern matching on IP ranges safer than manual if-else chains?
3. How does deep destructuring with `box` patterns work for nested packets?
4. What's the advantage of using pattern guards for subnet checking?

---

## Milestone 3: TCP/UDP Parsing and Port Range Matching

**Goal:** Parse transport layer and demonstrate range patterns for port filtering.

**Concepts:**
- Or-patterns for protocol variants
- Range patterns for port classification
- Bit manipulation for TCP flags
- Service detection

### Implementation Steps

#### Step 3.1: Define TCP Types

```rust
// TODO: Define TCP flags structure
#[derive(Debug, Clone, Copy)]
pub struct TcpFlags {
    // TODO: Add flag fields
    // Hint: pub fin: bool,
    // Hint: pub syn: bool,
    // Hint: pub rst: bool,
    // Hint: pub psh: bool,
    // Hint: pub ack: bool,
    // Hint: pub urg: bool,
}

impl TcpFlags {
    // TODO: Parse from byte using bit manipulation
    pub fn from_byte(byte: u8) -> Self {
        // Pseudocode:
        // TcpFlags {
        //     fin: (byte & 0x01) != 0,
        //     syn: (byte & 0x02) != 0,
        //     rst: (byte & 0x04) != 0,
        //     psh: (byte & 0x08) != 0,
        //     ack: (byte & 0x10) != 0,
        //     urg: (byte & 0x20) != 0,
        // }
        todo!()
    }
}

// TODO: Define TCP packet structure
#[derive(Debug, Clone)]
pub struct TcpPacket {
    // TODO: Add fields
    // Hint: pub src_port: u16,
    // Hint: pub dst_port: u16,
    // Hint: pub seq_num: u32,
    // Hint: pub ack_num: u32,
    // Hint: pub flags: TcpFlags,
    // Hint: pub window_size: u16,
    // Hint: pub payload: Vec<u8>,
}

impl TcpPacket {
    // TODO: Parse TCP packet
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // Pseudocode:
        // if data.len() < 20:
        //     return Err(ParseError::TooShort { expected: 20, found: data.len() })
        //
        // src_port = u16::from_be_bytes([data[0], data[1]])
        // dst_port = u16::from_be_bytes([data[2], data[3]])
        // seq_num = u32::from_be_bytes([data[4], data[5], data[6], data[7]])
        // ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]])
        // data_offset = (data[12] >> 4) * 4
        // flags = TcpFlags::from_byte(data[13])
        // window_size = u16::from_be_bytes([data[14], data[15]])
        //
        // header_len = data_offset as usize
        // payload = if data.len() > header_len { data[header_len..].to_vec() } else { vec![] }
        //
        // Ok(TcpPacket { ... })
        todo!()
    }
}
```

#### Step 3.2: Define UDP Types

```rust
// TODO: Define UDP packet structure (simpler than TCP)
#[derive(Debug, Clone)]
pub struct UdpPacket {
    // TODO: Add fields
    // Hint: pub src_port: u16,
    // Hint: pub dst_port: u16,
    // Hint: pub length: u16,
    // Hint: pub payload: Vec<u8>,
}

impl UdpPacket {
    // TODO: Parse UDP packet
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // Pseudocode:
        // if data.len() < 8:
        //     return Err(ParseError::TooShort { expected: 8, found: data.len() })
        //
        // src_port = u16::from_be_bytes([data[0], data[1]])
        // dst_port = u16::from_be_bytes([data[2], data[3]])
        // length = u16::from_be_bytes([data[4], data[5]])
        // payload = data[8..].to_vec()
        //
        // Ok(UdpPacket { ... })
        todo!()
    }
}
```

#### Step 3.3: Port Classification with Range Patterns

```rust
// TODO: Define port classes
#[derive(Debug, PartialEq)]
pub enum PortClass {
    // TODO: Add variants
    // Hint: Reserved,      // 0
    // Hint: WellKnown,     // 1-1023
    // Hint: Registered,    // 1024-49151
    // Hint: Dynamic,       // 49152-65535
}

// TODO: Classify port using range patterns
pub fn classify_port(port: u16) -> PortClass {
    // Pseudocode:
    // match port:
    //     0 => PortClass::Reserved
    //     1..=1023 => PortClass::WellKnown
    //     1024..=49151 => PortClass::Registered
    //     49152..=65535 => PortClass::Dynamic
    todo!()
}

// TODO: Define common services
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Service {
    // TODO: Add variants
    // Hint: Http, Https, SSH, FTP, SMTP, DNS, DHCP, Database, Unknown,
}

// TODO: Detect service using or-patterns
pub fn detect_service(port: u16, protocol: IpProtocol) -> Service {
    // Pseudocode:
    // match (port, protocol):
    //     (80 | 8080 | 8000, IpProtocol::TCP) => Service::Http
    //     (443 | 8443, IpProtocol::TCP) => Service::Https
    //     (22, IpProtocol::TCP) => Service::SSH
    //     (20 | 21, IpProtocol::TCP) => Service::FTP
    //     (25, IpProtocol::TCP) => Service::SMTP
    //     (3306 | 5432 | 1433 | 27017, IpProtocol::TCP) => Service::Database
    //     (53, IpProtocol::UDP | IpProtocol::TCP) => Service::DNS
    //     (67 | 68, IpProtocol::UDP) => Service::DHCP
    //     _ => Service::Unknown
    todo!()
}
```

#### Step 3.4: Update Packet Enum

```rust
// TODO: Add TCP and UDP to packet enum
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
    // TODO: Add TCP(TcpPacket),
    // TODO: Add UDP(UdpPacket),
    Raw(Vec<u8>),
}

// TODO: Helper to check TCP flags using matches! macro
pub fn is_tcp_syn(packet: &Packet) -> bool {
    // Pseudocode:
    // matches!(
    //     packet,
    //     Packet::TCP(TcpPacket { flags: TcpFlags { syn: true, ack: false, .. }, .. })
    // )
    todo!()
}

pub fn is_tcp_syn_ack(packet: &Packet) -> bool {
    // Pseudocode:
    // matches!(
    //     packet,
    //     Packet::TCP(TcpPacket { flags: TcpFlags { syn: true, ack: true, .. }, .. })
    // )
    todo!()
}
```

### Checkpoint Tests

```rust
#[test]
fn test_port_classification() {
    assert_eq!(classify_port(0), PortClass::Reserved);
    assert_eq!(classify_port(80), PortClass::WellKnown);
    assert_eq!(classify_port(8080), PortClass::Registered);
    assert_eq!(classify_port(50000), PortClass::Dynamic);
}

#[test]
fn test_service_detection() {
    assert_eq!(detect_service(80, IpProtocol::TCP), Service::Http);
    assert_eq!(detect_service(443, IpProtocol::TCP), Service::Https);
    assert_eq!(detect_service(22, IpProtocol::TCP), Service::SSH);
    assert_eq!(detect_service(53, IpProtocol::UDP), Service::DNS);
    assert_eq!(detect_service(9999, IpProtocol::TCP), Service::Unknown);
}
```

### Check Your Understanding

1. How do or-patterns simplify service detection across multiple ports?
2. Why are range patterns better than if-else for port classification?
3. How does the `matches!` macro make TCP flag checking more concise?
4. What's the benefit of exhaustive matching when adding new services?

---

## Milestone 4: Firewall Rule Engine with Guards and Complex Patterns

**Goal:** Implement a sophisticated firewall using pattern guards and complex matching.

**Concepts:**
- Pattern guards for multi-criteria matching
- Deep destructuring for rule evaluation
- Exhaustive action matching
- Option handling in patterns

### Implementation Steps

#### Step 4.1: Define Firewall Rules

```rust
// TODO: Define firewall rules
#[derive(Debug, Clone)]
pub enum FirewallRule {
    // TODO: Add rule variants
    // Hint: AllowAll,
    // Hint: DenyAll,
    // Hint: AllowPort { port: u16 },
    // Hint: DenyPort { port: u16 },
    // Hint: AllowPortRange { start: u16, end: u16 },
    // Hint: DenyPortRange { start: u16, end: u16 },
    // Hint: AllowIp { ip: Ipv4Address },
    // Hint: DenyIp { ip: Ipv4Address },
    // Hint: AllowSubnet { network: Ipv4Address, mask: u8 },
    // Hint: DenySubnet { network: Ipv4Address, mask: u8 },
    // Hint: AllowService(Service),
    // Hint: DenyService(Service),
    // Hint: Complex {
    //     src_ip: Option<Ipv4Address>,
    //     dst_ip: Option<Ipv4Address>,
    //     src_port: Option<u16>,
    //     dst_port: Option<u16>,
    //     protocol: Option<IpProtocol>,
    //     action: Action,
    // },
}

// TODO: Define actions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    // TODO: Add variants
    // Hint: Allow, Deny, Log, LogAndAllow, LogAndDeny,
}
```

#### Step 4.2: Implement Firewall Engine

```rust
// TODO: Define firewall structure
#[derive(Debug)]
pub struct Firewall {
    // TODO: Add fields
    // Hint: rules: Vec<FirewallRule>,
    // Hint: default_action: Action,
}

impl Firewall {
    pub fn new(default_action: Action) -> Self {
        // Pseudocode:
        // Firewall { rules: Vec::new(), default_action }
        todo!()
    }

    pub fn add_rule(&mut self, rule: FirewallRule) {
        // Pseudocode:
        // self.rules.push(rule)
        todo!()
    }

    // TODO: Check packet against all rules
    pub fn check_packet(&self, packet: &Packet) -> Action {
        // Pseudocode:
        // for each rule in &self.rules:
        //     if let Some(action) = self.match_rule(rule, packet):
        //         return action
        // return self.default_action
        todo!()
    }

    // TODO: Match a single rule using exhaustive pattern matching
    fn match_rule(&self, rule: &FirewallRule, packet: &Packet) -> Option<Action> {
        // Pseudocode:
        // match (rule, packet):
        //     (FirewallRule::AllowAll, _) => Some(Action::Allow)
        //     (FirewallRule::DenyAll, _) => Some(Action::Deny)
        //
        //     Port-based rules with exhaustive protocol matching:
        //     (FirewallRule::AllowPort { port }, Packet::TCP(tcp))
        //         if tcp.src_port == *port || tcp.dst_port == *port =>
        //         Some(Action::Allow)
        //
        //     (FirewallRule::AllowPort { port }, Packet::UDP(udp))
        //         if udp.src_port == *port || udp.dst_port == *port =>
        //         Some(Action::Allow)
        //
        //     Port range with pattern guards:
        //     (FirewallRule::AllowPortRange { start, end },
        //      Packet::TCP(TcpPacket { dst_port, .. }))
        //         if (*start..=*end).contains(dst_port) => Some(Action::Allow)
        //
        //     Deep destructuring for nested packets with guards:
        //     (FirewallRule::AllowIp { ip },
        //      Packet::Ethernet {
        //          inner: Some(box Packet::IPv4 { packet, .. }),
        //          ..
        //      })
        //         if packet.src_ip == *ip || packet.dst_ip == *ip => Some(Action::Allow)
        //
        //     Subnet matching with guards:
        //     (FirewallRule::AllowSubnet { network, mask },
        //      Packet::IPv4 { packet, .. })
        //         if Self::in_subnet(&packet.dst_ip, network, *mask) => Some(Action::Allow)
        //
        //     Service-based rules:
        //     (FirewallRule::AllowService(service), packet) => {
        //         let detected = Self::detect_service_from_packet(packet)
        //         if detected == *service { Some(Action::Allow) } else { None }
        //     }
        //
        //     Complex rule with multiple criteria:
        //     (FirewallRule::Complex { src_ip, dst_ip, src_port, dst_port, protocol, action }, packet) => {
        //         Extract packet info
        //         Check all criteria using Option::map and unwrap_or
        //         Return action if all match
        //     }
        //
        //     _ => None (no match)
        todo!()
    }

    // TODO: Helper for subnet matching
    fn in_subnet(ip: &Ipv4Address, network: &Ipv4Address, mask: u8) -> bool {
        // Pseudocode:
        // ip_bits = u32::from_be_bytes(ip.0)
        // net_bits = u32::from_be_bytes(network.0)
        // mask_bits = !0u32 << (32 - mask)
        // (ip_bits & mask_bits) == (net_bits & mask_bits)
        todo!()
    }

    // TODO: Detect service from packet
    fn detect_service_from_packet(packet: &Packet) -> Service {
        // Pseudocode:
        // match packet:
        //     Packet::TCP(tcp) => detect_service(tcp.dst_port, IpProtocol::TCP)
        //     Packet::UDP(udp) => detect_service(udp.dst_port, IpProtocol::UDP)
        //     _ => Service::Unknown
        todo!()
    }
}
```

#### Step 4.3: Packet Info Extractor

```rust
// TODO: Helper to extract packet info for complex rules
#[derive(Debug)]
struct PacketInfo {
    // TODO: Add fields
    // Hint: src_ip: Option<Ipv4Address>,
    // Hint: dst_ip: Option<Ipv4Address>,
    // Hint: src_port: Option<u16>,
    // Hint: dst_port: Option<u16>,
    // Hint: protocol: Option<IpProtocol>,
}

impl PacketInfo {
    // TODO: Extract info using deep destructuring
    fn extract(packet: &Packet) -> Option<Self> {
        // Pseudocode:
        // match packet:
        //     Full Ethernet -> IPv4 -> TCP stack:
        //     Packet::Ethernet {
        //         inner: Some(box Packet::IPv4 {
        //             packet: ipv4,
        //             inner: Some(box Packet::TCP(tcp)),
        //         }),
        //         ..
        //     } => Some(PacketInfo {
        //         src_ip: Some(ipv4.src_ip),
        //         dst_ip: Some(ipv4.dst_ip),
        //         src_port: Some(tcp.src_port),
        //         dst_port: Some(tcp.dst_port),
        //         protocol: Some(IpProtocol::TCP),
        //     })
        //
        //     Ethernet -> IPv4 -> UDP:
        //     (similar pattern for UDP)
        //
        //     Just IPv4:
        //     Packet::IPv4 { packet, .. } => Some(PacketInfo with IP info only)
        //
        //     Just TCP:
        //     Packet::TCP(tcp) => Some(PacketInfo with port info only)
        //
        //     _ => None
        todo!()
    }
}
```

### Checkpoint Tests

```rust
#[test]
fn test_firewall_allow_port() {
    let mut firewall = Firewall::new(Action::Deny);
    firewall.add_rule(FirewallRule::AllowPort { port: 80 });

    let tcp_packet = Packet::TCP(TcpPacket {
        src_port: 1234,
        dst_port: 80,
        seq_num: 0,
        ack_num: 0,
        flags: TcpFlags::from_byte(0x02),
        window_size: 8192,
        payload: vec![],
    });

    assert_eq!(firewall.check_packet(&tcp_packet), Action::Allow);
}

#[test]
fn test_subnet_matching() {
    let ip = Ipv4Address::new(192, 168, 1, 100);
    let network = Ipv4Address::new(192, 168, 1, 0);

    assert!(Firewall::in_subnet(&ip, &network, 24));
    assert!(!Firewall::in_subnet(&ip, &network, 32));
}
```

### Check Your Understanding

1. How do pattern guards enable multi-criteria firewall rules?
2. Why is deep destructuring useful for extracting packet info through layers?
3. How does exhaustive matching prevent firewall configuration bugs?
4. What's the advantage of Option handling in complex rule matching?

---

## Milestone 5: Connection Tracking, Statistics, and Deep Packet Inspection

**Goal:** Add stateful inspection, HTTP parsing, and comprehensive pattern matching.

**Concepts:**
- State machines with pattern matching
- While-let for stream processing
- Let-else for error handling
- Complex nested matching for threat detection

### Implementation Steps

#### Step 5.1: Connection Tracking

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

// TODO: Define connection key for tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionKey {
    // TODO: Add fields
    // Hint: pub src_ip: Ipv4Address,
    // Hint: pub dst_ip: Ipv4Address,
    // Hint: pub src_port: u16,
    // Hint: pub dst_port: u16,
    // Hint: pub protocol: IpProtocol,
}

impl ConnectionKey {
    // TODO: Create canonical key (bidirectional)
    fn canonical(&self) -> Self {
        // Pseudocode:
        // if self.src_ip.0 < self.dst_ip.0 || (self.src_ip == self.dst_ip && self.src_port < self.dst_port):
        //     return *self
        // else:
        //     return ConnectionKey with swapped src/dst
        todo!()
    }
}

// TODO: Define TCP connection states
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    // TODO: Add variants
    // Hint: TcpSynSent, TcpSynReceived, TcpEstablished, TcpFinWait, TcpClosed,
    // Hint: UdpActive, Unknown,
}

// TODO: Connection tracking structure
#[derive(Debug, Clone)]
pub struct Connection {
    // TODO: Add fields
    // Hint: pub key: ConnectionKey,
    // Hint: pub state: ConnectionState,
    // Hint: pub packets: usize,
    // Hint: pub bytes: usize,
    // Hint: pub start_time: Instant,
    // Hint: pub last_seen: Instant,
}

// TODO: Packet analyzer with connection tracking
pub struct PacketAnalyzer {
    // TODO: Add fields
    // Hint: connections: HashMap<ConnectionKey, Connection>,
    // Hint: statistics: Statistics,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        // Pseudocode:
        // PacketAnalyzer {
        //     connections: HashMap::new(),
        //     statistics: Statistics::default(),
        // }
        todo!()
    }

    // TODO: Process packet and update state
    pub fn process_packet(&mut self, packet: &Packet) {
        // Pseudocode:
        // self.statistics.total_packets += 1
        // self.update_statistics(packet)
        // if let Some(key) = self.extract_connection_key(packet):
        //     self.track_connection(key.canonical(), packet)
        todo!()
    }

    // TODO: Track connection state using pattern matching
    fn track_connection(&mut self, key: ConnectionKey, packet: &Packet) {
        // Pseudocode:
        // Get or create connection entry
        // conn.packets += 1
        // conn.last_seen = Instant::now()
        // self.update_connection_state(conn, packet)
        todo!()
    }

    // TODO: TCP state machine using exhaustive pattern matching
    fn update_connection_state(&mut self, conn: &mut Connection, packet: &Packet) {
        // Pseudocode:
        // match (packet, &conn.state):
        //     (Packet::TCP(TcpPacket { flags: TcpFlags { syn: true, ack: false, .. }, .. }),
        //      ConnectionState::Unknown) => {
        //         conn.state = ConnectionState::TcpSynSent
        //     }
        //
        //     (Packet::TCP(TcpPacket { flags: TcpFlags { syn: true, ack: true, .. }, .. }),
        //      ConnectionState::TcpSynSent) => {
        //         conn.state = ConnectionState::TcpSynReceived
        //     }
        //
        //     (Packet::TCP(TcpPacket { flags: TcpFlags { ack: true, syn: false, fin: false, .. }, .. }),
        //      ConnectionState::TcpSynReceived) => {
        //         conn.state = ConnectionState::TcpEstablished
        //     }
        //
        //     (Packet::TCP(TcpPacket { flags: TcpFlags { fin: true, .. }, .. }),
        //      ConnectionState::TcpEstablished) => {
        //         conn.state = ConnectionState::TcpFinWait
        //     }
        //
        //     (Packet::TCP(TcpPacket { flags: TcpFlags { rst: true, .. }, .. }), _) => {
        //         conn.state = ConnectionState::TcpClosed
        //     }
        //
        //     (Packet::UDP(_), _) => {
        //         conn.state = ConnectionState::UdpActive
        //     }
        //
        //     _ => {}
        todo!()
    }

    // TODO: Extract connection key using let-else
    fn extract_connection_key(&self, packet: &Packet) -> Option<ConnectionKey> {
        // Pseudocode:
        // info = PacketInfo::extract(packet)?
        // Some(ConnectionKey {
        //     src_ip: info.src_ip?,
        //     dst_ip: info.dst_ip?,
        //     src_port: info.src_port?,
        //     dst_port: info.dst_port?,
        //     protocol: info.protocol?,
        // })
        todo!()
    }

    // TODO: Get active connections using pattern guards
    pub fn get_active_connections(&self) -> Vec<&Connection> {
        // Pseudocode:
        // now = Instant::now()
        // self.connections.values()
        //     .filter(|conn| {
        //         now.duration_since(conn.last_seen) < Duration::from_secs(60)
        //             && !matches!(conn.state, ConnectionState::TcpClosed)
        //     })
        //     .collect()
        todo!()
    }

    // TODO: Cleanup old connections
    pub fn cleanup_old_connections(&mut self, max_age: Duration) {
        // Pseudocode:
        // now = Instant::now()
        // self.connections.retain(|_, conn| {
        //     now.duration_since(conn.last_seen) < max_age
        //         || !matches!(conn.state, ConnectionState::TcpClosed)
        // })
        todo!()
    }
}

// TODO: Statistics structure
#[derive(Debug, Default)]
pub struct Statistics {
    // TODO: Add fields
    // Hint: pub total_packets: usize,
    // Hint: pub ethernet_packets: usize,
    // Hint: pub ipv4_packets: usize,
    // Hint: pub tcp_packets: usize,
    // Hint: pub udp_packets: usize,
    // Hint: pub icmp_packets: usize,
    // Hint: pub raw_packets: usize,
}
```

#### Step 5.2: Connection Analysis with Pattern Matching

```rust
// TODO: Analyze connection for suspicious behavior
#[derive(Debug, PartialEq)]
pub enum ConnectionAnalysis {
    // TODO: Add variants
    // Hint: Normal, SynFlood, PossiblePortScan, LongLived, UdpQuery, Closed,
}

// TODO: Analyze using exhaustive pattern matching with guards
pub fn analyze_connection(conn: &Connection) -> ConnectionAnalysis {
    // Pseudocode:
    // match (&conn.state, conn.packets):
    //     (ConnectionState::TcpSynSent, p) if p > 100 => ConnectionAnalysis::SynFlood
    //     (ConnectionState::TcpSynSent, p) if p < 5 => ConnectionAnalysis::PossiblePortScan
    //     (ConnectionState::TcpEstablished, p) if p > 10000 => ConnectionAnalysis::LongLived
    //     (ConnectionState::UdpActive, p) if p < 3 => ConnectionAnalysis::UdpQuery
    //     (ConnectionState::TcpClosed, _) => ConnectionAnalysis::Closed
    //     _ => ConnectionAnalysis::Normal
    todo!()
}
```

#### Step 5.3: Stream Processing with While-Let

```rust
// TODO: Process packet stream using while-let
pub fn process_packet_stream<I>(analyzer: &mut PacketAnalyzer, mut packets: I)
where
    I: Iterator<Item = Packet>,
{
    // Pseudocode:
    // while let Some(packet) = packets.next():
    //     analyzer.process_packet(&packet)
    //     if analyzer.statistics.total_packets % 1000 == 0:
    //         analyzer.cleanup_old_connections(Duration::from_secs(300))
    todo!()
}
```

### Checkpoint Tests

```rust
#[test]
fn test_connection_tracking() {
    let mut analyzer = PacketAnalyzer::new();

    let syn = Packet::TCP(TcpPacket {
        src_port: 1234,
        dst_port: 80,
        seq_num: 100,
        ack_num: 0,
        flags: TcpFlags {
            syn: true,
            ack: false,
            fin: false,
            rst: false,
            psh: false,
            urg: false,
        },
        window_size: 8192,
        payload: vec![],
    });

    analyzer.process_packet(&syn);

    let active = analyzer.get_active_connections();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].state, ConnectionState::TcpSynSent);
}

#[test]
fn test_connection_analysis() {
    let syn_flood = Connection {
        key: ConnectionKey {
            src_ip: Ipv4Address::new(1, 1, 1, 1),
            dst_ip: Ipv4Address::new(2, 2, 2, 2),
            src_port: 1234,
            dst_port: 80,
            protocol: IpProtocol::TCP,
        },
        state: ConnectionState::TcpSynSent,
        packets: 150,
        bytes: 0,
        start_time: Instant::now(),
        last_seen: Instant::now(),
    };

    assert_eq!(analyze_connection(&syn_flood), ConnectionAnalysis::SynFlood);
}
```

### Check Your Understanding

1. How does pattern matching simplify TCP state machine implementation?
2. Why is while-let useful for stream processing?
3. How do pattern guards help identify suspicious connections?
4. What's the benefit of exhaustive matching in connection analysis?

---

## Complete Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_packet_analysis() {
        let mut analyzer = PacketAnalyzer::new();
        let mut firewall = Firewall::new(Action::Allow);

        // Block telnet
        firewall.add_rule(FirewallRule::DenyPort { port: 23 });

        // Allow HTTP/HTTPS
        firewall.add_rule(FirewallRule::AllowPortRange {
            start: 80,
            end: 443,
        });

        // Test with various packets...
    }

    #[test]
    fn test_port_scan_detection() {
        let mut analyzer = PacketAnalyzer::new();

        // Simulate port scan - many SYN packets to different ports
        for port in 1..=100 {
            let syn = Packet::TCP(TcpPacket {
                src_port: 54321,
                dst_port: port,
                seq_num: port as u32,
                ack_num: 0,
                flags: TcpFlags::from_byte(0x02),
                window_size: 8192,
                payload: vec![],
            });

            analyzer.process_packet(&syn);
        }

        // Should have many connections in SYN_SENT state
        let active = analyzer.get_active_connections();
        let syn_sent_count = active
            .iter()
            .filter(|c| c.state == ConnectionState::TcpSynSent)
            .count();

        assert!(syn_sent_count > 50);
    }
}
```

## Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_packet_parsing() {
        let ethernet_data = vec![
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // dst
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, // src
            0x08, 0x00, // IPv4
            // Minimal IPv4 header
            0x45, 0x00, 0x00, 0x3C, 0x1C, 0x46, 0x40, 0x00,
            0x40, 0x06, 0xB1, 0xE6, 0xC0, 0xA8, 0x01, 0x01,
            0x08, 0x08, 0x08, 0x08,
        ];

        let start = Instant::now();
        for _ in 0..100_000 {
            let _ = Packet::parse(&ethernet_data);
        }
        let elapsed = start.elapsed();
        println!("Packet parsing: {:?}", elapsed);
    }

    #[test]
    fn bench_firewall_rules() {
        let mut firewall = Firewall::new(Action::Deny);

        // Add 100 rules
        for port in 1..=100 {
            firewall.add_rule(FirewallRule::AllowPort { port });
        }

        let packet = Packet::TCP(TcpPacket {
            src_port: 1234,
            dst_port: 80,
            seq_num: 0,
            ack_num: 0,
            flags: TcpFlags::from_byte(0x02),
            window_size: 8192,
            payload: vec![],
        });

        let start = Instant::now();
        for _ in 0..100_000 {
            firewall.check_packet(&packet);
        }
        let elapsed = start.elapsed();
        println!("Firewall evaluation: {:?}", elapsed);
    }
}
```

## Extensions and Challenges

1. **HTTP Parsing**: Add HTTP request/response parsing and threat detection (SQL injection, XSS)
2. **IPv6 Support**: Extend to support IPv6 addresses and protocols
3. **PCAP Files**: Read/write packet capture files in PCAP format
4. **More Protocols**: Add DNS, DHCP, SMTP, FTP parsing
5. **TLS Inspection**: Inspect encrypted traffic metadata
6. **Distributed Capture**: Aggregate packets from multiple sources
7. **Real-time Visualization**: Create a terminal UI for live packet analysis
8. **Machine Learning**: Detect anomalies using statistical models

## Pattern Matching Features Demonstrated

✅ **Byte Slice Patterns**: Parsing binary protocol headers
✅ **Deep Destructuring**: Extracting data through Ethernet → IP → TCP layers
✅ **Range Patterns**: Port classification (1..=1023)
✅ **Pattern Guards**: Complex firewall rules and validation
✅ **Or-Patterns**: Service detection across multiple ports (80 | 8080 | 8000)
✅ **Exhaustive Matching**: All protocol variants handled
✅ **Matches! Macro**: Quick TCP flag checks
✅ **If-Let Chains**: Optional field handling in complex rules
✅ **While-Let**: Stream processing
✅ **Let-Else**: Error handling in extraction functions
✅ **Array Patterns**: IP address classification ([10, _, _, _])
✅ **Tuple Matching**: Protocol and port combinations

## Real-World Applications

- **Network Security**: Firewalls, IDS/IPS systems
- **Traffic Analysis**: Bandwidth monitoring, QoS enforcement
- **Protocol Testing**: Validate protocol implementations
- **Forensics**: Analyze network captures for security incidents
- **Performance Optimization**: Identify network bottlenecks
- **Compliance**: Monitor for policy violations

This project demonstrates how pattern matching naturally expresses network protocol analysis, making code both safe (exhaustive checking) and maintainable (clear structure).
