#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

struct siko_int {
    int64_t value;
};

struct siko_bool {
    int32_t value;
};

struct siko_string {
    char* value;
    int64_t length;
};

extern void Std_Basic_Util_siko_runtime_abort() {
    printf("siko_runtime_abort called\n");
    abort();
}

extern void Std_Basic_Util_siko_runtime_num(struct siko_int* v) {
    printf("%ld\n", v->value);
}

extern void Std_Basic_Util_siko_runtime_str(struct siko_string* v) {
    printf("%.*s\n", (int)v->length, v->value);
}

extern void Std_Basic_Util_siko_runtime_true(struct siko_bool* v) {
    v->value = 1;
}

extern void Std_Basic_Util_siko_runtime_false(struct siko_bool* v) {
    v->value = 0;
}

extern void Std_Basic_Util_siko_runtime_bool(struct siko_bool* v) {
    if (v->value) {
        printf("siko_runtime_bool true\n");    
    } else {
        printf("siko_runtime_num false\n");
    }
}

extern void Int_Int_add(struct siko_int* v1, struct siko_int* v2, struct siko_int* result) {
    //printf("add %ld %ld\n", v1->value, v2->value);
    result->value = v1->value + v2->value;
}

extern void Int_Int_sub(struct siko_int* v1, struct siko_int* v2, struct siko_int* result) {
    result->value = v1->value - v2->value;
}

extern void Int_Int_eq(struct siko_int* v1, struct siko_int* v2, struct siko_bool* result) {
    result->value = v1->value == v2->value;
}

extern void Int_Int_lessThan(struct siko_int* v1, struct siko_int* v2, struct siko_bool* result) {
    //printf("lessThan %ld %ld\n", v1->value, v2->value);
    if (v1->value < v2->value) {
        result->value = 1;
    } else {
        result->value = 0;
    }
}

extern void String_String_eq(struct siko_string* v1, struct siko_string* v2, struct siko_bool* result) {
    //printf("string eq!!\n");
    //Std_Basic_Util_siko_runtime_str(v1);
    //Std_Basic_Util_siko_runtime_str(v2);
    if (v1->length != v2->length) {
        result->value = 0;
        return;
    }
    result->value = strncmp(v1->value, v2->value, v1->length) == 0;
}
