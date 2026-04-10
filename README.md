# hrmon

This is our summative project for CST3115 & CST3119. It is a combination of a heart monitor and respiratory monitor for use in rough estimation of stress levels and a mobile app to display the data. This repository contains all code involved.

## back

rocket.rs backend. For assignment demonstration, this will be hosted on an external Caddy instance. Keeps track of latest 60 heart rate updates from user. Naive authentication. Uses first 10 updates for calibration of nominal resting heart rate. Not intended for a production implementation; very basic. Basic API docs, see [routes.rs](https://github.com/jack-avery/hrmon/blob/main/back/src/routes.rs) for advanced documentation and sample usage:

### `POST /flush_info`
Flush stored information and begin a new session from scratch.

### `POST /info`
Push info to the in-memory db.
> It is assumed that this is the average heart rate measured for the given epoch second.

### `GET /info`
Get info from the in-memory db, including the last 60 measurements, the calibrated average, and current projected user status.
