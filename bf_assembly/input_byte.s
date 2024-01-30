        .global _start

        .text
_start:

        # open(1, data_pointer, 1)
        mov     $0, %rax                # system call 0 is open
        mov     $1, %rdi                # file handle 0 is stdin
        mov     $data_pointer, %rsi     # address of string to output
        mov     $1, %rdx                # number of bytes
        syscall                         # invoke operating system to do the write

        # write(1, byte, 1)
        mov     $1, %rax                # system call 1 is write
        mov     $1, %rdi                # file handle 1 is stdout
        mov     $data_pointer, %rsi     # address of string to output
        mov     $1, %rdx                # number of bytes
        syscall                         # invoke operating system to do the write

        # exit(0)
        mov     $60, %rax               # system call 60 is exit
        xor     %rdi, %rdi              # we want return code 0
        syscall                         # invoke operating system to exit

        .data

data_pointer:
        .zero 1
