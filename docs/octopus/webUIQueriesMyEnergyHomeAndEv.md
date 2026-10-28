[< [My Energy Page](webUiQueriesMyEnergy.md)

# Observed API calls - My Energy Page GraphQL Queries

These queries are executed by the HOME + EV page which was added in autumn 2026 and they appear to be undocumented at the moment.

## getActiveMeterPointsWithAgreements

### Query

```gql
query getActiveMeterPointsWithAgreements($accountNumber: String!) {
  account(accountNumber: $accountNumber) {
    properties {
      electricityMeterPoints {
        mpan
        direction
        agreements(excludeFuture: true, includeInactive: false) {
          tariff {
            __typename
            ... on TariffType {
              productCode
              __typename
            }
          }
          __typename
        }
        __typename
      }
      __typename
    }
    __typename
  }
}
```

### Variables

```json
{
  "accountNumber": "A-B1C2D34E"
}
```

### Example Response

```json
{
  "data": {
    "account": {
      "properties": [
        {
          "electricityMeterPoints": [
            {
              "mpan": "1111111111111",
              "direction": "IMPORT",
              "agreements": [
                {
                  "tariff": {
                    "__typename": "HalfHourlyTariff",
                    "productCode": "IOG-SMB-VAR-24-10-29"
                  },
                  "__typename": "ElectricityAgreementType"
                }
              ],
              "__typename": "ElectricityMeterPointType"
            },
            {
              "mpan": "2222222222222",
              "direction": "EXPORT",
              "agreements": [
                {
                  "tariff": {
                    "__typename": "StandardTariff",
                    "productCode": "OUTGOING-VAR-24-10-26"
                  },
                  "__typename": "ElectricityAgreementType"
                }
              ],
              "__typename": "ElectricityMeterPointType"
            }
          ],
          "__typename": "PropertyType"
        }
      ],
      "__typename": "AccountType"
    }
  }
}
```

Description

## getDeviceConsumptionBreakdown

### Query

```gql
query getDeviceConsumptionBreakdown(
  $accountNumber: String!
  $first: Int!
  $utilityFilters: [UtilityFiltersInput!]
  $modelledUtilityFilters: [UtilityFiltersInput!]
  $startOn: Date
  $endOn: Date
  $startAt: DateTime
  $endAt: DateTime
  $timezone: String
  $cursor: String
) {
  account(accountNumber: $accountNumber) {
    properties {
      id
      electricityMeterPoints {
        agreements(excludeFuture: true, includeInactive: false) {
          tariff {
            __typename
            ... on TariffType {
              productCode
              __typename
            }
          }
          validFrom
          validTo
          __typename
        }
        __typename
      }
      measurements(
        first: $first
        utilityFilters: $utilityFilters
        startOn: $startOn
        endOn: $endOn
        startAt: $startAt
        endAt: $endAt
        timezone: $timezone
        after: $cursor
      ) {
        edges {
          node {
            value
            unit
            ... on IntervalMeasurementType {
              startAt
              endAt
              durationInSeconds
              __typename
            }
            metaData {
              statistics {
                costExclTax {
                  pricePerUnit {
                    amount
                    __typename
                  }
                  costCurrency
                  estimatedAmount
                  __typename
                }
                costInclTax {
                  pricePerUnit {
                    amount
                    __typename
                  }
                  costCurrency
                  estimatedAmount
                  __typename
                }
                value
                description
                label
                type
                __typename
              }
              __typename
            }
            __typename
          }
          __typename
        }
        pageInfo {
          hasNextPage
          hasPreviousPage
          startCursor
          endCursor
          __typename
        }
        __typename
      }
      modelledMeasurements: measurements(
        first: $first
        utilityFilters: $modelledUtilityFilters
        startOn: $startOn
        endOn: $endOn
        startAt: $startAt
        endAt: $endAt
        timezone: $timezone
        after: $cursor
      ) {
        edges {
          node {
            value
            unit
            ... on IntervalMeasurementType {
              startAt
              endAt
              durationInSeconds
              __typename
            }
            metaData {
              statistics {
                costExclTax {
                  pricePerUnit {
                    amount
                    __typename
                  }
                  costCurrency
                  estimatedAmount
                  __typename
                }
                costInclTax {
                  pricePerUnit {
                    amount
                    __typename
                  }
                  costCurrency
                  estimatedAmount
                  __typename
                }
                value
                description
                label
                type
                __typename
              }
              __typename
            }
            __typename
          }
          __typename
        }
        pageInfo {
          hasNextPage
          hasPreviousPage
          startCursor
          endCursor
          __typename
        }
        __typename
      }
      __typename
    }
    __typename
  }
}
```

### Variables

```json
{
  "accountNumber": "A-B1C2D34E",
  "first": 100,
  "startAt": "2026-10-08T11:00:00.000Z",
  "endAt": "2026-10-10T10:59:59.999Z",
  "timezone": "Europe/London",
  "utilityFilters": [
    {
      "electricityFilters": {
        "readingFrequencyType": "RAW_INTERVAL",
        "readingDirection": "CONSUMPTION",
        "marketSupplyPointId": "1111111111111"
      }
    }
  ],
  "modelledUtilityFilters": [
    {
      "electricityFilters": {
        "readingFrequencyType": "RAW_INTERVAL",
        "readingDirection": "CONSUMPTION",
        "marketSupplyPointId": "1111111111111",
        "modelledSubMeterReadings": true
      }
    }
  ]
}
```

### Example Response

```json
{
    "data": {
        "account": {
            "properties": [
                {
                    "id": "2267073",
                    "electricityMeterPoints": [
                        {
                            "agreements": [
                                {
                                    "tariff": {
                                        "__typename": "HalfHourlyTariff",
                                        "productCode": "IOG-SMB-VAR-24-10-29"
                                    },
                                    "validFrom": "2026-08-26T23:00:00+00:00",
                                    "validTo": null,
                                    "__typename": "ElectricityAgreementType"
                                }
                            ],
                            "__typename": "ElectricityMeterPointType"
                        },
                        {
                            "agreements": [
                                {
                                    "tariff": {
                                        "__typename": "StandardTariff",
                                        "productCode": "OUTGOING-VAR-24-10-26"
                                    },
                                    "validFrom": "2025-08-12T23:00:00+00:00",
                                    "validTo": null,
                                    "__typename": "ElectricityAgreementType"
                                }
                            ],
                            "__typename": "ElectricityMeterPointType"
                        }
                    ],
                    "measurements": {
                        "edges": [
                            {
                                "node": {
                                    "value": "0E-18",
                                    "unit": "kwh",
                                    "startAt": "2026-10-06T12:00:00+01:00",
                                    "endAt": "2026-10-06T12:30:00+01:00",
                                    "durationInSeconds": 1800,
                                    "__typename": "IntervalMeasurementType",
                                    "metaData": {
                                        "statistics": [
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "28.8273",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "30.26866",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "0",
                                                "description": null,
                                                "label": "Home Standard Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "6.5714",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "6.89997",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "0",
                                                "description": null,
                                                "label": "EV Low Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "28.8273",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "30.26866",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "0",
                                                "description": null,
                                                "label": "EV Standard Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": null,
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "1.06928",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": null,
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "1.12275",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": null,
                                                "description": null,
                                                "label": "STANDING_CHARGE_STANDARD_A",
                                                "type": "STANDING_CHARGE_COST",
                                                "__typename": "StatisticOutput"
                                            }
                                        ],
                                        "__typename": "MeasurementsMetadataOutput"
                                    }
                                },
                                "__typename": "MeasurementEdge"
                            },
                            {
                                "node": {
                                    "value": "0E-18",
                                    "unit": "kwh",
                                    "startAt": "2026-10-06T12:30:00+01:00",
                                    "endAt": "2026-10-06T13:00:00+01:00",
                                    "durationInSeconds": 1800,
                                    "__typename": "IntervalMeasurementType",
                                    "metaData": {
                                        "statistics": [
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "28.8273",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "30.26866",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "0",
                                                "description": null,
                                                "label": "Home Standard Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "6.5714",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "6.89997",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "0",
                                                "description": null,
                                                "label": "EV Low Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "28.8273",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "30.26866",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "0",
                                                "description": null,
                                                "label": "EV Standard Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": null,
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "1.06928",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": null,
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "1.12275",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": null,
                                                "description": null,
                                                "label": "STANDING_CHARGE_STANDARD_A",
                                                "type": "STANDING_CHARGE_COST",
                                                "__typename": "StatisticOutput"
                                            }
                                        ],
                                        "__typename": "MeasurementsMetadataOutput"
                                    }
                                },
                                "__typename": "MeasurementEdge"
                            },
                            {
                                "node": {
                                    "value": "3.419097661972046000",
                                    "unit": "kwh",
                                    "startAt": "2026-10-10T00:30:00+01:00",
                                    "endAt": "2026-10-10T01:00:00+01:00",
                                    "durationInSeconds": 1800,
                                    "__typename": "IntervalMeasurementType",
                                    "metaData": {
                                        "statistics": [
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "28.8273",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "30.26866",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "0",
                                                "description": null,
                                                "label": "Home Standard Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "6.5714",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "23.2424",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "6.89997",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "24.40452",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "3.5369",
                                                "description": null,
                                                "label": "Home Low Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "6.5714",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "22.46826",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "6.89997",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "23.59167",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "3.4191",
                                                "description": null,
                                                "label": "EV Low Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "28.8273",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": {
                                                        "amount": "30.26866",
                                                        "__typename": "PricePerUnit"
                                                    },
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "0",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": "0",
                                                "description": null,
                                                "label": "EV Standard Rate",
                                                "type": "CONSUMPTION_COST",
                                                "__typename": "StatisticOutput"
                                            },
                                            {
                                                "costExclTax": {
                                                    "pricePerUnit": null,
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "1.06928",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "costInclTax": {
                                                    "pricePerUnit": null,
                                                    "costCurrency": "GBP",
                                                    "estimatedAmount": "1.12275",
                                                    "__typename": "EstimatedMoneyType"
                                                },
                                                "value": null,
                                                "description": null,
                                                "label": "STANDING_CHARGE_STANDARD_A",
                                                "type": "STANDING_CHARGE_COST",
                                                "__typename": "StatisticOutput"
                                            }
                                        ],
                                        "__typename": "MeasurementsMetadataOutput"
                                    }
                                },
                                "__typename": "MeasurementEdge"
                            }
                        ],
                        "pageInfo": {
                            "hasNextPage": false,
                            "hasPreviousPage": false,
                            "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
                            "endCursor": "YXJyYXljb25uZWN0aW9uOjE5",
                            "__typename": "PageInfo"
                        },
                        "__typename": "MeasurementConnection"
                    },
                    "__typename": "PropertyType"
                }
            ],
            "__typename": "AccountType"
        }
    }
}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

## queryName

### Query

```gql
query
```

### Variables

```json
{}
```

### Example Response

```json
{}
```

Description

[My Energy Page - Get Your Geek >](webUiQueriesGetYourGeek.md)
