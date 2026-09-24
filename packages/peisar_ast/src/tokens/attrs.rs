use crate::utils::escape_attr;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Attributes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<(String, String)>>,
}

impl Attributes {
    pub fn to_html_attr_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref id) = self.id {
            parts.push(format!("id=\"{}\"", escape_attr(id)));
        }
        if !self.classes.is_none() {
            // let classes = self.classes.clone().unwrap();
            parts.push(format!(
                "class=\"{}\"",
                escape_attr(&self.classes.clone().unwrap().join(" "))
            ));
        }
        for (k, v) in &self.attributes.clone().unwrap() {
            if k == "style" {
                parts.push(format!("style=\"{}\"", escape_attr(v)));
            }
            parts.push(format!("{}=\"{}\"", escape_attr(k), escape_attr(v)));
        }
        parts.join(" ")
    }
}
