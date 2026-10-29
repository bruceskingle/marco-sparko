# Capture Devices

Capture devices are small always-on ESP32 boards with a screen, such as a CYD, which fetch data while this computer
sleeps, e.g. the Powerwall's every 5 minutes, and show the latest on their screens. The app reads what they fetched when
it next runs. **Devices** in Home's page list shows them.

## Adopting a device

A new device has its own settings page in a browser, at its address followed by /config, which is where it is put on
your Wi-Fi. Once it is on the Wi-Fi, adopt it here:

1. Enter its IP address under **Add a Capture Device**, and click **Find**. The app shows what the device is.
2. Click **Show Pairing Code**. The device shows a 6 digit code on its screen for 2 minutes.
3. Type the code and click **Adopt**.

The screen also shows the start of the device's certificate, which should be the same as the **Certificate** the app
shows. If it is not, something else on the network may be pretending to be the device, so do not adopt it.

Once adopted, only this profile can change the device's settings: its own /config page only says that an app manages
it, unless it cannot join the Wi-Fi, when it makes its own Wi-Fi network for its settings page again, with only the
Wi-Fi settings on it.

## Managing a device

- **Settings** shows each of the device's features, with its settings. Change them and click **Save**, then
  **Restart Now**, as they take effect when the device restarts. Passwords are never shown, only whether they are set:
  type a new one to replace it, or **Clear Password** to remove an optional one.
- **Refresh** asks the device again what it is, and **Restart** restarts it.
- **Release** gives the device back to its own /config page, and removes it from this profile.
- **Forget** only removes it from this profile, e.g. if it no longer works. It stays adopted until its **BOOT** button
  is held down for 5 seconds, which releases it.

The app keeps a key for each device it adopts in the profile, which lets it change the device's settings, so keep the
profile's files private. A redacted copy of a profile does not include its devices.
