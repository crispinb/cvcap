# cvcap 
> A minimal commandline capture tool for [Checkvist](https://checkvist.com)

## Usage
This is a summary. `cvcap help` shows all options
### Add a task from the commandline to the default list
  This form requires no subcommand, and takes no options other than `-v` (for verbose output)

* `cvcap "a task"` 

  adds a task to the top of a default list (chosen on first run)


### Add a task from a choice of sources to any list
This requires use of the `add` subcommand
* `cvcap add "a task" -l` 

  adds a task to a selected list (optionally saved as a new default)
* `cvcap add -c`

  adds a task from the clipboard
* `cat [file name] | cvcap add -s`
  
   Adds a task from stdin

* `cvcap add -cl` &nbsp;&nbsp;  - or - &nbsp;&nbsp;   `echo "task"  | cvcap add -sl`

   options can be combined

#### Bookmarks
* `cvcap add-bookmark [bookmark name]`

   Adds a bookmark to enable adding tasks to Checkvist lists other than the default list.
   A bookmark must already exist on the clipboard. This can be either the URL of a list 
   (ie. copied from the browser url bar) or the URL of an item (the Checkvist `copy permalink` command
    or `lc` shortcut). Adding a task with an item shortcut makes a child task.

* `cvcap add [task] -b [bookmark name]`
  
  Adds the task to the location pointed to by the bookmark

## Installation
Those familiar with the Rust tookchain will find it straightforward to install from git with `cargo install --git https://github.com/crispinb/cvcap/releases/tag/v0.1.5-alpha` or `cargo install --git https://github.com/crispinb/cvcap ` for the edgier version.

Earlier versions provided binaries which I've stopped updating for now. Feel free to submit an issue if you'd like binaries available.

## Configuration and Environment

The default configuration file is named `cvcap.toml`. By default it's in the standard config location for each platform.
This can be altered by setting the env var CVCAP_CONFIG_FILE_PATH to the full desired path to the file.

The Checkvist API login is stored in the logged-in user's system keyring for the platform (see https://github.com/hwchen/keyring-rs for platform details).
The credential ID (however that is defined on the platform' is 'cvcap-api-token' by default. This can be changed by setting the env var CVCAP_CREDENTIAL_ID


## Future plans
This is an intentionally simple tool to quickly capture text tasks to Checkvist. It will likely remain unpolished but serviceable. A few additional features I may add are:
* File attachments
* notes

I'm most interested though in using `cvcap` as inspiration for a more general-purpose capture tool, ie. a rules-driven interface to flexibly capture to a variety of sinks: checkvist, logseq, notion, etc.


## Note on cvcap / Checkvist (non-)relationship
This is a third party app using Checkvist's public API. This repo has no affiliation with Checkvist (apart from recommending it heartily)
