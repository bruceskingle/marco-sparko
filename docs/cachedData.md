[< Documentation Home](index.md)

# Cached Data
Marco Sparko will create a file in your home directory called ```.marco-sparko-cache``` which contains local copies of data retrieved from the various online data sources for which it is configured. Maintaining a local copy of data which has previously been fetched from a server makes the application significantly faster and reduces the load on the cloud services it accesses. Most services will place limits on the number of requests which can be made in any period of time, and some operations (such as generating session authentication tokens) may be sufficiently expensive that the imposed limit is quite small.

If every time the program runs it was to reauthenticate with the server, then you would quickly find the application failing with a "Too Many Requests" error or similar.

If you look in the ```.marco-sparko-cache``` directory you will find the files listed below. In each case the name ```profile``` refers to a profile defined in ```~/.marco-sparko``` and ```module``` refers to the name of a module such as (and for the time being only ever) ```octopus```

## profile-history.txt
This is a text file containing a list of commands previously typed. This is used to provide history completion on the Marco Sparko command line. If you press the ```Up Arrow``` key you will see previous commands which can be re-execured by pressing ```Return```. If you start to type a command which has been previously executed you may see a proposed completion in italics, if you press the ```Right Arrow``` key the command will be completed for you.

## profile-module.json
This is the credential cache for the given module in the given profile. It is important to reuse the credentials to prevent "Too Many Requests" errors from the server, but they are, of course, sensitive and you should not share them with anyone.

## profile-module
This is a directory (folder) containing files which contain various sets of data retrieved from the given module in the given profile.

It is envisaged that there will be an option to maintain this data in the cloud in future, and the data format is designed with the use of a NOSQL data store such as Amazon DynamoDb in mind. These data stores use a ```Hash Key``` and ```Sort Key``` structure. The name of each file in this directory is the ```Hash Key``` under which that data is stored. The contents of the files are lines which each represent one record, consisting of a ```Sort Key``` followed by a TAB character and then a JSON payload (without whitespace padding so there are no unescaped newlines).

The ```Hash Key``` is usually made up of multiple parts concatenated together and separated by # (hash) characters. For technical reasons, (to avoid hot-spots in the data store) where a constant string is used as part of the overall key, it is placed at the end so we have ```A-B1C2B345#104910337#StatementTransactions``` rather than ```StatementTransactions#A-B1C2B345#104910337```

In most cases the ```Sort Key``` is the cursor value used by the server for paginated queries, which facilitates the fetching of updates to time series data. The JSON payload is the response as received from the server.

There are three styles of file:

### Single Record
In this case the data set contains only a single record. An example of this is the ```#Viewer``` query for the Octopus module, which contains information about the user account whose authentication credential is used to access the service. Note that the single record may contain one or more arrays of data, for example the list of utility meters in an account. While this may change over time, it is not expected to do so in the same what that meter readings, for example, naturally grow on a regular basis.

### Multi Record
In this case the data set is expected to be reasonably small but may grow over time. The data set is stored in a single file with records stored in ascending date order. As new records are detected they will be appended to the file. The system will usually fetch all records from the start of the account as soon as any record is needed to facilitate correct maintenance of the data set. An example of data of this form is the list of Bills in an Octopus account.

### Time Series
In this case the dataset is expected to grow regularly. This data is chunked into time buckets, currently 1 calendar month. Within the data folder you will see folders with the name of recent (or the current) years. Within that folder you will find files containing data for a single month. When data is required for such a data set, the application will fetch all data from the start of the bucket (month) in which the required data sits. This means that for historic data each file usually contains a whole buckets worth of data, the file for the current period will, of course, be incomplete until the end of the period.

As with the Multi Record format, the data is stored in ascending date order, so as new data is retrieved it can be appended to the existing file.

[< Profiles](profiles.md)