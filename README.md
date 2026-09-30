# samolss-gui

## Requirements

There are maybe a few required packages, but i don't know

## Running

### Nix

This is the goated way, really easy.
If in the repo simply run:

```bash
nix run
```

Otherwise simply do:

```bash
nix run "github:inda26plusplus/samolss-gui"
```

### Cargo

I dont know, just do:

```bash
cargo run
```

in the repo. And debug whatever packages you may be missing

## Options

### Connect

Running the program with

```bash
--connect "<ip_address:port>"
```

will connect to the specified ip address and port, according to TCP (The Chess Protocol) this allows the opponent to chose the color. If this flag is omitted the program will instead listen for incoming connections on the protocols specified port: 6767. This also means that the port one supplies should be 6767.

### Host

Hosting allows for two flags.

Accept-only;

```bash
--accept-only "<ip_address>"
```

With accept-only specified the program will only accept connections from the ip address specified. If accept-only is not used any incoming connection will be allowed to connect.

Color:

```bash
--color <w|b>
```

With color specified the program will chose the specified color for its own player, w makes the player white and b makes the player black. The other color will be sent to the connecting client according to TCP (The Chess Protocol). If color is not used a random color will be chosen with a 50% chance for both colors.

## Examples

Hosting:

```bash
nix run
```

Hosting but only accepting localhost-connections:

```bash
nix run --accept-only "127.0.0.1"
```

Hosting and playing white:

```bash
nix run --color w
```

Accept-only and color can be combined:

```bash
nix run --accept-only "127.0.0.1" --color w # Will host a game, will only accept connections from localhost and the host will be white
```

Connecting to localhost:

```bash
nix run --connect "127.0.0.1:6767"
```

Connecting with a color will work, BUT the color will not be respected since the client does not choose the color:

```bash
nix run --connect "127.0.0.1:6767" --color w # Will connect to localhost on port 6767, but will still be assigned a color by the machine hosting
```
