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

extern struct siko_Tuple__t__t_ Std_Basic_Util_siko_runtime_str(struct String_String *v)
{
    struct siko_Tuple__t__t_ result;
    printf("%.*s\n", (int)v->field1, v->field0);
    return result;
}

extern int64_t String_String_eq(struct String_String *v1, struct String_String *v2)
{
    // printf("string eq0!!\n");
    // printf("string eq1!! %p %p\n", v1, v2);
    // printf("string eq2!! %p %p\n", v1->field0, v2->field0);
    // printf("string eq3!! %ld %ld\n", v1->field1, v2->field1);
    // Std_Basic_Util_siko_runtime_str(v1);
    // Std_Basic_Util_siko_runtime_str(v2);
    if (v1->field1 != v2->field1)
    {
        return 0;
    }
    return strncmp((const char *)v1->field0, (const char *)v2->field0, v1->field1) == 0;
}

struct String_String String_String_clone(struct String_String *v1) 
{
    return *v1;
}
