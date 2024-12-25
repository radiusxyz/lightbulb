pub mod core;
pub mod domain;
pub mod services;
pub mod utils;

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn auction_test() {
        assert_eq!(2 + 2, 4);
    }
}
