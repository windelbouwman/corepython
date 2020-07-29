
def myAdd(a: int, b: int) -> int:
    while a > 1:
        return a + b

def mySub(a: int, b: int) -> int:
    if a < 10:
        c = 133
        c = a - b
        return c + 7
    else:
        return 1337


def myFoo(a: float) -> float:
    return a + 3.14

def myBar(x: int) -> int:
    return myAdd(x, mySub(100, x))
