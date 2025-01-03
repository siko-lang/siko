#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

struct siko_Tuple__t__t_
{
};

struct String_String
{
    uint8_t *field0;
    int64_t field1;
};

void Std_Basic_Util_siko_runtime_abort();

struct siko_Tuple__t__t_ Std_Basic_Util_siko_runtime_str(struct String_String *v);

int64_t String_String_eq(struct String_String *v1, struct String_String *v2);

struct String_String String_String_clone(struct String_String *v1);
