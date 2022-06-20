  _   _   _   _   _  
 / \ / \ / \ / \ / \ 
( c | v | c | a | p )
 \_/ \_/ \_/ \_/ \_/ 

# Aims
* Simple cli for Checkvist, with main focus on quick capture (eg of clipboard) to a (preconfigured) list
* ?A service/daemon to run locally, accessed via OS ports, or Web sockets, or HTTP 
* Trello-ish interfaces
  + egui (quick / small)
  + React / Chakra 

# Tasks
* [X] ~~*remove command from `cvcap` - it's all 'add', so have a single command with arguments only*~~ [2022-06-21]
      ie. now just `cvcap 'text to add' [--choose-list, --from-clipboard, etc]`
      Needs a bit of clap research
* [ ] add argument to choose capture list (rather than use default)
      `cvcap add --choose-list` [ or -l]
      This offers the same list as during initial setup
      And then asks whether this should be saved as the new default
* [ ] --verbose turns on logging (ie. gets env_logger to log, regardless of env vars)
* [ ] capture from clipboard (can this be made all-platform?) `cvcap add --from-clipboard` [or -c]
* [ ] review for proper use of signals and stdin/stderr (see cmdline gitbook)
  * [ ] capture from stdin (eg `cat file | cvcap add`)
  * [ ] send errors to stderr
* [ ] before deployment stuff, consider how to split API crate and bin (we'll need the crate for the Trelloish UI), but without putting on crates.io. Can cargo.toml deps be added from github? Or local relative paths?
    lib - checkvist-api
    bin - cvcap (and later perhaps cvconv and chrello)
* [ ] install / deploy
      * just cargo install, or anything else?
  * linux
    * I reckon just a build of the binary is fine for linux users
    * [ ] windows installer
      * [ ] set up file sharing with the quickemu VM for testing
      * [ ] create with wixtools
* [ ] when saving a new config file, offer to show in file manager (or terminal)?
* [ ] possibly add a debug mode with error capture 
* [ ] set up CI (github actions will be fine)
* [ ] man pages

* make cli OS-friendly
  + https://rust-cli.github.io/book/index.html
      * [ ] replace "text" with stdin
* how to deal with auth on first run?
  - where to put the api token when retrieved?
    - possible crates
        - https://crates.io/crates/keyring     
          This looks good - widely downloaded, and surely the best place?
        - I might somehow add 1password for my own use (and as an alternative for other users)
          There are some wrappers: https://crates.io/search?q=1password
  + handle secrets https://crates.io/crates/secrecy for the token (wrapper type to avoid exposing during logging etc))

# Research Required
* how to create a daemon process similar to docker cli's, that doesn't need a service installation?

# Resources
* [Checkvist API](https://checkvist.com/auth/api)
* https://serde.rs/
* https://www.lpalmieri.com/posts/how-to-write-a-rest-client-in-rust-with-reqwest-and-wiremock/
  Looks like a good ref for making the client roughly prod-ready
* https://rust-cli.github.io/book/index.html
* egui backend? https://github.com/emilk/egui
* config file library https://github.com/rust-cli/confy
* config files without a library https://github.com/rust-adventure/lets-code-cli-config

## commandline UI
* https://crates.io/crates/tui
* https://crates.io/crates/cursive
  (more declarative alternative to tui)
* https://lunatic.solutions/blog/lunatic-chat/
* https://github.com/lunatic-solutions/chat implementation, which apparently has changed a lot since the article (because of underlying lunatic changes)
  In additiion to the wasm stuff, he uses TUI which might be usefully instructive. Looks difficult though.

## Release / packaging
* https://wixtoolset.org/

## Windows VM
* https://docs.fedoraproject.org/en-US/quick-docs/getting-started-with-virtualization/ 
* https://ask.fedoraproject.org/t/which-vm-for-a-windows-guest-in-2022/23242/2
