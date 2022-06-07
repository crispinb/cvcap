  ____ _               _           _     _             _ _ 
 / ___| |__   ___  ___| | ____   _(_)___| |_       ___| (_)
| |   | '_ \ / _ \/ __| |/ /\ \ / / / __| __|____ / __| | |
| |___| | | |  __/ (__|   <  \ V /| \__ \ ||_____| (__| | |
 \____|_| |_|\___|\___|_|\_\  \_/ |_|___/\__|     \___|_|_|

                                                           

# Aims
* A client library to access the main CRUD operations of the Checkvist API
* Simple cli, with main focus on quick capture (eg of clipboard) to a (preconfigured) list
* A service/daemon to run locally, accessed via OS ports, or Web sockets, or HTTP 
* Trello-ish interfaces
  * egui (quick / small)
  * React / Chakra 

# Tasks
* ultra simple (POC) capture-to-task
  + spec
    - [x] API client: method & test to push text to a new task in a list
    - commandline
      * [x] call API client with text with approx this UI: 
          `checkvistcli list 
          `checkvistcli add "text"` , with result:
          `task "text" successfully added to list [preconfigured list name]`
      * [ ] first run, ask for name of list to capture to, & persist in a config file 
      * [ ] add command to change default capture list

* make cli OS-friendly
  + https://rust-cli.github.io/book/index.html
      * [ ] replace "text" with stdin
* how to deal with auth on first run?
  * handle secrets https://crates.io/crates/secrecy for the token (wrapper type to avoid exposing during logging etc))

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
