#!/bin/bash
set -e

HOST_UNAME=$(uname)
TARGET_TRIPLE="$1"

if [[ -z "$TARGET_TRIPLE" ]]; then
  echo "No target specified. Defaulting to x86_64-unknown-linux-gnu"
  TARGET_TRIPLE="x86_64-unknown-linux-gnu"
fi
create_synology_package() {
  local package_name="bitcoin_de_trading_api"
  local version="1.0.0"  # You should extract this from your Cargo.toml
  local arch="$1"
  local spk_arch=""

  # Map Rust target triple to Synology architecture
  case "$arch" in
    "x86_64-unknown-linux-gnu")
      spk_arch="x86_64"
      ;;
    "aarch64-unknown-linux-gnu")
      spk_arch="aarch64"
      ;;
    *)
      echo "Error: Unsupported architecture for Synology package: $arch" >&2
      return 1
      ;;
  esac

  echo "Creating Synology package for architecture: $spk_arch"

  # Create package directory structure
  local package_dir="synology_package"
  rm -rf "$package_dir"
  mkdir -p "$package_dir/package"
  mkdir -p "$package_dir/package/bin"
  mkdir -p "$package_dir/package/ui"
  mkdir -p "$package_dir/package/conf"
  mkdir -p "$package_dir/package/scripts"
  mkdir -p "$package_dir/package/ui/config"

  # Copy binary
  cp "target/$arch/release/bitcoin_de_trading_api_client" "$package_dir/package/bin/"

  # Create INFO file
  cat > "$package_dir/INFO" <<EOF
package="$package_name"
version="$version"
os_min_ver="7.0-40000"
description="Bitcoin.de Trading API Client"
arch="$spk_arch"
maintainer="Your Name"
displayname="Bitcoin.de Trading API"
adminport="8080"
adminurl="/webman/3rdparty/$package_name/index.cgi"
dsmuidir="ui"
dsmappname="SYNO.SDS.$package_name"
support_conf_folder="yes"
startable="yes"
thirdparty="yes"
ctl_stop_timeout="60"
create_user="yes"
install_dep_services="apache-web"
EOF

  # Create package scripts
  cat > "$package_dir/package/scripts/start-stop-status" <<'EOF'
#!/bin/sh

PACKAGE="bitcoin_de_trading_api"
DNAME="Bitcoin.de Trading API"
INSTALL_DIR="/var/packages/$PACKAGE/target"
CONFIG_DIR="/var/packages/$PACKAGE/etc"
BINARY="$INSTALL_DIR/bin/bitcoin_de_trading_api_client"
PID_FILE="/var/packages/$PACKAGE/var/run/$PACKAGE.pid"
LOG_FILE="/var/packages/$PACKAGE/var/log/$PACKAGE.log"
PACKAGE_USER="${SYNOPKG_PKGNAME}"

start_daemon() {
    if [ -f "$PID_FILE" ] && kill -0 $(cat "$PID_FILE") > /dev/null 2>&1; then
        echo "$DNAME is already running"
        return 0
    fi

    echo "Starting $DNAME..."
    cd "$INSTALL_DIR"
    # Run explicitly as PACKAGE_USER
    sudo -u "$PACKAGE_USER" nohup "$BINARY" \
        --api-key "$(jq -r '.api_key' $CONFIG_DIR/config.json)" \
        --api-secret "$(jq -r '.api_secret' $CONFIG_DIR/config.json)" \
        > "$LOG_FILE" 2>&1 &
    echo $! > "$PID_FILE"
}

stop_daemon() {
    if [ ! -f "$PID_FILE" ] || ! kill -0 $(cat "$PID_FILE") > /dev/null 2>&1; then
        echo "$DNAME is not running"
        return 0
    fi

    echo "Stopping $DNAME..."
    kill $(cat "$PID_FILE")
    rm -f "$PID_FILE"
}

daemon_status() {
    if [ -f "$PID_FILE" ] && kill -0 $(cat "$PID_FILE") > /dev/null 2>&1; then
        echo "$DNAME is running"
        return 0
    else
        echo "$DNAME is not running"
        return 1
    fi
}

case $1 in
    start) start_daemon ;;
    stop) stop_daemon ;;
    status) daemon_status ;;
    restart) stop_daemon; start_daemon ;;
    *) echo "Usage: $0 {start|stop|status|restart}" ;;
esac

exit 0
EOF

  chmod +x "$package_dir/package/scripts/start-stop-status"

  # Create pre-install script
  cat > "$package_dir/package/scripts/preinst" <<'EOF'
#!/bin/sh
exit 0
EOF
  chmod +x "$package_dir/package/scripts/preinst"

  # Create post-install script with proper user detection
  cat > "$package_dir/package/scripts/postinst" <<'EOF'
#!/bin/sh

PACKAGE="connectify_rs"
CONFIG_DIR="/var/packages/$PACKAGE/etc"
VAR_DIR="/var/packages/$PACKAGE/var"
INSTALL_DIR="/var/packages/$PACKAGE/target"
PACKAGE_USER="${SYNOPKG_PKGNAME}"
PACKAGE_GROUP="${SYNOPKG_PKGNAME}"

# Ensure directories exist
mkdir -p "$CONFIG_DIR" "$VAR_DIR/log" "$VAR_DIR/run"
chmod 755 "$INSTALL_DIR" "$INSTALL_DIR/bin"
chmod 750 "$CONFIG_DIR" "$VAR_DIR" "$VAR_DIR/log" "$VAR_DIR/run"

# Set ownership explicitly
chown -R "$PACKAGE_USER:$PACKAGE_GROUP" "$CONFIG_DIR" "$VAR_DIR"
chown "$PACKAGE_USER:$PACKAGE_GROUP" "$INSTALL_DIR/bin/bitcoin_de_trading_api_client"
chmod 755 "$INSTALL_DIR/bin/bitcoin_de_trading_api_client"

# Create default configuration file if missing
if [ ! -f "$CONFIG_DIR/config.json" ]; then
cat > "$CONFIG_DIR/config.json" <<EOT
{
  "api_key": "",
  "api_secret": "",
  "log_level": "info"
}
EOT
chmod 640 "$CONFIG_DIR/config.json"
chown "$PACKAGE_USER:$PACKAGE_GROUP" "$CONFIG_DIR/config.json"
fi

# Ensure web UI (http) can read/write config if needed:
if id http >/dev/null 2>&1; then
    chown "$PACKAGE_USER:http" "$CONFIG_DIR/config.json"
    chmod 660 "$CONFIG_DIR/config.json"
fi

exit 0
EOF

  chmod +x "$package_dir/package/scripts/postinst"
  # Create pre-uninstall script
  cat > "$package_dir/package/scripts/preuninst" <<'EOF'
#!/bin/sh

# Package
PACKAGE="bitcoin_de_trading_api"

# Stop the service if running
/var/packages/$PACKAGE/scripts/start-stop-status stop

exit 0
EOF
    # Create post-uninstall script
    cat > "$package_dir/package/scripts/postuninst" <<'EOF'
#!/bin/sh
exit 0
EOF


  chmod +x "$package_dir/package/scripts/preuninst"
  chmod +x "$package_dir/package/scripts/postuninst"
  # Create web UI for configuration
  mkdir -p "$package_dir/package/ui/config"

  # Create index.cgi for web UI
# Complete the index.cgi file
cat > "$package_dir/package/ui/index.cgi" <<'EOF'
#!/bin/sh

# Load DSM UI library
. /usr/syno/synoman/webman/modules/authenticate.cgi

# Package name
PACKAGE="bitcoin_de_trading_api"
CONFIG_FILE="/var/packages/$PACKAGE/etc/config.json"
LOG_FILE="/var/packages/$PACKAGE/var/log/$PACKAGE.log"

# Function to read config
read_config() {
    if [ -f "$CONFIG_FILE" ]; then
        API_KEY=$(grep -o '"api_key"[^,}]*' "$CONFIG_FILE" | cut -d'"' -f4)
        API_SECRET=$(grep -o '"api_secret"[^,}]*' "$CONFIG_FILE" | cut -d'"' -f4)
        LOG_LEVEL=$(grep -o '"log_level"[^,}]*' "$CONFIG_FILE" | cut -d'"' -f4)
    else
        API_KEY=""
        API_SECRET=""
        LOG_LEVEL="info"
    fi
}

# Function to save config
save_config() {
    cat > "$CONFIG_FILE.tmp" <<EOT
{
  "api_key": "$API_KEY",
  "api_secret": "$API_SECRET",
  "log_level": "$LOG_LEVEL"
}
EOT
    # Use mv to ensure atomic file update
    mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
    chmod 640 "$CONFIG_FILE"

    # Restart service to apply new config
    /var/packages/$PACKAGE/scripts/start-stop-status restart
}

# Process form submission
if [ "$REQUEST_METHOD" = "POST" ]; then
    # Read POST data
    read -r QUERY_STRING

    # Parse form data
    API_KEY=$(echo "$QUERY_STRING" | grep -o 'api_key=[^&]*' | cut -d'=' -f2)
    API_SECRET=$(echo "$QUERY_STRING" | grep -o 'api_secret=[^&]*' | cut -d'=' -f2)
    LOG_LEVEL=$(echo "$QUERY_STRING" | grep -o 'log_level=[^&]*' | cut -d'=' -f2)
    ACTION=$(echo "$QUERY_STRING" | grep -o 'action=[^&]*' | cut -d'=' -f2)

    # URL decode
    API_KEY=$(echo "$API_KEY" | sed 's/+/ /g;s/%\([0-9A-F][0-9A-F]\)/\\\\\\x\1/g' | xargs -0 echo -e)
    API_SECRET=$(echo "$API_SECRET" | sed 's/+/ /g;s/%\([0-9A-F][0-9A-F]\)/\\\\\\x\1/g' | xargs -0 echo -e)
    LOG_LEVEL=$(echo "$LOG_LEVEL" | sed 's/+/ /g;s/%\([0-9A-F][0-9A-F]\)/\\\\\\x\1/g' | xargs -0 echo -e)

    # Handle service control actions
    if [ "$ACTION" = "start" ]; then
        /var/packages/$PACKAGE/scripts/start-stop-status start
        SAVED=0
    elif [ "$ACTION" = "stop" ]; then
        /var/packages/$PACKAGE/scripts/start-stop-status stop
        SAVED=0
    elif [ "$ACTION" = "restart" ]; then
        /var/packages/$PACKAGE/scripts/start-stop-status restart
        SAVED=0
    else
        # Save config
        save_config
        SAVED=1
    fi
else
    # Read current config
    read_config
    SAVED=0
fi

# Get service status
if /var/packages/$PACKAGE/scripts/start-stop-status status > /dev/null 2>&1; then
    SERVICE_STATUS="Running"
else
    SERVICE_STATUS="Stopped"
fi

# Get last 20 lines of log
if [ -f "$LOG_FILE" ]; then
    LOG_CONTENT=$(tail -n 20 "$LOG_FILE" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g; s/'"'"'/\&#39;/g')
else
    LOG_CONTENT="Log file not found"
fi

# Output HTML
echo "Content-type: text/html"
echo ""
cat <<EOT
<!DOCTYPE html>
<html>
<head>
    <title>Bitcoin.de Trading API Configuration</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 800px; margin: 0 auto; }
        .form-group { margin-bottom: 15px; }
        label { display: block; margin-bottom: 5px; font-weight: bold; }
        input[type="text"], input[type="password"], select {
            width: 100%;
            padding: 8px;
            box-sizing: border-box;
            border: 1px solid #ddd;
            border-radius: 4px;
        }
        button {
            background-color: #0078d7;
            color: white;
            border: none;
            padding: 10px 15px;
            border-radius: 4px;
            cursor: pointer;
            margin-right: 5px;
        }
        .success {
            background-color: #dff0d8;
            color: #3c763d;
            padding: 15px;
            margin-bottom: 20px;
            border-radius: 4px;
        }
        .status {
            margin-bottom: 20px;
            padding: 10px;
            border-radius: 4px;
            background-color: #f5f5f5;
        }
        .status-running {
            color: #3c763d;
            font-weight: bold;
        }
        .status-stopped {
            color: #a94442;
            font-weight: bold;
        }
        .control-buttons {
            margin-bottom: 20px;
        }
        .tab {
            overflow: hidden;
            border: 1px solid #ccc;
            background-color: #f1f1f1;
            margin-bottom: 20px;
        }
        .tab button {
            background-color: inherit;
            float: left;
            border: none;
            outline: none;
            cursor: pointer;
            padding: 14px 16px;
            transition: 0.3s;
            color: black;
        }
        .tab button:hover {
            background-color: #ddd;
        }
        .tab button.active {
            background-color: #0078d7;
            color: white;
        }
        .tabcontent {
            display: none;
            padding: 20px;
            border: 1px solid #ccc;
            border-top: none;
        }
        .log-container {
            background-color: #f5f5f5;
            padding: 10px;
            border-radius: 4px;
            font-family: monospace;
            white-space: pre-wrap;
            max-height: 400px;
            overflow-y: auto;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Bitcoin.de Trading API</h1>

        $([ $SAVED -eq 1 ] && echo '<div class="success">Configuration saved successfully!</div>')

        <div class="status">
            Status: <span class="status-$(echo $SERVICE_STATUS | tr '[:upper:]' '[:lower:]')">$SERVICE_STATUS</span>
        </div>

        <div class="control-buttons">
            <form method="post" style="display: inline;">
                <input type="hidden" name="action" value="start">
                <button type="submit">Start Service</button>
            </form>
            <form method="post" style="display: inline;">
                <input type="hidden" name="action" value="stop">
                <button type="submit">Stop Service</button>
            </form>
            <form method="post" style="display: inline;">
                <input type="hidden" name="action" value="restart">
                <button type="submit">Restart Service</button>
            </form>
        </div>

        <div class="tab">
            <button class="tablinks active" onclick="openTab(event, 'Config')">Configuration</button>
            <button class="tablinks" onclick="openTab(event, 'Logs')">Logs</button>
        </div>

        <div id="Config" class="tabcontent" style="display: block;">
            <form method="post">
                <div class="form-group">
                    <label for="api_key">API Key:</label>
                    <input type="text" id="api_key" name="api_key" value="$API_KEY" required>
                </div>

                <div class="form-group">
                    <label for="api_secret">API Secret:</label>
                    <input type="password" id="api_secret" name="api_secret" value="$API_SECRET" required>
                </div>

                <div class="form-group">
                    <label for="log_level">Log Level:</label>
                    <select id="log_level" name="log_level">
                        <option value="debug" $([ "$LOG_LEVEL" = "debug" ] && echo 'selected')>Debug</option>
                        <option value="info" $([ "$LOG_LEVEL" = "info" ] && echo 'selected')>Info</option>
                        <option value="warn" $([ "$LOG_LEVEL" = "warn" ] && echo 'selected')>Warning</option>
                        <option value="error" $([ "$LOG_LEVEL" = "error" ] && echo 'selected')>Error</option>
                    </select>
                </div>

                <button type="submit">Save Configuration</button>
            </form>
        </div>

        <div id="Logs" class="tabcontent">
            <h3>Service Logs</h3>
            <div class="log-container">$LOG_CONTENT</div>
            <p><small>Showing last 20 log entries. Full logs available at: $LOG_FILE</small></p>
        </div>
    </div>

    <script>
    function openTab(evt, tabName) {
        var i, tabcontent, tablinks;
        tabcontent = document.getElementsByClassName("tabcontent");
        for (i = 0; i < tabcontent.length; i++) {
            tabcontent[i].style.display = "none";
        }
        tablinks = document.getElementsByClassName("tablinks");
        for (i = 0; i < tablinks.length; i++) {
            tablinks[i].className = tablinks[i].className.replace(" active", "");
        }
        document.getElementById(tabName).style.display = "block";
        evt.currentTarget.className += " active";
    }
    </script>
</body>
</html>
EOT
EOF

  chmod +x "$package_dir/package/ui/index.cgi"
  # Create package.tgz
  (cd "$package_dir/package" && tar -czf "../package.tgz" .)

  # Create the SPK file
  (cd "$package_dir" && tar -cf "../${package_name}_${version}_${spk_arch}.spk" INFO package.tgz)

  echo "Synology package created: ${package_name}_${version}_${spk_arch}.spk"
  return 0
}

## check_target
check_target() {
  local target="$1"
  if ! rustup target list | grep -q "^$target (installed)"; then
    echo "Target $target not installed. Installing..."
    rustup target add "$target"

    # Verify installation was successful
    if ! rustup target list | grep -q "^$target (installed)"; then
      echo "Error: Failed to install target $target" >&2
      return 1
    fi

    echo "Successfully installed target $target"
  else
    echo "Target $target is already installed."
  fi
  return 0
}

# Example usage:
# check_target "x86_64-unknown-linux-gnu"
# check_target "aarch64-pc-windows-msvc"

echo "Building for target: $TARGET_TRIPLE on host: $HOST_UNAME"

if [[ "$HOST_UNAME" == "Darwin" ]]; then
  case "$TARGET_TRIPLE" in
    x86_64-unknown-linux-gnu)
      if ! command -v x86_64-unknown-linux-gnu-gcc >/dev/null 2>&1; then
        echo "Error: x86_64-unknown-linux-gnu-gcc not found in PATH!" >&2
        echo "Please install with:" >&2
        echo "  brew tap messense/macos-cross-toolchains" >&2
        echo "  brew install x86_64-unknown-linux-gnu" >&2
        exit 1
      fi
      export CC="x86_64-unknown-linux-gnu-gcc"
      export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="x86_64-unknown-linux-gnu-gcc"
      ;;
    x86_64-unknown-freebsd)
      if ! command -v x86_64-unknown-freebsd13-clang >/dev/null 2>&1; then
        echo "Error: x86_64-unknown-freebsd13-clang not found in PATH!" >&2
        echo "Please install with:" >&2
        echo "  brew tap messense/macos-cross-toolchains" >&2
        echo "  brew install x86_64-unknown-freebsd13" >&2
        exit 1
      fi
      if ! command -v ld.lld >/dev/null 2>&1; then
        echo "Error: ld.lld not found! Required for FreeBSD cross-compilation." >&2
        echo "Please install with:" >&2
        echo "  brew install llvm lld" >&2
        exit 1
      fi
      SYSROOT_PATH="$HOME/opt/freebsd-sysroot"
      if [[ ! -d "$SYSROOT_PATH" ]]; then
        echo "Error: FreeBSD sysroot not found at $SYSROOT_PATH" >&2
        echo "Please create it with:" >&2
        echo "  mkdir -p ~/opt/freebsd-sysroot" >&2
        echo "  curl -LO https://download.freebsd.org/ftp/releases/amd64/13.3-RELEASE/base.txz" >&2
        echo "  tar -xf base.txz -C ~/opt/freebsd-sysroot" >&2
        exit 1
      fi
      echo "Using FreeBSD sysroot at $SYSROOT_PATH"

      # Create temporary clang wrapper
      CLANG_WRAPPER="$(pwd)/clang_wrapper_for_cross_freebsd.sh"
      echo '#!/bin/bash' > "$CLANG_WRAPPER"
      echo 'exec x86_64-unknown-freebsd13-clang -fuse-ld=lld "$@"' >> "$CLANG_WRAPPER"
      chmod +x "$CLANG_WRAPPER"

      export CC="x86_64-unknown-freebsd13-clang"
      export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER="$CLANG_WRAPPER"
      export RUSTFLAGS="-C link-arg=--sysroot=$SYSROOT_PATH"

      # Optionally: remove the wrapper on script exit
      trap 'rm -f "$CLANG_WRAPPER"' EXIT
      ;;
    aarch64-unknown-linux-gnu)
      if ! command -v aarch64-unknown-linux-gnu-gcc >/dev/null 2>&1; then
        echo "Error: aarch64-unknown-linux-gnu-gcc not found in PATH!" >&2
        echo "Please install with:" >&2
        echo "  brew tap messense/macos-cross-toolchains" >&2
        echo "  brew install aarch64-unknown-linux-gnu" >&2
        exit 1
      fi
      export CC="aarch64-unknown-linux-gnu-gcc"
      export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-unknown-linux-gnu-gcc"
      ;;
    x86_64-pc-windows-gnu)
      if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        echo "Error: x86_64-w64-mingw32-gcc not found in PATH!" >&2
        echo "Please install with:" >&2
        echo "  brew install mingw-w64" >&2
        echo "  rustup target add x86_64-pc-windows-gnu" >&2
        exit 1
      fi
      export CC=x86_64-w64-mingw32-gcc
      export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=$CC
      ;;
    aarch64-pc-windows-msvc)
      if ! command -v lib.exe >/dev/null 2>&1; then
              echo "Error: lib.exe special shell script not found in PATH" >&2
              echo "The \`aarch64-pc-windows-msvc\` target requires a lib.exe wrapper for LLVM." >&2
              echo "" >&2
              echo "❶ Ensure LLVM is available (needed for llvm-ar)" >&2
              echo "   brew install llvm" >&2
              echo "" >&2
              echo "❷ Install the wrapper at /opt/homebrew/bin/lib.exe:" >&2
              cat <<'EOF'
#!/bin/bash
# Minimal shim translating “lib.exe” arguments to llvm‑ar.
OUT=""
OBJS=()

for arg in "$@"; do
  case "$arg" in
    -nologo) ;;                       # ignore
    -out:*)  OUT="${arg#-out:}" ;;    # strip “-out:”
    *)       OBJS+=("$arg") ;;        # collect object files
  esac
done

if [[ -z "$OUT" ]]; then
  echo "lib.exe wrapper error: -out:<file> missing" >&2
  exit 1
fi

exec /opt/homebrew/opt/llvm/bin/llvm-ar crs "$OUT" "${OBJS[@]}"
EOF
        echo "" >&2
        echo "❸ Make it executable:" >&2
        echo "   chmod +x /opt/homebrew/bin/lib.exe" >&2
        exit 1
      fi
      # Configure Windows ARM64 sysroot and paths
      SYSROOT="$HOME/opt/aarch64-windows"

      # Set C compiler flags for headers
      export CFLAGS_aarch64_pc_windows_msvc="--include-directory=$SYSROOT/include --include-directory=$SYSROOT/include/ucrt"

      # Set linker
      export CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER="lld-link"

      # Set rustflags directly instead of using config.toml
      export RUSTFLAGS="-C link-arg=/LIBPATH:$SYSROOT/lib/um/arm64 \
                        -C link-arg=/LIBPATH:$SYSROOT/lib/ucrt/arm64 \
                        -C link-arg=/LIBPATH:$SYSROOT/lib/arm64 \
                        -C link-arg=/LIBPATH:$SYSROOT/lib/aarch64-windows-msvc/ucrt \
                        -Lnative=$SYSROOT/lib/ucrt/arm64"

      echo "Configured Windows ARM64 cross-compilation environment"
      ;;
    x86_64-apple-darwin)
          if ! check_target "$TARGET_TRIPLE"; then
            echo "Failed to set up target $TARGET_TRIPLE. Aborting build." >&2
            exit 1
          fi
          ;;
    synology-x86_64)
          TARGET_TRIPLE="x86_64-unknown-linux-gnu"
          if ! command -v x86_64-unknown-linux-gnu-gcc >/dev/null 2>&1; then
            echo "Error: x86_64-unknown-linux-gnu-gcc not found in PATH!" >&2
            echo "Please install with:" >&2
            echo "  brew tap messense/macos-cross-toolchains" >&2
            echo "  brew install x86_64-unknown-linux-gnu" >&2
            exit 1
          fi
          export CC="x86_64-unknown-linux-gnu-gcc"
          export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="x86_64-unknown-linux-gnu-gcc"

          # Build for Synology
          if ! check_target "$TARGET_TRIPLE"; then
            echo "Failed to set up target $TARGET_TRIPLE. Aborting build." >&2
            exit 1
          fi

          # Build the project
          cargo build --release --target "$TARGET_TRIPLE"

          # Create Synology package
          create_synology_package "$TARGET_TRIPLE"

          # Skip the default build at the end
          exit 0
          ;;

        synology-aarch64)
          TARGET_TRIPLE="aarch64-unknown-linux-gnu"
          if ! command -v aarch64-unknown-linux-gnu-gcc >/dev/null 2>&1; then
            echo "Error: aarch64-unknown-linux-gnu-gcc not found in PATH!" >&2
            echo "Please install with:" >&2
            echo "  brew tap messense/macos-cross-toolchains" >&2
            echo "  brew install aarch64-unknown-linux-gnu" >&2
            exit 1
          fi
          export CC="aarch64-unknown-linux-gnu-gcc"
          export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-unknown-linux-gnu-gcc"

          # Build for Synology
          if ! check_target "$TARGET_TRIPLE"; then
            echo "Failed to set up target $TARGET_TRIPLE. Aborting build." >&2
            exit 1
          fi

          # Build the project
          cargo build --release --target "$TARGET_TRIPLE"

          # Create Synology package
          create_synology_package "$TARGET_TRIPLE"

          # Skip the default build at the end
          exit 0
          ;;
    *)
      echo "Error: Unsupported target: $TARGET_TRIPLE" >&2
      echo "Supported targets on macOS:" >&2
      echo "  - x86_64-unknown-linux-gnu" >&2
      echo "  - x86_64-unknown-linux-gnu" >&2
      echo "  - aarch64-unknown-linux-gnu" >&2
      echo "  - x86_64-unknown-freebsd" >&2
      echo "  - x86_64-pc-windows-gnu" >&2
      echo "  - x86_64-apple-darwin" >&2
      echo "  - aarch64-pc-windows-msvc" >&2
      echo "  - synology-x86_64" >&2
      echo "  - synology-aarch64" >&2
      exit 1
      ;;
  esac
else
  echo "Warning: Running on non-macOS host ($HOST_UNAME). Cross-compilation setup may not be necessary." >&2
fi

# Make sure the target is installed
if ! rustup target list | grep -q "^$TARGET_TRIPLE (installed)"; then
  echo "Target $TARGET_TRIPLE not installed. Installing..."
  rustup target add "$TARGET_TRIPLE"
fi

# Finally, build
cargo build --package connectify-backend --bin connectify-backend --all-features --release --target "$TARGET_TRIPLE"

## 6 · Windows AArch64 (**MSVC**) — fake `lib.exe`

#The `aarch64-pc-windows-msvc` target expects Microsoft’s `lib.exe`.
#We can satisfy Cargo/LLVM by dropping a tiny wrapper that forwards the
#MSVC‑style command‑line to `llvm-ar`:
#
#```bash
# ❶ Ensure LLVM is available (needed for llvm‑ar)
#brew install llvm

# ❷ Install the wrapper once:
#tee /opt/homebrew/bin/lib.exe >/dev/null <<'EOF'
##!/bin/bash
## Minimal shim translating “lib.exe” arguments to llvm‑ar.
#OUT=""
#OBJS=()
#
#for arg in "$@"; do
#  case "$arg" in
#    -nologo) ;;                       # ignore
#    -out:*)  OUT="${arg#-out:}" ;;    # strip “-out:”
#    *)       OBJS+=("$arg") ;;        # collect object files
#  esac
#done
#
#if [[ -z "$OUT" ]]; then
#  echo "lib.exe wrapper error: -out:<file> missing" >&2
#  exit 1
#fi
#
#exec /opt/homebrew/opt/llvm/bin/llvm-ar crs "$OUT" "${OBJS[@]}"
#EOF
#sudo chmod +x /opt/homebrew/bin/lib.exe
#```
#
#After that, add the Rust target and build:
#
#```bash
#rustup target add aarch64-pc-windows-msvc
#./cross_build_on_mac.sh aarch64-pc-windows-msvc
#```
#
#The script will detect the target, `lib.exe` will kick in during the
#link‑time creation of static libraries, and you’ll get a native **PE/COFF**
#binary for Windows on ARM64.

