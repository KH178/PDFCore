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
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Template {
    pub root: TemplateNode,
    pub name: Option<String>,
}

impl Template {
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_layout_node(&self) -> std::sync::Arc<dyn crate::core::layout::LayoutNode> {
        self.root.to_layout_node()
    }
}

use crate::core::layout::{LayoutNode as CoreLayoutNode, Column, Row, TextNode, ImageNode, Container};
use std::sync::Arc;

impl TemplateNode {
    pub fn to_layout_node(&self) -> Arc<dyn CoreLayoutNode> {
        match self {
            TemplateNode::Column { children, spacing } => {
                let nodes: Vec<Arc<dyn CoreLayoutNode>> = children.iter().map(|c| c.to_layout_node()).collect();
                Arc::new(Column { children: nodes, spacing: spacing.unwrap_or(0.0) })
            },
            TemplateNode::Row { children, spacing } => {
                let nodes = children.iter().map(|c| c.to_layout_node()).collect();
                Arc::new(Row { children: nodes, spacing: spacing.unwrap_or(0.0) })
            },
            TemplateNode::Text { content, size, color, background_color, width: _ } => {
                Arc::new(TextNode {
                     text: content.clone(), 
                     size: *size, 
                     color: *color, 
                     background_color: *background_color 
                })
            },
            TemplateNode::Container { child, padding, border } => {
                 Arc::new(Container {
                     child: child.to_layout_node(),
                     padding: padding.unwrap_or(0.0),
                     border_width: border.unwrap_or(0.0),
                 })
            },
            TemplateNode::Image { src: _, width, height } => {
                // Placeholder: Image loading needs document context. 
                // For now, we return a placeholder or error?
                // Or we assume image_index 0? 
                // We'll use index 0 and warn.
                Arc::new(ImageNode {
                    image_index: 0, 
                    width: *width, 
                    height: *height 
                })
            }
        }
    }
}
