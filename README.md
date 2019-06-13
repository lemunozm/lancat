[![lancat](https://img.shields.io/crates/v/lancat.svg)](https://crates.io/crates/lancat)
[![license](https://img.shields.io/crates/l/lancat.svg)](https://www.apache.org/licenses/LICENSE-2.0.txt)
[![downloads](https://img.shields.io/crates/d/lancat.svg)](https://crates.io/crates/lancat)

# What is *lancat*?
*lancat* is a tool that extends the behaviour of `cat` linux utility to the LAN.
It sends a multicast message for searching *lancat* listeners in the LAN,
then creates one-to-one tcp connection for each listener found in order to transfer the information
in a reliable way and without saturating the network.

# Installation
*lancat* is a [*rust*][rust] application. You can use the [*cargo*][cargo] package manager in order to install it.
```
$ cargo install lancat
```
If you have `~/.cargo/bin` in your PATH, you will be able to use *lancat* everywhere in your computer!

*Currently it only supports `linux`.*

# How it works?
It has two main modes: *write* to the LAN, and *read* from the LAN.

## To the LAN
To write data to the LAN, run `lancat` into *write* mode with `w`:
```
$ lancat -w
hello lan
```

Furthermore, you can redirect the *standard input* from a file to write it into the LAN:
```
$ lancat -w < to_share.txt
```

## From the LAN
To read data from the LAN, run `lancat` into *read* mode with `-r`:
```
$ lancat -r
=========== username - 192.168.1.35:43230 ===========
hello lan
```

In the case you want to send the incoming data to a certain file, you can redirect the *standard output*:
```
$ lancat -r -q > shared.txt
```
the `-q` flag (also `--quiet`) will avoid to write the *user name line* into the file.

## Filtering users
By default *lancat* notifies to the LAN with your OS user name.
It is possbile to change this name with the `-n` flag.

You can filter for writing or reading only for certain users:
```
$ lancat -w -u user1 user2
```
```
$ lancat -r -u user1 user2
```

In order to see which users are listening the lan, you can run *lancat* in *search mode* with `-s`:
```
$ lancat -s
Found 'user1' at: 192.168.1.72:44435
Found 'user2' at: 192.168.1.72:44439
Found 'user3' at: 192.168.1.54:44432
```

To see all available options see the help: `lancat --help`.

# Usage Examples
## Pair to pair LAN communication
### Default user names
Sending a message filtering by *user1*:
```
$ echo "Hello user1" | lancat -w -u user1
```
Receiving messages filtering by *user2*:
```
$ lancat -r -q -u user2
Hello user1
```
Only users with names *user1* and *user2* will be able participate in the communication.

### Aliasing names
Sending a message to *Juanito* identifying us as *Pepito*:
```
$ echo "Hello Juanito, I'm Pepito" | lancat -w -n Pepito -u Juanito
```
Receiving messages intended for *Juanito* that only *Pepito* sends:
```
$ lancat -r -q -n Juanito -u Pepito
Hello Juanito, I'm Pepito
```

# Changelog
#### v0.2.0
* Modified cli
#### v0.1.0
* *lancat* base

[rust]: https://www.rust-lang.org/
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
