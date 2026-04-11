# hrmon

This is our summative project for CST3115 & CST3119. It is a heart monitor for use in rough estimation of stress levels and a mobile app to display the data. This repository contains all code involved.

For running, see [running](#running).

## pi

Raspberry Pi 4 running stock Raspbian. Requires internet connection.

## front

Android 8.1 application developed with Kotlin. Running this on legitimate hardware has not been attempted; use Android Studio.

## back

rocket.rs backend. For assignment demonstration, this will be hosted on an external Caddy instance. Keeps track of latest 60 heart rate updates from user. Naive authentication. Uses first 10 updates for calibration of nominal resting heart rate. Not intended for a production implementation; very basic. Basic API docs, see [routes.rs](https://github.com/jack-avery/hrmon/blob/main/back/src/routes.rs) for advanced documentation and sample usage:

### `POST /flush_info`
Flush stored information and begin a new session from scratch.

### `POST /info`
Push info to the in-memory db.
> It is assumed that this is the average heart rate measured for the given epoch second.

### `GET /info`
Get info from the in-memory db, including the last 60 measurements, the calibrated average, and current projected user status.

## running

### Host the back-end
Without Docker and Caddy **(no TLS)**:
1. Install `rustup`.
2. Install the latest stable Rust toolchain: `rustup default stable`
3. Run the back-end: `cargo run`

With Docker and Caddy:
1. Install Docker, `docker-compose-v2` (or just Docker Compose on some package management systems), `docker buildx`, and `rustup`.
2. Install the latest stable Rust toolchain: `rustup default stable`
3. Compile the back-end application: `cd back && cargo build --release`
4. Create the Docker image: `docker buildx build -t back .`
5. Create the docker-compose.yml. A sample:
```yml
volumes:
  caddy-config:
  caddy-data:

networks:
  caddy:

services:
  caddy:
    container_name: caddy
    image: caddy:2.10.0-alpine # or newer, if desired
    ports:
      - '80:80/tcp'
      - '443:443/tcp'
      - '443:443/udp'
    networks:
      - caddy
    restart: unless-stopped
    volumes:
      - './Caddyfile:/etc/caddy/Caddyfile:ro'
      - caddy-config:/config
      - caddy-data:/data

  back:
    image: back
    container_name: back
    networks:
      - caddy
```
6. Create the Caddyfile. A sample:
```caddyfile
{
    # for TLS
    email my.email@example.com
}

example.com {
    tls internal
    handle_path /api/* {
        reverse_proxy back:8000
    }
}
```
7. Run the back-end: `docker compose up -d`.

### Run the Sensor
1. Attach the MAX30102 Pulse Oximeter to the Raspberry Pi.
2. Create a Python virtual environment: `python3 -m venv .venv`
3. Activate it by sourcing the activation script into your shell: `. .venv/bin/activate`
4. Install the Python requirements: `pip install -r pi/requirements.txt`
5. Modify `pi/sensor_reader.py`, line 17, `API_BASE_URL` to point to your back-end.
6. Run the sensor code: `./pi/sensor_reader.py`

### Run the Mobile App
1. We did not test running this on real hardware, so you should install [Android Studio](https://developer.android.com/studio).
2. You should be able to open `front` as a standard project.
3. Create a Google Pixel 2 emulated phone.
4. Modify `com.example.mobilestressmonitor.Constants.BASE_URL` to point to your back-end.
5. Run the project.
