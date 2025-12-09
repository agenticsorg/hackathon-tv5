
import os
import requests
import json
from dotenv import load_dotenv

# Load env
load_dotenv(dotenv_path='../.env', override=True) # It might not be there, but we have the values in context

PINECONE_API_KEY = "pcsk_3rvhUN_2a8aevZympt95fzPrc5us1bo9gXZLUM1MRFd3uBEsfcJKHxGncoBrXbxK6t4SkB"
PINECONE_HOST = "media-knowledge-graph-cb340d9.svc.aped-4627-b74a.pinecone.io"

def test_rest_stats():
    host = PINECONE_HOST
    if not host.startswith('http'):
        host = f"https://{host}"
    
    url = f"{host}/describe_index_stats"
    
    headers = {
        "Api-Key": PINECONE_API_KEY,
        "Content-Type": "application/json",
        "X-Pinecone-API-Version": "2024-07"
    }
    
    try:
        print(f"Testing POST to {url}...")
        response = requests.post(url, headers=headers, json={})
        
        print(f"Status Code: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            data = response.json()
            print(f"Total Vectors: {data.get('total_vector_count')}")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    test_rest_stats()
