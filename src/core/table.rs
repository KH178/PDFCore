
#[derive(Debug, Clone)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub header: String,
    pub width: f64,
    pub align: TextAlign,
}

#[derive(Debug, Clone)]
pub struct TableSettings {
    pub padding: f64,
    pub border_width: f64,
    pub header_height: f64,
    pub cell_height: f64, // Min height
    pub font_size: f64,   // Cell font size
}

impl Default for TableSettings {
    fn default() -> Self {
        TableSettings {
            padding: 5.0,
            border_width: 1.0,
            header_height: 30.0,
            cell_height: 20.0,
            font_size: 10.0,
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
