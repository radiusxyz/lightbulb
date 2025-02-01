use http::{HeaderValue, Method};
use thiserror::Error;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

/// Creates a CorsLayer from the given domain string.
/// - If the input is "*" (allow all), it permits GET/POST methods and all headers.
/// - Otherwise, it parses a comma-separated list of domains.
pub fn create_cors_layer(http_cors_domains: &str) -> Result<CorsLayer, CorsDomainError> {
    let cors = match http_cors_domains.trim() {
        "*" => CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers(Any),
        _ => {
            let iter = http_cors_domains.split(',');
            if iter.clone().any(|o| o.trim() == "*") {
                return Err(CorsDomainError::WildCardNotAllowed {
                    input: http_cors_domains.to_string(),
                });
            }
            let origins = iter
                .map(|domain| {
                    domain.trim().parse::<HeaderValue>().map_err(|_| {
                        CorsDomainError::InvalidHeader {
                            domain: domain.to_string(),
                        }
                    })
                })
                .collect::<Result<Vec<HeaderValue>, _>>()?;
            let origin = AllowOrigin::list(origins);
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(origin)
                .allow_headers(Any)
        }
    };
    Ok(cors)
}

#[derive(Debug, Error)]
pub enum CorsDomainError {
    #[error("{domain} is an invalid header value")]
    InvalidHeader { domain: String },
    #[error("Wildcard origin (`*`) cannot be passed as part of a list: {input}")]
    WildCardNotAllowed { input: String },
}
