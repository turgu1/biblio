use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::OsRng, SaltString};

fn main() {
    let password = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "admin".to_string());
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hashed) => println!("{}", hashed.to_string()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
