// -*- c -*-

// This is a line comment.

/*
 * This is a block comment.
 */

int g1;
int g2[4];

int assert(int expected, int actual, char *code)
{
    if (expected == actual)
    {
        printf("%s => %d\n", code, actual);
    }
    else
    {
        printf("%s => %d expected but got %d\n", code, expected, actual);
        exit(1);
    }
}

int main()
{
    assert(0, 0, "0");
    assert(42, 42, "42");
    assert(21, 5 + 20 - 4, "5 + 20 - 4");
    assert(41, 12 + 34 - 5, "12 + 34 - 5");
    assert(47, 5 + 6 * 7, "5 + 6 * 7");
    assert(15, 5 * (9 - 6), "5 * (9 - 6)");
    assert(4, (3 + 5) / 2, "(3 + 5) / 2");
    assert(10, -10 + 20, "-10 + 20");
    assert(1, (-3 + 5) / 2, "(-3 + 5) / 2");
    assert(6, (-3 * +4) / -2, "(-3*+4)/-2");
    assert(10, - -10, "- -10");
    assert(10, - -+10, "- -+10");

    assert(0, 0 == 1, "0==1");
    assert(1, 42 == 42, "42==42");
    assert(1, 0 != 1, "0!=1");
    assert(0, 42 != 42, "42!=42");

    assert(1, 0 < 1, "0 < 1");
    assert(0, 1 < 1, "1 < 1");
    assert(0, 2 < 1, "2 < 1");
    assert(1, 0 <= 1, "0 <= 1");
    assert(1, 1 <= 1, "1 <= 1");
    assert(0, 2 <= 1, "2 <= 1");

    assert(1, 1 > 0, "1 > 0");
    assert(0, 1 > 1, "1 > 1");
    assert(0, 1 > 2, "1 > 2");
    assert(1, 1 >= 0, "1 >= 0");
    assert(1, 1 >= 1, "1 >= 1");
    assert(0, 1 >= 2, "1 >= 2");

    assert(3, ({ int a; a=3; a; }), "int a; a=3; a;");
    assert(8, ({ int a; int z; a=3; z=5; a+z; }), "int a; int z; a=3; z=5; a+z;");
    assert(1, ({ int a=5; int z=4; a-z; }), "int a=5; int z=4; a-z;");
    assert(15, ({ int a=3; int z=5; a*z; }), "int a=3; int z=5; a*z;");
    assert(2, ({ int a=8; int z=4; a/z; }), "int a=8; int z=4; a/z;");
    assert(6, ({ int a; int b; a=b=3; a+b; }), "int a; int b; a=b=3; a+b;");

    printf("OK\n");
    // return 0;
}

// 838[addr]
// 333

// 8358
