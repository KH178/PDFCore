use serde::{Deserialize, Serialize};
use crate::core::color::Color;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum TemplateNode {
    /// A vertical column of elements
    Column {
        children: Vec<TemplateNode>,
        #[serde(default)]
        spacing: Option<f64>,
    },
    /// A horizontal row of elements
    Row {
        children: Vec<TemplateNode>,
        #[serde(default)]
        spacing: Option<f64>,
    },
    /// Text block with simple string
    Text {
        content: String,
        size: f64,
        #[serde(default)]
        color: Option<Color>,
        #[serde(default)]
        background_color: Option<Color>,
        #[serde(default)]
        width: Option<f64>, // Max width for wrapping
    },
    /// Image asset with source path (relative to template or absolute)
    Image {
        src: String,
        width: f64,
        height: f64,
    },
    /// Empty space or container
    Container {
        child: Box<TemplateNode>,
        #[serde(default)]
        padding: Option<f64>,
        #[serde(default)]
        border: Option<f64>,
    },
    /// Table with columns and rows
    Table {
        columns: Vec<crate::core::table::TableColumn>,
        #[serde(default)]
        rows: Vec<Vec<String>>,
        #[serde(default)]
        settings: crate::core::table::TableSettings,
        #[serde(default)]
        data: Option<String>, // For data binding (array source)
    },
    /// Page number placeholder
    PageNumber {
        format: String,
        size: f64,
        #[serde(default)]
        align: Option<String>,
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Template {
    pub root: TemplateNode,
    pub name: Option<String>,
    #[serde(skip)]
    pub assets: std::collections::HashMap<String, Vec<u8>>,
    #[serde(skip)]
    pub asset_indices: std::collections::HashMap<String, u32>,
}

impl Template {
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        let mut t: Template = serde_json::from_str(json)?;
        t.assets = std::collections::HashMap::new();
        t.asset_indices = std::collections::HashMap::new();
        Ok(t)
    }

    pub fn from_zip(path: &str) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        
        // 1. Read layout.json
        let mut layout_file = archive.by_name("layout.json").map_err(|_| "layout.json not found in archive".to_string())?;
        let mut json = String::new();
        std::io::Read::read_to_string(&mut layout_file, &mut json).map_err(|e| e.to_string())?;
        drop(layout_file);
        
        let mut template: Template = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        template.assets = std::collections::HashMap::new();
        template.asset_indices = std::collections::HashMap::new();
        
        // 2. Read all other files as assets
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();
            
            if name == "layout.json" || name.ends_with('/') { continue; }
            
            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut buffer).map_err(|e| e.to_string())?;
            template.assets.insert(name, buffer);
        }
        
        Ok(template)
    }

    pub fn to_layout_node(&self) -> std::sync::Arc<dyn crate::core::layout::LayoutNode> {
        self.root.to_layout_node(&serde_json::Value::Null, &self.asset_indices)
    }

    pub fn render(&self, data: &serde_json::Value) -> std::sync::Arc<dyn crate::core::layout::LayoutNode> {
        self.root.to_layout_node(data, &self.asset_indices)
    }
}

use crate::core::layout::{LayoutNode as CoreLayoutNode, Column, Row, TextNode, ImageNode, Container, TableNode, PageNumberNode};
use std::sync::Arc;
use serde_json::Value;

// Helper to resolve {{ variable.path }}
fn resolve_template_string(text: &str, data: &Value) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut i = 0;
    
    // We'll process by splitting on "{{". This is a naive implementation.
    // A robust one would check for balanced braces.
    // For v1.2, we assume valid syntax or simple nesting.
    
    let parts: Vec<&str> = text.split("{{").collect();
    if parts.len() == 1 {
        return text.to_string();
    }

    result.push_str(parts[0]);
    
    for part in &parts[1..] {
        if let Some(end_idx) = part.find("}}") {
            let var_name = part[..end_idx].trim();
            let remainder = &part[end_idx+2..];
            
            // Resolve var_name
            let value = resolve_json_path(var_name, data);
            result.push_str(&value.unwrap_or_else(|| format!("{{{{ {} }}}}", var_name)));
            result.push_str(remainder);
        } else {
            // Malformed? Just treat as text
            result.push_str("{{");
            result.push_str(part);
        }
    }
    
    result
}

fn resolve_json_path(path: &str, data: &Value) -> Option<String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data;
    
    for part in parts {
        match current {
            Value::Object(map) => {
                if let Some(v) = map.get(part) {
                    current = v;
                } else {
                    return None;
                }
            },
            Value::Array(arr) => {
                if let Ok(idx) = part.parse::<usize>() {
                    if let Some(v) = arr.get(idx) {
                        current = v;
                    } else {
                        return None;
                    }
                } else {
                     return None;
                }
            },
            _ => return None,
        }
    }
    
    match current {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => Some("".to_string()),
        _ => Some(current.to_string()),
    }
}

impl TemplateNode {
    pub fn to_layout_node(&self, data: &Value, asset_indices: &std::collections::HashMap<String, u32>) -> Arc<dyn CoreLayoutNode> {
        match self {
            TemplateNode::Column { children, spacing } => {
                let nodes: Vec<Arc<dyn CoreLayoutNode>> = children.iter().map(|c| c.to_layout_node(data, asset_indices)).collect();
                Arc::new(Column { children: nodes, spacing: spacing.unwrap_or(0.0) })
            },
            TemplateNode::Row { children, spacing } => {
                let nodes = children.iter().map(|c| c.to_layout_node(data, asset_indices)).collect();
                Arc::new(Row { children: nodes, spacing: spacing.unwrap_or(0.0) })
            },
            TemplateNode::Text { content, size, color, background_color, width: _ } => {
                // Resolve content
                let resolved = resolve_template_string(content, data);
                
                Arc::new(TextNode {
                     text: resolved, 
                     size: *size, 
                     color: *color, 
                     background_color: *background_color 
                })
            },
            TemplateNode::Container { child, padding, border } => {
                 Arc::new(Container {
                     child: child.to_layout_node(data, asset_indices),
                     padding: padding.unwrap_or(0.0),
                     border_width: border.unwrap_or(0.0),
                 })
            },
            TemplateNode::Image { src, width, height } => {
                // Resolve "src" from asset_indices
                // If not found, use 0 (default/placeholder) or we should error?
                // For now, default to 0 to avoid crashing if asset missing.
                let index = *asset_indices.get(src).unwrap_or(&0);
                
                Arc::new(ImageNode {
                    image_index: index, 
                    width: *width, 
                    height: *height 
                })
            },
            TemplateNode::Table { columns, rows, settings, data: data_path } => {
                 let mut final_rows = Vec::new();

                 // 1. If static rows exist, include them (with variable substitution!)
                 for r in rows {
                     let resolved_row: Vec<String> = r.iter()
                         .map(|cell| resolve_template_string(cell, data))
                         .collect();
                     final_rows.push(resolved_row);
                 }
                 
                 // 2. If data binding exists, fetch array and iterate
                 if let Some(path_str) = data_path {
                     let clean_path = if path_str.starts_with("{{") && path_str.ends_with("}}") {
                         path_str[2..path_str.len()-2].trim()
                     } else {
                         path_str.as_str()
                     };

                     if let Some(array_val) = get_value_by_path(clean_path, data) {
                         if let Value::Array(arr) = array_val {
                             for item in arr {
                                 let mut row_vec = Vec::new();
                                 for col in columns {
                                     if let Some(field) = &col.field {
                                          let val = resolve_json_path(field, item).unwrap_or_default();
                                          row_vec.push(val);
                                     } else {
                                          row_vec.push("".to_string());
                                     }
                                 }
                                 final_rows.push(row_vec);
                             }
                         }
                     }
                 }

                let table = crate::core::table::Table {
                    columns: columns.clone(),
                    rows: final_rows,
                    settings: settings.clone(),
                };
                Arc::new(TableNode { table })
            },
            TemplateNode::PageNumber { format, size, align } => {
                Arc::new(PageNumberNode {
                    format: format.clone(),
                    size: *size,
                    align: align.clone().unwrap_or_else(|| "left".to_string()),
                })
            }
        }
    }
}

fn get_value_by_path<'a>(path: &str, data: &'a Value) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data;
    
    for part in parts {
        match current {
            Value::Object(map) => {
                if let Some(v) = map.get(part) {
                    current = v;
                } else {
                    return None;
                }
            },
            Value::Array(arr) => {
                if let Ok(idx) = part.parse::<usize>() {
                    if let Some(v) = arr.get(idx) {
                        current = v;
                    } else {
                        return None;
                    }
                } else {
                     return None;
                }
            },
            _ => return None,
        }
    }
    Some(current)
}
