#include <stdio.h>
#include <stdint.h>

struct siko_int {
    int64_t value;
};

void siko_runtime_abort() {
    printf("siko_runtime_abort called\n");
}

void Other_Module_siko_runtime_abort() {
    printf("ccc called\n");
}

void Other_Module_siko_runtime_num(struct siko_int* v) {
    printf("siko_runtime_num %ld\n", v->value);
}

void Int_Int_add(struct siko_int* v1, struct siko_int* v2, struct siko_int* result) {
    result->value = v1->value + v2->value;
}

void Int_Int_sub(struct siko_int* v1, struct siko_int* v2, struct siko_int* result) {
    result->value = v1->value - v2->value;
}