# BufTest

## Goal
The goal is to identify how fast 16 byte buffers, or "packets", can be moved from one process to another through stdin/stdout.

The original problem is that we have seen very, very poor performance on corporate Windows laptops and we are trying to identify the cause.

## The Rust
The rust was the original implementation and a modification of the main application. If run with two arguments it will create two subprocesses that are linked over stdin and stdout. It is equivalent to the bash command:
```bash
$ mkfifo fifo
$ buftest one <fifo | buftest two > fifo
```

Windows doesn't have ready access to fifos, so the rust connects the processes in the parent process.
This was duplicated with the bash process on Windows and the applications run equivalently.

On my laptop this results in roughly 12,000,000 packets/sec for about 20 s, then it slows to 1,000,000 packet/s for about 20 second, then returns to the original speed.

## The C
After deciding that the slowdown was not due to Java (the previous implementation), Windows, or Windows antivirus, I rewrote the process in C. See buftest.c.

```bash
$ gcc -O4 -Wall -o buftest buftest.c  && ./buftest one <fifo | ./buftest two > fifo
```

This log displays a little differently and the C code is about half the speed, 6,000,000 packets/s. There is no slowdown.  The slow down seems to be the Rust.