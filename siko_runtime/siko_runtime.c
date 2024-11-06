#include <stdio.h>

void siko_runtime_abort() {
    printf("siko_runtime_abort called\n");
}

void Other_Module_siko_runtime_abort() {
    printf("ccc called\n");
}