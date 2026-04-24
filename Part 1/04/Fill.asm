// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Fill.asm

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.


@Loop
0;JEQ

(Loop)
@SCREEN
D=A
@KBD
D=A-D
@endAddr
M=D
@KBD
D=M
@Blacken
D;JNE
@Clear
D;JEQ

(Blacken)
@endAddr
M=M-1
D=M
@Loop
D;JLT
@SCREEN
A=D+A
M=-1
@Blacken
0;JEQ

(Clear)
@endAddr
M=M-1
D=M
@Loop
D;JLT
@SCREEN
A=D+A
M=0
@Clear
0;JEQ