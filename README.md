# lancat
Currently in process...

## Usage examples (don't implemented yet)
- Listen from LAN
```
$ lancat -l
```
- Listen from LAN filtering users
```
$ lancat -l -u user1 user2
```
- Send to LAN
```
$ echo "Hello lan users" | lancat
```
- Send to LAN filtering users
```
$ echo "Hello lan users user1 and user2" | lancat -u user1 user2
```
- Search for LAN users
```
$ lancat -s
```
