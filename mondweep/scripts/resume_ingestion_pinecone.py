
import json
import os
import time
import google.generativeai as genai
from pinecone import Pinecone, ServerlessSpec
from dotenv import load_dotenv

# Load env from parent directory (relative to this script)
script_dir = os.path.dirname(os.path.abspath(__file__))
env_path = os.path.join(script_dir, '../../.env')
load_dotenv(dotenv_path=env_path, override=True)

PINECONE_API_KEY = os.getenv("PINECONE_API_KEY", "").strip()
GOOGLE_API_KEY = os.getenv("GOOGLE_API_KEY", "").strip()

print(f"DEBUG: Loaded env from {env_path}")
print(f"DEBUG: Pinecone Key: {PINECONE_API_KEY[:10]}... (Length: {len(PINECONE_API_KEY)})" if PINECONE_API_KEY else "DEBUG: Pinecone Key: None")

if not PINECONE_API_KEY:
    print("❌ Missing PINECONE_API_KEY")
    exit(1)
if not GOOGLE_API_KEY:
    print("❌ Missing GOOGLE_API_KEY")
    exit(1)

# Configuration
INDEX_NAME = "media-knowledge-graph"
EMBEDDING_MODEL = "models/text-embedding-004"
BATCH_SIZE = 100
DATA_FILE = "data/full_knowledge_graph.json"
CHECKPOINT_FILE = "data/ingestion_checkpoint.json"
FAILED_LOG_FILE = "data/ingestion_failed.json"

# Initialize Clients
print("🔄 Initializing Clients...")
pc = Pinecone(api_key=PINECONE_API_KEY)
genai.configure(api_key=GOOGLE_API_KEY)

# Check Index
try:
    existing_indexes = [i.name for i in pc.list_indexes()]
    if INDEX_NAME not in existing_indexes:
        print(f"⚠️ Index {INDEX_NAME} not found. Creating...")
        pc.create_index(
            name=INDEX_NAME,
            dimension=768,
            metric="dotproduct",
            spec=ServerlessSpec(cloud="aws", region="us-east-1")
        )
except Exception as e:
    print(f"❌ Failed to verify/create index: {e}")
    exit(1)

index = pc.Index(INDEX_NAME)

# Load Checkpoint
last_id = None
if os.path.exists(CHECKPOINT_FILE):
    with open(CHECKPOINT_FILE, 'r') as f:
        try:
            ckpt = json.load(f)
            last_id = ckpt.get("lastId")
            print(f"🔄 Resuming from ID: {last_id}")
        except: pass

def save_checkpoint(last_processed_id):
    with open(CHECKPOINT_FILE, 'w') as f:
        json.dump({"lastId": str(last_processed_id), "timestamp": time.time()}, f)

def log_failure(item_id, title, error):
    entry = {"id": item_id, "title": title, "error": str(error)}
    existing = []
    if os.path.exists(FAILED_LOG_FILE):
        try:
            with open(FAILED_LOG_FILE, 'r') as f:
                existing = json.load(f)
        except: pass
    existing.append(entry)
    with open(FAILED_LOG_FILE, 'w') as f:
        json.dump(existing, f)


# Load Data using Streaming (ijson)
print(f"📖 Streaming data from {DATA_FILE}...")
# Use ijson to avoid loading 1GB+ into RAM which causes OOM/stalls
import ijson

# Processing Loop
# Processing Loop - Optimized with Parallelism
import concurrent.futures

BATCH_SIZE = 100
MAX_WORKERS = 15  # Adjust based on rate limits

batch_buffer = []
processed_count = 0
failed_count = 0
skip_mode = True if last_id else False

print(f"🚀 Starting Optimized Ingestion Loop (Workers: {MAX_WORKERS})...")

def generate_embedding(data_item):
    """
    Function to be run in parallel.
    Returns (item, vector_object) or (item, None) if failed.
    """
    movie, text = data_item
    retries = 3
    while retries > 0:
        try:
            result = genai.embed_content(
                model=EMBEDDING_MODEL,
                content=text,
                task_type="retrieval_document"
            )
            embedding = result['embedding']
            
            metadata = {
                "title": movie.get("title", "Unknown")[:1000],
                "year": movie.get("release_date", "")[:4] if movie.get("release_date") else "",
                "genres": str(movie.get("genres", ""))[:1000]
            }
            
            return movie, {
                "id": str(movie.get("id")),
                "values": embedding,
                "metadata": metadata
            }
        except Exception as e:
            if "429" in str(e):
                time.sleep(2 * (4 - retries)) # Exponential-ish backoff
            else:
                # print(f"⚠️ Error: {e}") 
                pass
            retries -= 1
            
    return movie, None


def process_batch(batch_items, index_obj):
    """
    Takes a list of raw movie objects, generates embeddings in parallel, and upserts.
    """
    # Prepare data for worker
    work_items = []
    for movie in batch_items:
        title = movie.get("title", "Unknown")
        overview = movie.get("overview", "")
        # Handle genres
        genres_list = movie.get("genres", [])
        if isinstance(genres_list, list):
            genres = ", ".join([str(g.get("name", "")) for g in genres_list if isinstance(g, dict)])
        else:
            genres = str(genres_list)
        
        # Mutate movie object temporarily to store clean genres for metadata (hacky but works)
        movie['genres'] = genres
        
        text = f"Title: {title}. Overview: {overview}. Genres: {genres}"
        work_items.append((movie, text))

    vectors_to_upsert = []
    failed_in_batch = 0
    
    with concurrent.futures.ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
        results = executor.map(generate_embedding, work_items)
        
    for movie, vector in results:
        if vector:
            vectors_to_upsert.append(vector)
        else:
            failed_in_batch += 1
            log_failure(movie.get('id'), movie.get('title'), "Embedding generation failed after retries")

    if vectors_to_upsert:
        try:
            index_obj.upsert(vectors=vectors_to_upsert)
            # Checkpoint with the ID of the LAST successfully upserted item in this batch
            # Ideally we checkpoint the last item of the input batch, assuming monotonic processing
            last_movie_in_batch = batch_items[-1]
            save_checkpoint(last_movie_in_batch.get("id"))
            # print(f"✅ Upserted batch of {len(vectors_to_upsert)}. Last ID: {last_movie_in_batch.get('id')}")
        except Exception as e:
            print(f"❌ Critical Batch Upsert Failed: {e}")
            # Consider dumping this batch to a retry file?
            
    return len(vectors_to_upsert), failed_in_batch


with open(DATA_FILE, 'rb') as f:
    movies_generator = ijson.items(f, 'data.movies.item')
    
    current_batch = []
    
    for movie in movies_generator:
        movie_id = str(movie.get("id"))
    
        # Skip logic
        if skip_mode:
            if movie_id == str(last_id):
                skip_mode = False
                print("✅ Found checkpoint. Resuming processing...")
            continue
            
        current_batch.append(movie)
        
        if len(current_batch) >= BATCH_SIZE:
             success, failed = process_batch(current_batch, index)
             processed_count += success
             failed_count += failed
             current_batch = []
             print(f"⏳ Processed {processed_count} items... (Failed: {failed_count})")

    # Flush final
    if current_batch:
        success, failed = process_batch(current_batch, index)
        processed_count += success
        failed_count += failed
        print("✅ Final batch upserted.")

print(f"🎉 Ingestion Complete. Processed: {processed_count}, Failed: {failed_count}")
