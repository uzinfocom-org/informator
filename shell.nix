flake:
{ pkgs, ... }:
let
  # Hostplatform system
  system = pkgs.hostPlatform.system;

  # Production package
  base = flake.packages.${system}.default;

  # Manifest via Cargo.toml
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
pkgs.mkShell {
  inputsFrom = [ base ];

  packages = with pkgs; [
    # Nix typeshi
    nixd
    statix
    deadnix
    nixfmt

    # Rust extras
    rustfmt
    clippy
    rust-analyzer
    cargo-watch

    # Database migration
    diesel-cli
    diesel-cli-ext
  ];

  # Touch only if you know what you're doing
  # (bash skill issues are not welcomed here)
  shellHook = ''
    # Check for .env file
    if [ ! -f .env ]; then
    read -r -p "Please enter your telegram bot token: " TELOXIDE_TOKEN;
      echo "export TELOXIDE_TOKEN=$TELOXIDE_TOKEN" > .env;
      echo "export WATCH_MODE=false" >> .env;
    fi

    # Load everything from .env
    source .env;

    # Lesss goooo...
    if [ $WATCH_MODE == true ]; then
      echo "Watchmode in background is enabled!"
      echo "Starting cargo watch session on backround..."
      echo

      start() {
        # Start watching for changes in the background
        cargo watch -x "run --bin ${manifest.name} -- env" &
        echo
        echo
        echo ===========================================================================
        echo "Press enter to continue on your terminal"
        echo "\> Don't close this terminal as bot is running on watch mode and see log"
        echo "\> Just open your editor from this terminal to make your editor read PATH"
        echo ===========================================================================

        # Store the PID of the background process to file & env
        CARGO_WATCH_PID=$!
        echo "$CARGO_WATCH_PID" > .daemon

        # Function to clean up the background process on exit
        cleanup() {
          kill $CARGO_WATCH_PID || true
          rm -rf ./.daemon
        }

        # Trap EXIT signal to run cleanup function
        trap cleanup EXIT INT TERM
      }

      # Check if there's already session running
      if [ -e ".daemon" ]; then
          echo "There's a session already running..."
          read -p "would you like to stop running instance? (y/n): " answer

          case "$answer" in
              [Yy]*)
                # Read old session file
                _pid=$(<.daemon)

                # Kill & remove old session
                kill $_pid || true

                # Cleanup daemon
                rm -rf ./.daemon

                # Cleanup the pid variable
                unset _pid

                # Start the damn bot
                start
                ;;
              [Nn]*)
                echo "Aight, no problem bro. I got you..."
                ;;
              *)
                echo "Invalid input, ignoring running instance and not running anything"
                ;;
          esac
      else
        # Start the damn bot
        start
      fi
    else
      echo "You disabled the watch mode on background."
      echo "Feel free to enable it back again at .env file!"
    fi
  '';
}
