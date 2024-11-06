#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

struct siko_int {
    int64_t value;
};

struct siko_bool {
    int32_t value;
};

void Std_Basic_Util_siko_runtime_abort() {
    printf("siko_runtime_abort called\n");
    abort();
}

void Other_Module_siko_runtime_num(struct siko_int* v) {
    printf("siko_runtime_num %ld\n", v->value);
}

void Other_Module_siko_runtime_bool(struct siko_bool* v) {
    if (v->value) {
        printf("siko_runtime_bool true\n");    
    } else {
        printf("siko_runtime_num false\n");
    }
}

void Int_Int_add(struct siko_int* v1, struct siko_int* v2, struct siko_int* result) {
    result->value = v1->value + v2->value;
}

void Int_Int_sub(struct siko_int* v1, struct siko_int* v2, struct siko_int* result) {
    result->value = v1->value - v2->value;
}

void Int_Int_eq(struct siko_int* v1, struct siko_int* v2, struct siko_bool* result) {
    result->value = v1->value == v2->value;
}

void Int_Int_lessThan(struct siko_int* v1, struct siko_int* v2, struct siko_bool* result) {
    result->value = v1->value < v2->value;
}
