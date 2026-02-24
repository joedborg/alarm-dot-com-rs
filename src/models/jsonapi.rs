//! JSON:API response types for Alarm.com's API.
//!
//! Alarm.com uses a variant of the JSON:API specification. Responses contain
//! `data` (single or array), `included` (related resources), and `relationships`.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Deserialize an `id` field that may be either a JSON string or number.
/// Alarm.com sometimes returns numeric IDs (e.g. `"id": 19991856`) instead
/// of the JSON:API-standard string form (`"id": "19991856"`).
fn deserialize_id<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        other => Ok(other.to_string()),
    }
}

/// A JSON:API document with a single resource in `data`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SingleDocument {
    pub data: Resource,
    #[serde(default)]
    pub included: Vec<Resource>,
}

/// A JSON:API document with multiple resources in `data`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultiDocument {
    pub data: Vec<Resource>,
    #[serde(default)]
    pub included: Vec<Resource>,
}

/// A JSON:API error document.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorDocument {
    pub errors: Vec<ApiError>,
}

/// A single error entry in a JSON:API error document.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiError {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
}

/// A JSON:API resource object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resource {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(default)]
    pub attributes: serde_json::Value,
    #[serde(default)]
    pub relationships: HashMap<String, Relationship>,
}

/// A JSON:API relationship object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relationship {
    pub data: RelationshipData,
}

/// Relationship data can be a single identifier or a list of identifiers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RelationshipData {
    Single(ResourceIdentifier),
    Many(Vec<ResourceIdentifier>),
    Null,
}

/// A JSON:API resource identifier (type + id).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceIdentifier {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: String,
}

impl Resource {
    /// Extract all related resource IDs for a given relationship key.
    pub fn related_ids(&self, key: &str) -> Vec<String> {
        match self.relationships.get(key) {
            Some(rel) => match &rel.data {
                RelationshipData::Single(ident) => vec![ident.id.clone()],
                RelationshipData::Many(idents) => idents.iter().map(|i| i.id.clone()).collect(),
                RelationshipData::Null => vec![],
            },
            None => vec![],
        }
    }
}

/// Try to parse a response body as either a single or multi document.
/// Falls back to error document detection.
#[derive(Debug, Clone)]
pub enum ApiResponse {
    Single(SingleDocument),
    Multi(MultiDocument),
    Error(ErrorDocument),
}

impl ApiResponse {
    /// Parse a JSON string into an `ApiResponse`.
    pub fn parse(body: &str) -> std::result::Result<Self, serde_json::Error> {
        // Try single document first
        if let Ok(doc) = serde_json::from_str::<SingleDocument>(body) {
            return Ok(ApiResponse::Single(doc));
        }
        // Try multi document
        if let Ok(doc) = serde_json::from_str::<MultiDocument>(body) {
            return Ok(ApiResponse::Multi(doc));
        }
        // Try error document
        if let Ok(doc) = serde_json::from_str::<ErrorDocument>(body) {
            return Ok(ApiResponse::Error(doc));
        }
        // Fall back: try to deserialize as generic Value and return unexpected
        Err(serde_json::Error::custom(
            "response did not match any known JSON:API format",
        ))
    }

    /// Get the data resources from the response.
    pub fn resources(&self) -> Vec<&Resource> {
        match self {
            ApiResponse::Single(doc) => vec![&doc.data],
            ApiResponse::Multi(doc) => doc.data.iter().collect(),
            ApiResponse::Error(_) => vec![],
        }
    }

    /// Get included resources from the response.
    pub fn included(&self) -> &[Resource] {
        match self {
            ApiResponse::Single(doc) => &doc.included,
            ApiResponse::Multi(doc) => &doc.included,
            ApiResponse::Error(_) => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_document() {
        let json = r#"{
            "data": {
                "id": "123",
                "type": "devices/partition",
                "attributes": { "state": 1, "desiredState": 1 },
                "relationships": {}
            },
            "included": []
        }"#;
        let resp = ApiResponse::parse(json).unwrap();
        assert!(matches!(resp, ApiResponse::Single(_)));
        let resources = resp.resources();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].id, "123");
    }

    #[test]
    fn parse_multi_document() {
        let json = r#"{
            "data": [
                { "id": "1", "type": "devices/sensor", "attributes": {}, "relationships": {} },
                { "id": "2", "type": "devices/sensor", "attributes": {}, "relationships": {} }
            ],
            "included": []
        }"#;
        let resp = ApiResponse::parse(json).unwrap();
        assert!(matches!(resp, ApiResponse::Multi(_)));
        assert_eq!(resp.resources().len(), 2);
    }

    #[test]
    fn parse_error_document() {
        let json = r#"{
            "errors": [
                { "status": "401", "code": "401", "title": "Unauthorized" }
            ]
        }"#;
        let resp = ApiResponse::parse(json).unwrap();
        assert!(matches!(resp, ApiResponse::Error(_)));
    }

    #[test]
    fn resource_related_ids() {
        let json = r#"{
            "id": "sys-1",
            "type": "systems/system",
            "attributes": {},
            "relationships": {
                "partitions": {
                    "data": [
                        { "id": "p-1", "type": "devices/partition" },
                        { "id": "p-2", "type": "devices/partition" }
                    ]
                },
                "primaryLock": {
                    "data": { "id": "l-1", "type": "devices/lock" }
                }
            }
        }"#;
        let res: Resource = serde_json::from_str(json).unwrap();
        assert_eq!(res.related_ids("partitions"), vec!["p-1", "p-2"]);
        assert_eq!(res.related_ids("primaryLock"), vec!["l-1"]);
        assert!(res.related_ids("nonexistent").is_empty());
    }
}
