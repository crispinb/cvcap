# cvcap 
> A minimal commandline capture tool for [Checkvist](https://checkvist.com)

## Usage
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

## Installation
* download a binary from [ Releases ](https://github.com/crispinb/cvcap/releases)
    
    Currently available for Linux, Windows, MacOS (Intel only)

* those familiar with the Rust tookchain will find it straightforward to install from git with `cargo install`. This is currently the best way to install on an ARM Mac, though the MacOS binary should run in a Rosetta terminal)

## Configuration and Environment

The default configuration file is named `cvcap.toml`. By default it's in the standard config location for each platform.
This can be altered by setting the env var CVCAP_CONFIG_FILE_PATH to the full desired path to the file.

The Checkvist API login is stored in the logged-in user's system keyring for the platform (see https://github.com/hwchen/keyring-rs for platform details).
The credential ID (however that is defined on the platform' is 'cvcap-api-token' by default. This can be changed by setting the env var CVCAP_CREDENTIAL_ID


## Future plans
This is an intentionally simple tool to quickly capture text tasks to Checkvist. It will likely remain unpolished but serviceable. A few additional features I expect to add are:
* a completely quiet option for scripting use
* simplify the UI eg. remove the 'add' subcommand 

### More speculative possibilities:
* adding non-text MIME types from files or clipboard
* richer Checkvist content (notes, attachments, due dates, priorities) 
* add task to somewhere other than the top of the list

## Note on cvcap / Checkvist (non-)relationship
This is a third party app using Checkvist's public API. This repo has no affiliation with Checkvist (apart from recommending it heartily)
