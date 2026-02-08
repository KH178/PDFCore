use serde::{Deserialize, Serialize};
use crate::core::color::Color;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Manifest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Style {
    pub size: Option<f64>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub align: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub padding: Option<f64>,
    pub spacing: Option<f64>,
    pub border: Option<f64>,
    pub header_height: Option<f64>,
    pub cell_height: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum TemplateNode {
    /// A vertical column of elements
    Column {
        children: Vec<TemplateNode>,
        #[serde(default)]
        spacing: Option<f64>,
        #[serde(default)]
        style: Option<String>,
    },
    /// A horizontal row of elements
    Row {
        children: Vec<TemplateNode>,
        #[serde(default)]
        spacing: Option<f64>,
        #[serde(default)]
        style: Option<String>,
    },
    /// Text block with simple string
    Text {
        content: String,
        #[serde(default)]
        size: Option<f64>,
        #[serde(default)]
        color: Option<Color>,
        #[serde(default)]
        background_color: Option<Color>,
        #[serde(default)]
        width: Option<f64>, // Max width for wrapping
        #[serde(default)]
        style: Option<String>,
    },
    /// Image asset with source path (relative to template or absolute)
    Image {
        src: String,
        #[serde(default)]
        width: Option<f64>,
        #[serde(default)]
        height: Option<f64>,
        #[serde(default)]
        style: Option<String>,
    },
    /// Empty space or container
    Container {
        child: Box<TemplateNode>,
        #[serde(default)]
        padding: Option<f64>,
        #[serde(default)]
        border: Option<f64>,
        #[serde(default)]
        style: Option<String>,
    },
    /// Table with columns and rows
    Table {
        columns: Vec<crate::core::table::TableColumn>,
        #[serde(default)]
        rows: Vec<Vec<String>>,
        #[serde(default)]
        settings: Option<crate::core::table::TableSettings>,
        #[serde(default)]
        data: Option<String>, // For data binding (array source)
        #[serde(default)]
        style: Option<String>,
    },
    /// Page number placeholder
    PageNumber {
        format: String,
        #[serde(default)]
        size: Option<f64>,
        #[serde(default)]
        align: Option<String>,
        #[serde(default)]
        style: Option<String>,
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Template {
    pub root: TemplateNode,
    #[serde(default)]
    pub manifest: Option<Manifest>,
    #[serde(default)]
    pub styles: HashMap<String, Style>,
    #[serde(skip)]
    pub assets: HashMap<String, Vec<u8>>,
    #[serde(skip)]
    pub asset_indices: HashMap<String, u32>,
}

impl Template {
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        let mut t: Template = serde_json::from_str(json)?;
        t.assets = HashMap::new();
        t.asset_indices = HashMap::new();
        Ok(t)
    }

    pub fn from_zip(path: &str) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("Failed to open zip file '{}': {}", path, e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;
        
        // 1. Read layout.json
        let mut layout_file = archive.by_name("layout.json").map_err(|_| "layout.json not found in archive (check if zip root is correct)".to_string())?;
        let mut json = String::new();
        std::io::Read::read_to_string(&mut layout_file, &mut json).map_err(|e| format!("Failed to read layout.json content: {}", e))?;
        drop(layout_file);
        
        let mut template: Template = serde_json::from_str(&json).map_err(|e| format!("Failed to parse layout.json: {}", e))?;
        template.assets = HashMap::new();
        template.asset_indices = HashMap::new();
        
        // 2. Read styles.json (optional)
        if let Ok(mut style_file) = archive.by_name("styles.json") {
            let mut style_json = String::new();
            if std::io::Read::read_to_string(&mut style_file, &mut style_json).is_ok() {
                if let Ok(styles) = serde_json::from_str::<HashMap<String, Style>>(&style_json) {
                    template.styles.extend(styles);
                }
            }
        }

        // 3. Read manifest.json (optional)
        if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
            let mut manifest_json = String::new();
            if std::io::Read::read_to_string(&mut manifest_file, &mut manifest_json).is_ok() {
                if let Ok(manifest) = serde_json::from_str::<Manifest>(&manifest_json) {
                    template.manifest = Some(manifest);
                }
            }
        }
        
        // 4. Read all other files as assets
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();
            
            if name == "layout.json" || name == "styles.json" || name == "manifest.json" || name.ends_with('/') { continue; }
            
            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut buffer).map_err(|e| e.to_string())?;
            template.assets.insert(name, buffer);
        }
        
        Ok(template)
    }

    pub fn to_layout_node(&self) -> std::sync::Arc<dyn crate::core::layout::LayoutNode> {
        self.root.to_layout_node(&serde_json::Value::Null, &self.asset_indices, &self.styles)
    }

    pub fn render(&self, data: &serde_json::Value) -> std::sync::Arc<dyn crate::core::layout::LayoutNode> {
        self.root.to_layout_node(data, &self.asset_indices, &self.styles)
    }
}

use crate::core::layout::{LayoutNode as CoreLayoutNode, Column, Row, TextNode, ImageNode, Container, TableNode, PageNumberNode};
use std::sync::Arc;
use serde_json::Value;

// Helper to resolve {{ variable.path }}
fn resolve_template_string(text: &str, data: &Value) -> String {
    let mut result = String::new();
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

// Helpers for style resolution
fn resolve_prop<T: Clone>(val: Option<T>, style: Option<&String>, styles: &HashMap<String, Style>, extractor: impl Fn(&Style) -> Option<T>, default: T) -> T {
    val.or_else(|| style.and_then(|name| styles.get(name)).and_then(|s| extractor(s)))
       .unwrap_or(default)
}

fn resolve_option<T: Clone>(val: Option<T>, style: Option<&String>, styles: &HashMap<String, Style>, extractor: impl Fn(&Style) -> Option<T>) -> Option<T> {
    val.or_else(|| style.and_then(|name| styles.get(name)).and_then(|s| extractor(s)))
}

impl TemplateNode {
    pub fn to_layout_node(&self, data: &Value, asset_indices: &HashMap<String, u32>, styles: &HashMap<String, Style>) -> Arc<dyn CoreLayoutNode> {
        match self {
            TemplateNode::Column { children, spacing, style } => {
                let nodes: Vec<Arc<dyn CoreLayoutNode>> = children.iter().map(|c| c.to_layout_node(data, asset_indices, styles)).collect();
                let spacing_val = resolve_prop(*spacing, style.as_ref(), styles, |s| s.spacing, 0.0);
                Arc::new(Column { children: nodes, spacing: spacing_val })
            },
            TemplateNode::Row { children, spacing, style } => {
                let nodes = children.iter().map(|c| c.to_layout_node(data, asset_indices, styles)).collect();
                let spacing_val = resolve_prop(*spacing, style.as_ref(), styles, |s| s.spacing, 0.0);
                Arc::new(Row { children: nodes, spacing: spacing_val })
            },
            TemplateNode::Text { content, size, color, background_color, width: _, style } => {
                // Resolve content
                let resolved = resolve_template_string(content, data);
                let size_val = resolve_prop(*size, style.as_ref(), styles, |s| s.size, 12.0);
                let color_val = resolve_option(*color, style.as_ref(), styles, |s| s.color);
                let bg_val = resolve_option(*background_color, style.as_ref(), styles, |s| s.background_color);

                Arc::new(TextNode {
                     text: resolved, 
                     size: size_val, 
                     color: color_val, 
                     background_color: bg_val 
                })
            },
            TemplateNode::Container { child, padding, border, style } => {
                 let padding_val = resolve_prop(*padding, style.as_ref(), styles, |s| s.padding, 0.0);
                 let border_val = resolve_prop(*border, style.as_ref(), styles, |s| s.border, 0.0);
                 
                 Arc::new(Container {
                     child: child.to_layout_node(data, asset_indices, styles),
                     padding: padding_val,
                     border_width: border_val,
                 })
            },
            TemplateNode::Image { src, width, height, style } => {
                let index = *asset_indices.get(src).unwrap_or(&0);
                let w_val = resolve_prop(*width, style.as_ref(), styles, |s| s.width, 100.0);
                let h_val = resolve_prop(*height, style.as_ref(), styles, |s| s.height, 100.0);
                
                Arc::new(ImageNode {
                    image_index: index, 
                    width: w_val, 
                    height: h_val 
                })
            },
            TemplateNode::Table { columns, rows, settings, data: data_path, style } => {
                 let mut final_rows = Vec::new();

                 // Resolve settings with styles
                 let mut resolved_settings = settings.clone().unwrap_or_default();
                 // Apply style overrides if settings were defaults or just to inherit
                 if let Some(style_name) = style {
                     if let Some(s) = styles.get(style_name) {
                         if let Some(v) = s.padding { resolved_settings.padding = v; }
                         if let Some(v) = s.border { resolved_settings.border_width = v; }
                         if let Some(v) = s.header_height { resolved_settings.header_height = v; }
                         if let Some(v) = s.cell_height { resolved_settings.cell_height = v; }
                         if let Some(v) = s.size { resolved_settings.font_size = v; }
                         if let Some(v) = s.color { resolved_settings.font_color = v; }
                     }
                 }

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
                    settings: resolved_settings,
                };
                Arc::new(TableNode { table })
            },
            TemplateNode::PageNumber { format, size, align, style } => {
                let size_val = resolve_prop(*size, style.as_ref(), styles, |s| s.size, 10.0);
                let align_val = resolve_prop(align.clone(), style.as_ref(), styles, |s| s.align.clone(), "left".to_string());
                
                Arc::new(PageNumberNode {
                    format: format.clone(),
                    size: size_val,
                    align: align_val,
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
