[< Octopus Home](index.md)

# Observations about the consumption query on ElectricityMeterType

The Meters page fetches half hourly readings with the `meterConsumption` query in
[meter.graphql](../../graphql/octopus/meter.graphql), which reads the `consumption` field of a meter node:

```gql
consumption(first: $first, grouping: $grouping, startAt: $startAt, timezone: $timezone, after: $after)
```

These observations come from the verbose trace of a fetch of an electricity meter with
`grouping: HALF_HOUR` and `startAt: 2025-01-01T00:00:00Z`, made in October 2026.

## There is no end time

The field takes a `startAt` but no end, so every request returns readings from `startAt` up to the present. A client
which only wants one month cannot ask for just that month, it has to page until it has what it wants and then stop.

## The page size is limited to 100

`first` can be at most 100, the server rejects larger values.

## The cursor is an offset

The cursors are base64 encodings of `arrayconnection:N`, where N is the index of the row in the result, e.g.
`YXJyYXljb25uZWN0aW9uOjQ5` is `arrayconnection:49`. Over a whole fetch they counted up from 0 with no gaps.

This is the default cursor of Graphene (the Python GraphQL library) when it paginates a list, which suggests that for
each page the server builds the whole list of readings from `startAt` to now and returns the slice from N + 1. That
would explain why requests with an early `startAt` are slow and can end in a gateway timeout: every page of a fetch from
January 2025 rebuilds 21 months of readings.

## The order of the readings changes on every request

The readings are not returned in time order, and the order is not the same from one request to the next:

- Only 20 of 262 pages were in time order, even within a page the order jumps back, e.g. 24 May, 26 May, 25 May.
- The rows come in runs which each move forwards through months of data before jumping back towards `startAt`, e.g.
  rows 0 to 1399 ran from 9 January 2025 to 29 August 2026, then rows 1649 to 2449 from 2 January to 30 July 2025.
- 25 of the 31 jumps back of more than 30 days fell exactly on a page boundary. If the server paged through one fixed
  list the jumps would fall anywhere, so the list is being built in a different order for each request.

## So readings are duplicated and others are never returned

Paging by offset through a list whose order changes between requests returns some rows more than once and never returns
others. A complete pass from `startAt` 1 January 2025 to the end of the data took 633 pages:

| | |
|---|---|
| Rows returned | 31,585 |
| Distinct half hours among them | 20,071 |
| Half hours of January 2025 returned | 1,026 of 1,488 |

Every duplicate had the same value as the original. The half hours which were never returned are not missing on the
server, they are simply never handed out by this way of paging, and fetching again returns a different incomplete set.

The cached data shows the effect: each month of 2025 is missing 25% to 45% of its half hours. Zero readings were not
dropped, they are stored as `"0"` because decimals are normalised when they are read.

## Consequences for fetching

- A fetch cannot tell whether it has every reading for a period from the order of the readings, a reading for the
  start of the period may arrive after readings from months later.
- A fetch which waits until a period is complete will page all the way to the end of the data, and the period may
  still be incomplete.
- Making requests for shorter periods does not help, because each request still covers everything from its `startAt`
  to now.

The fetching code now retries transient errors with the same cursor and saves what it has received if a fetch fails,
see [monthly_fixed_slot.rs](../../src/data_set/monthly_fixed_slot.rs), but it cannot make this query return a complete
set of readings.

# An Alternative is the measurements query on PropertyType

The Meters page fetches readings with the `homeAndEvConsumptionBreakdown` query, also in
[meter.graphql](../../graphql/octopus/meter.graphql), which reads the `measurements` field of a property, filtered by
the meter point:

```gql
measurements(first: $first, utilityFilters: $utilityFilters, startOn: $startOn, endOn: $endOn,
             startAt: $startAt, endAt: $endAt, timezone: $timezone, after: $cursor)
```

with an electricity filter giving the MPAN (`marketSupplyPointId`), `readingFrequencyType: RAW_INTERVAL` and
`readingDirection: CONSUMPTION`.

## The readings are the same

For the import meter the total units of each measurement,
adding up all of its rates, are exactly the same as the consumption query's value for the same half hour. 5,429 half
hours from April to October 2026 were in both caches with no differences at all, e.g. July 2026 is 1,069.198 kWh in
both. The older measurements in the cache are in an earlier storage format, so there was nothing to compare before
April 2026.

## Compared with the consumption query

| | consumption on a meter | measurements on a property |
|---|---|---|
| Identifies the meter by | meter node id | MPAN, in a utility filter |
| End time | none, always up to now | `endAt` (or `endOn`) |
| Page size | at most 100 | the app uses 1500 |
| Order and completeness | changes between requests, rows are duplicated and missed | each month in the cache from July to September 2026 has every half hour |
| Values | units only | units, cost, rate and the standing charge, split by rate (e.g. Home Low Rate, EV Standard Rate), or by time of use bucket in older readings (OFF_PEAK, STANDARD_RATE) |
| Aggregation | `grouping`: QUARTER_HOUR, HALF_HOUR, HOUR, DAY, WEEK, MONTH, QUARTER | `readingFrequencyType`: RAW_INTERVAL, FIVE_MIN_INTERVAL to HOUR_INTERVAL, DAY_INTERVAL, WEEK_INTERVAL and longer |
| Direction | separate meters for import and export | `readingDirection`: CONSUMPTION or GENERATION |
| Gas | a gas meter node | a gas filter (`GasFiltersInput`), not yet tried |

The measurements query can be limited to one month, so its requests stay small, and it returns complete months, so it
is the better source of half hourly readings for an electricity import meter.

## Still to check

- Whether `readingDirection: GENERATION` matches the export meter, whose readings are
  zero overnight and positive during the day. Generation may be the total generated rather than what was exported.
- Whether the gas filter returns the same readings as the consumption query for a gas meter.
- Whether the measurements query's own pagination is stable, the months seen so far were complete but its cursors have
  not been examined.
