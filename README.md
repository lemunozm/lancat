# What is *lancat*?
*lancat* is a tool that extends the behaviour of `cat` linux utility to the LAN.
It sends a multicast message for searching *lancat* listeners in the LAN,
then creates one-to-one tcp connection for each listener found in order to transfer the information
in a reliable way and without saturating the network.

# Installation
`lancat` is a [*rust*][rust] application. You can use the [*cargo*][cargo] package manager in order to install it.
```
$ cargo install lancat
```
If you have `~/.cargo/bin` in your PATH, you will be able to use *lancat* everywhere in your computer!

# How it works?
It has two main modes. First of them is to send to the LAN, and the second one is to listen from the LAN.

## To the LAN
To send data to the LAN, run:
```
$ lancat
hello lan
```

Furthermore, you can redirect the *standard input* from a file to send it into the LAN:
```
$ lancat < to_share.txt
```

## From the LAN
To listen data from the LAN, run `lancat` in listen mode with `-l`:
```
$ lancat -l
=========== username - 192.168.1.35:43230 ===========
hello lan
```

In the case you want to send the incoming data to a certain file, you can redirect the *standard output*:
```
$ lancat -l -q > shared.txt
```
the `-q` flag (also `--quiet`) will avoid to send the *user name line* into the file.

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
$ echo "Hello user1" | lancat -u user1
```
Receiving messages filtering by *user2*:
```
$ lancat -l -q -u user2
Hello user1
```
Only users with names *user1* and *user2* will be able to send / listen the communication.

### Aliasing names
Sending a message to *Juanito* identifying us as *Pepito*:
```
$ echo "Hello Juanito, I'm Pepito" | lancat -n Pepito -u Juanito
```
Receiving messages intended for *Juanito* that only *Pepito* sends:
```
$ lancat -l -q -n Juanito -u Pepito
Hello Juanito, I'm Pepito
```

[rust]: https://www.rust-lang.org/
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
