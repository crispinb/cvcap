  ____ _               _           _     _             _ _ 
 / ___| |__   ___  ___| | ____   _(_)___| |_       ___| (_)
| |   | '_ \ / _ \/ __| |/ /\ \ / / / __| __|____ / __| | |
| |___| | | |  __/ (__|   <  \ V /| \__ \ ||_____| (__| | |
 \____|_| |_|\___|\___|_|\_\  \_/ |_|___/\__|     \___|_|_|
                                                           
In scratch because I'm not sure if this will become a real project.
If it does, move to crispinb


# Aims
- A client library to access the main CRUD operations of the Checkvist API
- Simple cli, with main focus on quick capture (eg of clipboard) to a (preconfigured) list
- A service/daemon to run locally, accessed via OS ports, or Web sockets, or HTTP, from a locally-run Electron client that provides
  a Trelloish interface
  

# Tasks
* ultra simple (POC) capture-to-task
  * spec
    * [ ] API client: method & test to push text to a new task in a list
    * commandline
      * [ ] call API client with text with approx this UI: 
            `checkvistcli new-task "text"`, with result:
            `task "text" successfully added to list [preconfigured list name]`
* make cli usable
  * https://rust-cli.github.io/book/index.html
      * [ ] replace "text" with stdin
* handle secrets https://crates.io/crates/secrecy for the token (wrapper type to avoid exposing during logging etc))


# Research Required
- how to create a daemon process similar to docker cli's, that doesn't need a service installation?


# Resources
* [Checkvist API](https://checkvist.com/auth/api)
* https://www.lpalmieri.com/posts/how-to-write-a-rest-client-in-rust-with-reqwest-and-wiremock/
  Looks like a good ref for making the client roughly prod-ready
* https://rust-cli.github.io/book/index.html