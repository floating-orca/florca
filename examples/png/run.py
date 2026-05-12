#!/usr/bin/env python3

import base64
import json
import os
import subprocess
import tempfile
from PIL import Image

# Change the working directory to the location of this script
os.chdir(os.path.dirname(os.path.abspath(__file__)))

# Deploy the workflow?
deploy = input("Deploy the workflow? (Y/n) ").strip().lower()
if deploy == "n":
    print("Skipping deployment.")
else:
    subprocess.run(["florca", "deploy", "--workflow-directory", ".", "png"], check=True)

# Run the workflow, capture the output, and decode the base64-encoded PNG data
stdout = subprocess.check_output(
    ["florca", "run", "--deployment-name", "png", "--wait", "--json"]
)
encoded = json.loads(stdout)["output"]
bytes = base64.b64decode(encoded)

# Save the PNG data to a temporary file and open it
with tempfile.NamedTemporaryFile(suffix=".png") as f:
    f.write(bytes)
    f.flush()
    Image.open(f.name).show()
