#include <siko_runtime.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

extern void Std_Basic_Util_siko_runtime_abort()
{
    printf("siko_runtime_abort called\n");
    abort();
}

extern struct siko_Tuple__t__t_ Std_Basic_Util_siko_runtime_num(Int_Int v)
{
    struct siko_Tuple__t__t_ result;
    printf("%ld\n", v);
    return result;
}

extern struct siko_Tuple__t__t_ Std_Basic_Util_siko_runtime_str(struct String_String *v)
{
    struct siko_Tuple__t__t_ result;
    printf("%.*s\n", (int)v->field1, v->field0);
    return result;
}

extern struct Bool_Bool Std_Basic_Util_siko_runtime_true()
{
    struct Bool_Bool result;
    result.field0 = 1;
    return result;
}

extern struct Bool_Bool Std_Basic_Util_siko_runtime_false()
{
    struct Bool_Bool result;
    result.field0 = 0;
    return result;
}

extern struct siko_Tuple__t__t_ Std_Basic_Util_siko_runtime_bool(struct Bool_Bool v)
{
    struct siko_Tuple__t__t_ result;
    if (v.field0)
    {
        printf("siko_runtime_bool true\n");
    }
    else
    {
        printf("siko_runtime_num false\n");
    }
    return result;
}

extern Int_Int Int_Int_add(Int_Int v1, Int_Int v2)
{
    Int_Int result;
    // printf("add %ld %ld\n", v1->field0, v2->field0);
    result = v1 + v2;
    return result;
}

extern Int_Int Int_Int_sub(Int_Int v1, Int_Int v2)
{
    Int_Int result;
    result = v1 - v2;
    return result;
}

extern Int_Int Int_Int_mul(Int_Int v1, Int_Int v2)
{
    Int_Int result;
    result = v1 * v2;
    return result;
}

extern Int_Int Int_Int_div(Int_Int v1, Int_Int v2)
{
    Int_Int result;
    result = v1 / v2;
    return result;
}

extern struct Bool_Bool Int_Int_eq(Int_Int *v1, Int_Int *v2)
{
    // printf("Int_Int_eq %ld %ld\n", v1.field0, v2.field0);
    struct Bool_Bool result;
    result.field0 = *v1 == *v2;
    return result;
}

extern struct Bool_Bool Int_Int_lessThan(Int_Int *v1, Int_Int *v2)
{
    struct Bool_Bool result;
    // printf("lessThan %ld %ld\n", v1->field0, v2->field0);
    if (*v1 < *v2)
    {
        result.field0 = 1;
    }
    else
    {
        result.field0 = 0;
    }
    return result;
}

extern Int_Int Int_Int_clone(Int_Int *v)
{
    return *v;
}

extern struct Bool_Bool String_String_eq(struct String_String *v1, struct String_String *v2)
{
    // printf("string eq0!!\n");
    // printf("string eq1!! %p %p\n", v1, v2);
    // printf("string eq2!! %p %p\n", v1->field0, v2->field0);
    // printf("string eq3!! %ld %ld\n", v1->field1, v2->field1);
    // Std_Basic_Util_siko_runtime_str(v1);
    // Std_Basic_Util_siko_runtime_str(v2);
    struct Bool_Bool result;
    if (v1->field1 != v2->field1)
    {
        result.field0 = 0;
        return result;
    }
    result.field0 = strncmp((const char *)v1->field0, (const char *)v2->field0, v1->field1) == 0;
    return result;
}

struct String_String String_String_clone(struct String_String *v1) 
{
    return *v1;
}
