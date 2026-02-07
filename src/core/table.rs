use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub header: String,
    pub width: f64,
    #[serde(default = "default_text_align")]
    pub align: TextAlign,
    #[serde(default)]
    pub field: Option<String>, // For data binding
}

fn default_text_align() -> TextAlign {
    TextAlign::Left
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSettings {
    #[serde(default = "default_padding")]
    pub padding: f64,
    #[serde(default = "default_border")]
    pub border_width: f64,
    #[serde(default = "default_header_height")]
    pub header_height: f64,
    #[serde(default = "default_cell_height")]
    pub cell_height: f64,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
}

fn default_padding() -> f64 { 5.0 }
fn default_border() -> f64 { 1.0 }
fn default_header_height() -> f64 { 30.0 }
fn default_cell_height() -> f64 { 20.0 }
fn default_font_size() -> f64 { 10.0 }

impl Default for TableSettings {
    fn default() -> Self {
        TableSettings {
            padding: default_padding(),
            border_width: default_border(),
            header_height: default_header_height(),
            cell_height: default_cell_height(),
            font_size: default_font_size(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<Vec<String>>,
    pub settings: TableSettings,
}

impl Table {
    pub fn new(columns: Vec<TableColumn>) -> Self {
        Table {
            columns,
            rows: Vec::new(),
            settings: TableSettings::default(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        if row.len() == self.columns.len() {
            self.rows.push(row);
        } else {
            // Panic or ignore? For core, maybe we should extend or truncate, 
            // but for simplicity let's just push what we have or pad.
            // Let's ensure strict length matching for v1.
            let mut r = row;
            r.resize(self.columns.len(), String::new());
            self.rows.push(r);
        }
    }
}
