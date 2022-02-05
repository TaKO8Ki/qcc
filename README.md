# qcc
A toy C compiler written in Rust

```sh
$ docker run --rm -it --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/myapp -w /usr/src/myapp rust ./test.sh
```

```sh
$ ./target/release/qcc 'int main() { return ret32(); } int ret32() { return 32; }' > tmp.s
$ cc -o tmp tmp.s
$ ./tmp
$ echo $?
32
```

## Reference

https://github.com/rui314/chibicc
