# Logging in to the Powerwall

The app logs in to the gateway as its customer, the same as the gateway's own web page on your home network:

- **Gateway Address**, its IP address on your home network. Leave it empty and the app searches the network for it. If the gateway's address changes later, the app finds it again.
- **Email** and **Password** of the customer login. If the password has never been changed, it is the last 5 characters of the password on the label inside the gateway's door.

The app saves the email and password in the profile, so that it can log in again when the gateway asks it to. They only go to the gateway.

Check the password before logging in again after a failure. The gateway locks out the customer login for a while after repeated failures, which stops other programs which use it too, such as Home Assistant. If the gateway rejects the saved password, the app removes it, and shows this page.
