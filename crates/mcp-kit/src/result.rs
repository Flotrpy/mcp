//! Helpers for building `CallToolResult`s with both human-readable text and
//! machine-readable structured content.

use rmcp::{
    ErrorData,
    model::{CallToolResult, ContentBlock},
};
use serde::Serialize;

/// A successful result carrying `value` as structured content plus a short
/// text summary for clients that only render text.
pub fn structured<T: Serialize>(summary: impl Into<String>, value: &T) -> Result<CallToolResult, ErrorData> {
    let json = serde_json::to_value(value)
        .map_err(|e| ErrorData::internal_error(format!("could not serialize result: {e}"), None))?;
    let mut result = CallToolResult::success(vec![ContentBlock::text(summary.into())]);
    result.structured_content = Some(json);
    Ok(result)
}

/// A tool-level failure the model can read and recover from (as opposed to a
/// protocol error). The message should say how to fix the call.
pub fn tool_error(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}

/// Map a bad-argument condition to a JSON-RPC invalid-params error.
pub fn invalid_params(message: impl Into<String>) -> ErrorData {
    ErrorData::invalid_params(message.into(), None)
}
