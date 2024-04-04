pub use self::native_tls::*;
pub use self::reqwest::*;

mod native_tls;
mod reqwest;

#[cfg(test)]
mod tests;
