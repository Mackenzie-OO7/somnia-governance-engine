use crate::auth::wallet_auth::WalletAuthService;
use crate::utils::errors::GovernanceError;
use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use ethers::core::types::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub address: Address,
    pub token: String,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
}

impl AuthenticatedUser {
    pub fn new(address: Address, token: String) -> Self {
        Self {
            address,
            token,
            authenticated_at: chrono::Utc::now(),
        }
    }
}

/// Middleware to require authentication for protected routes
pub async fn require_auth(
    State(auth_service): State<WalletAuthService>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            )
        })?;

    // Extract Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization header format".to_string(),
            )
        })?;

    // Verify token
    let auth_token = auth_service
        .verify_token(token)
        .await
        .map_err(|e| {
            tracing::error!("Token verification error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token verification failed".to_string(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token".to_string(),
            )
        })?;

    // Create authenticated user and add to request extensions
    let authenticated_user = AuthenticatedUser::new(auth_token.address, token.to_string());
    request.extensions_mut().insert(authenticated_user);

    Ok(next.run(request).await)
}

/// Optional authentication middleware - doesn't fail if no auth provided
pub async fn optional_auth(
    State(auth_service): State<WalletAuthService>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract and verify token
    if let Some(auth_header) = request
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Ok(Some(auth_token)) = auth_service.verify_token(token).await {
                let authenticated_user = AuthenticatedUser::new(auth_token.address, token.to_string());
                request.extensions_mut().insert(authenticated_user);
            }
        }
    }

    next.run(request).await
}

/// Middleware to extract user address from authenticated request
pub fn extract_user_address(request: &Request) -> Option<Address> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .map(|user| user.address)
}

/// Rate limiting middleware based on user address
pub async fn rate_limit_by_address(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // Get user address if authenticated
    let user_address = extract_user_address(&request);
    
    // TODO: Implement actual rate limiting logic here
    // For now, we'll just pass through
    // In production, this would:
    // 1. Track requests per address
    // 2. Implement sliding window or token bucket
    // 3. Return 429 Too Many Requests if limit exceeded
    
    if let Some(_address) = user_address {
        // Authenticated user - different rate limits
        tracing::debug!("Rate limiting authenticated user");
    } else {
        // Anonymous user - more restrictive limits
        tracing::debug!("Rate limiting anonymous user");
    }

    Ok(next.run(request).await)
}

/// CORS middleware specifically for the governance API
pub async fn governance_cors(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // Add CORS headers
    let headers = response.headers_mut();
    
    headers.insert(
        "access-control-allow-origin",
        HeaderValue::from_static("*"), // In production, be more specific
    );
    
    headers.insert(
        "access-control-allow-methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    
    headers.insert(
        "access-control-allow-headers",
        HeaderValue::from_static("content-type, authorization"),
    );
    
    headers.insert(
        "access-control-max-age",
        HeaderValue::from_static("3600"),
    );

    response
}

/// Middleware to log requests with user context
pub async fn request_logging(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_address = extract_user_address(&request);
    
    let start_time = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start_time.elapsed();
    
    let status = response.status();
    
    match user_address {
        Some(address) => {
            tracing::info!(
                method = %method,
                uri = %uri,
                status = %status,
                duration_ms = %duration.as_millis(),
                user_address = %format!("{:?}", address),
                "Request processed"
            );
        }
        None => {
            tracing::info!(
                method = %method,
                uri = %uri,
                status = %status,
                duration_ms = %duration.as_millis(),
                "Anonymous request processed"
            );
        }
    }
    
    response
}

/// Security headers middleware
pub async fn security_headers(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Security headers
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    
    headers.insert(
        "x-frame-options",
        HeaderValue::from_static("DENY"),
    );
    
    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block"),
    );
    
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    // Don't add HSTS in development
    if cfg!(not(debug_assertions)) {
        headers.insert(
            "strict-transport-security",
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }

    response
}

/// Helper function to get authenticated user from request
pub fn get_authenticated_user(request: &Request) -> Result<&AuthenticatedUser, GovernanceError> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| GovernanceError::invalid_signature("User not authenticated"))
}

/// Response types for authentication endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl ApiResponse<()> {
    pub fn success_empty() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user() {
        let address = Address::zero();
        let token = "test_token".to_string();
        let user = AuthenticatedUser::new(address, token.clone());
        
        assert_eq!(user.address, address);
        assert_eq!(user.token, token);
        assert!(user.authenticated_at <= chrono::Utc::now());
    }

    #[test]
    fn test_api_response() {
        let success_response = ApiResponse::success("test data");
        assert!(success_response.success);
        assert_eq!(success_response.data, Some("test data"));
        assert!(success_response.error.is_none());

        let error_response: ApiResponse<String> = ApiResponse::error("test error".to_string());
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert_eq!(error_response.error, Some("test error".to_string()));

        let empty_response = ApiResponse::success_empty();
        assert!(empty_response.success);
        assert!(empty_response.data.is_none());
        assert!(empty_response.error.is_none());
    }
}