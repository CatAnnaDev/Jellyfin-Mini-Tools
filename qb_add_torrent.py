import json
import requests

QB_API_URL = "http://localhost:8080/api/v2"
USERNAME = "admin"
PASSWORD = "adminadmin"
JSON_FILE_PATH = "/Users/anna/RustroverProjects/torrent_linker/anime.json"

def login_to_qbittorrent():
    login_url = f"{QB_API_URL}/auth/login"
    payload = {"username": USERNAME, "password": PASSWORD}
    session = requests.Session()
    response = session.post(login_url, data=payload)
    if response.status_code == 200 and response.text == "Ok.":
        print("[+] Connexion réussie à qBittorrent.")
        return session
    else:
        print("[-] Échec de la connexion à qBittorrent.")
        print("Réponse : ", response.text)
        exit()

def add_torrent_to_qbittorrent(session, torrent_path, save_path):
    upload_url = f"{QB_API_URL}/torrents/add"
    files = {"torrents": open(torrent_path, "rb")}
    data = {"savepath": save_path, "autoTMM": "false"}
    response = session.post(upload_url, files=files, data=data)
    if response.status_code == 200:
        print(f"[+] Torrent ajouté : {torrent_path} vers {save_path}")
    else:
        print(f"[-] Échec de l'ajout du torrent : {torrent_path}")
        print("Réponse : ", response.text)

def import_torrents_from_json():
    with open(JSON_FILE_PATH, "r") as json_file:
        data = json.load(json_file)
        if not data:
            print("[-] Aucun torrent trouvé dans le fichier JSON.")
            return

        session = login_to_qbittorrent()
        for torrent_path, save_path in data.items():
            add_torrent_to_qbittorrent(session, torrent_path, save_path)

if __name__ == "__main__":
    import_torrents_from_json()
