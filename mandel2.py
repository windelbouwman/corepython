
from x import putc
from x import put_float

def mandel():
    x = 0
    while x < 10:
        putc(65+x)
        putc(10)
        x = x + 1

def mandel2():
    a = [1, 2, 3, 10]
    b = [[4, 5, 6], [7, 8], a]
    c = [3.14, 2.7, 13.37, 42.42]
    for x in a:
        putc(65 + x)
        putc(10)
    # for x in reversed(b):
    for y in b:
        putc(10)
        for x in y:
            putc(65 + x)
            putc(32)

    putc(10)
    for f in c:
        put_float(f)
        putc(32)
    putc(10)

# mandel()
