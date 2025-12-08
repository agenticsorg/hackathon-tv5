import os
import subprocess
from pathlib import Path


def main():
    project_root = Path(__file__).parent.parent
    os.chdir(project_root)
    os.environ.setdefault("KAGGLE_CONFIG_DIR", str(project_root / ".kaggle"))

    out_dir = project_root / "data/tmdb"
    out_dir.mkdir(parents=True, exist_ok=True)
    zip_path = out_dir / "tmdb-movies-dataset-2023-930k-movies.zip"

    print("Downloading TMDB 2024 dataset via Kaggle...")
    cmd = [
        "bash",
        "-lc",
        f"pip install -q kaggle && kaggle datasets download -d asaniczka/tmdb-movies-dataset-2023-930k-movies -p {out_dir} --force",
    ]
    code = subprocess.call(cmd)
    if code != 0:
        raise SystemExit("Kaggle download failed. Ensure KAGGLE_CONFIG_DIR and credentials are set.")

    # Unzip
    print("Extracting TMDB zip...")
    subprocess.check_call(["bash", "-lc", f"cd {out_dir} && unzip -o *.zip"])  # extracts TMDB_movie_dataset_v11.csv
    print("TMDB dataset ready in data/tmdb/")


if __name__ == "__main__":
    main()

