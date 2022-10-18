# LUWAK Secure & Minimalize JS Runtime
## Table of Contents

- [Build](#build)
- [Installing and Updating](#installing-and-updating)
  - [Install & Update Script](#install--update-script)
    - [Additional Notes](#additional-notes)
    - [Troubleshooting on Linux](#troubleshooting-on-linux)
    - [Ansible](#ansible)
## Build

```bash
git clone https://github.com/graphteon/luwak.git
cd luwak
cargo build --release
```

## Installing and Updating

### Install & Update Script

To **install** or **update** luwak, you should run the [install script][2]. To do that, you may either download and run the script manually, or use the following cURL or Wget command:
```sh
curl -o- https://raw.githubusercontent.com/graphteon/luwak/main/install.sh | bash
```
```sh
wget -qO- https://raw.githubusercontent.com/graphteon/luwak/main/install.sh | bash
```

Running either of the above commands downloads a script and runs it. The script attempts to add the source lines from the snippet below to the correct profile file (`~/.zshrc`, or `~/.bashrc`).

```sh
export PATH="$PATH:$HOME/.luwak/bin"
```

#### Troubleshooting on Linux

On Linux, after running the install script, if you get `luwak: command not found` or see no feedback from your terminal after you type `command -v luwak`, simply close your current terminal, open a new terminal, and try verifying again.
Alternatively, you can run the following commands for the different shells on the command line:

*bash*: `source ~/.bashrc`

*zsh*: `source ~/.zshrc`

*ksh*: `. ~/.profile`

These should pick up the `luwak` command.

On ubuntu if you get error ssl version, you can run the following commands on the command line:

```
wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.16_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2.16_amd64.deb
```

#### Ansible

You can use a task:

```yaml
- name: Install luwak
  ansible.builtin.shell: >
    curl -o- https://raw.githubusercontent.com/graphteon/luwak/main/install.sh | bash
```