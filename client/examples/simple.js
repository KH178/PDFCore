
const { PDFCoreClient } = require('../sdk/dist/client');
const path = require('path');

async function run() {
    const client = new PDFCoreClient();
    const templatePath = path.join(__dirname, '../../examples/invoice/invoice_template_fixed_v2.pdfCoret');
    const outputPath = path.join(__dirname, 'output.pdf');

    console.log("Rendering invoice to", outputPath);
    
    // Sample data
    // Sample data matching layout.json
    const data = {
        invoice_number: "INV-2024-001",
        date: "2024-02-08",
        due_date: "2024-02-22",
        customer: {
            name: "Acme Corp",
            email: "billing@acme.com",
            address: "123 Business Rd, Tech City"
        },
        items: [
            { desc: "Consulting Services", qty: 10, price: 150.00, total: 1500.00 },
            { desc: "Software License", qty: 1, price: 499.00, total: 499.00 }
        ],
        subtotal: 1999.00,
        tax_rate: 10,
        tax: 199.90,
        grand_total: 2198.90
    };

    try {
        await client.render({
            template: templatePath,
            data: data,
            output: outputPath
        });
        console.log("Success!");
    } catch (e) {
        console.error("Error:", e);
    }
}

run();
