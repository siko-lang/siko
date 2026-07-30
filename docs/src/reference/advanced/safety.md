title: Safety
layout: reference

# Safety in Siko

Siko uses a garbage collector but it also has an FFI layer so it can call C code or write/read memory using raw pointers. The unit of safety is a function. By default, a function is safe. A function using unsafe operations must be marked `@safe` or `@unsafe`: `@safe` promises that its public contract is safe, while callers of an `@unsafe` function must themselves be `@safe` or `@unsafe`. A function without unsafe operations cannot use either annotation.

Raw pointers cannot cross a safe function boundary. A function with `*T` or `void*` in a parameter or return type must be `@unsafe`; `@safe` is rejected for such a signature.

Safety must be known before monomorphization. A trait instance method must therefore be safe or `@safe`. Effect handlers instead match their effect method: a safe effect method requires a safe handler, while an `@unsafe` effect method requires an `@unsafe` handler. These rules apply per method, including methods supplied through a named effect instance.

A function is considered to be unsafe if it uses any of the following operations:

- has any expression that has pointer type
- calls an unsafe function
- installs an unsafe effect handler
- calls an extern function
- dereferences a pointer
- takes the address of something
- calls sizeof
- creates a raw function pointer
- calls a raw function pointer
- transmutes a value into another type.

The design philosophy is that anything that reveals platform specific details and does not fit into a java/python esque feel of the language needs to be explicitly marked as unsafe.

The language must guarantee that literally everything you can do in safe mode is safe.
