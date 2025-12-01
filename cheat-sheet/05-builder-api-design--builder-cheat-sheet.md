### Builder Cheat Sheet
```rust
// ===== BASIC BUILDER PATTERN =====
#[derive(Debug)]
struct User {
    username: String,
    email: String,
    age: Option<u32>,
    active: bool,
}

struct UserBuilder {
    username: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    active: bool,
}

impl UserBuilder {
    fn new() -> Self {
        UserBuilder {
            username: None,
            email: None,
            age: None,
            active: true,
        }
    }
    
    fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }
    
    fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
    
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
    
    fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    
    fn build(self) -> Result<User, String> {
        Ok(User {
            username: self.username.ok_or("username is required")?,
            email: self.email.ok_or("email is required")?,
            age: self.age,
            active: self.active,
        })
    }
}

fn basic_builder_example() {
    let user = UserBuilder::new()
        .username("alice".to_string())
        .email("alice@example.com".to_string())
        .age(30)
        .build()
        .unwrap();
    
    println!("{:?}", user);
}

// ===== BUILDER WITH REFERENCES =====
struct ConfigBuilder<'a> {
    host: Option<&'a str>,
    port: Option<u16>,
    timeout: Option<u64>,
}

impl<'a> ConfigBuilder<'a> {
    fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            timeout: None,
        }
    }
    
    fn host(mut self, host: &'a str) -> Self {
        self.host = Some(host);
        self
    }
    
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    fn build(self) -> Config {
        Config {
            host: self.host.unwrap_or("localhost").to_string(),
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
        }
    }
}

#[derive(Debug)]
struct Config {
    host: String,
    port: u16,
    timeout: u64,
}

// ===== BUILDER WITH DEFAULTS =====
#[derive(Debug, Default)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
    timeout: u64,
}

impl ServerConfig {
    fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }
}

#[derive(Default)]
struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    workers: Option<usize>,
    timeout: Option<u64>,
}

impl ServerConfigBuilder {
    fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }
    
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    fn workers(mut self, workers: usize) -> Self {
        self.workers = Some(workers);
        self
    }
    
    fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    fn build(self) -> ServerConfig {
        ServerConfig {
            host: self.host.unwrap_or_else(|| "0.0.0.0".to_string()),
            port: self.port.unwrap_or(8080),
            workers: self.workers.unwrap_or(4),
            timeout: self.timeout.unwrap_or(30),
        }
    }
}

fn builder_with_defaults() {
    let config = ServerConfig::builder()
        .host("localhost")
        .port(3000)
        .build();
    
    println!("{:?}", config);
}

// ===== TYPESTATE BUILDER PATTERN =====
// Compile-time enforcement of builder state

use std::marker::PhantomData;

// State markers
struct Empty;
struct HasUsername;
struct HasEmail;

// Builder with typestate
struct TypedUserBuilder<UsernameState, EmailState> {
    username: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    _username_state: PhantomData<UsernameState>,
    _email_state: PhantomData<EmailState>,
}

impl TypedUserBuilder<Empty, Empty> {
    fn new() -> Self {
        TypedUserBuilder {
            username: None,
            email: None,
            age: None,
            _username_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<EmailState> TypedUserBuilder<Empty, EmailState> {
    fn username(self, username: String) -> TypedUserBuilder<HasUsername, EmailState> {
        TypedUserBuilder {
            username: Some(username),
            email: self.email,
            age: self.age,
            _username_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<UsernameState> TypedUserBuilder<UsernameState, Empty> {
    fn email(self, email: String) -> TypedUserBuilder<UsernameState, HasEmail> {
        TypedUserBuilder {
            username: self.username,
            email: Some(email),
            age: self.age,
            _username_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<UsernameState, EmailState> TypedUserBuilder<UsernameState, EmailState> {
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
}

// Only allow build when both username and email are set
impl TypedUserBuilder<HasUsername, HasEmail> {
    fn build(self) -> User {
        User {
            username: self.username.unwrap(),
            email: self.email.unwrap(),
            age: self.age,
            active: true,
        }
    }
}

fn typestate_builder_example() {
    let user = TypedUserBuilder::new()
        .username("alice".to_string())
        .email("alice@example.com".to_string())
        .age(30)
        .build();
    
    // This won't compile - missing required fields:
    // let user = TypedUserBuilder::new()
    //     .username("alice".to_string())
    //     .build(); // ERROR: no method `build` found
    
    println!("{:?}", user);
}

// ===== TYPESTATE PATTERN FOR STATE MACHINES =====
// Door state machine with typestate

struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        println!("Creating locked door");
        Door { _state: PhantomData }
    }
    
    fn unlock(self) -> Door<Unlocked> {
        println!("Unlocking door");
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        println!("Locking door");
        Door { _state: PhantomData }
    }
    
    fn open(&self) {
        println!("Opening door");
    }
}

fn door_state_example() {
    let door = Door::<Locked>::new();
    // door.open(); // ERROR: method not found for Door<Locked>
    
    let door = door.unlock();
    door.open(); // OK
    
    let door = door.lock();
    // door.open(); // ERROR: method not found for Door<Locked>
}

// ===== CONNECTION STATE MACHINE =====
struct Disconnected;
struct Connecting;
struct Connected;
struct Failed;

struct Connection<State> {
    address: String,
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new(address: String) -> Self {
        Connection {
            address,
            _state: PhantomData,
        }
    }
    
    fn connect(self) -> Result<Connection<Connecting>, Connection<Failed>> {
        println!("Attempting to connect to {}", self.address);
        Ok(Connection {
            address: self.address,
            _state: PhantomData,
        })
    }
}

impl Connection<Connecting> {
    fn establish(self) -> Result<Connection<Connected>, Connection<Failed>> {
        println!("Establishing connection...");
        Ok(Connection {
            address: self.address,
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    fn send(&self, data: &str) {
        println!("Sending data: {}", data);
    }
    
    fn receive(&self) -> String {
        "Received data".to_string()
    }
    
    fn disconnect(self) -> Connection<Disconnected> {
        println!("Disconnecting");
        Connection {
            address: self.address,
            _state: PhantomData,
        }
    }
}

impl Connection<Failed> {
    fn retry(self) -> Connection<Disconnected> {
        println!("Retrying connection");
        Connection {
            address: self.address,
            _state: PhantomData,
        }
    }
}

fn connection_state_example() {
    let conn = Connection::<Disconnected>::new("localhost:8080".to_string());
    
    let conn = match conn.connect() {
        Ok(conn) => conn,
        Err(failed) => return,
    };
    
    let conn = match conn.establish() {
        Ok(conn) => conn,
        Err(failed) => return,
    };
    
    conn.send("Hello");
    let data = conn.receive();
    println!("{}", data);
    
    let conn = conn.disconnect();
}

// ===== FILE HANDLE STATE MACHINE =====
struct Closed;
struct Open;

struct File<State> {
    path: String,
    _state: PhantomData<State>,
}

impl File<Closed> {
    fn new(path: String) -> Self {
        File {
            path,
            _state: PhantomData,
        }
    }
    
    fn open(self) -> std::io::Result<File<Open>> {
        println!("Opening file: {}", self.path);
        Ok(File {
            path: self.path,
            _state: PhantomData,
        })
    }
}

impl File<Open> {
    fn read(&self) -> String {
        format!("Contents of {}", self.path)
    }
    
    fn write(&mut self, data: &str) {
        println!("Writing to {}: {}", self.path, data);
    }
    
    fn close(self) -> File<Closed> {
        println!("Closing file: {}", self.path);
        File {
            path: self.path,
            _state: PhantomData,
        }
    }
}

// ===== TRANSACTION STATE MACHINE =====
struct NotStarted;
struct InProgress;
struct Committed;
struct RolledBack;

struct Transaction<State> {
    id: u64,
    _state: PhantomData<State>,
}

impl Transaction<NotStarted> {
    fn new(id: u64) -> Self {
        Transaction {
            id,
            _state: PhantomData,
        }
    }
    
    fn begin(self) -> Transaction<InProgress> {
        println!("Beginning transaction {}", self.id);
        Transaction {
            id: self.id,
            _state: PhantomData,
        }
    }
}

impl Transaction<InProgress> {
    fn execute(&mut self, query: &str) {
        println!("Executing in transaction {}: {}", self.id, query);
    }
    
    fn commit(self) -> Transaction<Committed> {
        println!("Committing transaction {}", self.id);
        Transaction {
            id: self.id,
            _state: PhantomData,
        }
    }
    
    fn rollback(self) -> Transaction<RolledBack> {
        println!("Rolling back transaction {}", self.id);
        Transaction {
            id: self.id,
            _state: PhantomData,
        }
    }
}

fn transaction_example() {
    let tx = Transaction::<NotStarted>::new(1);
    let mut tx = tx.begin();
    
    tx.execute("INSERT INTO users VALUES (1, 'alice')");
    tx.execute("INSERT INTO users VALUES (2, 'bob')");
    
    let tx = tx.commit();
}

// ===== PROTOCOL STATE MACHINE =====
struct Handshake;
struct Authenticated;
struct Active;

struct Protocol<State> {
    session_id: String,
    _state: PhantomData<State>,
}

impl Protocol<Handshake> {
    fn new() -> Self {
        Protocol {
            session_id: uuid::Uuid::new_v4().to_string(),
            _state: PhantomData,
        }
    }
    
    fn authenticate(self, token: &str) -> Result<Protocol<Authenticated>, String> {
        if token == "valid_token" {
            Ok(Protocol {
                session_id: self.session_id,
                _state: PhantomData,
            })
        } else {
            Err("Invalid token".to_string())
        }
    }
}

impl Protocol<Authenticated> {
    fn activate(self) -> Protocol<Active> {
        Protocol {
            session_id: self.session_id,
            _state: PhantomData,
        }
    }
}

impl Protocol<Active> {
    fn send_message(&self, message: &str) {
        println!("Sending: {}", message);
    }
    
    fn receive_message(&self) -> String {
        "Received message".to_string()
    }
}

// ===== BUILDER WITH TYPESTATE AND VALIDATION =====
struct NoHost;
struct HasHost;
struct NoPort;
struct HasPort;

struct ServerBuilder<HostState, PortState> {
    host: Option<String>,
    port: Option<u16>,
    workers: usize,
    _host_state: PhantomData<HostState>,
    _port_state: PhantomData<PortState>,
}

impl ServerBuilder<NoHost, NoPort> {
    fn new() -> Self {
        ServerBuilder {
            host: None,
            port: None,
            workers: 4,
            _host_state: PhantomData,
            _port_state: PhantomData,
        }
    }
}

impl<PortState> ServerBuilder<NoHost, PortState> {
    fn host(self, host: impl Into<String>) -> ServerBuilder<HasHost, PortState> {
        ServerBuilder {
            host: Some(host.into()),
            port: self.port,
            workers: self.workers,
            _host_state: PhantomData,
            _port_state: PhantomData,
        }
    }
}

impl<HostState> ServerBuilder<HostState, NoPort> {
    fn port(self, port: u16) -> ServerBuilder<HostState, HasPort> {
        ServerBuilder {
            host: self.host,
            port: Some(port),
            workers: self.workers,
            _host_state: PhantomData,
            _port_state: PhantomData,
        }
    }
}

impl<HostState, PortState> ServerBuilder<HostState, PortState> {
    fn workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }
}

impl ServerBuilder<HasHost, HasPort> {
    fn build(self) -> Server {
        Server {
            host: self.host.unwrap(),
            port: self.port.unwrap(),
            workers: self.workers,
        }
    }
}

#[derive(Debug)]
struct Server {
    host: String,
    port: u16,
    workers: usize,
}

fn typed_server_builder_example() {
    let server = ServerBuilder::new()
        .host("localhost")
        .port(8080)
        .workers(8)
        .build();
    
    println!("{:?}", server);
    
    // Won't compile - missing required fields:
    // let server = ServerBuilder::new()
    //     .host("localhost")
    //     .build(); // ERROR
}

// ===== PHANTOM TYPE FOR UNITS =====
struct Meters;
struct Kilometers;

struct Distance<Unit> {
    value: f64,
    _unit: PhantomData<Unit>,
}

impl Distance<Meters> {
    fn meters(value: f64) -> Self {
        Distance {
            value,
            _unit: PhantomData,
        }
    }
    
    fn to_kilometers(self) -> Distance<Kilometers> {
        Distance {
            value: self.value / 1000.0,
            _unit: PhantomData,
        }
    }
}

impl Distance<Kilometers> {
    fn kilometers(value: f64) -> Self {
        Distance {
            value,
            _unit: PhantomData,
        }
    }
    
    fn to_meters(self) -> Distance<Meters> {
        Distance {
            value: self.value * 1000.0,
            _unit: PhantomData,
        }
    }
}

fn distance_example() {
    let d1 = Distance::<Meters>::meters(5000.0);
    let d2 = d1.to_kilometers();
    println!("Distance: {} km", d2.value);
    
    // Won't compile - different units:
    // let d3 = Distance::<Meters>::meters(100.0);
    // let d4 = Distance::<Kilometers>::kilometers(1.0);
    // let sum = d3.value + d4.value; // Types are different!
}

// ===== COMPILE-TIME GUARANTEES =====
struct Draft;
struct Published;

struct BlogPost<State> {
    title: String,
    content: String,
    _state: PhantomData<State>,
}

impl BlogPost<Draft> {
    fn new(title: String) -> Self {
        BlogPost {
            title,
            content: String::new(),
            _state: PhantomData,
        }
    }
    
    fn write(&mut self, content: &str) {
        self.content.push_str(content);
    }
    
    fn publish(self) -> BlogPost<Published> {
        println!("Publishing: {}", self.title);
        BlogPost {
            title: self.title,
            content: self.content,
            _state: PhantomData,
        }
    }
}

impl BlogPost<Published> {
    fn view(&self) -> String {
        format!("{}\n\n{}", self.title, self.content)
    }
    
    // Cannot write to published post
}

fn blog_post_example() {
    let mut post = BlogPost::<Draft>::new("My Post".to_string());
    post.write("This is the content.");
    // post.view(); // ERROR: method not available on Draft
    
    let post = post.publish();
    println!("{}", post.view());
    // post.write("More content"); // ERROR: method not available on Published
}

// ===== PAYMENT STATE MACHINE =====
struct Pending;
struct Authorized;
struct Captured;
struct Refunded;

struct Payment<State> {
    amount: u64,
    transaction_id: String,
    _state: PhantomData<State>,
}

impl Payment<Pending> {
    fn new(amount: u64) -> Self {
        Payment {
            amount,
            transaction_id: uuid::Uuid::new_v4().to_string(),
            _state: PhantomData,
        }
    }
    
    fn authorize(self) -> Result<Payment<Authorized>, String> {
        println!("Authorizing payment of ${}", self.amount);
        Ok(Payment {
            amount: self.amount,
            transaction_id: self.transaction_id,
            _state: PhantomData,
        })
    }
}

impl Payment<Authorized> {
    fn capture(self) -> Payment<Captured> {
        println!("Capturing payment");
        Payment {
            amount: self.amount,
            transaction_id: self.transaction_id,
            _state: PhantomData,
        }
    }
}

impl Payment<Captured> {
    fn refund(self) -> Payment<Refunded> {
        println!("Refunding payment");
        Payment {
            amount: self.amount,
            transaction_id: self.transaction_id,
            _state: PhantomData,
        }
    }
}

// Mock uuid for example
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self { Uuid }
        pub fn to_string(&self) -> String { "uuid".to_string() }
    }
}

fn payment_example() {
    let payment = Payment::<Pending>::new(10000);
    let payment = payment.authorize().unwrap();
    let payment = payment.capture();
    // Can only refund captured payments
    let payment = payment.refund();
}
```