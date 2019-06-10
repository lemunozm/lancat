# What is *lancat*?
*lancat* is a tool that extend the behavior of `cat` linux utility to the LAN.
It sends a multicast message for searching *lancat* listeners in the LAN,
then creates one-to-one tcp connection for each listener found in order to transfer the information
in a reliable way and without saturating the network.

# Installation
`lancat` is a [*rust*][rust] application, so you can use the [*cargo*][cargo] package manager in order to install it.
For it, place into the repository and run:
```
$ cargo install --path .
```
In you have `~/.cargo/bin` in your PATH, you will be able to use *lancat*  everywhere in your computer!

# How it works?
It has two main modes, for writing to the LAN and for reading from the LAN.

## To the LAN
For writing data to the LAN, run:
```
$ lancat
hello lan
```

Also, you can redirect the *standard input* from a file to write it into the LAN:
```
$ lancat < to_share.txt
```

## From the LAN
For listening data from the LAN run `lancat` in listen mode with `-l`:
```
$ lancat -l
=========== username - ip:port ===========
hello lan
```

Or if you want to write the incoming data to a file, you can redirect the *standard output*:
```
$ lancat -l -q > shared.txt
```
the `-q` flag (also `--quiet`) will avoid to write the *user name line* into the file.

## Filtering users
By default *lancat* notifies to the LAN with your OS user name.
It is possbile to change this name with the `-n` flag.

You can filter for sending or receiving messages only for certain users:
```
$ lancat -u user1 user2
```
```
$ lancat -l -u user1 user2
```

In order to see which users are listening the lan, you can run *lancat* in the *search mode* with `-s`
```
$ lancat -s
```

For see all available options see the help: `lancat --help`.

# Usage Examples
## Pair to pair LAN communication
### Default user names
We send a message filtering by *user1*:
```
$ echo "Hello user1" | lancat -u user1
```
We recive messages filtering by *user2*:
```
$ lancat -l -q -u user2
Hello user1
```
Only users with names *user1* and *user2* will be able to write / read the communication.

### Aliasing names
We send a message only to *Pepito* identifying as *Pepito*:
```
$ echo "Hello Juanito, I'm Pepito" | lancat -n Pepito -u Juanito
```
We recive messages intended only to *Juanito* that only *Pepito* writes:
```
$ lancat -l -q -n Juanito -u Pepito
Hello Juanito, I'm Pepito
```

[rust]: https://www.rust-lang.org/
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
