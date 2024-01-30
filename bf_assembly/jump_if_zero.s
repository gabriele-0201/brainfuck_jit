        .global _start

        .text
_start:

        movb data_pointer, %al
        cmp $0, %al
        je skip # jne - jump not equal

        # write(1, byte, 1)
        mov     $1, %rax                # system call 1 is write
        mov     $1, %rdi                # file handle 1 is stdout
        mov     $msg1, %rsi     # address of string to output
        mov     $17, %rdx                # number of bytes
        syscall                         # invoke operating system to do the write

skip:
        # write(1, byte, 1)
        mov     $1, %rax                # system call 1 is write
        mov     $1, %rdi                # file handle 1 is stdout
        mov     $msg2, %rsi     # address of string to output
        mov     $2, %rdx                # number of bytes
        syscall                         # invoke operating system to do the write

        # exit(0)
        mov     $60, %rax               # system call 60 is exit
        xor     %rdi, %rdi              # we want return code 0
        syscall                         # invoke operating system to exit

        .data

data_pointer:
        .zero 1
msg1:
    .ascii "should be skipped"
msg2:
    .ascii "ok"
