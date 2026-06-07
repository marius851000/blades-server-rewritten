# Blades Rewritten Server

This is a re‑implementation of the Blades server, as documented via reverse‑engineering (both binary analysis and packet capture).

## How to run:

1. Have a PostgreSQL database.
2. Have Rust installed.
3. Have the diesel cli installed.
4. Configuring the database:
   - Create a `.env` file with the `DATABASE_URL` being the PostgreSQL connection string, as expected by diesel.
   - Run `diesel migration run` to apply the database migrations.
5. Extracting game data (generate the `data` folder):
   - Use the unity asset ripper to extract the game data from the APK
   - Execute `scripts/data_parser/main.py`, setting the output file to `data/parsed.json`
   - Execute `scripts/generate_download_from_dump.py` (after fixing the path) with a capture that downloaded the full game (so the client can download it)
6. Configure mitmproxy:
   - Run `mitmweb --mode wireguard@51820 -s scripts/mitmproxy_script.py --set tls_version_client_min=UNBOUNDED` (adapt port as needed. You can use `--set web_port=...`). This will redirect HTTP request to port 8000.
   - On the mobile device, configure the wireguard tunnel (or whatever other way you prefer to capture HTTP/HTTPS traffic)
7. Android: build a patched APK that trust user‑installed certs:
   - generate an APK from the app and copy it to `build-app/source-package.apk`.
   - Run `build_patched_apk.sh`
   - Install the generated APK
8. Run the server (finally): Run `cargo run -- run --connection-string "<same as DATABASE_URL in .env>" --host 127.0.0.1 --port 8000 --static-data ./data --enet-listen-addr 127.0.0.1:8001 --enet-public-addr <machine network/public IP>:8001`

## Some SQL notes:
Remember to use FOR NO KEY UPDATE in your select if you’re gonna write back the modified result (obviously in the same transaction). Take care of deadlock too! (the for FOR NO KEY UPDATE should handle that in most cases).
