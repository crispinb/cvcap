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
* [X] ~~*add argument to choose capture list (rather than use default)*~~ [2022-06-21]
      `cvcap add --choose-list` [ or -l]
      This offers the same list as during initial setup
      And then asks whether this should be saved as the new default

* AUTH
  * [x] get and store auth token on first run
    will need to ask user for email and Open API key (available from https://checkvist.com/auth/profile)
  * [X] ~~*retrieve auth token when available*~~ [2022-06-22]
  * [x] refresh auth token when refused
    - 401 error handler for all the checkvist methods to automatically retry token?
  * [X] ~~*re-get auth token when token refresh fails*~~ [2022-06-24]
      - quick and dirty: delete token from keyring, and ask user to run again to re-login
* [X] ~~*add main error handling / reporting*~~ [2022-06-24]
  - use fn main ONLY To call a command function, then report on its errors (perhaps also display help)
* [X] ~~*add Checkvist username to status*~~ [2022-06-26]
* [X] ~~*show status when invoked without args (logged in / default list)*~~ [2022-06-26]
* [X] ~~*save new token after refresh (currently main has no idea when the token is refreshed)*~~ [2022-06-29]
* [ ] review for proper use of signals and stdin/stderr (see cmdline gitbook)
  * [ ] send errors to stderr (I think they are already?)
* [ ] consolidate (or decide on) error! and/or .context
* [ ] before deployment stuff, consider how to split API crate and bin (we'll need the crate for the Trelloish UI), but without putting on crates.io. Can cargo.toml deps be added from github? Or local relative paths?
    lib - checkvist-api
    bin - cvcap (and later perhaps cvconv and chrello)
    NB - remember the 2 will need different deps
* [ ] test on win
* [ ] install / deploy
      * just cargo install, or anything else?
  * linux
    * I reckon just a build of the binary is fine for linux users
    * [ ] windows installer
      * [ ] set up file sharing with the quickemu VM for testing
      * [ ] create with wixtools
## post 1st release
* [ ] replace re-login approach (instead of asking user to run again, launch login immediately on user confirmation)
* [ ] capture from stdin (eg `cat file | cvcap add`)
* [ ] set up CI (github actions will be fine)
* [ ] capture from clipboard (can this be made all-platform?) `cvcap add --from-clipboard` [or -c]
  - how to turn off main 'task' content requirement (which would conflict with the clipboard content)?
* [ ] add user feedback during network calls (spinner? but wouldn't that need async? I could do it on a thread I guess)
* [X] ~~*stop env_logger logging errors when there's no RUST_LOG set (should be entirely opt-in)*~~ [2022-06-26]
* [ ] --verbose turns on logging (ie. gets env_logger to log, regardless of env vars)
* [ ] add support for 2fa key  when getting token from API? (https://checkvist.com/auth/api#task_data)
  * PENDING https://discuss.checkvist.com/t/2fa-in-auth-api/729/4
* [ ] maybe add a timestamp to to the token to reduce an unecessary network call when (nearly) expired.
* [ ] perhaps add a thread to check/refresh creds while user is interacting. We could keep the age of the
  thread, we know they only last 24 hours, so can automatically refresh in the background, probably before 
  the user's even started typin

# Possible further features
* man pages
* non-text types (from clipboard or a specified/piped file)
* specify new item's position in list
* when saving a new config file, offer to show in file manager (or terminal)?

# Resources
* [Checkvist API](https://checkvist.com/auth/api)
* https://serde.rs/
* https://www.lpalmieri.com/posts/how-to-write-a-rest-client-in-rust-with-reqwest-and-wiremock/
  Looks like a good ref for making the client roughly prod-ready
* https://rust-cli.github.io/book/index.html
* egui backend? https://github.com/emilk/egui
* config file library https://github.com/rust-cli/confy
* config files without a library https://github.com/rust-adventure/lets-code-cli-config

## secrets / security
* https://crates.io/crates/keyring     
* https://crates.io/crates/secrecy
* https://crates.io/search?q=1password



## commandline UI
+ https://rust-cli.github.io/book/index.html
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
