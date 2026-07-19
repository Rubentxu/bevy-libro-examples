// Capítulo 26B. BRP — Bevy Remote Protocol concepts
// JSON-RPC 2.0 message structure and validation.

/// JSON-RPC 2.0 request structure
#[derive(Clone, Debug, PartialEq)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json_lite::Value,
}

/// JSON-RPC 2.0 response structure
#[derive(Clone, Debug, PartialEq)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<serde_json_lite::Value>,
    pub error: Option<RpcError>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// Simplified JSON value type (avoids external dependency)
pub mod serde_json_lite {
    #[derive(Clone, Debug, PartialEq)]
    pub enum Value {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    impl Value {
        pub fn from_str(s: &str) -> Self { Value::String(s.to_string()) }
        pub fn from_num(n: f64) -> Self { Value::Number(n) }
        pub fn from_bool(b: bool) -> Self { Value::Bool(b) }

        pub fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
                _ => None,
            }
        }
    }
}

/// BRP methods supported by Bevy
pub mod methods {
    pub const BEVY_QUERY: &str = "bevy/query";
    pub const BEVY_GET: &str = "bevy/get";
    pub const BEVY_SPAWN: &str = "bevy/spawn";
    pub const BEVY_INSERT: &str = "bevy/insert";
    pub const BEVY_REMOVE: &str = "bevy/remove";
    pub const BEVY_LIST: &str = "bevy/list";
}

/// Validate a BRP request
pub fn validate_request(req: &RpcRequest) -> Result<(), RpcError> {
    if req.jsonrpc != "2.0" {
        return Err(RpcError {
            code: -32600,
            message: "Invalid Request: jsonrpc must be '2.0'".to_string(),
        });
    }

    if req.method.is_empty() {
        return Err(RpcError {
            code: -32600,
            message: "Invalid Request: method is required".to_string(),
        });
    }

    Ok(())
}

/// Build a bevy/query response
pub fn build_query_response(entities: Vec<(u64, Vec<String>)>) -> RpcResponse {
    let result: Vec<serde_json_lite::Value> = entities
        .into_iter()
        .map(|(id, components)| {
            serde_json_lite::Value::Object(vec![
                ("id".to_string(), serde_json_lite::Value::from_num(id as f64)),
                ("components".to_string(), serde_json_lite::Value::Array(
                    components.into_iter().map(|c| serde_json_lite::Value::from_str(&c)).collect()
                )),
            ])
        })
        .collect();

    RpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(serde_json_lite::Value::Array(result)),
        error: None,
    }
}

/// HTTP endpoint path for BRP
pub const BRP_ENDPOINT: &str = "/bevy/api";

/// Default port for BRP server
pub const DEFAULT_PORT: u16 = 15702;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_valid_request() {
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: methods::BEVY_QUERY.to_string(),
            params: serde_json_lite::Value::Null,
        };
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn validate_wrong_version() {
        let req = RpcRequest {
            jsonrpc: "1.0".to_string(),
            id: 1,
            method: "test".to_string(),
            params: serde_json_lite::Value::Null,
        };
        let result = validate_request(&req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32600);
    }

    #[test]
    fn validate_empty_method() {
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "".to_string(),
            params: serde_json_lite::Value::Null,
        };
        assert!(validate_request(&req).is_err());
    }

    #[test]
    fn build_query_response_structure() {
        let entities = vec![
            (42, vec!["Health".to_string(), "Position".to_string()]),
            (99, vec!["Enemy".to_string()]),
        ];

        let response = build_query_response(entities);

        assert!(response.error.is_none());
        assert!(response.result.is_some());

        if let Some(serde_json_lite::Value::Array(arr)) = &response.result {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Result should be an array");
        }
    }

    #[test]
    fn json_value_object_get() {
        let obj = serde_json_lite::Value::Object(vec![
            ("name".to_string(), serde_json_lite::Value::from_str("test")),
            ("value".to_string(), serde_json_lite::Value::from_num(42.0)),
        ]);

        assert!(obj.get("name").is_some());
        assert!(obj.get("missing").is_none());
    }

    #[test]
    fn json_value_array() {
        let arr = serde_json_lite::Value::Array(vec![
            serde_json_lite::Value::from_num(1.0),
            serde_json_lite::Value::from_num(2.0),
            serde_json_lite::Value::from_num(3.0),
        ]);

        if let serde_json_lite::Value::Array(items) = &arr {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Should be array");
        }
    }

    #[test]
    fn rpc_error_codes() {
        let err = RpcError {
            code: -32601,
            message: "Method not found".to_string(),
        };
        assert!(err.code < 0);
        assert!(!err.message.is_empty());
    }

    #[test]
    fn brp_methods_are_namespaced() {
        assert!(methods::BEVY_QUERY.starts_with("bevy/"));
        assert!(methods::BEVY_LIST.starts_with("bevy/"));
    }

    #[test]
    fn brp_endpoint_path() {
        assert!(BRP_ENDPOINT.starts_with("/"));
        assert!(BRP_ENDPOINT.contains("bevy"));
    }

    #[test]
    fn default_port_is_valid() {
        assert!(DEFAULT_PORT > 1024, "Should use non-privileged port");
    }
}
