set history save on
set remotetimeout 240
target extended-remote electropi:3333
set print asm-demangle on
monitor allowsimulation 0
load
