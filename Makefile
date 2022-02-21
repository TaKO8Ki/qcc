test: build
	./target/debug/qcc test.c > tmp.s
	cc -static -o tmp tmp.s
	./tmp

build:
	cargo build
