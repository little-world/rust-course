// Pattern 10: Configuration with Defaults using Cow
use std::borrow::Cow;

struct Config<'a> {
    host: Cow<'a, str>,
    database: Cow<'a, str>,
}

impl<'a> Config<'a> {
    fn new(host: &'a str) -> Self {
        Config {
            host: Cow::Borrowed(host),
            database: Cow::Borrowed("default_db"),
        }
    }

    fn with_database(mut self, db: String) -> Self {
        self.database = Cow::Owned(db);
        self
    }
}

fn main() {
    let config1 = Config::new("localhost");
    println!("Config1: host={}, db={}", config1.host, config1.database);

    let config2 = Config::new("production.example.com")
        .with_database("prod_db".to_string());
    println!("Config2: host={}, db={}", config2.host, config2.database);

    println!("Cow config example completed");
}
