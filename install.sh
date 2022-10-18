#!/bin/sh

get_latest_release() {
  curl --silent "https://api.github.com/repos/$1/releases/latest" |
    grep '"tag_name":' |
    sed -E 's/.*"([^"]+)".*/\1/'
}

LATEST_RELEASE=$(get_latest_release "graphteon/luwak")
BIN_URL="https://github.com/graphteon/luwak/releases/download/${LATEST_RELEASE}"
BIN_OSX_X86="${BIN_URL}/luwak-macOS-latest"
BIN_LINUX_X86="${BIN_URL}/luwak-ubuntu-latest"

PATH_LUWAK_BIN="${HOME}/.luwak/bin"

if [[ ! -d "$PATH_LUWAK_BIN" ]]; then
    mkdir -p $PATH_LUWAK_BIN
fi

if [[ $OSTYPE == 'darwin'* ]]; then
    echo "${BIN_OSX_X86}"
    curl -o "${PATH_LUWAK_BIN}/luwak" -L "${BIN_OSX_X86}"
    chmod +x "${PATH_LUWAK_BIN}/luwak"
elif [[ $OSTYPE == 'linux-gnu' ]]; then
    curl -o "${PATH_LUWAK_BIN}/luwak" -L "${BIN_LINUX_X86}"
    chmod +x "${PATH_LUWAK_BIN}/luwak"
else
    echo 'Not supported OS!'
    exit 0
fi

if [ "/bin/zsh" == "$SHELL" ] || [ "/usr/bin/zsh" == "$SHELL" ]; then
  PROFILE_NAME="${HOME}/.zshrc"
elif [ "/bin/bash" == "$SHELL" ] || [ "/usr/bin/bash" == "$SHELL" ]; then
  PROFILE_NAME="${HOME}/.bashrc"
else
    echo 'Add this environment to your shell config!'
    echo 'export PATH="$PATH:$HOME/.luwak/bin"'
fi

IS_LUWAK_INSTALLED=$(grep -c ".luwak/bin" "$PROFILE_NAME")

if [ $IS_LUWAK_INSTALLED -eq 0 ]; then
echo 'export PATH="$PATH:$HOME/.luwak/bin"' >> $PROFILE_NAME
fi

echo "Done..."