import os
import sys
import subprocess


def main():
    target_dir = os.environ.get("BASE_MODEL_DIR", "models/base-minilm")
    os.makedirs(target_dir, exist_ok=True)
    # Prefer sentence-transformers quick download path
    code = subprocess.call(
        [
            sys.executable,
            "-c",
            "from sentence_transformers import SentenceTransformer; SentenceTransformer('sentence-transformers/all-MiniLM-L6-v2').save('{}')".format(
                target_dir
            ),
        ]
    )
    if code != 0:
        print("Falling back to huggingface-cli download...")
        code = subprocess.call(
            [
                "bash",
                "-lc",
                f"huggingface-cli download sentence-transformers/all-MiniLM-L6-v2 --local-dir {target_dir}",
            ]
        )
    if code == 0:
        print(f"Model downloaded to {target_dir}")
    else:
        print("Failed to download model. Ensure git-lfs and huggingface-cli are available.")
        sys.exit(1)


if __name__ == "__main__":
    main()

