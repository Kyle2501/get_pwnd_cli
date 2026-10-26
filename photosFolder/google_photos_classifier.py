import os
import glob
import json
import base64
import requests
from google_auth_oauthlib.flow import InstalledAppFlow
from google.auth.transport.requests import AuthorizedSession
from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials

# Scopes restricted to app-created data boundary rules
SCOPES = [
    'https://www.googleapis.com/auth/photoslibrary.appendonly',
    'https://www.googleapis.com/auth/photoslibrary.edit.appcreateddata'
]

CREDENTIALS_FILE = 'client_secret.json'
TOKEN_FILE = 'token_append.json'
LOCAL_IMAGE_DIR = './local_photos_to_upload/*'
OLLAMA_API_URL = 'http://localhost:11434/api/generate'
OLLAMA_MODEL = 'llama3.2-vision'

def get_authenticated_session():
    """Handles OAuth 2.0 user authentication flow and token caching."""
    creds = None
    if os.path.exists(TOKEN_FILE):
        creds = Credentials.from_authorized_user_file(TOKEN_FILE, SCOPES)
    
    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            creds.refresh(Request())
        else:
            flow = InstalledAppFlow.from_client_secrets_file(CREDENTIALS_FILE, SCOPES)
            creds = flow.run_local_server(port=8080)
        with open(TOKEN_FILE, 'w') as token:
            token.write(creds.to_json())
            
    return AuthorizedSession(creds)

def classify_image_with_ollama(image_path):
    """Passes local image to local Ollama vision model to extract category and caption."""
    with open(image_path, "rb") as img_file:
        encoded_image = base64.b64encode(img_file.read()).decode('utf-8')
        
    prompt = (
        "Analyze this image and categorize it strictly into one of these exact folders: "
        "[Receipts, Street Art, Hardware, Nature, Documents, Pets, Architecture, Uncategorized]. "
        "Respond in valid JSON format with keys: 'category' (string) and 'caption' (string description)."
    )
    
    payload = {
        "model": OLLAMA_MODEL,
        "prompt": prompt,
        "images": [encoded_image],
        "stream": False,
        "format": "json"
    }
    
    try:
        response = requests.post(OLLAMA_API_URL, json=payload, timeout=60)
        result = response.json()
        data = json.loads(result.get('response', '{}'))
        return data.get('category', 'Uncategorized'), data.get('caption', 'Uploaded via automated pipeline')
    except Exception as e:
        print(f"[-] Vision classification failed for {image_path}: {e}")
        return "Uncategorized", "Automated upload"

def get_or_create_app_album(session, album_title):
    """
    Finds an app-created album by title or creates a new one. 
    Note: The API only lists albums created by your specific OAuth client ID.
    """
    albums_url = 'https://photoslibrary.googleapis.com/v1/albums'
    
    # Paginate through existing app albums to find a match
    page_token = None
    while True:
        params = {'pageSize': 50}
        if page_token:
            params['pageToken'] = page_token
            
        res = session.get(albums_url, params=params)
        if res.status_code == 200:
            data = res.json()
            for album in data.get('albums', []):
                if album.get('title') == album_title:
                    return album.get('id')
            page_token = data.get('nextPageToken')
            if not page_token:
                break
        else:
            break

    # If album doesn't exist, create it
    create_res = session.post(albums_url, json={'album': {'title': album_title}})
    if create_res.status_code == 200:
        return create_res.json().get('id')
    
    print(f"[-] Error creating album '{album_title}': {create_res.text}")
    return None

def upload_and_categorize():
    session = get_authenticated_session()
    image_paths = glob.glob(LOCAL_IMAGE_DIR)
    
    if not image_paths:
        print(f"[-] No local images found matching pattern: {LOCAL_IMAGE_DIR}")
        return

    print(f"[+] Found {len(image_paths)} local image(s) to process.")

    for path in image_paths:
        if not path.lower().endswith(('.png', '.jpg', '.jpeg', '.webp', '.heic')):
            continue
            
        print(f"\n[•] Processing: {path}")
        
        # 1. Classify image locally
        category, caption = classify_image_with_ollama(path)
        print(f"    ↳ Classified as: [{category}] | Caption: {caption}")
        
        # 2. Get or create target Google Photos album
        album_id = get_or_create_app_album(session, category)
        
        # 3. Step 1 of upload: Send raw binary bytes to obtain uploadToken
        with open(path, "rb") as f:
            image_bytes = f.read()
            
        upload_headers = {'Content-type': 'application/octet-stream'}
        upload_res = session.post(
            'https://photoslibrary.googleapis.com/v1/uploads', 
            headers=upload_headers, 
            data=image_bytes
        )
        
        if upload_res.status_code != 200:
            print(f"[-] Failed uploading bytes for {path}: {upload_res.text}")
            continue
            
        upload_token = upload_res.text
        
        # 4. Step 2 of upload: Commit item to library and assign to album via batchCreate
        media_item_payload = {
            "albumId": album_id,
            "newMediaItems": [
                {
                    "description": caption,
                    "simpleMediaItem": {
                        "uploadToken": upload_token,
                        "fileName": os.path.basename(path)
                    }
                }
            ]
        }
        
        create_res = session.post(
            'https://photoslibrary.googleapis.com/v1/mediaItems:batchCreate',
            headers={'Content-Type': 'application/json'},
            json=media_item_payload
        )
        
        if create_res.status_code == 200:
            res_json = create_res.json()
            result_status = res_json.get('newMediaItemResults', [{}])[0].get('status', {}).get('message', 'Success')
            print(f"    ✔ Successfully added to Google Photos album '{category}' ({result_status})")
        else:
            print(f"    [-] Failed finalizing media item for {path}: {create_res.text}")

if __name__ == '__main__':
    upload_and_categorize()
