const fs = require('fs');

// Minimal 1x1 Red Pixel PNG
const base64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';
const buffer = Buffer.from(base64, 'base64');

fs.writeFileSync('logo.png', buffer);
console.log('Created logo.png (1x1 pixel)');
