mod nu;

use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{Span, Value};

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub struct KDL;

fn build_document(document: &KdlDocument) -> Value {
    let cols: Vec<String> = document
        .nodes()
        .iter()
        .map(|node| node.name().to_string())
        .collect();
    let vals = document.nodes().iter().map(build_node).collect();
    let span = Span::new(
        document.span().offset(),
        document.span().offset() + document.len(),
    );

    Value::record(cols, vals, span)
}

fn build_node(node: &KdlNode) -> Value {
    let entries: Vec<Value> = node.entries().iter().map(build_entry).collect();

    let span = Span::new(node.span().offset(), node.span().offset() + node.len());

    if let Some(children) = node.children() {
        let children = build_document(children);

        if entries.is_empty() {
            return children;
        }

        let entries = if entries.len() == 1 {
            entries[0].clone()
        } else {
            Value::list(entries, Span::unknown())
        };

        Value::Record {
            cols: vec!["entries".to_string(), "children".to_string()],
            vals: vec![entries, children],
            span,
        }
    } else {
        if entries.is_empty() {
            Value::nothing(Span::unknown())
        } else if entries.len() == 1 {
            entries[0].clone()
        } else {
            Value::list(entries, Span::unknown())
        }
    }
}

fn build_entry(entry: &KdlEntry) -> Value {
    let span = Span::new(entry.span().offset(), entry.span().offset() + entry.len());

    let value = match entry.value() {
        KdlValue::RawString(val) => Value::string(val.to_string(), span),
        KdlValue::String(val) => Value::string(val.to_string(), span),
        KdlValue::Base2(val) => Value::int(*val, span),
        KdlValue::Base8(val) => Value::int(*val, span),
        KdlValue::Base10(val) => Value::int(*val, span),
        KdlValue::Base16(val) => Value::int(*val, span),
        KdlValue::Base10Float(val) => Value::float(*val, span),
        KdlValue::Bool(val) => Value::bool(*val, span),
        KdlValue::Null => Value::nothing(span),
    };

    match entry.name() {
        Some(name) => Value::Record {
            cols: vec![name.value().to_string()],
            vals: vec![value],
            span,
        },
        None => value,
    }
}

impl KDL {
    pub fn from(&self, _call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> {
        let doc: KdlDocument = input
            .as_string()
            .expect("input is not a string")
            .parse()
            .expect("failed to parse KDL");
        Ok(build_document(&doc))
    }
}
