//! Security headers for Nexus Protocol server.

/// Security headers to be included in all HTTP/WebSocket responses.
pub struct SecurityHeaders {
    pub frame_options: &'static str,
    pub content_type_options: &'static str,
    pub referrer_policy: &'static str,
    pub permissions_policy: &'static str,
    pub hsts_max_age: u32,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            frame_options: "DENY",
            content_type_options: "nosniff",
            referrer_policy: "strict-origin-when-cross-origin",
            permissions_policy: "camera=(), microphone=(), geolocation=()",
            hsts_max_age: 63072000, // 2 years
        }
    }
}

impl SecurityHeaders {
    /// Convert to HTTP header format.
    pub fn to_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            ("X-Frame-Options", self.frame_options.to_string()),
            ("X-Content-Type-Options", self.content_type_options.to_string()),
            ("Referrer-Policy", self.referrer_policy.to_string()),
            ("Permissions-Policy", self.permissions_policy.to_string()),
            (
                "Strict-Transport-Security",
                format!("max-age={}", self.hsts_max_age),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_headers() {
        let headers = SecurityHeaders::default();
        assert_eq!(headers.frame_options, "DENY");
        assert_eq!(headers.hsts_max_age, 63072000);
    }

    #[test]
    fn test_header_conversion() {
        let headers = SecurityHeaders::default();
        let converted = headers.to_headers();
        assert!(converted.len() >= 4);
    }
}