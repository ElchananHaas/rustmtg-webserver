use game::client_message::ClientMessage;
use schemars::schema_for;

pub fn write_types() {
    let mut out: String = String::new();
    let schema = schema_for!(ClientMessage);
    let str_schema: String = serde_json::to_string_pretty(&schema).unwrap();
    out.push_str(&str_schema);
    let _ = std::fs::write("ui/rust_types.json", out);
}
