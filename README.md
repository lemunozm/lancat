# What is *lancat*?
*lancat* is an utility that extend the behavior of `cat` tool to the LAN.
It sends a multicast message for searching *lancat* listeners in the LAN,
and creates an one-to-one tcp connection for each listener found to transfer the information,
in a reliable way and without saturate the network.

# Installation
`lancat` is a *rust* application, so you need the *cargo rust*'s utility installed in your computer in order to compile it.
Place into the repository and run:
```
$ cargo install --path .
```
In you have `~/.cargo/bin` in your PATH, you will be able to use *lancat* in everywhere in your computer!.

# How it works?
It has two main modes, for writing to the LAN and for reading from the LAN.

## To the LAN
For write data to the LAN, run:
```
$ lancat
hello lan
```

Also, you can redirect the *standard input* from a file to write it into the LAN:
```
$ lancat < to_share.txt
```

## From the LAN
For listen the data from the LAN run `lancat -l` (`-l` will enable the listen mode):
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
You can change it with the `-n` flag.

You can filter for sending messages only to certain users:
```
$ lancat -u -q user1 user2
```
Only user1 and user2 will listen.

And tou can filter also to receive messages only from concrete users:
```
$ lancat -l -u user1 user2
```
Only user1 and user2 will be listened.

In order to see which users are listening the lan, you can run *lancat* in the search mode with `-s`
```
$ lancat -s
```

For see all available options see the help: `lancat --help`.

# Usage examples
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
Only users with names user1 and user2 will be able to write / read the communication.

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

