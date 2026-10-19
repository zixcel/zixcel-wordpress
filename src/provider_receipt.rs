use serde_json::{Value, json};
use zix_operation_contracts::Binding;
pub(crate) fn wrap(
    value: Value,
    operation: &str,
    binding: &Binding,
    approved_plan_sha256: Option<String>,
) -> Value {
    json!({"value":value,"invocation":{"operation":operation,"binding":binding,
 "approved_plan_sha256":approved_plan_sha256}})
}
