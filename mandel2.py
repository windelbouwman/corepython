
from x import putc

def mandel():
    x = 0
    while x < 10:
        putc(65+x)
        putc(10)
        x = x + 1

def mandel2():
    a = [1, 2, 3, 10]
    for x in a:
        putc(65 + x)
        putc(10)


# mandel()
