#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

struct siko_Tuple_
{
};

struct Bool_Bool
{
    int32_t field0;
};

struct String_String
{
    uint8_t *field0;
    int64_t field1;
};

typedef int64_t Int_Int;

void Std_Basic_Util_siko_runtime_abort();

struct siko_Tuple_ Std_Basic_Util_siko_runtime_num(Int_Int v);

struct siko_Tuple_ Std_Basic_Util_siko_runtime_str(struct String_String *v);

struct Bool_Bool Std_Basic_Util_siko_runtime_true();

struct Bool_Bool Std_Basic_Util_siko_runtime_false();

struct siko_Tuple_ Std_Basic_Util_siko_runtime_bool(struct Bool_Bool v);

Int_Int Int_Int_add(Int_Int v1, Int_Int v2);

Int_Int Int_Int_sub(Int_Int v1, Int_Int v2);

Int_Int Int_Int_mul(Int_Int v1, Int_Int v2);

Int_Int Int_Int_div(Int_Int v1, Int_Int v2);

struct Bool_Bool Int_Int_eq(Int_Int *v1, Int_Int *v2);

struct Bool_Bool Int_Int_lessThan(Int_Int *v1, Int_Int *v2);

Int_Int Int_Int_clone(Int_Int *v);

struct Bool_Bool String_String_eq(struct String_String *v1, struct String_String *v2);
