use ai_pdf_writer::core::template::Template;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Verifying Editor JSON export with Shapes...");

    // 1. Create a simulated Editor Export JSON with Shapes
    // We'll mimic what `toPDFCore` produces for:
    // - A Rectangle (Container)
    // - A Line (Thin Container)
    // - A Data URL Image (Circle) - *Wait, Rust engine needs to handle Data URL source or we mock it*
    
    // For this test, we'll use a placeholder path for the image to avoid base64 complexity in Rust source,
    // assuming the Editor export logic works. 
    // BUT we want to verify the STRUCTURAL compatibility.
    
    let editor_json = json!({
        "root": {
            "type": "Column",
            "children": [
                {
                    "type": "Canvas",
                    "width": 595.0,
                    "height": 842.0,
                    "children": [
                        // Rectangle
                        {
                            "type": "Container",
                            "style": {
                                "x": 50, "y": 50, "width": 100, "height": 100, "backgroundColor": "#ff0000"
                            },
                             "child": { "type": "Text", "content": "" }
                        },
                        // Line
                        {
                            "type": "Container",
                            "style": {
                                "x": 50, "y": 200, "width": 200, "height": 2, "backgroundColor": "#000000"
                            },
                             "child": { "type": "Text", "content": "" }
                        },
                        // Circle (exported as Image)
                         {
                            "type": "Image",
                            "src": "placeholder_circle.png", 
                            "style": {
                                "x": 300, "y": 50, "width": 100, "height": 100
                            },
                            "width": 100,
                            "height": 100
                        }
                    ]
                }
            ]
        },
        "styles": {},
        "manifest": { "name": "Shapes Verification" }
    });

    // 2. Serialize to string to simulate reading from file
    let json_str = serde_json::to_string_pretty(&editor_json)?;
    println!("Generated JSON:\n{}", json_str);

    // 3. Deserialize using Template::from_json
    // We expect this to execute without error if the schema matches
    match Template::from_json(&json_str) {
        Ok(template) => {
            println!("✅ Successfully verified JSON schema compatibility!");
            println!("Template Root Type: {:?}", template.root);
        },
        Err(e) => {
            eprintln!("❌ JSON Verification Failed: {}", e);
            let err: Box<dyn std::error::Error> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
            return Err(err);
        }
    }

    Ok(())
}
