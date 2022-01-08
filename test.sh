#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  ./target/debug/qcc "$input" > tmp.s
  cc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

if [ ! -f "./target/debug/qcc" ]; then
    cargo build
fi

assert 0 '{ return 0; }'
assert 42 '{ 42; }'
assert 21 '{ return 5+20-4; }'
assert 41 '{ 12 + 34 - 5 ; }'
assert 47 '{ return 5+6*7; }'
assert 15 '{ 5*(9-6); }'
assert 4 '{ return (3+5)/2; }'
assert 10 '{ -10+20; }'
assert 1 '{ return (-3+5)/2; }'
assert 6 '{ (-3*+4)/-2; }'

assert 0 '{ return 0==1; }'
assert 1 '{ 42==42; }'
assert 1 '{ return 0!=1; }'
assert 0 '{ 42!=42; }'

assert 1 '{ return 0<1; }'
assert 0 '{ 1<1; }'
assert 0 '{ return 2<1; }'
assert 1 '{ 0<=1; }'
assert 1 '{ return 1<=1; }'
assert 0 '{ 2<=1; }'

assert 1 '{ return 1>0; }'
assert 0 '{ 1>1; }'
assert 0 '{ return 1>2; }'
assert 1 '{ 1>=0; }'
assert 1 '{ return 1>=1; }'
assert 0 '{ 1>=2; }'

assert 3 '{ a=3; a; }'
assert 8 '{ a=3; z=5; return a+z; }'
assert 1 '{ a=5; z=4; a-z; }'
assert 15 '{ a=3; z=5; return a*z; }'
assert 2 '{ a=8; z=4; a/z; }'
assert 6 '{ a=b=3; return a+b; }'

assert 3 '{ foo=3; foo; }'
assert 8 '{ foo123=3; bar=5; return foo123+bar; }'

assert 1 '{ return 1; 2; 3; }'
assert 2 '{ 1; return 2; 3; }'
assert 3 '{ 1; 2; return 3; }'

assert 3 '{ {1; {2;} return 3;} }'
assert 5 '{ ;;; return 5; }'

echo OK
