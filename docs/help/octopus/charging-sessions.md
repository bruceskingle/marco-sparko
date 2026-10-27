# Charging Sessions

Each time the car charged, between the [dates chosen](../date-range.md): when it started and ended, how long it took, how much energy was added at what average power, what it cost and the price per kWh.

- **Type** is how the session was charged, e.g. **Smart** when Octopus scheduled it, and **Public** for a public charger.
- **% Battery** is the energy added as a percentage of the battery size configured in Octopus, which is not the car's charge level.
- **Dispatches** are the times Octopus scheduled charging within the session.

Show them as a **Table**, **Graph** or **Trend**, see [using the graphs](../graphs.md). ![Categories](../icons/categories.svg) **Categories** chooses which types of session are shown.

![Reload](../icons/reload.svg) **Reload** downloads the sessions shown again, fetching the data from the server and overwriting the cached values.
