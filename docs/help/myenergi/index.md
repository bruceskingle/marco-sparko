# myenergi

The myenergi module fetches data about myenergi devices, such as the zappi EV charger, from myenergi's servers. It is new, and for now it only shows what it fetches, to see what the data is like:

- **Days**, the totals of each device's records of each day, in kWh. **Fetch** gets the status of every device and the last few days of their records. Days are UTC days, and a day is fetched again until it has ended.
- **Cache Files**, every file it has fetched, as myenergi sent it.

[Logging in](login.md) explains what it needs to fetch your data.
