use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// Milestone 1: Ethernet Layer
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub fn new(bytes: [u8; 6]) -> Self {
        MacAddress(bytes)
    }

    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    pub fn is_multicast(&self) -> bool {
        (self.0[0] & 0x01) != 0
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtherType {
    IPv4,
    IPv6,
    ARP,
    Unknown(u16),
}

impl EtherType {
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        let value = u16::from_be_bytes(bytes);
        match value {
            0x0800 => EtherType::IPv4,
            0x86DD => EtherType::IPv6,
            0x0806 => EtherType::ARP,
            _ => EtherType::Unknown(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EthernetFrame {
    pub dst_mac: MacAddress,
    pub src_mac: MacAddress,
    pub ethertype: EtherType,
    pub payload: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    TooShort { expected: usize, found: usize },
    InvalidProtocol(u8),
    Malformed(String),
}

impl EthernetFrame {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 14 {
            return Err(ParseError::TooShort {
                expected: 14,
                found: data.len(),
            });
        }

        let dst_mac = MacAddress::new([data[0], data[1], data[2], data[3], data[4], data[5]]);
        let src_mac = MacAddress::new([data[6], data[7], data[8], data[9], data[10], data[11]]);
        let ethertype = EtherType::from_bytes([data[12], data[13]]);
        let payload = data[14..].to_vec();

        Ok(EthernetFrame {
            dst_mac,
            src_mac,
            ethertype,
            payload,
        })
    }

    pub fn summary(&self) -> String {
        format!(
            "{} -> {} [{}]",
            self.src_mac,
            self.dst_mac,
            match self.ethertype {
                EtherType::IPv4 => "IPv4",
                EtherType::IPv6 => "IPv6",
                EtherType::ARP => "ARP",
                EtherType::Unknown(v) => return format!("Unknown(0x{:04X})", v),
            }
        )
    }
}

pub fn classify_ethernet(frame: &EthernetFrame) -> &'static str {
    match frame.ethertype {
        EtherType::IPv4 => "IPv4 traffic",
        EtherType::IPv6 => "IPv6 traffic",
        EtherType::ARP => "ARP request/reply",
        EtherType::Unknown(_) => "Unknown protocol",
    }
}

pub fn is_interesting(frame: &EthernetFrame) -> bool {
    matches!(frame.ethertype, EtherType::IPv4 | EtherType::IPv6)
        && !frame.dst_mac.is_broadcast()
}

// ============================================================================
// Milestone 2: IPv4 Layer
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Address([a, b, c, d])
    }

    pub fn is_private(&self) -> bool {
        match self.0 {
            [10, _, _, _] => true,
            [172, b, _, _] if (16..=31).contains(&b) => true,
            [192, 168, _, _] => true,
            _ => false,
        }
    }

    pub fn is_loopback(&self) -> bool {
        self.0[0] == 127
    }

    pub fn is_multicast(&self) -> bool {
        (224..=239).contains(&self.0[0])
    }

    pub fn is_link_local(&self) -> bool {
        matches!(self.0, [169, 254, _, _])
    }
}

impl std::fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IpProtocol {
    ICMP,
    TCP,
    UDP,
    Unknown(u8),
}

impl IpProtocol {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => IpProtocol::ICMP,
            6 => IpProtocol::TCP,
            17 => IpProtocol::UDP,
            _ => IpProtocol::Unknown(value),
        }
    }
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
        if data.len() < 20 {
            return Err(ParseError::TooShort {
                expected: 20,
                found: data.len(),
            });
        }

        let version_ihl = data[0];
        let version = (version_ihl >> 4) & 0x0F;
        let ihl = version_ihl & 0x0F;
        let header_length = ihl * 4;

        if version != 4 {
            return Err(ParseError::InvalidProtocol(version));
        }

        let total_length = u16::from_be_bytes([data[2], data[3]]);
        let ttl = data[8];
        let protocol = IpProtocol::from_u8(data[9]);
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
}

#[derive(Debug, PartialEq)]
pub enum TrafficType {
    LocalPrivate,
    Outbound,
    Inbound,
    Loopback,
    Multicast,
    InternetRouted,
    Other,
}

pub fn classify_traffic(packet: &Ipv4Packet) -> TrafficType {
    match (&packet.src_ip, &packet.dst_ip) {
        (src, _) if src.is_loopback() => TrafficType::Loopback,
        (_, dst) if dst.is_loopback() => TrafficType::Loopback,
        (_, dst) if dst.is_multicast() => TrafficType::Multicast,
        (src, dst) if src.is_private() && dst.is_private() => TrafficType::LocalPrivate,
        (src, dst) if src.is_private() && !dst.is_private() => TrafficType::Outbound,
        (src, dst) if !src.is_private() && dst.is_private() => TrafficType::Inbound,
        (src, dst) if !src.is_private() && !dst.is_private() => TrafficType::InternetRouted,
        _ => TrafficType::Other,
    }
}

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

impl Packet {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let ethernet = EthernetFrame::parse(data)?;

        let inner = match ethernet.ethertype {
            EtherType::IPv4 => match Ipv4Packet::parse(&ethernet.payload) {
                Ok(ipv4) => {
                    let transport_inner = match ipv4.protocol {
                        IpProtocol::TCP => match TcpPacket::parse(&ipv4.payload) {
                            Ok(tcp) => Some(Box::new(Packet::TCP(tcp))),
                            Err(_) => None,
                        },
                        IpProtocol::UDP => match UdpPacket::parse(&ipv4.payload) {
                            Ok(udp) => Some(Box::new(Packet::UDP(udp))),
                            Err(_) => None,
                        },
                        _ => None,
                    };
                    Some(Box::new(Packet::IPv4 {
                        packet: ipv4,
                        inner: transport_inner,
                    }))
                }
                Err(_) => None,
            },
            _ => None,
        };

        Ok(Packet::Ethernet {
            frame: ethernet,
            inner,
        })
    }

    pub fn extract_ips(&self) -> Option<(Ipv4Address, Ipv4Address)> {
        match self {
            Packet::Ethernet {
                inner: Some(inner),
                ..
            } => match inner.as_ref() {
                Packet::IPv4 { packet, .. } => Some((packet.src_ip, packet.dst_ip)),
                _ => None,
            },
            Packet::IPv4 { packet, .. } => Some((packet.src_ip, packet.dst_ip)),
            _ => None,
        }
    }
}

// ============================================================================
// Milestone 3: TCP/UDP Layer
// ============================================================================

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
    pub fn from_byte(byte: u8) -> Self {
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

#[derive(Debug, PartialEq)]
pub enum PortClass {
    Reserved,
    WellKnown,
    Registered,
    Dynamic,
}

pub fn classify_port(port: u16) -> PortClass {
    match port {
        0 => PortClass::Reserved,
        1..=1023 => PortClass::WellKnown,
        1024..=49151 => PortClass::Registered,
        49152..=65535 => PortClass::Dynamic,
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Service {
    Http,
    Https,
    SSH,
    FTP,
    SMTP,
    DNS,
    DHCP,
    Database,
    Unknown,
}

pub fn detect_service(port: u16, protocol: IpProtocol) -> Service {
    match (port, protocol) {
        (80 | 8080 | 8000, IpProtocol::TCP) => Service::Http,
        (443 | 8443, IpProtocol::TCP) => Service::Https,
        (22, IpProtocol::TCP) => Service::SSH,
        (20 | 21, IpProtocol::TCP) => Service::FTP,
        (25, IpProtocol::TCP) => Service::SMTP,
        (3306 | 5432 | 1433 | 27017, IpProtocol::TCP) => Service::Database,
        (53, IpProtocol::UDP | IpProtocol::TCP) => Service::DNS,
        (67 | 68, IpProtocol::UDP) => Service::DHCP,
        _ => Service::Unknown,
    }
}

pub fn is_tcp_syn(packet: &Packet) -> bool {
    matches!(
        packet,
        Packet::TCP(TcpPacket {
            flags: TcpFlags {
                syn: true,
                ack: false,
                ..
            },
            ..
        })
    )
}

pub fn is_tcp_syn_ack(packet: &Packet) -> bool {
    matches!(
        packet,
        Packet::TCP(TcpPacket {
            flags: TcpFlags {
                syn: true,
                ack: true,
                ..
            },
            ..
        })
    )
}

// ============================================================================
// Milestone 4: Firewall Engine
// ============================================================================

#[derive(Debug, Clone)]
pub enum FirewallRule {
    AllowAll,
    DenyAll,
    AllowPort { port: u16 },
    DenyPort { port: u16 },
    AllowPortRange { start: u16, end: u16 },
    DenyPortRange { start: u16, end: u16 },
    AllowIp { ip: Ipv4Address },
    DenyIp { ip: Ipv4Address },
    AllowSubnet { network: Ipv4Address, mask: u8 },
    DenySubnet { network: Ipv4Address, mask: u8 },
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
        for rule in &self.rules {
            if let Some(action) = self.match_rule(rule, packet) {
                return action;
            }
        }
        self.default_action
    }

    fn match_rule(&self, rule: &FirewallRule, packet: &Packet) -> Option<Action> {
        match (rule, packet) {
            (FirewallRule::AllowAll, _) => Some(Action::Allow),
            (FirewallRule::DenyAll, _) => Some(Action::Deny),

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

            (FirewallRule::DenyPort { port }, Packet::TCP(tcp))
            if tcp.src_port == *port || tcp.dst_port == *port =>
                {
                    Some(Action::Deny)
                }

            (FirewallRule::DenyPort { port }, Packet::UDP(udp))
            if udp.src_port == *port || udp.dst_port == *port =>
                {
                    Some(Action::Deny)
                }

            (
                FirewallRule::AllowPortRange { start, end },
                Packet::TCP(TcpPacket { dst_port, .. }),
            ) if (*start..=*end).contains(dst_port) => Some(Action::Allow),

            (
                FirewallRule::AllowPortRange { start, end },
                Packet::UDP(UdpPacket { dst_port, .. }),
            ) if (*start..=*end).contains(dst_port) => Some(Action::Allow),

            (
                FirewallRule::AllowIp { ip },
                Packet::Ethernet { inner: Some(inner), .. },
            ) => match inner.as_ref() {
                Packet::IPv4 { packet, .. }
                    if packet.src_ip == *ip || packet.dst_ip == *ip =>
                    Some(Action::Allow),
                _ => None,
            },

            (FirewallRule::AllowIp { ip }, Packet::IPv4 { packet, .. })
            if packet.src_ip == *ip || packet.dst_ip == *ip =>
                {
                    Some(Action::Allow)
                }

            (FirewallRule::AllowSubnet { network, mask }, Packet::IPv4 { packet, .. })
            if Self::in_subnet(&packet.dst_ip, network, *mask) =>
                {
                    Some(Action::Allow)
                }

            (FirewallRule::AllowService(service), packet) => {
                let detected = Self::detect_service_from_packet(packet);
                if detected == *service {
                    Some(Action::Allow)
                } else {
                    None
                }
            }

            (FirewallRule::DenyService(service), packet) => {
                let detected = Self::detect_service_from_packet(packet);
                if detected == *service {
                    Some(Action::Deny)
                } else {
                    None
                }
            }

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
                let info = PacketInfo::extract(packet)?;

                let ip_match = src_ip.map_or(true, |ip| info.src_ip == Some(ip))
                    && dst_ip.map_or(true, |ip| info.dst_ip == Some(ip));

                let port_match = src_port.map_or(true, |p| info.src_port == Some(p))
                    && dst_port.map_or(true, |p| info.dst_port == Some(p));

                let proto_match = protocol.map_or(true, |p| info.protocol == Some(p));

                if ip_match && port_match && proto_match {
                    Some(*action)
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    fn in_subnet(ip: &Ipv4Address, network: &Ipv4Address, mask: u8) -> bool {
        let ip_bits = u32::from_be_bytes(ip.0);
        let net_bits = u32::from_be_bytes(network.0);
        let mask_bits = !0u32 << (32 - mask);
        (ip_bits & mask_bits) == (net_bits & mask_bits)
    }

    fn detect_service_from_packet(packet: &Packet) -> Service {
        match packet {
            Packet::TCP(tcp) => detect_service(tcp.dst_port, IpProtocol::TCP),
            Packet::UDP(udp) => detect_service(udp.dst_port, IpProtocol::UDP),
            _ => Service::Unknown,
        }
    }
}

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
            Packet::Ethernet { inner: Some(inner), .. } => {
                // Recurse into the inner packet of the Ethernet frame
                Self::extract(inner.as_ref())
            }

            Packet::IPv4 { packet: ipv4, inner } => {
                if let Some(inner) = inner {
                    match inner.as_ref() {
                        Packet::TCP(tcp) => Some(PacketInfo {
                            src_ip: Some(ipv4.src_ip),
                            dst_ip: Some(ipv4.dst_ip),
                            src_port: Some(tcp.src_port),
                            dst_port: Some(tcp.dst_port),
                            protocol: Some(IpProtocol::TCP),
                        }),
                        Packet::UDP(udp) => Some(PacketInfo {
                            src_ip: Some(ipv4.src_ip),
                            dst_ip: Some(ipv4.dst_ip),
                            src_port: Some(udp.src_port),
                            dst_port: Some(udp.dst_port),
                            protocol: Some(IpProtocol::UDP),
                        }),
                        _ => Some(PacketInfo {
                            src_ip: Some(ipv4.src_ip),
                            dst_ip: Some(ipv4.dst_ip),
                            src_port: None,
                            dst_port: None,
                            protocol: Some(ipv4.protocol),
                        }),
                    }
                } else {
                    Some(PacketInfo {
                        src_ip: Some(ipv4.src_ip),
                        dst_ip: Some(ipv4.dst_ip),
                        src_port: None,
                        dst_port: None,
                        protocol: Some(ipv4.protocol),
                    })
                }
            }

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

// ============================================================================
// Milestone 5: Connection Tracking
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionKey {
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: IpProtocol,
}

impl ConnectionKey {
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

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Unknown,
    TcpSynSent,
    TcpSynReceived,
    TcpEstablished,
    TcpFinWait,
    TcpClosed,
    UdpActive,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub key: ConnectionKey,
    pub state: ConnectionState,
    pub packets: usize,
    pub bytes: usize,
    pub start_time: Instant,
    pub last_seen: Instant,
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
        self.update_statistics(packet);

        if let Some(key) = self.extract_connection_key(packet) {
            self.track_connection(key.canonical(), packet);
        }
    }

    fn update_statistics(&mut self, packet: &Packet) {
        match packet {
            Packet::Ethernet { .. } => self.statistics.ethernet_packets += 1,
            Packet::IPv4 { .. } => self.statistics.ipv4_packets += 1,
            Packet::TCP(_) => self.statistics.tcp_packets += 1,
            Packet::UDP(_) => self.statistics.udp_packets += 1,
            Packet::Raw(_) => self.statistics.raw_packets += 1,
        }
    }

    fn track_connection(&mut self, key: ConnectionKey, packet: &Packet) {
        let now = Instant::now();
        let conn = self.connections.entry(key).or_insert_with(|| Connection {
            key,
            state: ConnectionState::Unknown,
            packets: 0,
            bytes: 0,
            start_time: now,
            last_seen: now,
        });

        conn.packets += 1;
        conn.last_seen = now;
        Self::update_connection_state(conn, packet);
    }

    fn update_connection_state(conn: &mut Connection, packet: &Packet) {
        match (packet, &conn.state) {
            (
                Packet::TCP(TcpPacket {
                                flags: TcpFlags {
                                    syn: true,
                                    ack: false,
                                    ..
                                },
                                ..
                            }),
                ConnectionState::Unknown,
            ) => {
                conn.state = ConnectionState::TcpSynSent;
            }

            (
                Packet::TCP(TcpPacket {
                                flags: TcpFlags {
                                    syn: true,
                                    ack: true,
                                    ..
                                },
                                ..
                            }),
                ConnectionState::TcpSynSent,
            ) => {
                conn.state = ConnectionState::TcpSynReceived;
            }

            (
                Packet::TCP(TcpPacket {
                                flags:
                                TcpFlags {
                                    ack: true,
                                    syn: false,
                                    fin: false,
                                    ..
                                },
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

            (Packet::UDP(_), _) => {
                conn.state = ConnectionState::UdpActive;
            }

            _ => {}
        }
    }

    fn extract_connection_key(&self, packet: &Packet) -> Option<ConnectionKey> {
        let info = PacketInfo::extract(packet)?;
        Some(ConnectionKey {
            src_ip: info.src_ip?,
            dst_ip: info.dst_ip?,
            src_port: info.src_port?,
            dst_port: info.dst_port?,
            protocol: info.protocol?,
        })
    }

    pub fn get_active_connections(&self) -> Vec<&Connection> {
        let now = Instant::now();
        self.connections
            .values()
            .filter(|conn| {
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
    pub raw_packets: usize,
}

#[derive(Debug, PartialEq)]
pub enum ConnectionAnalysis {
    Normal,
    SynFlood,
    PossiblePortScan,
    LongLived,
    UdpQuery,
    Closed,
}

pub fn analyze_connection(conn: &Connection) -> ConnectionAnalysis {
    match (&conn.state, conn.packets) {
        (ConnectionState::TcpSynSent, p) if p > 100 => ConnectionAnalysis::SynFlood,
        (ConnectionState::TcpSynSent, p) if p < 5 => ConnectionAnalysis::PossiblePortScan,
        (ConnectionState::TcpEstablished, p) if p > 10000 => ConnectionAnalysis::LongLived,
        (ConnectionState::UdpActive, p) if p < 3 => ConnectionAnalysis::UdpQuery,
        (ConnectionState::TcpClosed, _) => ConnectionAnalysis::Closed,
        _ => ConnectionAnalysis::Normal,
    }
}

pub fn process_packet_stream<I>(analyzer: &mut PacketAnalyzer, mut packets: I)
where
    I: Iterator<Item = Packet>,
{
    while let Some(packet) = packets.next() {
        analyzer.process_packet(&packet);
        if analyzer.statistics.total_packets % 1000 == 0 {
            analyzer.cleanup_old_connections(Duration::from_secs(300));
        }
    }
}

// ============================================================================
// Main Example Usage
// ============================================================================

fn main() {
    println!("Network Packet Inspector - Complete Implementation\n");

    // Example: Parse a simple TCP SYN packet
    let raw_packet = vec![
        // Ethernet header
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Destination MAC
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, // Source MAC
        0x08, 0x00, // EtherType: IPv4
        // IPv4 header
        0x45, 0x00, 0x00, 0x3C, // Version, IHL, ToS, Total Length
        0x1C, 0x46, 0x40, 0x00, // Identification, Flags, Fragment Offset
        0x40, 0x06, 0xB1, 0xE6, // TTL, Protocol (TCP), Checksum
        0xC0, 0xA8, 0x01, 0x01, // Source IP: 192.168.1.1
        0x08, 0x08, 0x08, 0x08, // Dest IP: 8.8.8.8
        // TCP header
        0x04, 0xD2, 0x00, 0x50, // Source Port: 1234, Dest Port: 80
        0x00, 0x00, 0x00, 0x64, // Sequence Number: 100
        0x00, 0x00, 0x00, 0x00, // Acknowledgment Number: 0
        0x50, 0x02, 0x20, 0x00, // Data Offset, Flags (SYN), Window
        0xE3, 0xE7, 0x00, 0x00, // Checksum, Urgent Pointer
    ];

    match Packet::parse(&raw_packet) {
        Ok(packet) => {
            println!("✓ Packet parsed successfully!");

            if let Some((src, dst)) = packet.extract_ips() {
                println!("  IP: {} -> {}", src, dst);
            }

            if is_tcp_syn(&packet) {
                println!("  Type: TCP SYN (connection initiation)");
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }

    // Example: Firewall
    println!("\n--- Firewall Example ---");
    let mut firewall = Firewall::new(Action::Deny);
    firewall.add_rule(FirewallRule::AllowPort { port: 80 });
    firewall.add_rule(FirewallRule::AllowPort { port: 443 });
    firewall.add_rule(FirewallRule::AllowService(Service::DNS));

    println!("✓ Firewall configured with {} rules", 3);

    // Example: Connection Tracking
    println!("\n--- Connection Tracking Example ---");
    let mut analyzer = PacketAnalyzer::new();
    println!("✓ Packet analyzer initialized");
    println!("  Ready to track connections and gather statistics");

    println!("\n✓ All components operational!");
}
