[< Octopus Energy Plugin](index.md)

# Observed API calls - My Energy Page - Get Your Geek GraphQL Queries
As soon as the details in the "Get your geek on" panel are changed, a series of requests are made to fetch the data



## smartMeterConsumption__first
### Query
```gql
"query smartMeterConsumption__first($meterId: ID!, $grouping: ConsumptionGroupings!, $startAt: DateTime!, $first: Int, $timezone: String!, $cursor: String) {
  node(id: $meterId) {
    ... on ElectricityMeterType {
      ...meterComparisonFields__first
      importMeter {
        id
        __typename
      }
      __typename
    }
    ... on GasMeterType {
      ...meterComparisonFields__first
      __typename
    }
    __typename
  }
}

fragment meterComparisonFields__first on Meter {
  consumptionUnits
  consumption(first: $first, grouping: $grouping, startAt: $startAt, timezone: $timezone, after: $cursor) {
    ...consumptionFields
    __typename
  }
  serialNumber
  __typename
}

fragment consumptionFields on ConsumptionConnection {
  edges {
    node {
      consumption: value
      intervalStart: startAt
      intervalEnd: endAt
      time: startAt
      __typename
    }
    __typename
  }
  pageInfo {
    endCursor
    hasNextPage
    __typename
  }
  __typename
}
```

### Variables
```json
{
    "meterId":"RWxlY3RyaWNpdHlNZXRlclR5cGU6MzY1NzQ2NQ==",
    "grouping":"HALF_HOUR",
    "startAt":"2024-08-01T00:00:00+01:00",
    "first":100,
    "timezone":"Europe/London"
}
```

### Example Response
```json
{
    "data": {
        "node": {
            "consumptionUnits": "kWh",
            "consumption": {
                "edges": [
                    {
                        "node": {
                            "consumption": "0.00800000000000000000",
                            "intervalStart": "2024-08-01T00:00:00+01:00",
                            "intervalEnd": "2024-08-01T00:30:00+01:00",
                            "__typename": "ConsumptionType"
                        },
                        "__typename": "ConsumptionEdge"
                    },
                    {
                        "node": {
                            "consumption": "3.7630000000000000",
                            "intervalStart": "2024-08-01T00:30:00+01:00",
                            "intervalEnd": "2024-08-01T01:00:00+01:00",
                            "__typename": "ConsumptionType"
                        },
                        "__typename": "ConsumptionEdge"
                    },
                    { 
                        "many": "Nodes not shown here ====================================================================="
                    },
                    {
                        "node": {
                            "consumption": "1.5450000000000000",
                            "intervalStart": "2024-08-03T01:00:00+01:00",
                            "intervalEnd": "2024-08-03T01:30:00+01:00",
                            "__typename": "ConsumptionType"
                        },
                        "__typename": "ConsumptionEdge"
                    },
                    {
                        "node": {
                            "consumption": "0.20100000000000000000",
                            "intervalStart": "2024-08-03T01:30:00+01:00",
                            "intervalEnd": "2024-08-03T02:00:00+01:00",
                            "__typename": "ConsumptionType"
                        },
                        "__typename": "ConsumptionEdge"
                    }
                ],
                "pageInfo": {
                    "endCursor": "YXJyYXljb25uZWN0aW9uOjk5",
                    "hasNextPage": true,
                    "__typename": "PageInfo"
                },
                "__typename": "ConsumptionConnection"
            },
            "serialNumber": "21E1111111",
            "__typename": "ElectricityMeterType",
            "importMeter": null
        }
    }
}
```
Fetches the first 100 half hourly consumption records starting at the selected date (1st August in this case). This is an example of a paged query. The results have been edited for brevity, the value "many": "Nodes not shown here" is not actually returned in the results but shows where data has been removed.

## smartMeterConsumption__first - Page 2
### Query
As above

### Variables
```json
{
    "meterId":"RWxlY3RyaWNpdHlNZXRlclR5cGU6MzY1NzQ2NQ==",
    "grouping":"HALF_HOUR",
    "startAt":"2024-08-01T00:00:00+01:00",
    "first":100,
    "timezone":"Europe/London",
    "cursor":"YXJyYXljb25uZWN0aW9uOjk5"
}
```

### Example Response
```json
{
    "data": {
        "node": {
            "consumptionUnits": "kWh",
            "consumption": {
                "edges": [
                    {
                        "node": {
                            "consumption": "1.9220000000000000",
                            "intervalStart": "2024-08-03T02:00:00+01:00",
                            "intervalEnd": "2024-08-03T02:30:00+01:00",
                            "__typename": "ConsumptionType"
                        },
                        "__typename": "ConsumptionEdge"
                    },
                    {
                        "node": {
                            "consumption": "0.65100000000000000000",
                            "intervalStart": "2024-08-03T02:30:00+01:00",
                            "intervalEnd": "2024-08-03T03:00:00+01:00",
                            "__typename": "ConsumptionType"
                        },
                        "__typename": "ConsumptionEdge"
                    },
                    { 
                        "many": "Nodes not shown here ====================================================================="
                    },
                    {
                        "node": {
                            "consumption": "3.8870000000000000",
                            "intervalStart": "2024-08-05T03:00:00+01:00",
                            "intervalEnd": "2024-08-05T03:30:00+01:00",
                            "__typename": "ConsumptionType"
                        },
                        "__typename": "ConsumptionEdge"
                    },
                    {
                        "node": {
                            "consumption": "3.8740000000000000",
                            "intervalStart": "2024-08-05T03:30:00+01:00",
                            "intervalEnd": "2024-08-05T04:00:00+01:00",
                            "__typename": "ConsumptionType"
                        },
                        "__typename": "ConsumptionEdge"
                    }
                ],
                "pageInfo": {
                    "endCursor": "YXJyYXljb25uZWN0aW9uOjE5OQ==",
                    "hasNextPage": true,
                    "__typename": "PageInfo"
                },
                "__typename": "ConsumptionConnection"
            },
            "serialNumber": "21E1111111",
            "__typename": "ElectricityMeterType",
            "importMeter": null
        }
    }
}
```
This is an example of fetching the next page of results. Note that the variables are the same as the initial call except for the addition of the `cursor` attribute from the `pageInfo` element of the first result set.

This is then repeated with the `cursor` value from each page until a page with the value `false` for `hasNextPage` is received.


[Prototype Queries >](prototypeQueries.md)
