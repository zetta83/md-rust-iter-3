use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("gRPC transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),
    #[error("gRPC error ({code}): {message}")]
    GrpcStatus {
        code: tonic::Code,
        message: String,
    },
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("permission denied")]
    PermissionDenied,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("server error: {0}")]
    Internal(String),
}

impl From<tonic::Status> for BlogClientError {
    fn from(s: tonic::Status) -> Self {
        match s.code() {
            tonic::Code::NotFound => BlogClientError::NotFound(s.message().to_string()),
            tonic::Code::Unauthenticated => BlogClientError::Unauthorized,
            tonic::Code::PermissionDenied => BlogClientError::PermissionDenied,
            tonic::Code::InvalidArgument => BlogClientError::InvalidRequest(s.message().to_string()),
            tonic::Code::Internal => BlogClientError::Internal(s.message().to_string()),
            code => BlogClientError::GrpcStatus {
                code,
                message: s.message().to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(code: tonic::Code, msg: &str) -> tonic::Status {
        tonic::Status::new(code, msg)
    }

    // --- From<tonic::Status>: семантические варианты ---

    #[test]
    fn not_found_status_becomes_not_found() {
        let err = BlogClientError::from(status(tonic::Code::NotFound, "post not found"));
        assert!(matches!(err, BlogClientError::NotFound(_)));
        assert_eq!(err.to_string(), "resource not found: post not found");
    }

    #[test]
    fn unauthenticated_becomes_unauthorized() {
        let err = BlogClientError::from(status(tonic::Code::Unauthenticated, "bad token"));
        assert!(matches!(err, BlogClientError::Unauthorized));
    }

    #[test]
    fn permission_denied_becomes_permission_denied() {
        let err = BlogClientError::from(status(tonic::Code::PermissionDenied, "forbidden"));
        assert!(matches!(err, BlogClientError::PermissionDenied));
    }

    #[test]
    fn invalid_argument_becomes_invalid_request() {
        let err = BlogClientError::from(status(tonic::Code::InvalidArgument, "bad input"));
        assert!(matches!(err, BlogClientError::InvalidRequest(_)));
        assert_eq!(err.to_string(), "invalid request: bad input");
    }

    #[test]
    fn internal_status_becomes_internal() {
        let err = BlogClientError::from(status(tonic::Code::Internal, "db crash"));
        assert!(matches!(err, BlogClientError::Internal(_)));
        assert_eq!(err.to_string(), "server error: db crash");
    }

    // --- From<tonic::Status>: catch-all ---

    #[test]
    fn unmapped_code_becomes_grpc_status() {
        let err = BlogClientError::from(status(tonic::Code::Unavailable, "service down"));
        match err {
            BlogClientError::GrpcStatus { code, message } => {
                assert_eq!(code, tonic::Code::Unavailable);
                assert_eq!(message, "service down");
            }
            other => panic!("expected GrpcStatus, got {:?}", other),
        }
    }

    #[test]
    fn message_is_preserved_in_not_found() {
        let msg = "user 42 not found";
        let err = BlogClientError::from(status(tonic::Code::NotFound, msg));
        if let BlogClientError::NotFound(m) = err {
            assert_eq!(m, msg);
        } else {
            panic!("expected NotFound");
        }
    }

    // --- Display ---

    #[test]
    fn unauthorized_display() {
        assert_eq!(BlogClientError::Unauthorized.to_string(), "unauthorized");
    }

    #[test]
    fn permission_denied_display() {
        assert_eq!(BlogClientError::PermissionDenied.to_string(), "permission denied");
    }
}
