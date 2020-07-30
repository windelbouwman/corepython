
from x import putc

def mandel():
    # """Print a mandelbrot fractal to the console, yielding after each character is printed"""
    w = 50.0
    h = 50.0
    y = 0.0
    while y < h:
        x = 0.0
        while x < w:
            Zr = 0.0
            Zi = 0.0
            Tr = 0.0
            Ti = 0.0
            Cr = 2.0 * x / w - 1.5
            Ci = 2.0 * y / h - 1.0

            i = 0
            while i < 50 and Tr + Ti <= 4.0:
                Zi = 2.0 * Zr * Zi + Ci
                Zr = Tr - Ti + Cr
                Tr = Zr * Zr
                Ti = Zi * Zi
                i = i + 1

            if Tr + Ti <= 4.0:
                putc('*')
            else:
                putc(' ')

            x = x + 1.0

        putc('n')
        y = y + 1.0

# run the mandelbrot
# mandel()

