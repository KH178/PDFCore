export const PDF_TEMPLATES = {
  blank: {
    "root": {
      "type": "Column",
      "children": []
    },
    "styles": {},
    "manifest": { "name": "Blank Document" }
  },
  
  invoice: {
    "root": {
      "type": "Column",
      "children": [
        {
          "type": "Text",
          "content": "INVOICE",
          "style": { "fontSize": "32px", "fontWeight": "bold", "color": "#1e293b", "y": 50, "x": 50 }
        },
        {
          "type": "Text",
          "content": "Invoice # INV-2026-001\nDate: Oct 24, 2026",
          "style": { "fontSize": "12px", "color": "#64748b", "y": 90, "x": 50 }
        },
        {
          "type": "Table",
          "columns": [
            { "header": "Description", "width": 300 },
            { "header": "Hrs/Qty", "width": 80 },
            { "header": "Rate", "width": 80 },
            { "header": "Amount", "width": 100 }
          ],
          "rows": [
            ["PDF Engine Development", "40", "$150", "$6000"],
            ["Visual Editor UI", "20", "$120", "$2400"],
            ["WASM Integration", "10", "$150", "$1500"]
          ],
          "style": { "y": 150, "x": 50 },
          "settings": {
            "header_bg": { "r": 241/255, "g": 245/255, "b": 249/255 },
            "header_color": { "r": 15/255, "g": 23/255, "b": 42/255 },
            "border_width": 1,
            "border_color": { "r": 226/255, "g": 232/255, "b": 240/255 },
            "padding": 12,
            "striped": false,
            "alternate_row_color": { "r": 255/255, "g": 255/255, "b": 255/255 }
          }
        },
        {
           "type": "Text",
           "content": "Total Due: $9,900.00",
           "style": { "fontSize": "18px", "fontWeight": "bold", "color": "#1e293b", "y": 300, "x": 350 }
        }
      ]
    },
    "styles": {},
    "manifest": { "name": "Standard Invoice" }
  },

  comprehensive: {
    // We can embed the comprehensive_test.json structure here, 
    // or keep it simple for the editor drag-and-drop representation.
    "root": {
      "type": "Column",
      "children": [
        {
          "type": "Container",
          "style": { "x": 50, "y": 50, "width": 495, "height": 100, "backgroundColor": "#f8fafc", "borderWidth": "2px", "borderColor": "#cbd5e1", "borderRadius": "8px" },
          "child": { "type": "Text", "content": "Elaborate PDF Layout Test\nThis template demonstrates nested containers, shapes, and tables." }
        },
        {
          "type": "Circle",
          "style": { "x": 50, "y": 180, "width": 80, "height": 80, "backgroundColor": "#3b82f6" }
        },
        {
          "type": "Container",
          "style": { "x": 150, "y": 180, "width": 120, "height": 80, "backgroundColor": "#10b981", "borderRadius": "16px" }
        },
        {
          "type": "Line",
          "style": { "x": 50, "y": 280, "width": 495, "height": 2, "backgroundColor": "#334155" }
        },
        {
           "type": "Table",
           "columns": [{"header": "ID", "width": 100}, {"header": "Status", "width": 395}],
           "rows": [["1001", "Success"], ["1002", "Pending"], ["1003", "Failed"]],
           "style": { "x": 50, "y": 300 }
        }
      ]
    },
    "styles": {},
    "manifest": { "name": "Comprehensive Template" }
  }
};
