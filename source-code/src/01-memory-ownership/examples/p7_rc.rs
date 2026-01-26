// Pattern 7: Shared Configuration with Rc
use std::rc::Rc;

struct Config {
    database_url: String,
    max_connections: usize,
}

#[allow(dead_code)]
struct DatabasePool { config: Rc<Config> }
#[allow(dead_code)]
struct CacheService { config: Rc<Config> }

fn main() {
    // Share config across components
    let config = Rc::new(Config {
        database_url: "postgres://localhost/db".into(),
        max_connections: 100,
    });

    println!("Ref count: {}", Rc::strong_count(&config)); // 1

    let _db = DatabasePool { config: Rc::clone(&config) };
    let _cache = CacheService { config: Rc::clone(&config) };

    println!("Ref count: {}", Rc::strong_count(&config)); // 3
    println!("Config: {}, max {}", config.database_url, config.max_connections);
    println!("Rc example completed");
}
