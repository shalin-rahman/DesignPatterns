//! A small, dependency-free, deterministic JSON writer shared by every
//! report format this workspace produces. Canonical key order is exactly
//! insertion order; callers are responsible for inserting in the already-
//! sorted order their data comes in.

use std::fmt::Write as _;

pub enum Json {
    Null,
    String(String),
    Number(u64),
    Float(f64),
    Array(Vec<Json>),
    Object(Vec<(&'static str, Json)>),
}

impl Json {
    pub fn render(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Number(value) => {
                let _ = write!(out, "{value}");
            }
            Self::Float(value) => {
                let _ = write!(out, "{value}");
            }
            Self::String(value) => render_string(value, out),
            Self::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    item.render(out);
                }
                out.push(']');
            }
            Self::Object(fields) => {
                out.push('{');
                for (index, (key, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    render_string(key, out);
                    out.push(':');
                    value.render(out);
                }
                out.push('}');
            }
        }
    }

    #[must_use]
    pub fn to_json_string(&self) -> String {
        let mut out = String::new();
        self.render(&mut out);
        out
    }
}

fn render_string(value: &str, out: &mut String) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}
