[< Documentation Home](index.md)

# Profiles

Marco Sparko will create a file in your home directory called ```.marco-sparko``` which contains the stored credentials it needs to access the various data providers.

When any module is initialised, if there is a current profile then the configuration will be added to that profile, otherwise a new profile called ```default``` will be created. The ```.marco-sparko``` file is a JSON document containing an array of profiles similar to this:

```
[
  {
    "name": "default",
    "modules": {
      "octopus": {
        "apiKey": "sk_live_XXXXXXXXXXXXXXXXXXXXXX",
        "billingTimezone": "Europe/London"
      }
    }
  },
  {
    "name": "test_profile",
    "modules": {
      "octopus": {
        "apiKey": "sk_live_XXXXXXXXXXXXXXXXXXXXXX",
        "billingTimezone": "Europe/London"
      }
    }
  }
]
```

The first profile listed will be used unless another is specified by passing the commandline parameter ```--profile=test_profile``` where test_profile is the name of the profile to be used. Profiles are entirely independent of each other, they have separate credentials, may have different combinations of modules enabled, and have separate data caches.

[Cached Data >](cachedData.md)