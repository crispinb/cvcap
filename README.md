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
* ultra simple (POC) capture-to-task
  + spec
    - [x] API client: method & test to push text to a new task in a list
    - commandline
      * [x] call API client with text with approx this UI: 
          `checkvistcli list 
          `checkvistcli add "text"` , with result:
          `task "text" successfully added to list [preconfigured list name]`
      * [ ] first run (no config file).
          - (just assuming api token for now)
          - get list of lists (need new api method)
          - present and offer which list
          - present to user and ask for default list
          - offer to save as default
            - store these in toml file if yes
            - tell the user where to find the file
          - add task

## platform issues

  + [ ] get Windows 10 running in a VM

## First run

* [ ] ask for name of list to capture to, & persist in a config file 
* [ ] add command to change default capture list

## Misc features 

* [ ] capture from stdin
* [ ] capture from clipboard (can this be made all-platform?)
* [ ] when saving a new config file, offer to show in file manager (or terminal)?

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
* https://lunatic.solutions/blog/lunatic-chat/
  https://github.com/lunatic-solutions/chat implementation, which apparently has changed a lot since the article (because of underlying lunatic changes)
  In additiion to the wasm stuff, he uses TUI which might be usefully instructive. Looks difficult though.

## Windows VM
* https://docs.fedoraproject.org/en-US/quick-docs/getting-started-with-virtualization/ 
* https://ask.fedoraproject.org/t/which-vm-for-a-windows-guest-in-2022/23242/2
