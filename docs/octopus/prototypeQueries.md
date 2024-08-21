[< Web UI Queries](webUiQueries.md)
# Prototype Queries
This section describes some prototype queries which might make a good starting point for a GraphQL application. The [Web UI Queries](webUiQueries.md) page includes some instructions about how to try these for yourself.



### Initial Authentication with Email and Password
#### Query
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

#### Variables
```json
{
  	"input": {
      "email": "name@domain.com",
      "password": "secret"
    }
}
```

#### Example Response
```json
{
  "data": {
    "obtainKrakenToken": {
      "refreshToken": "qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwer",
      "refreshExpiresIn": 1724844646,
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
This should **only be used with an interactive application where you will prompt for the password from a user, DO NOT store the password anywhere and DO NOT try this query in the playground**.

Once this request is made you should delete (i.e. write zeros over the stored value byte by byte) the stored password. It is possible for data stored in memory to remain accessible even after a process has terminated in some situations.

Once you have authenticated it is possible to retrieve the current APIKey via the `view` query, which could then be stored if desired.

The `token` value needs to be provided as the value of the `Authorization` header for all further requests.

### Initial Authentication with APIKey
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

### Invalidate All Current Sessions
#### Query
```gql
mutation forceReauthentication($forceReAuth: ForceReauthenticationInput!){
  forceReauthentication(input: $forceReAuth) {
    tokensInvalidated
    effectiveAt
  }
}
```

#### Variables
```json
{
  	"forceReAuth": {
      "includeThirdParties": true
    }
}
```

In the event that the APIKey is compromised you can invalidate the existing key and generate a new one in the Account Dashboard (https://octopus.energy/dashboard/new/accounts/personal-details/api-access). Unfortunately this does not invalidate any current refresh tokens so you should also call this mutation which will invalidate all current sessions and their refresh tokens.

### Refresh Authentication
#### Query
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

#### Variables
```json
{
    "refresh": {
      "refreshToken": "qwertyuiop1234567890qwertyuiop1234567890qwertyuiop1234567890qwer"
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

### Fetch User's Current APIKey
#### Query
```gql
query getApiKey {
    viewer {
        liveSecretKey
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
    "viewer": {
      "liveSecretKey": "sk_XXXX_XXXXXXXXXXXXXXXXXXXXXXXX"
    }
  }
}
```
This query can be used to obtain the current APIKey, which may be useful after a password based authentication.

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description

### queryName
#### Query
```gql
query
```

#### Variables
```json
{}
```

#### Example Response
```json
{}
```
Description