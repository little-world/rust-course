// Pattern 6: Mock and Stub Patterns - Trait-Based Mocking
// Demonstrates idiomatic Rust mocking through trait abstraction.

use std::sync::Mutex;

// ============================================================================
// Example: Trait-Based Mocking
// ============================================================================

// Define a trait for the dependency
trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
}

// Real implementation
struct SmtpEmailService {
    server: String,
}

impl EmailService for SmtpEmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        // Actually send email via SMTP
        println!("Sending to {} via {}: {} - {}", to, self.server, subject, body);
        Ok(())
    }
}

// Mock for testing
struct MockEmailService {
    sent_emails: Mutex<Vec<(String, String, String)>>,
}

impl MockEmailService {
    fn new() -> Self {
        MockEmailService {
            sent_emails: Mutex::new(Vec::new()),
        }
    }

    fn emails_sent(&self) -> Vec<(String, String, String)> {
        self.sent_emails.lock().unwrap().clone()
    }
}

impl EmailService for MockEmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        self.sent_emails.lock().unwrap().push((
            to.to_string(),
            subject.to_string(),
            body.to_string(),
        ));
        Ok(())
    }
}

// Application code uses the trait
struct UserService<'a, E: EmailService> {
    email_service: &'a E,
}

impl<'a, E: EmailService> UserService<'a, E> {
    fn register_user(&self, email: &str) -> Result<(), String> {
        // ... registration logic ...
        println!("Registering user: {}", email);

        self.email_service.send_email(
            email,
            "Welcome!",
            "Thanks for registering",
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration_sends_email() {
        let mock = MockEmailService::new();
        let service = UserService {
            email_service: &mock,
        };

        service.register_user("user@example.com").unwrap();

        let emails = mock.emails_sent();
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].0, "user@example.com");
        assert_eq!(emails[0].1, "Welcome!");
    }
}

fn main() {
    // Demo with real implementation
    let real_service = SmtpEmailService {
        server: "smtp.example.com".to_string(),
    };
    let user_service = UserService {
        email_service: &real_service,
    };

    println!("Production use:");
    let _ = user_service.register_user("real@example.com");

    // Demo with mock
    let mock_service = MockEmailService::new();
    let test_user_service = UserService {
        email_service: &mock_service,
    };

    println!("\nTest use:");
    let _ = test_user_service.register_user("test@example.com");
    println!("Emails captured: {:?}", mock_service.emails_sent());
}
