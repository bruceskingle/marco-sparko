# marco-sparko
### The Energy Explorer

Marco Sparko is an application and a set of libraries to access the APIs of various energy service providers, written in the Rust programming language. As we install more and more smart energy devices in our homes it becomes more and more difficult to get an overall view of usage and to get the most out o the resources at our disposal.

This is a hobby project of mine, it is not supported, endorsed or approved by Octopus Energy, Tesla, Myenergi or anyone else.

The source code is all available on GitHub and available under the MIT license.

The documentation has been written primarily for my benefit, but I have added some extra instructions which I hope would enable someone unfamiliar with the various APIs or GraphQL to be able to get going, and I hope it may be helpful to someone.

I will endeavor to respond to any comments or questions (email: github at skingle.org), but please bear in mind that this is a hobby project.

# Where It Started
I am a customer of Octopus Energy and I have solar panels, a Tesla PowerWall battery and a Myenergi Zappi smart EV charger. Occasionally Octopus offer `Power Ups` where electricity is free for a few hours on an ad hoc basis. In order to make the maximum use of these (and to help consume the surplus electricity generation which is the reason for Power Ups in the first place), it's necessary to reconfigure the PowerWall before the Power Up starts, and because the PowerWall app does not provide a mechanism to add one off periods to the utility rate plan, to remember to replace the normal configuration after the Power Up has ended.

If I forget to put undo the change after a Power Up then the next day the PowerWall will charge up from the grid on what it thinks is free power, at the peak rate :o(

So I thought it would be cool to have a program which could manage that for me.

Now I have moved onto the Intelligent Go tariff and Octopus controls when the Zappi actually charges, even if it charges during the day, they only charge for the power used at the overnight rate. So now, in order to get an idea of how much I'm spending, I need to get data from the Zappi as well as overall consumption data from Octopus.

# The Final Vision
I intend Marco Sparko to become a platform supporting plugins to access various APIs including:

* Octopus Energy
* Myenergi
* Tesla

To provide an integrated view of energy usage across all of those platforms. I envisage:

* A command line utility to allow me to pull down usage data from all services in a form which can be loaded into a spreadsheet for analysis.
* A GUI which can allow me to explore data across all platforms
* An embedded system running on a Raspberry Pi or similar small computer which can act as a zwave controller, as well as being able to access these APIs.

This would enable me to switch on discretionary loads (such as an electric immersion heater for hot water) when power is cheap or free, or when there is surplus solar generation.

I would like to make the plugins entirely independent of the core code base so that plugins could be developed independently of access to the core source code (although it's available as Open Source anyway)

# The Current Reality
The basic structure of the platform is in place and I have the bare bones of a plugin for the Octopus API. All it can do at the moment is print a summary of the Octopus account.

The Octopus plugin is actually tightly bound into the core code base, and there are still a few circular dependencies between the two parts of the code base, but I have a plan for how I will address this.

# Documentation
The [Documentation Home](docs/index.md) is the starting point for access to all of the documentation.

Plugins provide access to various utilities and services, the following sections provide more detailed information about them:


[Octopus Energy](https://github.com/bruceskingle/marco-sparko/blob/main/docs/octopus/index.md)
