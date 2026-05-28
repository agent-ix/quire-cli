//! Static binary CLI wrapping quire-rs (render, parse, extract, validate).

/// Placeholder entry point.
pub fn hello() -> &'static str {
    "hello from quire_cli"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_returns_greeting() {
        assert!(hello().contains("quire_cli"));
    }
}
