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

void Std_Basic_Util_siko_runtime_abort();

void Std_Basic_Util_siko_runtime_num(struct siko_int* v);

void Std_Basic_Util_siko_runtime_str(struct siko_string* v);

void Std_Basic_Util_siko_runtime_true(struct siko_bool* v);

void Std_Basic_Util_siko_runtime_false(struct siko_bool* v);

void Std_Basic_Util_siko_runtime_bool(struct siko_bool* v);

void Int_Int_add(struct siko_int* v1, struct siko_int* v2, struct siko_int* result);

void Int_Int_sub(struct siko_int* v1, struct siko_int* v2, struct siko_int* result);

void Int_Int_eq(struct siko_int* v1, struct siko_int* v2, struct siko_bool* result);

void Int_Int_lessThan(struct siko_int* v1, struct siko_int* v2, struct siko_bool* result);

void String_String_eq(struct siko_string* v1, struct siko_string* v2, struct siko_bool* result);
