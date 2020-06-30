from ctypes import cdll, c_char_p
from sys import platform

if platform == 'darwin':
    prefix = 'lib'
    ext = 'dylib'
elif platform == 'win32':
    prefix = ''
    ext = 'dll'
else:
    prefix = 'lib'
    ext = 'so'


lib = cdll.LoadLibrary('target/debug/{}eva_c.{}'.format(prefix, ext))

lib.tasks()
lib.add(
    c_char_p("math assignment".encode()),
    c_char_p("5 Jul 2020 00:00".encode()),
    c_char_p("3".encode()),
    c_char_p("9".encode())
)
lib.set(
    c_char_p("2".encode()),
    c_char_p("duration".encode()),
    c_char_p("10".encode())
)
lib.tasks()
