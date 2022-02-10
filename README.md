# qcc
A toy C compiler written in Rust

```sh
$ docker run --rm -it --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/myapp -w /usr/src/myapp rust ./test.sh
```

```sh
$ ./target/release/qcc 'int main() { return fib(9); } int fib(int x) { if (x<=1) return 1; return fib(x-1) + fib(x-2); }' > tmp.s
$ cc -o tmp tmp.s
$ ./tmp
$ echo $?
55
```

## Reference

https://github.com/rui314/chibicc
