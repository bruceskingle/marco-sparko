[< Web UI Queries](webUiQueries.md)
# Prototype Queries
This section describes some prototype queries which might make a good starting point for a GraphQL application. The [Web UI Queries](webUiQueries.md) page includes some instructions about how to try these for yourself.

In most cases I have included all the available attributes in the query on the basis that it's easier to delete the ones you don't want than to look up the details of ones which are missing.

## Initial Authentication with Email and Password
## Query
```gql
mutation login($input: ObtainJSONWebTokenInput!) {
  obtainKrakenToken(input: $input) {
    refreshToken
    refreshExpiresIn
    payload
    token
  }
}
```

## Variables
```json
{
  	"input": {
      "email": "name@domain.com",
      "password": "secret"
    }
}
```

## Example Response
```json
{
  "data": {
    "obtainKrakenToken": {
      "refreshToken": "redacted_refresh_token_pZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtp",
      "refreshExpiresIn": 1725024454,
      "payload": {
        "sub": "kraken|account-user:3235447",
        "gty": "API-KEY",
        "email": "dan@archer.org",
        "tokenUse": "access",
        "iss": "https://api.octopus.energy/v1/graphql/",
        "iat": 1724155372,
        "exp": 1724158972,
        "origIat": 1724155372
      },
      "token": "reacted_jwt_token_3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtpZCI6InF3ZW"
    }
  }
}
```
This should **only be used with an interactive application where you will prompt for the password from a user, DO NOT store the password anywhere and DO NOT try this query in the playground**.

Once this request is made you should delete (i.e. write zeros over the stored value byte by byte) the stored password. It is possible for data stored in memory to remain accessible even after a process has terminated in some situations.

Once you have authenticated it is possible to retrieve the current APIKey via the `view` query, which could then be stored if desired.

The `token` value needs to be provided as the value of the `Authorization` header for all further requests.

## Initial Authentication with APIKey
If you intend to store the authentication credential, for unattended operation or as a user convenience then authenticate using the APIKey instead. The same query above can be used with the following variables:

```json
{
    "input": {
        "APIKey": "sk_XXXX_XXXXXXXXXXXXXXXXXXXXXXXX"
    }
}
```

See The [Web UI Queries](webUiQueries.md) page for details on how to obtain this from the Account dashboard.

The result will be the same as above.

## Invalidate All Current Sessions
### Query
```gql
mutation forceReauthentication($forceReAuth: ForceReauthenticationInput!){
  forceReauthentication(input: $forceReAuth) {
    tokensInvalidated
    effectiveAt
  }
}
```

### Variables
```json
{
  	"forceReAuth": {
      "includeThirdParties": true
    }
}
```

In the event that the APIKey is compromised you can invalidate the existing key and generate a new one in the Account Dashboard (https://octopus.energy/dashboard/new/accounts/personal-details/api-access). Unfortunately this does not invalidate any current refresh tokens so you should also call this mutation which will invalidate all current sessions and their refresh tokens.

## Refresh Authentication
### Query
```gql
mutation refreshKrakenToken($refresh: ObtainJSONWebTokenInput!) {
  obtainKrakenToken(input: $refresh) {
    refreshToken
    refreshExpiresIn
    payload
    token
  }
}
```

### Variables
```json
{
    "refresh": {
      "refreshToken": "redacted_refresh_token_pZCI6InF3ZWxsLWtub3duL2p3a3MuanNvbiIsImtp"
    }
}
```

The authentication token returned by `obtainKrakenToken` is only valid for a short time (currently 1 hour). This means that the token needs to be periodically refreshed if the application lives for longer than that. There are two strategies to deal with this, one is to detect the error returned from any request when the token has expired, which looks like this:

```json
{
  "errors": [
    {
      "message": "Signature of the JWT has expired.",
      "locations": [
        {
          "line": 3,
          "column": 37
        }
      ],
      "path": [
        "account"
      ],
      "extensions": {
        "errorType": "APPLICATION",
        "errorCode": "KT-CT-1124",
        "errorDescription": ""
      }
    }
  ],
  "data": {
    "account": null
  }
}
```

The other is to remember when the token is due to expire and refresh it before that time. The expiry time of the token is provided in the `exp` attribute of the response from `obtainKrakenToken`. This is a timestamp in seconds from Jan 1 1970 (a.k.a a Unix timestamp in seconds).

When the token expires you can either use this mutation passing the `refreshToken` provided in the original response from `obtainKrakenToken`, or re-execute the initial authentication. In the case that a password was used for the initial authentication you should use this method in preference to retaining the password for security reasons.

Note that the refresh token also expires (currently after 10 days), the time of that expiry is provided in the 'refreshExpiresIn' attribute of the response. (This is a misnamed attribute the value is an absolute timestamp not an offset from some other point in time).

## Fetch User's Current APIKey
### Query
```gql
query getApiKey {
    viewer {
        liveSecretKey
    }
}
```

### Variables
```json
{}
```

### Example Response
```json
{
  "data": {
    "viewer": {
      "liveSecretKey": "sk_XXXX_XXXXXXXXXXXXXXXXXXXXXXXX"
    }
  }
}
```
This query can be used to obtain the current APIKey, which may be useful after a password based authentication.

## Fetch Account User
Fetch the main details for the logged in user, and all of their accounts. No parameters are required.
### Query
```gql
query getAccountUser {
    viewer
    {
        id
        givenName
        familyName
        email
        mobile
        landline
        title
        pronouns
        isDeceased
        liveSecretKey
        dateOfBirth
        fullName
        preferredName
        alternativePhoneNumbers
        hasFamilyIssues
        isInHardship
        isOptedInToWof
        details {
        namespace
        value
        }
        specialCircumstances {
        isSharingConsentGiven
        records {
            ... on SpecialCircumstanceRecordType {
            id
            summary
            internalCode
            gasPSRCode
            electricityPSRCode
            }
            ... on TemporarySpecialCircumstanceRecordType {
            id
            summary
            internalCode
            gasPSRCode
            electricityPSRCode
            expiryDate
            }
        }
        }
        preferences {
        isOptedInToClientMessages
        isOptedInToOfferMessages
        isOptedInToRecommendedMessages
        isOptedInToUpdateMessages
        isOptedInToThirdPartyMessages
        emailFormat
        isUsingInvertedEmailColours
        fontSizeMultiplier
        isOptedInMeterReadingConfirmations
        isOptedInToSmsMessages
        preferredHoldMusic
        }
        accounts {
            ... on AccountType {
                id
                number
                status
                brand
                balance(includeAllLedgers: false)
                overdueBalance
                urn
                billingName
                billingSubName
                billingEmail
                billingAddress
                billingAddressLine1
                billingAddressLine2
                billingAddressLine3
                billingAddressLine4
                billingAddressLine5
                billingAddressPostcode
                billingCountryCode
                billingDeliveryPointIdentifier
                splitBillingAddress
                address {
                    name
                    organization
                    streetAddress
                    structuredStreetAddress
                    dependentLocality
                    locality
                    administrativeArea
                    postalCode
                    sortingCode
                    country
                    deliveryPointIdentifier
                }
                metadata {
                    key
                    value
                }
                canRequestRefund
                requestRefundEligibility {
                    canRequestRefund
                    reason
                }
                referralsCreated
                billingOptions {
                    periodStartDay
                    periodLength
                    periodLengthMultiplier
                    isFixed
                    currentBillingPeriodStartDate
                    currentBillingPeriodEndDate
                    nextBillingDate
                }
                accountType
                business{
                    name
                    number
                    businessType
                }
                commsDeliveryPreference
                documentAccessibility
                references {
                    namespace
                    value
                    createdAt
                    updatedAt
                    account {
                        number
                    }
                }
                maximumRefund {
                    amount
                    reasonToRecommendAmount
                    recommendedBalance
                }
                campaigns {
                    campaignExpiryDate
                    name
                    slug
                    expiryDate
                    startDate
                }
                isInHardship
                createdAt
                preferredLanguageForComms
                properties(activeFrom: "2024-08-12T23:00:00.000Z") {
                    id
                    postcode
                    address
                    richAddress {
                        name
                        organization
                        streetAddress:
                        structuredStreetAddress
                        dependentLocality
                        locality
                        administrativeArea
                        postalCode
                        sortingCode
                        country
                        deliveryPointIdentifier
                    }
                    splitAddress
                    occupancyPeriods {
                        id
                        effectiveFrom
                        effectiveTo
                        isOccupier
                    }
                    coordinates {
                        latitude
                        longitude
                    }
                }
                projectedBalance
                shouldReviewPayments
                recommendedBalanceAdjustment
                canChangePayments
                cotReadingWindowDays
                canBeWithdrawn
                currentEstimatedSsd
                earliestPossibleSsd
                latestPossibleSsd

                operationsTeam {
                        id
                        teamName
                        isOffline
                        isAcceptingCalls
                    }
                canInputMeterReadingsViaIvr
                hasActiveDunningProcess
                hasActiveCollectionsProceedings
                isEligibleForElectricityReadingIncentive
                isEligibleForGasReadingIncentive
                isInBlockingMigration
            }
        }
    }
}
```

### Variables
```json
{}
```

### Example Response
```json
{
  "data": {
    "viewer": {
      "id": "3235447",
      "givenName": "Dan",
      "familyName": "Archer",
      "email": "dan@archer.org",
      "mobile": "+44787123456",
      "landline": "",
      "title": "",
      "pronouns": null,
      "isDeceased": false,
      "liveSecretKey": "redacted_api_key_AAAAAAAAAAAAAAA",
      "dateOfBirth": null,
      "fullName": "Dan Archer",
      "preferredName": "Dan",
      "alternativePhoneNumbers": [],
      "hasFamilyIssues": false,
      "isInHardship": false,
      "isOptedInToWof": true,
      "details": [],
      "specialCircumstances": {
        "isSharingConsentGiven": null,
        "records": []
      },
      "preferences": {
        "isOptedInToClientMessages": false,
        "isOptedInToOfferMessages": false,
        "isOptedInToRecommendedMessages": true,
        "isOptedInToUpdateMessages": false,
        "isOptedInToThirdPartyMessages": false,
        "emailFormat": "HTML",
        "isUsingInvertedEmailColours": false,
        "fontSizeMultiplier": 1,
        "isOptedInMeterReadingConfirmations": false,
        "isOptedInToSmsMessages": true,
        "preferredHoldMusic": null
      },
      "accounts": [
        {
          "id": "3403670",
          "number": "A-B1C2D34E",
          "status": "ACTIVE",
          "brand": "OCTOPUS_ENERGY",
          "balance": 39303,
          "overdueBalance": 0,
          "urn": "",
          "billingName": "Dan Archer",
          "billingSubName": null,
          "billingEmail": null,
          "billingAddress": "BROOKFIELD FARM, AMBRIDGE, BORTCHESTER, BORSETSHIRE, , BB12 3AM",
          "billingAddressLine1": "BROOKFIELD FARM",
          "billingAddressLine2": "",
          "billingAddressLine3": "AMBRIDGE",
          "billingAddressLine4": "BORTCHESTER",
          "billingAddressLine5": "BORSETSHIRE",
          "billingAddressPostcode": "BB12 3AM",
          "billingCountryCode": "GB",
          "billingDeliveryPointIdentifier": null,
          "splitBillingAddress": [
            "BROOKFIELD FARM",
            "",
            "AMBRIDGE",
            "BORTCHESTER",
            "BORSETSHIRE"
          ],
          "address": {
            "name": "",
            "organization": "",
            "streetAddress": "BROOKFIELD FARM\nAMBRIDGE\nBORTCHESTER\nBORSETSHIRE",
            "structuredStreetAddress": null,
            "dependentLocality": "",
            "locality": "",
            "administrativeArea": "",
            "postalCode": "BB12 3AM",
            "sortingCode": "",
            "country": "GB",
            "deliveryPointIdentifier": ""
          },
          "metadata": [],
          "canRequestRefund": false,
          "requestRefundEligibility": {
            "canRequestRefund": false,
            "reason": "HAS_NOT_GIVEN_READINGS_RECENTLY"
          },
          "referralsCreated": 0,
          "billingOptions": {
            "periodStartDay": null,
            "periodLength": null,
            "periodLengthMultiplier": null,
            "isFixed": false,
            "currentBillingPeriodStartDate": "2024-08-22",
            "currentBillingPeriodEndDate": null,
            "nextBillingDate": null
          },
          "accountType": "DOMESTIC",
          "business": null,
          "commsDeliveryPreference": "EMAIL",
          "documentAccessibility": null,
          "references": [],
          "maximumRefund": {
            "amount": 29082,
            "reasonToRecommendAmount": "MAX_AVAILABLE_AMOUNT",
            "recommendedBalance": 10221
          },
          "campaigns": [
            {
              "campaignExpiryDate": null,
              "name": "Octoplus",
              "slug": "octoplus",
              "expiryDate": null,
              "startDate": "2023-10-26"
            },
            {
              "campaignExpiryDate": null,
              "name": "Octoplus Saving Sessions",
              "slug": "octoplus-saving-sessions",
              "expiryDate": null,
              "startDate": "2023-10-26"
            },
            {
              "campaignExpiryDate": null,
              "name": "Power-Ups UKPN",
              "slug": "power_ups_ukpn",
              "expiryDate": null,
              "startDate": "2023-08-18"
            },
            {
              "campaignExpiryDate": null,
              "name": "SMETS2_CALL_LIST",
              "slug": "smets2-call-list",
              "expiryDate": null,
              "startDate": null
            }
          ],
          "isInHardship": false,
          "createdAt": "2021-05-04T10:17:30.242231+00:00",
          "preferredLanguageForComms": null,
          "properties": [
            {
              "id": "2930512",
              "postcode": "BB12 3AM",
              "address": "Brookfield Farm, Ambridge, Bortchester, Borsetshire, BB12 3AM",
              "richAddress": null,
              "splitAddress": [
                "BROOKFIELD FARM",
                "",
                "AMBRIDGE",
                "BORTCHESTER",
                "BORSETSHIRE"
              ],
              "occupancyPeriods": [
                {
                  "id": "3524305",
                  "effectiveFrom": "2021-05-03T23:00:00+00:00",
                  "effectiveTo": null,
                  "isOccupier": false
                }
              ],
              "coordinates": {
                "latitude": 52.505811411470816,
                "longitude": -1.8095484735073384
              }
            }
          ],
          "projectedBalance": -4497,
          "shouldReviewPayments": false,
          "recommendedBalanceAdjustment": null,
          "canChangePayments": true,
          "cotReadingWindowDays": 2,
          "canBeWithdrawn": true,
          "currentEstimatedSsd": "2024-08-28",
          "earliestPossibleSsd": "2024-08-29",
          "latestPossibleSsd": "2024-10-07",
          "operationsTeam": {
            "id": 3309,
            "teamName": "SMART-LDN-1",
            "isOffline": false,
            "isAcceptingCalls": true
          },
          "canInputMeterReadingsViaIvr": false,
          "hasActiveDunningProcess": false,
          "hasActiveCollectionsProceedings": false,
          "isEligibleForElectricityReadingIncentive": true,
          "isEligibleForGasReadingIncentive": false,
          "isInBlockingMigration": false
        }
      ]
    }
  }
}
```
Description

## getMeterDetails
### Query
```gql
query getMeters($accountNumber: String!, $propertiesActiveFrom: DateTime) {
  account(accountNumber: $accountNumber) {
    status
    number
    balance
    electricityAgreements(active: true) {
      tariff {
        ... on StandardTariff {
          id
        	fullName
          productCode
          standingCharge
          preVatStandingCharge
          unitRate
          preVatUnitRate
        }
        ... on DayNightTariff {
          id
        	fullName
          productCode
          standingCharge
          preVatStandingCharge
          dayRate
          preVatDayRate
          nightRate
          preVatNightRate
        }
        ... on ThreeRateTariff {
          id
        	fullName
          productCode
          standingCharge
          preVatStandingCharge
          dayRate
          preVatDayRate
          nightRate
          preVatNightRate
          offPeakRate
          preVatOffPeakRate
        }
        ... on HalfHourlyTariff {
          id
        	fullName
          productCode
          standingCharge
          preVatStandingCharge
          unitRates {
            validFrom
            validTo
            value
            preVatValue
          }
        }
        ... on PrepayTariff {
          id
        	fullName
          productCode
          standingCharge
          preVatStandingCharge
          unitRate
          preVatUnitRate
        }
       	__typename
      }
    }
    gasAgreements(active:true) {
      tariff {
        fullName
        productCode
        standingCharge
        unitRate
      }
    }
    properties(activeFrom: $propertiesActiveFrom) {
      id
      electricityMeterPoints {
        id
        mpan
        meters {
          id
          hasAndAllowsHhReadings
          serialNumber
          isTradPrepay
          smartDevices {
            deviceId
          }
        }
      }
      gasMeterPoints {
        id
        mprn
        meters {
          id
          hasAndAllowsHhReadings
          serialNumber
          smartDevices {
            deviceId
          }
        }
      }
    }
  }
}
```

### Variables
```json
{
  "accountNumber":"A-B1C2D34E",
  "propertiesActiveFrom":"2024-08-12T23:00:00.000Z"
}
```

### Example Response
```json
{
  "data": {
    "account": {
      "status": "ACTIVE",
      "number": "A-B1C2D34E",
      "balance": 39303,
      "electricityAgreements": [
        {
          "tariff": {
            "id": "11353",
            "fullName": "Outgoing Octopus 12M Fixed May 2019",
            "productCode": "OUTGOING-FIX-12M-19-05-13",
            "standingCharge": 0,
            "preVatStandingCharge": 0,
            "unitRate": 15,
            "preVatUnitRate": 15,
            "__typename": "StandardTariff"
          }
        },
        {
          "tariff": {
            "id": "175911",
            "fullName": "Intelligent Octopus Go",
            "productCode": "INTELLI-VAR-22-10-14",
            "standingCharge": 47.8485,
            "preVatStandingCharge": 45.57,
            "unitRates": [
              {
                "validFrom": "2024-08-24T22:30:00+00:00",
                "validTo": "2024-08-25T04:30:00+00:00",
                "value": 7.00035,
                "preVatValue": 6.667
              },
              {
                "validFrom": "2024-08-25T04:30:00+00:00",
                "validTo": "2024-08-25T22:30:00+00:00",
                "value": 24.39255,
                "preVatValue": 23.231
              },
              {
                "validFrom": "2024-08-25T22:30:00+00:00",
                "validTo": "2024-08-26T04:30:00+00:00",
                "value": 7.00035,
                "preVatValue": 6.667
              }
            ],
            "__typename": "HalfHourlyTariff"
          }
        }
      ],
      "gasAgreements": [
        {
          "tariff": {
            "fullName": "Flexible Octopus",
            "productCode": "VAR-22-11-01",
            "standingCharge": 28.9485,
            "unitRate": 5.401725
          }
        }
      ],
      "properties": [
        {
          "id": "2930512",
          "electricityMeterPoints": [
            {
              "id": "2875805",
              "mpan": "1111111111111",
              "meters": [
                {
                  "id": "3657465",
                  "hasAndAllowsHhReadings": true,
                  "serialNumber": "21E1111111",
                  "isTradPrepay": false,
                  "smartDevices": [
                    {
                      "deviceId": "01-01-01-01-01-01-01-01"
                    }
                  ]
                }
              ]
            },
            {
              "id": "3347939",
              "mpan": "2222222222222",
              "meters": [
                {
                  "id": "3839934",
                  "hasAndAllowsHhReadings": true,
                  "serialNumber": "21E1111111",
                  "isTradPrepay": false,
                  "smartDevices": [
                    {
                      "deviceId": "01-01-01-01-01-01-01-01"
                    }
                  ]
                }
              ]
            }
          ],
          "gasMeterPoints": [
            {
              "id": "2383770",
              "mprn": "3333333333",
              "meters": [
                {
                  "id": "3274816",
                  "hasAndAllowsHhReadings": true,
                  "serialNumber": "E6S22222222222",
                  "smartDevices": [
                    {
                      "deviceId": "02-02-02-02-02-02-02-02"
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  }
}
```
Fetches the current agreements (which contain the tariff) and all meter details.

Notice that the tariff rates are returned for the current and next day (with respect to the date the call is being made).

This gives us the `meterId` (the `id` attribute of each `meter` object), the `mpan` or `mprn` and the `serialNumber` for each meter which allows us to call any method to fetch readings or consumption values.

## getElectricityMeterReadings
### Query
```gql
query getElectricityMeterReadings {
    electricityMeterReadings(
        accountNumber: "A-B1C2D34E"
        meterId: "3657465"
        first: 50
    ) {
        totalCount
        edgeCount
        edges {
            node {
                id
                readAt
                readingSource
                source
                registers {
                    identifier
                    name
                    value
                    digits
                    isQuarantined
                }
            }
        }
    }
}
```

### Variables
```json
{}
```

### Example Response
```json
{
  "data": {
    "electricityMeterReadings": {
      "totalCount": 44,
      "edgeCount": 44,
      "edges": [
        {
          "node": {
            "id": "533539646",
            "readAt": "2024-08-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "25612.44300",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "519155216",
            "readAt": "2024-07-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "25314.90500",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "504613787",
            "readAt": "2024-06-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "25158.29100",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "492413383",
            "readAt": "2024-05-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "25087.24700",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "478397854",
            "readAt": "2024-04-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "24898.54500",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "461836920",
            "readAt": "2024-03-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "24518.81700",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "420426713",
            "readAt": "2024-02-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "23732.80700",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "388713239",
            "readAt": "2024-01-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "22715.70300",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "375209860",
            "readAt": "2023-12-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "21646.05500",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "371195701",
            "readAt": "2023-12-01T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "20817.05700",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "364979081",
            "readAt": "2023-11-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "20455.44300",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "353391443",
            "readAt": "2023-10-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "19439.74000",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "450941903",
            "readAt": "2023-10-10T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "19191.25300",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "450941943",
            "readAt": "2023-10-01T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "19058.72200",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "341436376",
            "readAt": "2023-09-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "18907.81900",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "331701317",
            "readAt": "2023-08-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "18297.93200",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "320550992",
            "readAt": "2023-07-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "17845.11800",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "320551022",
            "readAt": "2023-07-01T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "17547.45500",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "308682442",
            "readAt": "2023-06-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "17461.63400",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "308682455",
            "readAt": "2023-05-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "17144.81300",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "272571724",
            "readAt": "2023-04-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "16638.40800",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "272571743",
            "readAt": "2023-04-01T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "16362.64800",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "247550683",
            "readAt": "2023-03-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "16125.91000",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "203513159",
            "readAt": "2023-02-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "15397.62700",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "160230919",
            "readAt": "2023-01-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "14332.81800",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "160230931",
            "readAt": "2023-01-01T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "13525.60100",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "153333714",
            "readAt": "2022-12-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "13174.57100",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "148138660",
            "readAt": "2022-11-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "11992.27000",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "141845354",
            "readAt": "2022-10-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "11029.56100",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "141845365",
            "readAt": "2022-10-01T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "10515.17300",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "135199559",
            "readAt": "2022-09-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "10235.34400",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "130298908",
            "readAt": "2022-08-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "9734.03600",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "125320877",
            "readAt": "2022-07-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "9208.13000",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "120035896",
            "readAt": "2022-06-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "8844.49200",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "115109416",
            "readAt": "2022-05-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "8192.87400",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "109252810",
            "readAt": "2022-04-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "7791.94500",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "103128656",
            "readAt": "2022-03-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "7347.30300",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "98549839",
            "readAt": "2022-02-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "6650.77200",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "93718140",
            "readAt": "2022-01-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "5359.93100",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "88411003",
            "readAt": "2021-12-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "3838.83200",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "82921009",
            "readAt": "2021-11-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "2650.96400",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "75269965",
            "readAt": "2021-09-21T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "771.44300",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "72682837",
            "readAt": "2021-08-26T00:00:00+00:00",
            "readingSource": "Smart reading",
            "source": "SMART_METER",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "20.79200",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        },
        {
          "node": {
            "id": "73078181",
            "readAt": "2021-08-25T10:58:28+00:00",
            "readingSource": "Data collector reading",
            "source": "DATA_COLLECTOR",
            "registers": [
              {
                "identifier": "1",
                "name": "Standard",
                "value": "0.00000",
                "digits": 5,
                "isQuarantined": false
              }
            ]
          }
        }
      ]
    }
  }
}
```
Description

## Get Energy Products Available at some Postcode
### Query
```gql
query energyProducts($postcode:String!) {
    energyProducts(first: 20,brand: "OCTOPUS_ENERGY", postcode:$postcode, filterBy:DOMESTIC, availability:AVAILABLE) {
        edges {
            node {
                id
                fullName
                displayName
                description
                availableFrom
                availableTo
                isHidden
                code
                direction
                notes
                isVariable
                isGreen
                isBusiness
                isChargedHalfHourly
                isPrepay
                isDefault
                isOccupier
                term
                isAvailable
                isUnavailable
                isFixed
                isDomestic
                includesEpgReduction
                exitFees
                exitFeesType
                tags
                __typename
                tariffs(first: 20,postcode:$postcode) {
                    edges {
                        node {
                            ... on TariffType {
                                id
                                displayName
                                fullName
                                description
                                productCode
                                standingCharge
                                preVatStandingCharge
                                tariffCode
                            }
                            ... on StandardTariff {
                                unitRate
                                unitRateEpgApplied
                                preVatUnitRate
                            }
                            ... on DayNightTariff {
                                dayRate

                                # Is EPG applied to the unit rate.
                                dayRateEpgApplied
                                nightRate

                                # Is EPG applied to the unit rate.
                                nightRateEpgApplied
                                preVatDayRate
                                preVatNightRate
                            }
                            ... on ThreeRateTariff {
                                dayRate

                                # Is EPG applied to the unit rate.
                                dayRateEpgApplied
                                nightRate

                                # Is EPG applied to the unit rate.
                                nightRateEpgApplied
                                offPeakRate

                                # Is EPG applied to the unit rate.
                                offPeakRateEpgApplied
                                preVatDayRate
                                preVatNightRate
                                preVatOffPeakRate
                            }  
                            __typename
                            }
                        }

                    }
                }
            }
        }
  }
```

### Variables
```json
{
  "postcode":"BB12 3AM"
}
```

### Example Response
```json
{
  "data": {
    "energyProducts": {
      "edges": [
        {
          "node": {
            "id": "14360",
            "fullName": "Octopus 12M Fixed August 2024 v3",
            "displayName": "Octopus 12M Fixed",
            "description": "This tariff features 100% renewable electricity and fixes your unit rates and standing charge for 12 months.",
            "availableFrom": "2024-08-20T00:00:00+01:00",
            "availableTo": null,
            "isHidden": false,
            "code": "OE-FIX-12M-24-08-20",
            "direction": "IMPORT",
            "notes": "",
            "isVariable": false,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": false,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": 12,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": true,
            "isDomestic": true,
            "includesEpgReduction": true,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "242336",
                    "displayName": "Octopus 12M Fixed",
                    "fullName": "Octopus 12M Fixed August 2024 v3",
                    "description": "This tariff features 100% renewable electricity and fixes your unit rates and standing charge for 12 months.",
                    "productCode": "OE-FIX-12M-24-08-20",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-1R-OE-FIX-12M-24-08-20-A",
                    "unitRate": 24.15,
                    "unitRateEpgApplied": false,
                    "preVatUnitRate": null,
                    "__typename": "StandardTariff"
                  }
                },
                {
                  "node": {
                    "id": "242337",
                    "displayName": "Octopus 12M Fixed",
                    "fullName": "Octopus 12M Fixed August 2024 v3",
                    "description": "This tariff features 100% renewable electricity and fixes your unit rates and standing charge for 12 months.",
                    "productCode": "OE-FIX-12M-24-08-20",
                    "standingCharge": 48.39,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-2R-OE-FIX-12M-24-08-20-A",
                    "dayRate": 29.79,
                    "dayRateEpgApplied": false,
                    "nightRate": 12.5,
                    "nightRateEpgApplied": false,
                    "preVatDayRate": null,
                    "preVatNightRate": null,
                    "__typename": "DayNightTariff"
                  }
                },
                {
                  "node": {
                    "id": "75840",
                    "displayName": "Octopus 12M Fixed",
                    "fullName": "Octopus 12M Fixed August 2024 v3",
                    "description": "This tariff features 100% renewable electricity and fixes your unit rates and standing charge for 12 months.",
                    "productCode": "OE-FIX-12M-24-08-20",
                    "standingCharge": 28.95,
                    "preVatStandingCharge": null,
                    "tariffCode": "G-1R-OE-FIX-12M-24-08-20-A",
                    "__typename": "GasTariffType"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "14140",
            "fullName": "Aira Flexible",
            "displayName": "Aira Flexible",
            "description": "Aira Flexible offers great value and 100% renewable electricity. As a variable tariff, your prices can rise and fall with wholesale prices - but we'll always give you notice of a change.",
            "availableFrom": "2024-07-01T00:00:00+01:00",
            "availableTo": null,
            "isHidden": false,
            "code": "AIRA-VAR-24-07-01",
            "direction": "IMPORT",
            "notes": "This is a copy of the \"Flexible Octopus\" product, created to be used by the Aira affiliate",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": false,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": true,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "234536",
                    "displayName": "Aira Flexible",
                    "fullName": "Aira Flexible",
                    "description": "Aira Flexible offers great value and 100% renewable electricity. As a variable tariff, your prices can rise and fall with wholesale prices - but we'll always give you notice of a change.",
                    "productCode": "AIRA-VAR-24-07-01",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-1R-AIRA-VAR-24-07-01-A",
                    "unitRate": 23.08,
                    "unitRateEpgApplied": false,
                    "preVatUnitRate": null,
                    "__typename": "StandardTariff"
                  }
                },
                {
                  "node": {
                    "id": "234537",
                    "displayName": "Aira Flexible",
                    "fullName": "Aira Flexible",
                    "description": "Aira Flexible offers great value and 100% renewable electricity. As a variable tariff, your prices can rise and fall with wholesale prices - but we'll always give you notice of a change.",
                    "productCode": "AIRA-VAR-24-07-01",
                    "standingCharge": 48.39,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-2R-AIRA-VAR-24-07-01-A",
                    "dayRate": 28.52,
                    "dayRateEpgApplied": false,
                    "nightRate": 11.96,
                    "nightRateEpgApplied": false,
                    "preVatDayRate": null,
                    "preVatNightRate": null,
                    "__typename": "DayNightTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "8028",
            "fullName": "Flexible Octopus",
            "displayName": "Flexible Octopus",
            "description": "Flexible Octopus offers great value and 100% renewable electricity. As a variable tariff, your prices can rise and fall with wholesale prices - but we'll always give you notice of a change.",
            "availableFrom": "2023-03-28T10:35:00+01:00",
            "availableTo": null,
            "isHidden": false,
            "code": "VAR-22-11-01",
            "direction": "IMPORT",
            "notes": "Payment-price-switching product\r\nRenamed from Flexible Octopus November 2022 v1 as part of SVT price change in October 2023",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": false,
            "isPrepay": false,
            "isDefault": true,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": true,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "176152",
                    "displayName": "Flexible Octopus",
                    "fullName": "Flexible Octopus",
                    "description": "Flexible Octopus offers great value and 100% renewable electricity. As a variable tariff, your prices can rise and fall with wholesale prices - but we'll always give you notice of a change.",
                    "productCode": "VAR-22-11-01",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-1R-VAR-22-11-01-A",
                    "unitRate": 23.08,
                    "unitRateEpgApplied": false,
                    "preVatUnitRate": null,
                    "__typename": "StandardTariff"
                  }
                },
                {
                  "node": {
                    "id": "176153",
                    "displayName": "Flexible Octopus",
                    "fullName": "Flexible Octopus",
                    "description": "Flexible Octopus offers great value and 100% renewable electricity. As a variable tariff, your prices can rise and fall with wholesale prices - but we'll always give you notice of a change.",
                    "productCode": "VAR-22-11-01",
                    "standingCharge": 48.39,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-2R-VAR-22-11-01-A",
                    "dayRate": 28.52,
                    "dayRateEpgApplied": false,
                    "nightRate": 11.96,
                    "nightRateEpgApplied": false,
                    "preVatDayRate": null,
                    "preVatNightRate": null,
                    "__typename": "DayNightTariff"
                  }
                },
                {
                  "node": {
                    "id": "49383",
                    "displayName": "Flexible Octopus",
                    "fullName": "Flexible Octopus",
                    "description": "Flexible Octopus offers great value and 100% renewable electricity. As a variable tariff, your prices can rise and fall with wholesale prices - but we'll always give you notice of a change.",
                    "productCode": "VAR-22-11-01",
                    "standingCharge": 28.95,
                    "preVatStandingCharge": null,
                    "tariffCode": "G-1R-VAR-22-11-01-A",
                    "__typename": "GasTariffType"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "14202",
            "fullName": "Intelligent Octopus Go - EV Saver",
            "displayName": "Intelligent Octopus Go - EV Saver",
            "description": "An EV tariff exclusively for customers that lease through Octopus EV",
            "availableFrom": "2024-07-17T00:00:00+01:00",
            "availableTo": null,
            "isHidden": true,
            "code": "INTELLI-VAR-OEV-24-07-17",
            "direction": "IMPORT",
            "notes": "",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": true,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": false,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "236702",
                    "displayName": "Intelligent Octopus Go - EV Saver",
                    "fullName": "Intelligent Octopus Go - EV Saver",
                    "description": "An EV tariff exclusively for customers that lease through Octopus EV",
                    "productCode": "INTELLI-VAR-OEV-24-07-17",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": 45.57,
                    "tariffCode": "E-1R-INTELLI-VAR-OEV-24-07-17-A",
                    "dayRate": 24.39,
                    "dayRateEpgApplied": null,
                    "nightRate": 6,
                    "nightRateEpgApplied": null,
                    "preVatDayRate": 23.23,
                    "preVatNightRate": 5.71,
                    "__typename": "DayNightTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "14133",
            "fullName": "Aira Zero",
            "displayName": "Aira Zero",
            "description": "Aira Zero is a heat pump tariff with eight hours of super cheap electricity every day to warm your home.",
            "availableFrom": "2024-07-01T00:00:00+01:00",
            "availableTo": null,
            "isHidden": true,
            "code": "AIRA-ZERO-24-07-01",
            "direction": "IMPORT",
            "notes": "This is a copy of the \"Cosy Octopus\" product, created to be used by the Aira affiliate",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": true,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": false,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "234298",
                    "displayName": "Aira Zero",
                    "fullName": "Aira Zero",
                    "description": "Aira Zero is a heat pump tariff with eight hours of super cheap electricity every day to warm your home.",
                    "productCode": "AIRA-ZERO-24-07-01",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": 45.57,
                    "tariffCode": "E-1R-AIRA-ZERO-24-07-01-A",
                    "dayRate": 33.9,
                    "dayRateEpgApplied": null,
                    "nightRate": 11.46,
                    "nightRateEpgApplied": null,
                    "offPeakRate": 23.38,
                    "offPeakRateEpgApplied": null,
                    "preVatDayRate": 32.29,
                    "preVatNightRate": 10.91,
                    "preVatOffPeakRate": 22.27,
                    "__typename": "ThreeRateTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "14138",
            "fullName": "Octopus Tracker July 2024 v1",
            "displayName": "Octopus Tracker",
            "description": "Tracker gives you the most transparent energy pricing in the UK. Every day, we update the price of your energy based on an independently published wholesale market price. The unit rate is capped at 100p/kWh for electricity and 30p/kWh for gas (including VAT).",
            "availableFrom": "2024-07-01T00:00:00+01:00",
            "availableTo": null,
            "isHidden": true,
            "code": "SILVER-24-07-01",
            "direction": "IMPORT",
            "notes": "",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": false,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": 12,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": true,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [
              "tracker"
            ],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "234480",
                    "displayName": "Octopus Tracker",
                    "fullName": "Octopus Tracker July 2024 v1",
                    "description": "Tracker gives you the most transparent energy pricing in the UK. Every day, we update the price of your energy based on an independently published wholesale market price. The unit rate is capped at 100p/kWh for electricity and 30p/kWh for gas (including VAT).",
                    "productCode": "SILVER-24-07-01",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-1R-SILVER-24-07-01-A",
                    "unitRate": 19.98,
                    "unitRateEpgApplied": false,
                    "preVatUnitRate": null,
                    "__typename": "StandardTariff"
                  }
                },
                {
                  "node": {
                    "id": "73463",
                    "displayName": "Octopus Tracker",
                    "fullName": "Octopus Tracker July 2024 v1",
                    "description": "Tracker gives you the most transparent energy pricing in the UK. Every day, we update the price of your energy based on an independently published wholesale market price. The unit rate is capped at 100p/kWh for electricity and 30p/kWh for gas (including VAT).",
                    "productCode": "SILVER-24-07-01",
                    "standingCharge": 27.47,
                    "preVatStandingCharge": null,
                    "tariffCode": "G-1R-SILVER-24-07-01-A",
                    "__typename": "GasTariffType"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "13979",
            "fullName": "Agile Octopus April 2024 v1",
            "displayName": "Agile Octopus",
            "description": "With Agile Octopus, you get access to half-hourly energy prices, tied to wholesale prices and updated daily.  The unit rate is capped at 100p/kWh (including VAT).",
            "availableFrom": "2024-04-03T00:00:00+01:00",
            "availableTo": null,
            "isHidden": true,
            "code": "AGILE-24-04-03",
            "direction": "IMPORT",
            "notes": "",
            "isVariable": true,
            "isGreen": true,
            "isBusiness": false,
            "isChargedHalfHourly": true,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": 12,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": false,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [
              "agile"
            ],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "230015",
                    "displayName": "Agile Octopus",
                    "fullName": "Agile Octopus April 2024 v1",
                    "description": "With Agile Octopus, you get access to half-hourly energy prices, tied to wholesale prices and updated daily.  The unit rate is capped at 100p/kWh (including VAT).",
                    "productCode": "AGILE-24-04-03",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-1R-AGILE-24-04-03-A",
                    "unitRate": 9.66,
                    "unitRateEpgApplied": false,
                    "preVatUnitRate": null,
                    "__typename": "StandardTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "10338",
            "fullName": "Intelligent Octopus Flux Import",
            "displayName": "Intelligent Octopus Flux Import",
            "description": "Power your home with 100% renewable energy on this Octopus Energy electricity tariff designed exclusively for solar and battery owners.",
            "availableFrom": "2023-07-13T00:00:00+01:00",
            "availableTo": null,
            "isHidden": true,
            "code": "INTELLI-FLUX-IMPORT-23-07-14",
            "direction": "IMPORT",
            "notes": "",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": true,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": false,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "182257",
                    "displayName": "Intelligent Octopus Flux Import",
                    "fullName": "Intelligent Octopus Flux Import",
                    "description": "Power your home with 100% renewable energy on this Octopus Energy electricity tariff designed exclusively for solar and battery owners.",
                    "productCode": "INTELLI-FLUX-IMPORT-23-07-14",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": 45.57,
                    "tariffCode": "E-1R-INTELLI-FLUX-IMPORT-23-07-14-A",
                    "dayRate": 27.7,
                    "dayRateEpgApplied": null,
                    "nightRate": 20.78,
                    "nightRateEpgApplied": null,
                    "preVatDayRate": 26.38,
                    "preVatNightRate": 19.79,
                    "__typename": "DayNightTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "8985",
            "fullName": "Octopus Flux Import",
            "displayName": "Octopus Flux Import",
            "description": "Power your home with 100% renewable energy on this Octopus Energy electricity tariff designed exclusively for solar and battery owners.",
            "availableFrom": "2023-02-14T00:00:00+00:00",
            "availableTo": null,
            "isHidden": true,
            "code": "FLUX-IMPORT-23-02-14",
            "direction": "IMPORT",
            "notes": "",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": true,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": false,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "178297",
                    "displayName": "Octopus Flux Import",
                    "fullName": "Octopus Flux Import",
                    "description": "Power your home with 100% renewable energy on this Octopus Energy electricity tariff designed exclusively for solar and battery owners.",
                    "productCode": "FLUX-IMPORT-23-02-14",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": 45.57,
                    "tariffCode": "E-1R-FLUX-IMPORT-23-02-14-A",
                    "dayRate": 32.32,
                    "dayRateEpgApplied": null,
                    "nightRate": 13.85,
                    "nightRateEpgApplied": null,
                    "offPeakRate": 23.08,
                    "offPeakRateEpgApplied": null,
                    "preVatDayRate": 30.78,
                    "preVatNightRate": 13.19,
                    "preVatOffPeakRate": 21.98,
                    "__typename": "ThreeRateTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "8490",
            "fullName": "Cosy Octopus",
            "displayName": "Cosy Octopus",
            "description": "Cosy Octopus is a heat pump tariff with eight hours of super cheap electricity every day to warm your home.",
            "availableFrom": "2022-12-13T00:00:00+00:00",
            "availableTo": null,
            "isHidden": true,
            "code": "COSY-22-12-08",
            "direction": "IMPORT",
            "notes": "",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": true,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": false,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "177076",
                    "displayName": "Cosy Octopus",
                    "fullName": "Cosy Octopus",
                    "description": "Cosy Octopus is a heat pump tariff with eight hours of super cheap electricity every day to warm your home.",
                    "productCode": "COSY-22-12-08",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": 45.57,
                    "tariffCode": "E-1R-COSY-22-12-08-A",
                    "dayRate": 33.9,
                    "dayRateEpgApplied": null,
                    "nightRate": 11.46,
                    "nightRateEpgApplied": null,
                    "offPeakRate": 23.38,
                    "offPeakRateEpgApplied": null,
                    "preVatDayRate": 32.29,
                    "preVatNightRate": 10.91,
                    "preVatOffPeakRate": 22.27,
                    "__typename": "ThreeRateTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "7833",
            "fullName": "Octopus Go",
            "displayName": "Octopus Go",
            "description": "The smart EV tariff with super cheap electricity between 00:30 - 05:30 every night",
            "availableFrom": "2022-10-14T00:00:00+01:00",
            "availableTo": null,
            "isHidden": true,
            "code": "GO-VAR-22-10-14",
            "direction": "IMPORT",
            "notes": "\" October 2022 v1\" removed from full_name 2024-07-01",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": true,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": false,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "175897",
                    "displayName": "Octopus Go",
                    "fullName": "Octopus Go",
                    "description": "The smart EV tariff with super cheap electricity between 00:30 - 05:30 every night",
                    "productCode": "GO-VAR-22-10-14",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": 45.57,
                    "tariffCode": "E-1R-GO-VAR-22-10-14-A",
                    "dayRate": 24.39,
                    "dayRateEpgApplied": null,
                    "nightRate": 8.5,
                    "nightRateEpgApplied": null,
                    "preVatDayRate": 23.23,
                    "preVatNightRate": 8.1,
                    "__typename": "DayNightTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "7834",
            "fullName": "Intelligent Octopus Go",
            "displayName": "Intelligent Octopus Go",
            "description": "With Intelligent Octopus Go EV tariff, you have access to a super low electricity rate between 23:30 - 05:30 every night, plus it smart-charges your car at the cheapest and greenest times overnight.",
            "availableFrom": "2022-10-14T00:00:00+01:00",
            "availableTo": null,
            "isHidden": true,
            "code": "INTELLI-VAR-22-10-14",
            "direction": "IMPORT",
            "notes": "\" October 2022 v1\" removed from full_name 2024-07-01",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": true,
            "isPrepay": false,
            "isDefault": false,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": false,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "175911",
                    "displayName": "Intelligent Octopus Go",
                    "fullName": "Intelligent Octopus Go",
                    "description": "With Intelligent Octopus Go EV tariff, you have access to a super low electricity rate between 23:30 - 05:30 every night, plus it smart-charges your car at the cheapest and greenest times overnight.",
                    "productCode": "INTELLI-VAR-22-10-14",
                    "standingCharge": 47.85,
                    "preVatStandingCharge": 45.57,
                    "tariffCode": "E-1R-INTELLI-VAR-22-10-14-A",
                    "dayRate": 24.39,
                    "dayRateEpgApplied": null,
                    "nightRate": 7,
                    "nightRateEpgApplied": null,
                    "preVatDayRate": 23.23,
                    "preVatNightRate": 6.67,
                    "__typename": "DayNightTariff"
                  }
                }
              ]
            }
          }
        },
        {
          "node": {
            "id": "191",
            "fullName": "Octopus Key and Card",
            "displayName": "Octopus Key and Card",
            "description": "Non-smart prepayment tariff",
            "availableFrom": "2018-10-10T00:00:00+01:00",
            "availableTo": null,
            "isHidden": true,
            "code": "PREPAY-VAR-18-09-21",
            "direction": "IMPORT",
            "notes": "",
            "isVariable": true,
            "isGreen": false,
            "isBusiness": false,
            "isChargedHalfHourly": false,
            "isPrepay": true,
            "isDefault": true,
            "isOccupier": false,
            "term": null,
            "isAvailable": true,
            "isUnavailable": false,
            "isFixed": false,
            "isDomestic": true,
            "includesEpgReduction": true,
            "exitFees": 0,
            "exitFeesType": "NONE",
            "tags": [],
            "__typename": "EnergyProductType",
            "tariffs": {
              "edges": [
                {
                  "node": {
                    "id": "6393",
                    "displayName": "Octopus Key and Card",
                    "fullName": "Octopus Key and Card",
                    "description": "Non-smart prepayment tariff",
                    "productCode": "PREPAY-VAR-18-09-21",
                    "standingCharge": 47.86,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-1R-PREPAY-VAR-18-09-21-A",
                    "unitRate": 22.3,
                    "unitRateEpgApplied": false,
                    "preVatUnitRate": null,
                    "__typename": "StandardTariff"
                  }
                },
                {
                  "node": {
                    "id": "6421",
                    "displayName": "Octopus Key and Card",
                    "fullName": "Octopus Key and Card",
                    "description": "Non-smart prepayment tariff",
                    "productCode": "PREPAY-VAR-18-09-21",
                    "standingCharge": 48.4,
                    "preVatStandingCharge": null,
                    "tariffCode": "E-2R-PREPAY-VAR-18-09-21-A",
                    "dayRate": 28.66,
                    "dayRateEpgApplied": false,
                    "nightRate": 10.34,
                    "nightRateEpgApplied": false,
                    "preVatDayRate": null,
                    "preVatNightRate": null,
                    "__typename": "DayNightTariff"
                  }
                },
                {
                  "node": {
                    "id": "2115",
                    "displayName": "Octopus Key and Card",
                    "fullName": "Octopus Key and Card",
                    "description": "Non-smart prepayment tariff",
                    "productCode": "PREPAY-VAR-18-09-21",
                    "standingCharge": 28.95,
                    "preVatStandingCharge": null,
                    "tariffCode": "G-1R-PREPAY-VAR-18-09-21-A",
                    "__typename": "GasTariffType"
                  }
                }
              ]
            }
          }
        }
      ]
    }
  }
}
```
Note that this query returns instances of `EnergyTariffType` which does not include `HalfHourlyTariff` or `PrepayTariff` so in this sense at least, Intelligent Go is not an Energy Product. 

## getBills
### Query
```gql
query getBills($accountNumber: String!) {
  account(accountNumber: $accountNumber) {
    status
    number
    balance
    bills(first: 1) {
      pageInfo {
        startCursor
        hasNextPage
      }
      edges {
        node {
          billType
          fromDate
          toDate
          issuedDate
          __typename
          ...on StatementType {
            closingBalance
            openingBalance
            isExternalBill
            transactions(
            first: 100
            ) {
              pageInfo {
                startCursor
                hasNextPage
              }
              edges {
                node {
                  postedDate
                  createdAt
                  amounts {
                    net
                    tax
                    gross
                  }
                  balanceCarriedForward
                  isHeld
                  isIssued
                  title
                  isReversed
                  hasStatement
                  note
                  ... on Charge {
                    consumption {
                      startDate
                      endDate
                      quantity
                      unit
                      usageCost
                      supplyCharge
                    }
                    isExport
                  }
                  __typename
                }
              }
            }

            userId
            toAddress
            paymentDueDate
            consumptionStartDate
            consumptionEndDate
            reversalsAfterClose
            status
            heldStatus {
              isHeld
              reason
            }
            totalCharges {
              netTotal
              taxTotal
              grossTotal
            }
            totalCredits {
              netTotal
              taxTotal
              grossTotal
            }
          }
        }
      }
    }
  }
}
```

### Variables
```json
{
  "accountNumber":"A-B1C2D34E"
}
```

### Example Response
```json
{
  "data": {
    "account": {
      "status": "ACTIVE",
      "number": "A-B1C2D34E",
      "balance": 39303,
      "bills": {
        "pageInfo": {
          "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
          "hasNextPage": true
        },
        "edges": [
          {
            "node": {
              "billType": "STATEMENT",
              "fromDate": "2024-07-22",
              "toDate": "2024-08-21",
              "issuedDate": "2024-08-22",
              "__typename": "StatementType",
              "closingBalance": 39303,
              "openingBalance": 17791,
              "isExternalBill": false,
              "transactions": {
                "pageInfo": {
                  "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
                  "hasNextPage": false
                },
                "edges": [
                  {
                    "node": {
                      "postedDate": "2024-08-20",
                      "createdAt": "2024-08-21T21:36:10.492186+00:00",
                      "amounts": {
                        "net": 2711,
                        "tax": 136,
                        "gross": 2847
                      },
                      "balanceCarriedForward": 39303,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Gas",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-07-21",
                        "endDate": "2024-08-20",
                        "quantity": "360.7100",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": false,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-08-20",
                      "createdAt": "2024-08-21T21:32:19.902722+00:00",
                      "amounts": {
                        "net": -2716,
                        "tax": 0,
                        "gross": -2716
                      },
                      "balanceCarriedForward": 42150,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Electricity",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-08-13",
                        "endDate": "2024-08-20",
                        "quantity": "181.0500",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": true,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-08-20",
                      "createdAt": "2024-08-21T21:32:01.991119+00:00",
                      "amounts": {
                        "net": 2854,
                        "tax": 143,
                        "gross": 2997
                      },
                      "balanceCarriedForward": 39434,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Electricity",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-08-08",
                        "endDate": "2024-08-20",
                        "quantity": "334.7100",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": false,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-08-14",
                      "createdAt": "2024-08-15T11:55:19.400763+00:00",
                      "amounts": {
                        "net": 478,
                        "tax": 24,
                        "gross": 502
                      },
                      "balanceCarriedForward": 42431,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Powerups Reward",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "__typename": "Credit"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-08-12",
                      "createdAt": "2024-08-21T21:32:19.073366+00:00",
                      "amounts": {
                        "net": -2407,
                        "tax": 0,
                        "gross": -2407
                      },
                      "balanceCarriedForward": 41929,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Electricity",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-07-21",
                        "endDate": "2024-08-12",
                        "quantity": "300.8200",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": true,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-08-07",
                      "createdAt": "2024-08-21T21:32:01.008991+00:00",
                      "amounts": {
                        "net": 4104,
                        "tax": 205,
                        "gross": 4309
                      },
                      "balanceCarriedForward": 39522,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Electricity",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-07-21",
                        "endDate": "2024-08-07",
                        "quantity": "322.5100",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": false,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-07-29",
                      "createdAt": "2024-08-01T03:09:50.202838+00:00",
                      "amounts": {
                        "net": 24790,
                        "tax": 0,
                        "gross": 0
                      },
                      "balanceCarriedForward": 43831,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Direct debit",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": null,
                      "__typename": "Payment"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-07-24",
                      "createdAt": "2024-07-25T10:53:30.897903+00:00",
                      "amounts": {
                        "net": 543,
                        "tax": 28,
                        "gross": 571
                      },
                      "balanceCarriedForward": 19041,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Powerups Reward",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "__typename": "Credit"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-07-24",
                      "createdAt": "2024-07-25T10:43:02.339290+00:00",
                      "amounts": {
                        "net": 177,
                        "tax": 9,
                        "gross": 186
                      },
                      "balanceCarriedForward": 18470,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Powerups Reward",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "__typename": "Credit"
                    }
                  },
                  {
                    "node": {
                      "postedDate": "2024-07-24",
                      "createdAt": "2024-07-25T10:17:07.255688+00:00",
                      "amounts": {
                        "net": 469,
                        "tax": 24,
                        "gross": 493
                      },
                      "balanceCarriedForward": 18284,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Powerups Reward",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "__typename": "Credit"
                    }
                  }
                ]
              },
              "userId": 3235447,
              "toAddress": "dan@archer.org",
              "paymentDueDate": "2024-09-06",
              "consumptionStartDate": null,
              "consumptionEndDate": null,
              "reversalsAfterClose": "NONE",
              "status": "CLOSED",
              "heldStatus": {
                "isHeld": false,
                "reason": null
              },
              "totalCharges": {
                "netTotal": 4546,
                "taxTotal": 484,
                "grossTotal": 5030
              },
              "totalCredits": {
                "netTotal": 1667,
                "taxTotal": 85,
                "grossTotal": 1752
              }
            }
          }
        ]
      }
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
{
  "data": {
    "account": {
      "status": "ACTIVE",
      "number": "A-B1C2D34E",
      "balance": 39303,
      "bills": {
        "pageInfo": {
          "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
          "hasNextPage": true
        },
        "edges": [
          {
            "node": {
              "id": "236646425",
              "billType": "STATEMENT",
              "fromDate": "2024-07-22",
              "toDate": "2024-08-21",
              "issuedDate": "2024-08-22",
              "__typename": "StatementType",
              "closingBalance": 39303,
              "openingBalance": 17791,
              "isExternalBill": false,
              "transactions": {
                "pageInfo": {
                  "startCursor": "YXJyYXljb25uZWN0aW9uOjA=",
                  "hasNextPage": false
                },
                "edges": [
                  {
                    "node": {
                      "id": "-1871040199",
                      "postedDate": "2024-08-20",
                      "createdAt": "2024-08-21T21:36:10.492186+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": 2711,
                        "tax": 136,
                        "gross": 2847
                      },
                      "balanceCarriedForward": 39303,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Gas",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-07-21",
                        "endDate": "2024-08-20",
                        "quantity": "360.7100",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": false,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "id": "-1871043601",
                      "postedDate": "2024-08-20",
                      "createdAt": "2024-08-21T21:32:19.902722+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": -2716,
                        "tax": 0,
                        "gross": -2716
                      },
                      "balanceCarriedForward": 42150,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Electricity",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-08-13",
                        "endDate": "2024-08-20",
                        "quantity": "181.0500",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": true,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "id": "-1871044025",
                      "postedDate": "2024-08-20",
                      "createdAt": "2024-08-21T21:32:01.991119+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": 2854,
                        "tax": 143,
                        "gross": 2997
                      },
                      "balanceCarriedForward": 39434,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Electricity",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-08-08",
                        "endDate": "2024-08-20",
                        "quantity": "334.7100",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": false,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "id": "-1896251302",
                      "postedDate": "2024-08-14",
                      "createdAt": "2024-08-15T11:55:19.400763+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": 478,
                        "tax": 24,
                        "gross": 502
                      },
                      "balanceCarriedForward": 42431,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Powerups Reward",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "__typename": "Credit"
                    }
                  },
                  {
                    "node": {
                      "id": "-1871043620",
                      "postedDate": "2024-08-12",
                      "createdAt": "2024-08-21T21:32:19.073366+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": -2407,
                        "tax": 0,
                        "gross": -2407
                      },
                      "balanceCarriedForward": 41929,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Electricity",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-07-21",
                        "endDate": "2024-08-12",
                        "quantity": "300.8200",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": true,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "id": "-1871044052",
                      "postedDate": "2024-08-07",
                      "createdAt": "2024-08-21T21:32:01.008991+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": 4104,
                        "tax": 205,
                        "gross": 4309
                      },
                      "balanceCarriedForward": 39522,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Electricity",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "consumption": {
                        "startDate": "2024-07-21",
                        "endDate": "2024-08-07",
                        "quantity": "322.5100",
                        "unit": "kWh",
                        "usageCost": 0,
                        "supplyCharge": 0
                      },
                      "isExport": false,
                      "__typename": "Charge"
                    }
                  },
                  {
                    "node": {
                      "id": "-1949392858",
                      "postedDate": "2024-07-29",
                      "createdAt": "2024-08-01T03:09:50.202838+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": 24790,
                        "tax": 0,
                        "gross": 0
                      },
                      "balanceCarriedForward": 43831,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Direct debit",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": null,
                      "__typename": "Payment"
                    }
                  },
                  {
                    "node": {
                      "id": "-1973989678",
                      "postedDate": "2024-07-24",
                      "createdAt": "2024-07-25T10:53:30.897903+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": 543,
                        "tax": 28,
                        "gross": 571
                      },
                      "balanceCarriedForward": 19041,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Powerups Reward",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "__typename": "Credit"
                    }
                  },
                  {
                    "node": {
                      "id": "-1974036696",
                      "postedDate": "2024-07-24",
                      "createdAt": "2024-07-25T10:43:02.339290+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": 177,
                        "tax": 9,
                        "gross": 186
                      },
                      "balanceCarriedForward": 18470,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Powerups Reward",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "__typename": "Credit"
                    }
                  },
                  {
                    "node": {
                      "id": "-1974103763",
                      "postedDate": "2024-07-24",
                      "createdAt": "2024-07-25T10:17:07.255688+00:00",
                      "accountNumber": "A-B1C2D34E",
                      "amounts": {
                        "net": 469,
                        "tax": 24,
                        "gross": 493
                      },
                      "balanceCarriedForward": 18284,
                      "isHeld": false,
                      "isIssued": true,
                      "title": "Powerups Reward",
                      "billingDocumentIdentifier": "236646425",
                      "isReversed": false,
                      "hasStatement": true,
                      "note": "",
                      "__typename": "Credit"
                    }
                  }
                ]
              },
              "userId": 3235447,
              "toAddress": "dan@archer.org",
              "paymentDueDate": "2024-09-06",
              "consumptionStartDate": null,
              "consumptionEndDate": null,
              "reversalsAfterClose": "NONE",
              "status": "CLOSED",
              "heldStatus": {
                "isHeld": false,
                "reason": null
              },
              "totalCharges": {
                "netTotal": 4546,
                "taxTotal": 484,
                "grossTotal": 5030
              },
              "totalCredits": {
                "netTotal": 1667,
                "taxTotal": 85,
                "grossTotal": 1752
              }
            }
          }
        ]
      }
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
