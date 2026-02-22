const https = require('https');
const fs = require('fs');
const file = fs.createWriteStream("examples/browser/assets/Roboto-Regular.ttf");
https.get("https://github.com/google/fonts/raw/main/apache/roboto/static/Roboto-Regular.ttf", function(response) {
  response.pipe(file);
  file.on('finish', function() {
    file.close(() => console.log('Download complete.'));
  });
}).on('error', function(err) {
  fs.unlink("examples/browser/assets/Roboto-Regular.ttf", () => {}); // Delete the file async. (But we don't check the result)
  console.error("Error downloading file: " + err.message);
});
