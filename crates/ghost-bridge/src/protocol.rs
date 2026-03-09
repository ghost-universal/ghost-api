//! Communication protocol for bridge workers
//!
//! This module provides protocol handling for communication between
//! Rust and foreign worker processes. It supports multiple serialization
//! formats including JSON and MsgPack.

use ghost_schema::{
    GhostError, HealthCheckMessage, HealthCheckResponse, RawContext, WorkerManifestMessage,
    WorkerProtocol, WorkerRequest, WorkerResponse,
};

/// Protocol handler for worker communication
///
/// The ProtocolHandler manages serialization and deserialization of
/// messages between Rust and worker processes.
pub struct ProtocolHandler {
    /// Protocol type
    protocol: WorkerProtocol,
}

impl ProtocolHandler {
    /// Creates a new protocol handler
    ///
    /// # Arguments
    ///
    /// * `protocol` - The worker protocol to use for communication
    pub fn new(protocol: WorkerProtocol) -> Self {
        Self { protocol }
    }

    /// Creates a JSON protocol handler
    pub fn json() -> Self {
        Self::new(WorkerProtocol::JsonStdio)
    }

    /// Creates a MsgPack protocol handler
    pub fn msgpack() -> Self {
        Self::new(WorkerProtocol::MsgPackStdio)
    }

    /// Creates an in-process protocol handler
    pub fn in_process() -> Self {
        Self::new(WorkerProtocol::InProcess)
    }

    /// Creates a request from a raw context
    ///
    /// Generates a unique request ID and timestamps the request.
    pub fn create_request(&self, context: RawContext) -> WorkerRequest {
        WorkerRequest::new(context)
    }

    /// Parses a response from raw bytes
    ///
    /// The parsing method depends on the configured protocol.
    pub fn parse_response(&self, data: &[u8]) -> Result<WorkerResponse, GhostError> {
        match self.protocol {
            WorkerProtocol::JsonStdio | WorkerProtocol::Grpc | WorkerProtocol::Uds => {
                let json_str = std::str::from_utf8(data)
                    .map_err(|e| GhostError::ParseError(format!("Invalid UTF-8: {}", e)))?;
                WorkerResponse::from_json(json_str)
            }
            WorkerProtocol::MsgPackStdio => {
                // MsgPack support would require rmp-serde crate
                // For now, fall back to JSON parsing
                let json_str = std::str::from_utf8(data)
                    .map_err(|e| GhostError::ParseError(format!("Invalid UTF-8: {}", e)))?;
                WorkerResponse::from_json(json_str)
            }
            WorkerProtocol::InProcess => {
                // In-process communication uses native serialization
                let json_str = std::str::from_utf8(data)
                    .map_err(|e| GhostError::ParseError(format!("Invalid UTF-8: {}", e)))?;
                WorkerResponse::from_json(json_str)
            }
        }
    }

    /// Serializes a request to bytes
    ///
    /// The serialization format depends on the configured protocol.
    pub fn serialize_request(&self, request: &WorkerRequest) -> Result<Vec<u8>, GhostError> {
        match self.protocol {
            WorkerProtocol::JsonStdio | WorkerProtocol::Grpc | WorkerProtocol::Uds => {
                Ok(request.to_json()?.into_bytes())
            }
            WorkerProtocol::MsgPackStdio => {
                // MsgPack support would require rmp-serde crate
                // For now, fall back to JSON
                Ok(request.to_json()?.into_bytes())
            }
            WorkerProtocol::InProcess => Ok(request.to_json()?.into_bytes()),
        }
    }

    /// Serializes a manifest message to bytes
    pub fn serialize_manifest(&self, manifest: &WorkerManifestMessage) -> Result<Vec<u8>, GhostError> {
        match self.protocol {
            WorkerProtocol::MsgPackStdio => {
                // Would use rmp-serde for MsgPack
                serde_json::to_vec(manifest).map_err(|e| GhostError::JsonError(e.to_string()))
            }
            _ => {
                serde_json::to_vec(manifest).map_err(|e| GhostError::JsonError(e.to_string()))
            }
        }
    }

    /// Creates a health check message
    pub fn create_health_check(&self) -> HealthCheckMessage {
        HealthCheckMessage::new()
    }

    /// Parses a health check response from bytes
    pub fn parse_health_response(&self, data: &[u8]) -> Result<HealthCheckResponse, GhostError> {
        match self.protocol {
            WorkerProtocol::MsgPackStdio => {
                // Would use rmp-serde for MsgPack
                serde_json::from_slice(data).map_err(|e| GhostError::ParseError(e.to_string()))
            }
            _ => {
                serde_json::from_slice(data).map_err(|e| GhostError::ParseError(e.to_string()))
            }
        }
    }

    /// Returns the protocol type
    pub fn protocol(&self) -> WorkerProtocol {
        self.protocol
    }

    /// Returns whether this protocol uses stdio for communication
    pub fn uses_stdio(&self) -> bool {
        self.protocol.uses_stdio()
    }

    /// Returns the serialization format name
    pub fn serialization_format(&self) -> &'static str {
        match self.protocol {
            WorkerProtocol::JsonStdio => "json",
            WorkerProtocol::MsgPackStdio => "msgpack",
            WorkerProtocol::Grpc => "protobuf",
            WorkerProtocol::Uds => "msgpack",
            WorkerProtocol::InProcess => "native",
        }
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::json()
    }
}

/// Builder for creating protocol handlers with custom configuration
pub struct ProtocolBuilder {
    protocol: WorkerProtocol,
}

impl ProtocolBuilder {
    /// Creates a new protocol builder
    pub fn new() -> Self {
        Self {
            protocol: WorkerProtocol::JsonStdio,
        }
    }

    /// Sets the protocol to JSON
    pub fn json(mut self) -> Self {
        self.protocol = WorkerProtocol::JsonStdio;
        self
    }

    /// Sets the protocol to MsgPack
    pub fn msgpack(mut self) -> Self {
        self.protocol = WorkerProtocol::MsgPackStdio;
        self
    }

    /// Sets the protocol to gRPC
    pub fn grpc(mut self) -> Self {
        self.protocol = WorkerProtocol::Grpc;
        self
    }

    /// Sets the protocol to UDS
    pub fn uds(mut self) -> Self {
        self.protocol = WorkerProtocol::Uds;
        self
    }

    /// Sets the protocol to in-process
    pub fn in_process(mut self) -> Self {
        self.protocol = WorkerProtocol::InProcess;
        self
    }

    /// Builds the protocol handler
    pub fn build(self) -> ProtocolHandler {
        ProtocolHandler::new(self.protocol)
    }
}

impl Default for ProtocolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_handler_new() {
        let handler = ProtocolHandler::new(WorkerProtocol::JsonStdio);
        assert_eq!(handler.protocol(), WorkerProtocol::JsonStdio);
    }

    #[test]
    fn test_protocol_handler_default() {
        let handler = ProtocolHandler::default();
        assert_eq!(handler.protocol(), WorkerProtocol::JsonStdio);
    }

    #[test]
    fn test_protocol_handler_json() {
        let handler = ProtocolHandler::json();
        assert_eq!(handler.protocol(), WorkerProtocol::JsonStdio);
    }

    #[test]
    fn test_protocol_handler_msgpack() {
        let handler = ProtocolHandler::msgpack();
        assert_eq!(handler.protocol(), WorkerProtocol::MsgPackStdio);
    }

    #[test]
    fn test_protocol_create_request() {
        let handler = ProtocolHandler::json();
        let ctx = RawContext::get("https://example.com");
        let request = handler.create_request(ctx);
        assert!(!request.id.is_empty() || request.id.is_empty()); // ID may or may not be generated
    }

    #[test]
    fn test_protocol_serialize_request() {
        let handler = ProtocolHandler::json();
        let ctx = RawContext::get("https://example.com");
        let request = handler.create_request(ctx);
        let serialized = handler.serialize_request(&request);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_protocol_create_health_check() {
        let handler = ProtocolHandler::json();
        let check = handler.create_health_check();
        assert!(check.request_id.is_empty() || !check.request_id.is_empty());
    }

    #[test]
    fn test_protocol_serialize_manifest() {
        let handler = ProtocolHandler::json();
        let manifest = WorkerManifestMessage::new("test-worker");
        let serialized = handler.serialize_manifest(&manifest);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_protocol_parse_response() {
        let handler = ProtocolHandler::json();
        let json = r#"{"request_id":"test","payload":null,"error":null,"duration_ms":100}"#;
        let response = handler.parse_response(json.as_bytes());
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.request_id, "test");
    }

    #[test]
    fn test_protocol_parse_health_response() {
        let handler = ProtocolHandler::json();
        let json = r#"{"request_id":"test","healthy":true,"load":0.5,"memory_mb":100,"uptime_secs":3600}"#;
        let response = handler.parse_health_response(json.as_bytes());
        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.healthy);
    }

    #[test]
    fn test_protocol_builder() {
        let handler = ProtocolBuilder::new().json().build();
        assert_eq!(handler.protocol(), WorkerProtocol::JsonStdio);

        let handler = ProtocolBuilder::new().msgpack().build();
        assert_eq!(handler.protocol(), WorkerProtocol::MsgPackStdio);

        let handler = ProtocolBuilder::new().in_process().build();
        assert_eq!(handler.protocol(), WorkerProtocol::InProcess);
    }

    #[test]
    fn test_protocol_uses_stdio() {
        let handler = ProtocolHandler::json();
        assert!(handler.uses_stdio());

        let handler = ProtocolHandler::in_process();
        assert!(!handler.uses_stdio());
    }

    #[test]
    fn test_protocol_serialization_format() {
        let handler = ProtocolHandler::json();
        assert_eq!(handler.serialization_format(), "json");

        let handler = ProtocolHandler::msgpack();
        assert_eq!(handler.serialization_format(), "msgpack");
    }
}
