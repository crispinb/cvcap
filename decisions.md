## How to do cli integration tests
* how to manage platform state (config file, keyring)
  
    * mocks

        not keen. Complex and too much risk of the tests themselves becoming a source of error. Also there's nothing like interacting with a real enviroment to expose an app's limitations
    * configure the run environment for the tests eg via containers
      
      Cross might be the simplest option here as I'm using it anyway.
      Postpone 'till CI setup
    * configure the app for the tests eg via env vars

       prefer as the most flexible. Can run on dev machine, or on containers when set up
  

* how to run tests

    ie. as they'll be slow, I want them to run independently from other tests

## How to check for piped input?
We need to do this, because reading from stdin blocks, so we can't just check whether or not there's content.

Some apps use '-', which I find to be ugly UX.

Some read from stdin pipe if there are no args, which doesn't work here as we want `-l` and `-v` available with piped content.

So we need to find out whether or not stdin *is a tty*. If it isn't, then we presume piped content.

This is platform-dependent and fairly complex. See https://github.com/softprops/atty/blob/master/src/lib.rs
I thought about just copying relevant functions from atty, but it's tiny so will just use as-is.