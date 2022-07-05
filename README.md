# cvcap 
> A minimal commandline capture tool for [Checkvist](https://checkvist.com)

## Usage
* `cvcap "a task"` 

  adds a task to the top of a default list (chosen on first run)
* `cvcap "a task" -l` 

  adds a task to a selected list (optionally saved as a new default)


## Installation
* download a binary from [ Releases ](https://github.com/crispinb/cvcap/releases)
    
    Currently available for Linux, Windows
* For Rust users: 

  `cargo install --git  git@github.com:crispinb/cvcap.git`

## Future plans
This is an intentionally simple tool to quickly capture text tasks Checkvist, so I'll keep it simple, but some features I expect to add are:
* capture task text from clipboard or stdin
* a MacOS build
* a completely quiet option for scripting use

And more speculative possibilities:
* adding non-text MIME types from files or clipboard
* richer Checkvist content (notes, attachments, due dates, priorities) 
* add task to somewhere other than the top of the list