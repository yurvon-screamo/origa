from pathlib import Path

import requests
from safetensors.torch import save_file
from ultralytics import YOLO

pt_uri = "https://huggingface.co/malaysia-ai/YOLOv8X-DocLayNet-Full-1024-42/resolve/main/weights/best.pt?download=true"

# Define directories
yolo_dir = Path("origa_ui/public/yolo")
yolo_dir.mkdir(parents=True, exist_ok=True)

pt_path = yolo_dir / "best.pt"
safetensors_path = yolo_dir / "layout.safetensors"

print("Downloading model...")
response = requests.get(pt_uri)
with open(pt_path, "wb") as f:
    f.write(response.content)

model = YOLO(pt_path)
print("Classes:", model.names)
print("nc (number of classes):", len(model.names))

state_dict = model.model.state_dict()


def rename_key(key: str) -> str:
    replacements = {
        "model.0.": "net.b1.0.",
        "model.1.": "net.b1.1.",
        "model.2.m.": "net.b2.0.bottleneck.",
        "model.2.": "net.b2.0.",
        "model.3.": "net.b2.1.",
        "model.4.m.": "net.b2.2.bottleneck.",
        "model.4.": "net.b2.2.",
        "model.5.": "net.b3.0.",
        "model.6.m.": "net.b3.1.bottleneck.",
        "model.6.": "net.b3.1.",
        "model.7.": "net.b4.0.",
        "model.8.m.": "net.b4.1.bottleneck.",
        "model.8.": "net.b4.1.",
        "model.9.": "net.b5.0.",
        "model.12.m.": "fpn.n1.bottleneck.",
        "model.12.": "fpn.n1.",
        "model.15.m.": "fpn.n2.bottleneck.",
        "model.15.": "fpn.n2.",
        "model.16.": "fpn.n3.",
        "model.18.m.": "fpn.n4.bottleneck.",
        "model.18.": "fpn.n4.",
        "model.19.": "fpn.n5.",
        "model.21.m.": "fpn.n6.bottleneck.",
        "model.21.": "fpn.n6.",
        "model.22.": "head.",
    }

    for old, new in replacements.items():
        if key.startswith(old):
            key = key.replace(old, new, 1)
            break

    return key


new_state_dict = {
    rename_key(k): v
    for k, v in state_dict.items()
    if "anchors" not in k and "strides" not in k
}  # remove extra if any

save_file(new_state_dict, safetensors_path)
