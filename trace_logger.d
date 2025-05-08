#!/usr/sbin/dtrace -s

#pragma D option quiet

pid$target::open:entry,
pid$target::write:entry,
pid$target::sendto:entry,
pid$target::door_call:entry
{
    printf("%s(%d, %s)\n", probefunc, arg0, copyinstr(arg1));
}

syscall::open:entry
/pid == $target/
{
    printf("open: %s\n", copyinstr(arg0));
}

syscall::write:entry
/pid == $target/
{
    printf("write: fd=%d, data=%s\n", arg0, copyinstr(arg1));
}

syscall::sendto:entry
/pid == $target/
{
    printf("sendto: fd=%d, data=%s\n", arg0, copyinstr(arg1));
}

syscall::door_call:entry
/pid == $target/
{
    printf("door_call: fd=%d\n", arg0);
}
