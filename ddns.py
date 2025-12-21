import os
import time
import requests

IP_FILE = "/data/last_ip.txt"

API_KEY = os.environ["PORKBUN_API_KEY"]
SECRET = os.environ["PORKBUN_SECRET_API_KEY"]
DOMAIN = os.environ["DOMAIN"]
SUB = os.environ["SUBDOMAIN"]
INTERVAL = int(os.environ.get("CHECK_INTERVAL", 300))

IP_URL = os.environ.get("IP_CHECK_URL", "https://api.ipify.org")

def get_public_ip():
    r = requests.get(IP_URL, timeout=10)
    r.raise_for_status()
    return r.text.strip()

def update_porkbun(ip):
    url = f"https://api.porkbun.com/api/json/v3/dns/editByNameType/{DOMAIN}/A/{SUB}"
    payload = {
        "apikey": API_KEY,
        "secretapikey": SECRET,
        "content": ip,
        "ttl": "300"
    }
    r = requests.post(url, json=payload, timeout=10)
    r.raise_for_status()

def main():
    last_ip = None
    if os.path.exists(IP_FILE):
        last_ip = open(IP_FILE).read().strip()

    while True:
        try:
            ip = get_public_ip()
            if ip != last_ip:
                update_porkbun(ip)
                os.makedirs("/data", exist_ok=True)
                with open(IP_FILE, "w") as f:
                    f.write(ip)
                last_ip = ip
                print("IP updated:", ip)
            else:
                print("No change:", ip)
        except Exception as e:
            print("Error:", e)

        time.sleep(INTERVAL)

if __name__ == "__main__":
    main()
