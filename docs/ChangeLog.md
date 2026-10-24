# Release 0.4.4

## New
- **Drill down by double clicking.** On the Graph view, double click a bar to see it in more detail: a month shows its
  weeks, a week its days, and a day its individual half hour readings.
- **Zoom by dragging.** Click and drag across bars in the graph to show just those bars.
- **Right click menu on the graph.** Right click a bar for:
  - **Zoom Out**, back to what was shown before the last drill down or zoom.
  - **Previous** and **Next**, the same period before or after the one shown.
  - **Export this bar**, the readings in that bar as CSV or JSON.
  - **Reload this bar** and **Reload this month**, which fetch the data again from Octopus, in case it has been
    corrected since it was downloaded. While a reload runs the graph stays on screen, with a dialog showing its
    progress and a Cancel button.
- **Quick date ranges.** The date range dialog has buttons for This Week, 7 Days, This Month and 30 Days, and for the
  period before each of them: Last Week, Prev 7 Days, Last Month and Prev 30 Days.
- **Previous and next period buttons** either side of the date range, to step back or forward by the same period. The
  next button is greyed out when the period already reaches the present.

## Improved
- The labels along the bottom of the graph are centred on their bars for days, weeks and months, and start at the
  start of the day for rate periods and half hour readings. A label which would not fit is left out.
- Data which has already been downloaded is no longer downloaded again just because only part of a month was looked
  at.
- The browser style right click menu (with Reload and other options which could leave the app in a broken state) no
  longer appears.

# Release 0.4.3

## New
- **Measurements.** A new Measurements page shows the consumption and costs of each meter point:
  - As a **table** of readings, a **graph** of stacked bars, or a **trend** graph which compares years, with a line
    for each year.
  - In **cost** or **units**, for each half hour, for each period at the same rates, or totalled by day, week or month.
  - Broken down into the rates Octopus reports, e.g. Home and EV usage at the low and standard rates. The Categories
    menu chooses which are shown, their colours and line styles, and can show a total instead.
  - The trend graph can show 7 or 30 day moving averages, or the cumulative total for each year. Its total is shown by
    default.
  - For any date and time range, with progress shown while data is downloaded, and a Cancel button.
  - **Export** of what is shown as CSV or JSON.
- **Properties.** A new Properties page shows each property's electricity and gas meter points, meters and tariffs.
- **Profiles:**
  - Create a new profile from the profile menu.
  - Each profile other than the one in use has a **…** menu with **Size…**, which shows how much disk space its
    downloaded data uses, and **Delete…**, which deletes it. Its login tokens are deleted immediately and its
    downloaded data is moved to the Trash or Recycle Bin.
  - The profile and downloaded data can be kept in another directory, e.g. on another drive, instead of your home
    directory. See [Profiles](profiles.md#moving-the-data-directory). The directory in use is shown on the Home page.
- The window opens at the size and position it had when it was last closed.

## Improved
- Bills downloaded since the last time they were viewed are highlighted.
- Graph axis labels show just the numbers, with the units (e.g. kwh/day) shown once above them.
- When Octopus rejects a request, e.g. when logging in, the message shown includes the reason it gave.
- If the saved login can no longer be used, the login page says why.
- Network errors and rate limits while downloading are retried automatically.

## Fixed
- Some Windows computers could not connect to Octopus at all, failing with "403 Forbidden". Marco Sparko now connects
  in the same way on every computer, whatever the version of Windows.
- Login details saved on Windows are now reliably readable only by you.
