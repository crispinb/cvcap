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

## Future plans
This is an intentionally simple tool to quickly capture text tasks to Checkvist. It will likely remain unpolished but serviceable. A few additional features I expect to add are:
* capture task text from clipboard or stdin
* a completely quiet option for scripting use

### More speculative possibilities:
* adding non-text MIME types from files or clipboard
* richer Checkvist content (notes, attachments, due dates, priorities) 
* add task to somewhere other than the top of the list

### Unlikely, shelved, or abandoned
* a MacOS build
  
  Has proved too much of a headache, at least for now. I build for non-Linux platforms using [cross-rs](https://github.com/cross-rs/cross). Excellent though that is, the procedure Apple's developer-hostile hypercorporate legalism mandates to build for MacOS (https://github.com/cross-rs/cross-toolchains#apple-targets) will take more time than I opt to spend on it.
   
    Mac users familiar with the Rust toolchain will find it quite straightforward to use via `cargo install`

## Note on cvcap / Checkvist (non-)relationship
*This is a third party app using Checkvist's public API. This repo has no affiliation with Checkvist (apart from recommending it heartily)*
