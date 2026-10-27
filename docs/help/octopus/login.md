# Logging in to Octopus

The app needs to log in to your Octopus account to fetch your data. There are two ways:

- **Login with Email and Password**, the same as for the Octopus web site and app.
- **Use API Key**, a key Octopus gives you for programs like this one. Find it in your Octopus account online under *Personal details*, *Developer settings*, see [Getting started](../../gettingStarted/index.md).

With email and password, the app saves the login token Octopus gives it, not your password. With an API key, it saves the key in the profile. Either way it stays logged in, and if the token expires or the key is changed, this page is shown again.

Your login only goes to Octopus, the app has no server of its own.
