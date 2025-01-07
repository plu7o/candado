echo "Downloading Binary..."

# Define variables
REPO="plu7o/candado"
DEST_DIR="$HOME/.local/share"
FILE_NAME="x86_64-unknown-linux-musl.tar.gz"

# Fetch latest release asset URL
DOWNLOAD_URL=$(curl -s https://api.github.com/repos/$REPO/releases/latest | jq -r ".assets[] | select(.name | contains(\"$FILE_NAME\")).browser_download_url")

if [ -n "$DOWNLOAD_URL" ]; then
    echo "Downloading latest release from $DOWNLOAD_URL..."
    wget -P "$DEST_DIR" "$DOWNLOAD_URL"
    echo "File downloaded to $DEST_DIR"
else
    echo "Error: File not found in the latest release."
fi
echo "Unpacking archive..."
tar -xzf $HOME/.local/share/candadocli-1.0.0-x86_64-unknown-linux-musl.tar.gz -C $HOME/.local/share/
echo "Installing candado -> $HOME/.local/bin/candado"
mv $HOME/.local/share/candadocli-1.0.0-x86_64-unknown-linux-musl/candadocli $HOME/.local/bin/candado
echo "Cleaning up install..."
rm $HOME/.local/share/candadocli-1.0.0-x86_64-unknown-linux-musl.tar.gz
rm -rf $HOME/.local/share/candadocli-1.0.0-x86_64-unknown-linux-musl/
echo "Done"
echo ""
echo "run:" 
echo "  'candado vault init' to initialize vault OR candado gen <COMMAND> to generate secrets"
echo ""
echo "For more information visit: 'https://github.com/plu7o/candado'"
echo "Report issues here: 'https://github.com/plu7o/candado/issues'"
