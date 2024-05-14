# WSL GPG Relay

This implements a way to access a GPG agent running on Windows from WSL. This is useful for signing and decrypting files using a hardware token like Yubikey within the Linux environment.

## Scheme

```
┌───────────────────────────────────────────────┐     
│                   WSL Host                    │     
│   ┌─────────┐                  ┌───────────┐  │     
│   │   WSL   │                  │           │  │     
│   │         ├───────┐    ┌─────┤ GPG Agent │  │     
│   │GPG Relay│       ▼    ▼     │           │  │     
│   └─────────┘    ┌──────────┐  └─────┬─────┘  │     
│       ▲          │          │        │        │     
│       │          │TCP Socket│        │   ┌────┴────┐
│       │          │          │        └──►│ Yubikey │
│       │          └──────────┘            └────┬────┘
│       │                                       │     
│ ┌─────┼─────────────────────────────────────┐ │     
│ │     │           WSL Guest                 │ │     
│ │ ┌───┴───┐       ┌────────┐      ┌───────┐ │ │     
│ │ │       │       │  Unix  │      │       │ │ │     
│ │ │ Socat ├──────►│        │◄─────┤  GPG  │ │ │     
│ │ │       │       │ Socket │      │       │ │ │     
│ │ └───────┘       └────────┘      └───────┘ │ │     
│ │                                           │ │     
│ └───────────────────────────────────────────┘ │     
│                                               │     
└───────────────────────────────────────────────┘
```

## Host

GPG4Win is required to be installed on Windows and the GPG agent must be running. This can be achieved by running `gpg-connect-agent /bye` in a command prompt. A shortcut to this invocation can be put into `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup` to start the agent automatically on login.

The GPG agent should be configured to enable putty support. You can edit `%APPDATA%\Roaming\gnupg\gpg-agent.conf` add the following line (restart the agent process afterwards):

``` 
enable-putty-support
```

## Guest

A user systemd unit will spawn a `socat` process that will listen on a Unix socket and pipe stdin/stdout into a `wsl-gpg-relay.exe` process that is spawned on the host and relays the traffic to the Host's GPG agent.

### Build the Relay

The Rust project needs to be built with a `--target x86_64-pc-windows-gnu` flag to produce a Windows binary. This requires the target and compiler tools to be installed. On Ubuntu, it would look like this:

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt-get install gcc-mingw-w64-x86-64
```

The binary can then be built with:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The resulting binary should be copied to the host (starting windows binaries on the guest's filesystem is significantly slower) and the path to it should be configured in the systemd unit file.

```bash
vim wsl-gpg-relay.service
```

### Setup the Service

```bash
mkdir -p ~/.config/systemd/user
cp wsl-gpg-relay.service ~/.config/systemd/user/
systemctl --user enable wsl-gpg-relay
systemctl --user start wsl-gpg-relay
```

Verify that the service is running:

```bash
systemctl --user status wsl-gpg-relay
```

## References and Prior Art

This project is inspired by [this blog post](https://jardazivny.medium.com/the-ultimate-guide-to-yubikey-on-wsl2-part-1-dce2ff8d7e45) and [this project](https://github.com/Lexicality/wsl-relay)
