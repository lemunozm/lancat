# lancat
Currently in process...

## Usage examples
- Listen from lan
```
$ lancat -l
```
- Listen from lan filtering users
```
$ lancat -l user1 user2
```
- Send to lan
```
$ echo "Hello lan users" | lancat
```
- Send to lan filtering users
```
$ echo "Hello lan users user1 and user2" | lancat user1 user2
```
- Search for users
```
$ lancat -s
```
- Search for users periodically each 1 seconds
```
$ lancat -s 1
```
