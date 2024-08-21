[< Octopus Energy Plugin](index.md)

# Web UI Queries
This section describes the queries made by the Octopus Web UI [https://octopus.energy/dashboard](https://octopus.energy/dashboard). While this is purely internal implementation detail which may change at any time, it's informative to look at as a way of understanding how to navigate to the information you may want to extract via your own API calls.


The [Chrome Inspector](../ChromeInspector.md) page describes how you can use the Inspector function of the Chrome browser to observe these queries for yourself.

 ## Redaction
 Some of the values in the examples below, such as account numbers and meter IDs have been redacted. Characters which appear to be alpha or alpha-numeric have been replaced with `A`, characters which appear to be numeric have been replaced with `9`. In some cases there is documentary evidence of what the actual type of the returned values are (e.g. the GraphQL schema defines the `number` attribute of `AccountInterface` to be `String` (interestingly not `String!` although one imagines that this is not an optional attribute)), in other cases there is not. There *appears* to be further structure in some values, e.g. Account numbers appear to have a hyphen in the second character. In order to aid recognition when comparing examples here to actual returned values some of this apparent structure may be represented in the redaction, so a redacted account number may be shown as `A-AAAAAAAA`. This apparent structure *cannot* be relied upon, when writing code you should code against the official schema so an account number should be stored as an unbounded optional (nullable) string value despite the fact that all observed values appear to be of fixed length with some internal structure.

## Trying These Calls For Yourself
For each call in the following section we show the `Query` and `Variables`, these values can be copied and pasted into https://api.octopus.energy/v1/graphql-playground to execute the query interactively. The official API documentation contains a link to https://api.octopus.energy/v1/graphql which is another very similar, but apparently older, version of the same thing.

Before you can do this you need to authenticate (log in), this can be done using your email address and password, or API key. I strongly recommend *not* storing your password anywhere including interactive GraphQL UIs, so the first step is to find your API Key. 

This is shown at https://octopus.energy/dashboard/new/accounts/personal-details/api-access

![alt text](../APIkey.png)

If you click on the API key it will be copied to your clipboard.

Now navigate to https://api.octopus.energy/v1/graphql-playground, you should see this:

![alt text](../GraphQlPlayground.png)

Paste the following query into the main panel on the left where it says `# Write your query or mutation here`

```gql
mutation obtainKrakenToken($input: ObtainJSONWebTokenInput!) {
  obtainKrakenToken(input: $input) {
    payload
    token
  }
}
```

Now click on `QUERY VARIABLES` at the bottom and paste the following into the panel which opens below, replacing the APIKey placeholder with your key:
```gql
{
    "input": {
        "APIKey": "sk_XXXX_XXXXXXXXXXXXXXXXXXXXXXXX"
    }
}
```

Now click the big play button in the centre and on the right hand side you should see a response similar to this:

```gql
{
  "data": {
    "obtainKrakenToken": {
      "payload": {
        "sub": "kraken|account-user:9999999",
        "gty": "API-KEY",
        "email": "you@your.domain",
        "tokenUse": "access",
        "iss": "https://api.octopus.energy/v1/graphql/",
        "iat": 1724155372,
        "exp": 1724158972,
        "origIat": 1724155372
      },
      "token": 
      "qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop12345678
      qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop123456789
      qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop123456789
      qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop123456789
      qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop123456789
      qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop123456789
      qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop123456789
      qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop123456789
      qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwertyuiop123456789
      qwertyuiop1234567890qwertyuiop1234567890qwert"
    }
  }
}
```
This is a [JSON Web Token (JWT)](https://jwt.io/) which is a short lived bearer token (i.e. a token, possession of which is considered as proof of identity). Although this is a short lived token you should treat it as sensitive and not store it anywhere unnecessarily. If you paste the `token` value into the JWT website above you will see the contents and as you will see, the payload of the JWT is also provided in the GraphQL response as the `payload` attribute as a convenience.

I believe the jwt.io site is safe and does not copy the tokens pasted into it, but cannot guarantee this. Extreme caution should be used with other sites which offer similar functionality.

The `iat` attribute is the time when the token was issued, if you paste the value `1724155372` into https://www.epochconverter.com/ you will see that this is a Unix timestamp (starting from 1st January 1970) in seconds for Tuesday, 20 August 2024 12:02:52.

The `exp` attribute is the expiry time for the token, in this case `1724158972` or Tuesday, 20 August 2024 13:02:52, so we can see that the token is valid for a maximum of 1 hour.

If you are writing an application to call the API you will need to refresh the token each time before it expires. Using the graphql-playground you will need to repeat the process above and update the token as and when it expires.

Once you have a token, click on the `HTTP HEADERS` tab at the bottom left of the window and paste in this, replacing the token placeholder with the value you just received:

```gql
{
  "Authorization":	"qwertyuiop1234567890"
}
```

For each of the queries shown below you can try for yourself by pasting the `Query` and `Variables` into the graphql-playground as we did above. In the variables section any placeholder of the form `AAAAA` or `99999` must be replaced with the appropriate value for your account.

## Observed API calls - Preamble
This section describes the initial sequence of HTTP requests which I have observed as a result of entering the URL https://octopus.energy/dashboard and my notes as to what is happening. These calls are *NOT* part of the published API and I do *NOT* recommend calling them yourself. They are included here as a way of understanding how the Octopus dashboard UI works in order to understand better the usage of the official API calls which follow.

In the examples here I had previously logged in, if you are not authenticated you will be redirected to the login page and there will be more calls not shown here before the sequence below will follow.

### GET https://octopus.energy/dashboard/
This is a simple HTTP GET request which the results in a `302 Found` response. This is a simple HTTP redirect to `/dashboard/accounts/A-AAAAAAAA/` (where the last element is your actual account number) which you can see in the `Location` response header.

The actual web dashboard application obviously knows the user's account number by this point, our GraphQL application will not.

There are then several other redirects eventually getting to

### GET https://octopus.energy/dashboard/new/accounts/A-AAAAAAAA/
This is the actual HTML for the web UI.

## Observed API calls - GraphQL Queries
This section describes the sequence of GraphQL API calls which I have observed and my notes as to what is happening. As I have already mentioned, this is internal implementation detail and there is every reason to expect that this will change over time, so don't be surprised if you look for yourself and see something different.



### getOctoplusEligibilityandFeatureFlags
#### Query
```gql
query getOctoplusEligibilityandFeatureFlags($accountNumber: String!) {
  octoplusAccountInfo(accountNumber: $accountNumber) {
    isOctoplusEnrolled
    octoplusEligibility {
      isEligible
      __typename
    }
    __typename
  }
  octoplusFeatureFlags {
    shouldClientDisplayOctoplus
    __typename
  }
}
"
```

#### Variables
```gql
{"accountNumber":"A-AAAAAAAA"}
```

#### Example Response
```gql
{
  "data": {
    "octoplusAccountInfo": {
      "isOctoplusEnrolled": true,
      "octoplusEligibility": {
        "isEligible": true,
        "__typename": "OctoplusEligibilityType"
      },
      "__typename": "OctoplusAccountInfoType"
    },
    "octoplusFeatureFlags": {
      "shouldClientDisplayOctoplus": true,
      "__typename": "OctoplusFeatureFlagsType"
    }
  }
}
```
This is the first GraphQL call made and it takes the Account Number as a parameter, which must have been retrieved by one of the earlier (non-GraphQL) calls. The account number is in the URL path of some of the redirect URLs so it could be being picked up from there or it may come from one of the other non-GraphQL calls which we have not described. This information is also available in the `My account` section of the website so you can get your own account number that way if you prefer.

In any event, this information is available elsewhere (see `getLoggedInUser` below) so if you are writing a GraphQL application you should get this value like that.

This query is retrieving some information about `Octoplus` which appears to control some aspect of the UI.

The query requests the type names of several of the returned objects (by including `__typename` in the list of fields in the query). It's not clear why the web UI does this, and these will be omitted in most examples. If you want to see the schema definition of the returned type for any query you can just add `__typename` to the list of fields in any query, then click on `SCHEMA` on the right hand edge of the graphql-playground window, type `Ctl-F` (or `COMMAND-f` on a Mac) to find, paste in the typename in the query response (e.g. `OctoplusEligibilityType` in this example) and press `RETURN`.

You should then see the schema definition for the type like this:
![Schema Example](Schema.png)
### getAccountInfo
#### Query
```gql
query getAccountInfo($accountNumber: String!) {
  account(accountNumber: $accountNumber) {
    activeReferralSchemes {
      domestic {
        referralUrl
        referrerRewardAmount
      }
    }
    balance
    accountType
  }
}
```

#### Variables
```gql
{"accountNumber":"A-AAAAAAAA"}
```

#### Example Response
```gql
{
    "data": {
        "account": {
            "activeReferralSchemes": {
                "domestic": {
                    "referralUrl": "https://share.octopus.energy/new-mouse-310",
                    "referrerRewardAmount": 5000
                }
            },
            "balance": 44333,
            "accountType": "DOMESTIC"
        }
    }
}
```

This query fetches some information about the account including the type (`DOMESTIC`) and the current balance (£443.33 in credit in this example). As we can see, currency amounts are represented as scaled integers (or you could think of it as the amount in pence rather than pounds), presumably to avoid issues with rounding floating point numbers.
### getLoggedInUser
#### Query
```gql
query getLoggedInUser {
  viewer {
    accounts {
      number,
      __typename
    },
    __typename
  }
}

```

#### Variables
```gql
{}
```

#### Example Response
```json
{
    "data": {
        "viewer": {
            "accounts": [
                {
                    "number": "A-AAAAAAAA",
                    "__typename": "AccountType"
                }
            ],
            "__typename": "AccountUserType"
        }
    }
}
```

This would most likely be the starting query for a pure GraphQL application. It takes no parameters and returns the Account number for the logged in user. Note that the `accounts` attribute is an array value so you could conceivably receive multiple account numbers back although probably not for domestic consumers.

This is an indication that although Octopus publish their APIs and encourage use of them by consumer end users, they are only one (and probably in many ways, the least important) audience for the API.

Note that ``getLoggedInUser`` is just a label being applied to this query, you will not find that name in the Schema. The actual GraphQL query being accessed is the `viewer` query which provides information about a number of object types linked to the current logged in user (or viewer).

As you can see, the query requests the `__typename` of each of the returned types and the response indicates that the returned object is an `AccountUserType` containing an array with a single objects of type `AccountType`. The schema indicates that `AccountUserType` contains an array of `AccountInterface` objects.

Here is part of the schema definition of `AccountType`:

```gql
# User objects are the core of the authentication system. They typically represent a customer who manages a portfolio of one or more accounts.
type AccountUserType {
  id: ID!

  # List of accounts that the user is linked to either via portfolio role or account role.
  accounts(
    # Optionally filter the user's accounts to only return those linked to portfolios on the specified brands.
    allowedBrandCodes: [BrandChoices]

    # Optionally restrict user accounts to only return those linked to portfolios on public facing brands.
    restrictToPublicFacingBrands: Boolean

    # Optionally restrict user accounts to only return those with the specified account numbers.
    restrictToAccountNumbers: [String]

    # Optionally exclude accounts with any of the given account types.
    excludeAccountTypes: [AccountTypeChoices]

    # Optionally exclude accounts that have never had an agreement.
    excludeAccountsWithoutAgreements: Boolean
  ): [AccountInterface]

# many lines not shown

}
```
And here is part of the definition of `AccountInterface`:

```gql
interface AccountInterface {
  # The brand of the account.
  brand: String

  # The current status of the account.
  status: AccountStatus

# many lines not shown

}
```

If you try to add `eligibilityForWarmHomeDiscount` to the list of attributes returned for the accounts it will not work, because that is not an attribute of `AccountInterface`, although it *is* an attribute of `AccountType`. 

It the case of queries which return an interface, GraphQL servers return the `__typename` of the underlying object rather than the name of the interface. It is also possible to query attributes of the concrete type using GraphQL's `on typename` syntax, which is similar to a programming language type cast. So the query could be rewritten to receive the `getWHDEligibility` attribute for the account like this:

```gql
query getLoggedInUser {
  viewer {
    accounts {
      number
      ...on AccountType {
        eligibilityForWarmHomeDiscount {
            isEligible
        }
      }
    }
  }
}
```

It is unclear why the `AccountInterface` interface exists because the only implementing type in the schema is `AccountType`. The web UI makes a separate request to fetch the `getWHDEligibility` attribute, which may indicate that it's unsafe to rely on the concrete type in this way, or it may not, there is no way to tell.

### getLoggedInUser (again)
#### Query
```gql
query getLoggedInUser {
  viewer {
    preferredName
  }
}

```

#### Variables
```json
{}
```

#### Example Response
```gql
{
  "data": {
    "viewer": {
      "preferredName": "Bruce"
    }
  }
}
```

This is a separate call (repeating the name getLoggedInUser) to fetch the user's preferred name. This could have been done as part of the previous call.
### getWHDEligibility
#### Query
```gql
query getWHDEligibility($accountNumber: String!) {
  account(accountNumber: $accountNumber) {
    eligibilityForWarmHomeDiscount {
      isEligible
    }
  }
}
```

#### Variables
```json
{
    "accountNumber":"A-AAAAAAAA"
}
```

#### Example Response
```json
{
    "data": {
        "viewer": {
            "accounts": [
                {
                    "number": "A-AAAAAAAA",
                    "__typename": "AccountType"
                }
            ],
            "__typename": "AccountUserType"
        }
    }
}
```

This request fetches the account's eligibility for the Warm Home Discount. This could be done in the `viewer` queries above as already discussed.

### getPropertiesMeterPoints
#### Query
```gql
query getPropertiesMeterPoints($accountNumber: String!, $propertiesActiveFrom: DateTime) {
  account(accountNumber: $accountNumber) {
    properties(activeFrom: $propertiesActiveFrom) {
      id
      electricityMeterPoints {
        id
        meters(includeInactive: false) {
          id
          smartDevices {
            deviceId
          }
        }
      }
      gasMeterPoints {
        id
        meters {
          id
          smartDevices {
            deviceId
          }
        }
      }
    }
  }
}
```

#### Variables
```json
{
    "accountNumber":"A-AAAAAAAA",
    "propertiesActiveFrom":"2024-08-12T23:00:00.000Z"
}
```

#### Example Response
```json
{
    "data": {
        "account": {
            "properties": [
                {
                    "id": "999999",
                    "electricityMeterPoints": [
                        {
                            "id": "999999",
                            "meters": [
                                {
                                    "id": "999999",
                                    "smartDevices": [
                                        {
                                            "deviceId": "AA-AA-AA-AA-AA-AA-AA-AA"
                                        }
                                    ]
                                }
                            ]
                        },
                        {
                            "id": "999999",
                            "meters": [
                                {
                                    "id": "999999",
                                    "smartDevices": [
                                        {
                                            "deviceId": "AA-AA-AA-AA-AA-AA-AA-AA"
                                        }
                                    ]
                                }
                            ]
                        }
                    ],
                    "gasMeterPoints": [
                        {
                            "id": "999999",
                            "meters": [
                                {
                                    "id": "999999",
                                    "smartDevices": [
                                        {
                                            "deviceId": "AA-AA-AA-AA-AA-AA-AA-AA"
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
This query enumerates all the properties (physical locations) associated with the given account, and for each property, all the gas and electricity meter points and their meters and smart devices.

Note that the `id` attribute of the meters here is the value required for the `meterId` parameter of other queries to fetch meter readings and the like.

It is unclear why the particular value for the `propertiesActiveFrom` parameter was chosen, this request was made on August 20 so the given date is about 12 days prior to that.

### getAccount
#### Query
```gql
query getAccount($accountNumber: String!, $propertiesActiveFrom: DateTime) {
  account(accountNumber: $accountNumber) {
    applications(first: 1) {
      edges {
        node {
          salesSubchannel
        }
      }
    }
    accountType
    brand
    electricityAgreements(active: true) {
      tariff {
        ... on StandardTariff {
          productCode
        }
        ... on DayNightTariff {
          productCode
        }
        ... on ThreeRateTariff {
          productCode
        }
        ... on HalfHourlyTariff {
          productCode
        }
        ... on PrepayTariff {
          productCode
        }
      }
    }
    status
    number
    balance
    canRenewTariff
    recommendedBalanceAdjustment
    smets2Interest
    canChangePayments
    directDebitInstructions(first: 1) {
      edges {
        node {
          id
        }
      }
    }
    campaigns {
      name
    }
    properties(activeFrom: $propertiesActiveFrom) {
      id
      occupancyPeriods {
        effectiveTo
      }
      electricityMeterPoints {
        id
        mpan
        smartStartDate
        profileClass
        meters {
          hasAndAllowsHhReadings
          serialNumber
          isTradPrepay
          smartDevices {
            paymentMode
            deviceId
          }
          isReadyForTopup
        }
      }
      gasMeterPoints {
        id
        mprn
        smartStartDate
        meters {
          hasAndAllowsHhReadings
          serialNumber
          isTradPrepay
          smartDevices {
            paymentMode
            deviceId
          }
          isReadyForTopup
        }
      }
      isSmets2InstallationAllowed
    }
    bills(first: 1) {
      edges {
        node {
          issuedDate
        }
      }
    }
    transactions(first: 1) {
      edges {
        node {
          id
          amount
          postedDate
          isHeld
          isIssued
          title
          statementId
        }
      }
    }
    repayments(first: 1, statuses: [REQUESTED, APPROVED, SUBMITTED]) {
      edges {
        node {
          id
        }
      }
    }
  }
  fanClubStatus(accountNumber: $accountNumber) {
    discountSource
    accountNumbers
    propertyIds
  }
  balanceForecast(accountNumber: $accountNumber) {
    isAvailable
  }
  viewer {
    preferences {
      isOptedInToUpdateMessages
      isOptedInToOfferMessages
    }
  }
}

```

#### Variables
```json
{
    "accountNumber":"A-AAAAAAAA",
    "propertiesActiveFrom":"2024-08-12T23:00:00.000Z"
}
```

#### Example Response
```json
{
    "data": {
        "account": {
            "applications": {
                "edges": [
                    {
                        "node": {
                            "salesSubchannel": "",
                        },
                    }
                ],
            },
            "accountType": "DOMESTIC",
            "brand": "OCTOPUS_ENERGY",
            "electricityAgreements": [
                {
                    "tariff": {
                        "productCode": "OUTGOING-FIX-12M-19-05-13",
                    },
                },
                {
                    "tariff": {
                        "productCode": "INTELLI-VAR-22-10-14",
                    },
                }
            ],
            "status": "ACTIVE",
            "number": "A-AAAAAAAA",
            "balance": 44333,
            "canRenewTariff": false,
            "recommendedBalanceAdjustment": null,
            "smets2Interest": "INTERESTED",
            "canChangePayments": true,
            "directDebitInstructions": {
                "edges": [
                    {
                        "node": {
                            "id": "9999999",
                        },
                    }
                ],
            },
            "campaigns": [
                {
                    "name": "Octoplus",
                },
                {
                    "name": "Octoplus Saving Sessions",
                },
                {
                    "name": "Power-Ups UKPN",
                },
                {
                    "name": "SMETS2_CALL_LIST",
                }
            ],
            "properties": [
                {
                    "id": "9999999",
                    "occupancyPeriods": [
                        {
                            "effectiveTo": null,
                        }
                    ],
                    "electricityMeterPoints": [
                        {
                            "id": "9999999",
                            "mpan": "9999999999999",
                            "smartStartDate": "9999-99-99",
                            "profileClass": 1,
                            "meters": [
                                {
                                    "hasAndAllowsHhReadings": true,
                                    "serialNumber": "99A9999999",
                                    "isTradPrepay": false,
                                    "smartDevices": [
                                        {
                                            "paymentMode": "CREDIT",
                                            "deviceId": "AA-AA-AA-AA-AA-AA-AA-AA",
                                        }
                                    ],
                                    "isReadyForTopup": false,
                                }
                            ]
                        },
                        {
                            "id": "9999999",
                            "mpan": "9999999999999",
                            "smartStartDate": "9999-99-99",
                            "profileClass": 8,
                            "meters": [
                                {
                                    "hasAndAllowsHhReadings": true,
                                    "serialNumber": "99A9999999",
                                    "isTradPrepay": false,
                                    "smartDevices": [
                                        {
                                            "paymentMode": "CREDIT",
                                            "deviceId": "AA-AA-AA-AA-AA-AA-AA-AA",
                                        }
                                    ],
                                    "isReadyForTopup": false,
                                }
                            ]
                        }
                    ],
                    "gasMeterPoints": [
                        {
                            "id": "9999999",
                            "mprn": "9999999999",
                            "smartStartDate": "9999-99-99",
                            "meters": [
                                {
                                    "hasAndAllowsHhReadings": true,
                                    "serialNumber": "A9A99999999999",
                                    "isTradPrepay": false,
                                    "smartDevices": [
                                        {
                                            "paymentMode": "CREDIT",
                                            "deviceId": "AA-AA-AA-AA-AA-AA-AA-AA",
                                        }
                                    ],
                                    "isReadyForTopup": false,
                                }
                            ]
                        }
                    ],
                    "isSmets2InstallationAllowed": false,
                }
            ],
            "bills": {
                "edges": [
                    {
                        "node": {
                            "issuedDate": "9999-99-99",
                        },
                    }
                ],
            },
            "transactions": {
                "edges": [
                    {
                        "node": {
                            "id": "-999999999",
                            "amount": 502,
                            "postedDate": "9999-99-99",
                            "isHeld": false,
                            "isIssued": false,
                            "title": "Powerups Reward",
                            "statementId": "999999999",
                        },
                    }
                ],
            },
            "repayments": {
                "edges": [],
            },
        },
        "fanClubStatus": [],
        "balanceForecast": {
            "isAvailable": false,
        },
        "viewer": {
            "preferences": {
                "isOptedInToUpdateMessages": false,
                "isOptedInToOfferMessages": false,
            },
        }
    }
}
```
This looks like a good candidate for the second query of a GraphQL application. It requires the account number as a parameter (and what looks like a more or less arbitrary effective date for properties) and returns a whole host of information including:

* Account Type (DOMESTIC)
* Brand (OCTOPUS_ENERGY)
* Account Status (ACTIVE)
* Balance
* ID of direct debit mandates
* All Gas and Electricity meter points including
    * meterId
    * mpan or mprn
    * Serial Number
    * Smart Device ID
* Date of most recent bill
* Most recent bill transaction
* Most recent repayment

### getOctoPointsBalance
#### Query
```gql
query getOctoPointsBalance {
  loyaltyPointLedgers {
    balanceCarriedForward
  }
}
```

#### Variables
```json
{}
```

#### Example Response
```json
{
    "data": {
        "loyaltyPointLedgers": [
            {
                "balanceCarriedForward": "1524",
            },
            {
                "balanceCarriedForward": "1516",
            },
            {
                "balanceCarriedForward": "1508",
            },
            {
                "balanceCarriedForward": "1308",
            },
            {
                "balanceCarriedForward": "500",
            }
        ]
    }
}
```
Fetches OctoPoints balances.

### getSmartMeterInstallationEligibility
#### Query
```gql
query getSmartMeterInstallationEligibility($accountNumber: String!) {
  account(accountNumber: $accountNumber) {
    smets2Interest
    smets2RefusalReason
    properties {
      id
      isSmets2InstallationAllowed
      electricityMeterPoints {
        meters {
          meterType
        }
      }
      gasMeterPoints {
        meters {
          mechanism
        }
      }
    }
  }
}

```

#### Variables
```json
{
    "accountNumber":"A-AAAAAAAA"
}
```

#### Example Response
```json
{
    "data": {
        "account": {
            "smets2Interest": "INTERESTED",
            "smets2RefusalReason": null,
            "properties": [
                {
                    "id": "9999999",
                    "isSmets2InstallationAllowed": false,
                    "electricityMeterPoints": [
                        {
                            "meters": [
                                {
                                    "meterType": "S2ADE"
                                }
                            ]
                        },
                        {
                            "meters": [
                                {
                                    "meterType": "S2ADE"
                                }
                            ]
                        }
                    ],
                    "gasMeterPoints": [
                        {
                            "meters": [
                                {
                                    "mechanism": "S2"
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
Fetch eligibility for smart meter installation.

### getAccount (again)
#### Query
```gql
query getAccount($accountNumber: String!, $propertiesActiveFrom: DateTime) {
  account(accountNumber: $accountNumber) {
    cotReadingWindowDays
    properties(activeFrom: $propertiesActiveFrom) {
      id
      address
      occupancyPeriods {
        effectiveTo
      }
      electricityMeterPoints {
        id
        meters(includeInactive: false) {
          id
          serialNumber
          requiresCotFinalReading
        }
        enrolment {
          status
        }
        status
      }
      gasMeterPoints {
        id
        meters {
          id
          serialNumber
          requiresCotFinalReading
        }
        enrolment {
          status
        }
        status
      }
    }
  }
}

```

#### Variables
```json
{
    "accountNumber":"A-AAAAAAAA",
    "propertiesActiveFrom":"2024-08-12T23:00:00.000Z"
}
```

#### Example Response
```json
{
    "data": {
        "account": {
            "cotReadingWindowDays": 2,
            "properties": [
                {
                    "id": "9999999",
                    "address": "AAAAAAAAA, AAAAAAAAAAAA, AAAAAAAAAAAAAAAAA, AAAAAAAAAAAAAA, AAAA AAA",
                    "occupancyPeriods": [
                        {
                            "effectiveTo": null
                        }
                    ],
                    "electricityMeterPoints": [
                        {
                            "id": "9999999",
                            "meters": [
                                {
                                    "id": "9999999",
                                    "serialNumber": "99A9999999",
                                    "requiresCotFinalReading": false
                                }
                            ],
                            "enrolment": null,
                            "status": "ON_SUPPLY"
                        },
                        {
                            "id": "9999999",
                            "meters": [
                                {
                                    "id": "9999999",
                                    "serialNumber": "99A9999999",
                                    "requiresCotFinalReading": false
                                }
                            ],
                            "enrolment": null,
                            "status": "ON_SUPPLY"
                        }
                    ],
                    "gasMeterPoints": [
                        {
                            "id": "9999999",
                            "meters": [
                                {
                                    "id": "9999999",
                                    "serialNumber": "A9A99999999999",
                                    "requiresCotFinalReading": false
                                }
                            ],
                            "enrolment": null,
                            "status": "ON_SUPPLY"
                        }
                    ]
                }
            ]
        }
    }
}
```
Yet another fetch of account details with slightly different attributes including address.

### paymentSchedules
#### Query
```gql
query paymentSchedules($accountNumber: String!, $statuses: [DirectDebitInstructionStatus]) {
  account(accountNumber: $accountNumber) {
    directDebitInstructions(first: 5, statuses: $statuses) {
      pageInfo {
        startCursor
      }
    }
    paymentSchedules(first: 10, canCreatePayment: true) {
      edges {
        node {
          supplementaryLedger {
            ledgerType
          }
          id
          paymentAmount
          paymentDay
          validTo
          validFrom
          isVariablePaymentAmount
          reason
          paymentAdequacyAdjustment
          paymentAdequacyAdjustmentExpiryDate
          totalDebtAmount
          paymentFrequency
          paymentFrequencyMultiplier
        }
      }
    }
  }
}
```

#### Variables
```json
{
    "accountNumber":"A-AAAAAAAA",
    "statuses":["ACTIVE","PROVISIONAL"]}
```

#### Example Response
```json
{
    "data": {
        "account": {
            "directDebitInstructions": {
                "pageInfo": {
                    "startCursor": "YXJyYXljb25uZWN0aW9uOjA="
                }
            },
            "paymentSchedules": {
                "edges": [
                    {
                        "node": {
                            "supplementaryLedger": null,
                            "id": "99999999",
                            "paymentAmount": 10221,
                            "paymentDay": 28,
                            "validTo": null,
                            "validFrom": "9999-99-99",
                            "isVariablePaymentAmount": false,
                            "reason": "GENERAL_ACCOUNT_PAYMENT",
                            "paymentAdequacyAdjustment": 0,
                            "paymentAdequacyAdjustmentExpiryDate": null,
                            "totalDebtAmount": 0,
                            "paymentFrequency": "Monthly",
                            "paymentFrequencyMultiplier": 1
                        }
                    },
                    {
                        "node": {
                            "supplementaryLedger": {
                                "ledgerType": "GOODS"
                            },
                            "id": "99999999",
                            "paymentAmount": 0,
                            "paymentDay": 1,
                            "validTo": null,
                            "validFrom": "9999-99-99",
                            "isVariablePaymentAmount": true,
                            "reason": "GENERAL_ACCOUNT_PAYMENT",
                            "paymentAdequacyAdjustment": 0,
                            "paymentAdequacyAdjustmentExpiryDate": null,
                            "totalDebtAmount": 0,
                            "paymentFrequency": "Monthly",
                            "paymentFrequencyMultiplier": 1
                        }
                    },
                    {
                        "node": {
                            "supplementaryLedger": {
                                "ledgerType": "ELECTRIC_JUICE"
                            },
                            "id": "99999999",
                            "paymentAmount": 0,
                            "paymentDay": 1,
                            "validTo": null,
                            "validFrom": "9999-99-99",
                            "isVariablePaymentAmount": true,
                            "reason": "GENERAL_ACCOUNT_PAYMENT",
                            "paymentAdequacyAdjustment": 0,
                            "paymentAdequacyAdjustmentExpiryDate": null,
                            "totalDebtAmount": 0,
                            "paymentFrequency": "Monthly",
                            "paymentFrequencyMultiplier": 1
                        }
                    }
                ]
            }
        }
    }
}
```
Fetches the schedule on which direct debits will be made. I think the first one (without a `supplementaryLedger`) is the regular monthly charge for gas and electricity, the second (`GOODS`) one relates to supply of physical goods, possibly related to a heat pump install, and the `ELECTRIC_JUICE` one relates to Electroverse car charging.

### getLedgers
#### Query
```gql
query getLedgers($accountNumber: String!) {
  account(accountNumber: $accountNumber) {
    ledgers {
      ledgerType
      balance
    }
  }
}
```

#### Variables
```json
{
    "accountNumber":"A-AAAAAAAA"
}
```

#### Example Response
```json
{
    "data": {
        "account": {
            "ledgers": [
                {
                    "ledgerType": "MAIN",
                    "balance": 44333
                },
                {
                    "ledgerType": "GOODS",
                    "balance": -288550
                },
                {
                    "ledgerType": "ELECTRIC_JUICE",
                    "balance": 0
                }
            ]
        }
    }
}
```
Fetch the current balance on each individual ledger relating to the account. Again I believe that
`MAIN` is the regular monthly charge for gas and electricity, `GOODS` relates to supply of physical goods, possibly related to a heat pump install, and `ELECTRIC_JUICE` relates to Electroverse car charging.

I have  heat pump install pending and the negative GOODS charge is not reflected in my current account balance so something elsewhere must indicate that this payment is not yet due.

[Prototype Queries >](prototypeQueries.md)