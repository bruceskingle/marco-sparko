# Marco Sparko Capture Device

Firmware for an ESP32 which polls what Marco Sparko needs while the computer it runs on sleeps, built on
[sparko-embedded-std](../../sparko-embedded-std), which must be checked out next to this repo. For now it only polls
the Tesla gateway, and only runs on a CYD (ESP32-2432S028R), whose screen shows a clock with the latest numbers below.

## Building and flashing

From this directory, with the ESP Rust toolchain installed (see sparko-embedded-std):

- `cargo build --release`
- `cargo run --release` flashes a CYD plugged in by USB, and shows its log.
- `python3 log.py` shows its log without restarting it (needs pyserial). Not `espflash monitor`, which restarts
  it, or with `--no-reset` leaves it stopped in its bootloader.

In VS Code, Terminal > Run Task has **Device: Flash** and **Device: Log**.

## Setting it up

The first time it starts, it makes its own WiFi network: join it and the config page opens. Enter your WiFi details,
then in the TeslaCapture section:

- **tesla_host**: the gateway's IP address.
- **tesla_email** and **tesla_password**: the gateway's customer login, as in the app. A password the gateway
  rejects is removed, so it is not tried again. Repeated failures lock out the customer login, for Home Assistant
  too.
- **tesla_schedule**: optional, when to poll, a cron spec with seconds first, `0 */5 * * * *` (every 5 minutes) if
  empty.
- **tesla_capacity**: optional, how many samples to keep, 144 (12 hours at 5 minutes) if empty. The log shows the free
  heap after each poll.

The config page never shows the password, and it is not written to the log: leave it empty to keep the saved one.
The page has no login though, so anyone on the network can change the settings, including tesla_host, which would
send the password to another address. TODO: a login for the config page.

Then enter the device's IP address on the app's Powerwall page.

## What the app reads

`GET /capture/tesla/records?since=<RFC 3339 time>` returns `{"device": {...}, "records": [...]}`:

- `device`: its version, when and why it last started (`resetReason`, e.g. `Brownout`), how many records it keeps,
  and its free heap.
- `records`: those after `since`, or all of them, oldest first, each as the app's own Tesla snapshots, with `slot`,
  `requested`, `received`, `capturedBy` and the `responses` of `meters/aggregates`, `system_status/soe`,
  `system_status/grid_status` and `operation`, keeping only the fields the app uses.

The records are only kept in RAM, so they are lost when the device restarts.

The same records are at `https://<device>/api/capture/tesla/records`, on the device's HTTPS API, which also has
`/api/info`, `/api/config` and `/api/restart`, see sparko-embedded-std's CLAUDE.md. Its certificate is the device's
own, so use `curl -k` for now; the app will pin its fingerprint, from `/api/info`, when it adopts the device.
