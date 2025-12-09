
import os
import json
from pinecone import Pinecone
from dotenv import load_dotenv

# Load env from parent directory (relative to this script)
script_dir = os.path.dirname(os.path.abspath(__file__))
env_path = os.path.join(script_dir, '../../.env')
load_dotenv(dotenv_path=env_path, override=True)

PINECONE_API_KEY = os.getenv("PINECONE_API_KEY", "").strip()
INDEX_NAME = "media-knowledge-graph"

if not PINECONE_API_KEY:
    print("Error: PINECONE_API_KEY not found")
    exit(1)

try:
    pc = Pinecone(api_key=PINECONE_API_KEY)
    index = pc.Index(INDEX_NAME)
    # The host is accessible via the index object properties in recent SDK versions, 
    # but let's print the describe_index result to be sure or just the host property.
    description = pc.describe_index(INDEX_NAME)
    print(f"HOST: {description.host}")
    
    stats = index.describe_index_stats()
    print(json.dumps(stats.to_dict(), indent=2))
except Exception as e:
    print(f"Error: {e}")
