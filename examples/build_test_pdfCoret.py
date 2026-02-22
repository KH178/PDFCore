import json
import zipfile

data = json.load(open('examples/comprehensive_test.json'))

layout = {
    'root': data['root'],
    'header': data.get('header'),
    'footer': data.get('footer'),
    'settings': data.get('settings')
}

styles = data.get('styles', {})
manifest = data.get('manifest', {})

with zipfile.ZipFile('examples/comprehensive_test.pdfCoret', 'w') as zf:
    zf.writestr('layout.json', json.dumps(layout, indent=2))
    zf.writestr('styles.json', json.dumps(styles, indent=2))
    zf.writestr('manifest.json', json.dumps(manifest, indent=2))

print("Created comprehensive_test.pdfCoret")
