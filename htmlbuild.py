#!/usr/bin/env python3
import base64
import os
import re

SRC_DIR = "src"
OUTPUT_FILE = "secret_game.html"

# Read CSS
with open(os.path.join(SRC_DIR, "ui/styles.css"), "r") as f:
    css_content = f.read()

# Collect JS modules and convert to Data URIs for the inlined importmap
import_map = {"imports": {}}
for root, _, files in os.walk(SRC_DIR):
    for file in files:
        if file.endswith(".js"):
            full_path = os.path.join(root, file)
            rel_path = os.path.relpath(full_path, SRC_DIR).replace("\\", "/")
            module_key = f"./src/{rel_path}"
            
            with open(full_path, "r") as f:
                code = f.read()
            
            encoded = base64.b64encode(code.encode("utf-8")).decode("utf-8")
            import_map["imports"][module_key] = f"data:text/javascript;base64,{encoded}"

# Direct import for main entrypoint
main_data_uri = import_map["imports"]["./src/main.js"]

template = f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Secret Game</title>
  <style>
{css_content}
  </style>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js"></script>
  <script type="importmap">
    {str(import_map).replace("'", '"')}
  </script>
</head>
<body>
  <div id="canvas-container"></div>
  <div class="scanlines"></div>
  <script type="module">
    import {{ initGame }} from './src/main.js';
    initGame(window.THREE);
  </script>
</body>
</html>
"""

with open(OUTPUT_FILE, "w") as f:
    f.write(template)

print(f"Bake complete: {OUTPUT_FILE} created.")
