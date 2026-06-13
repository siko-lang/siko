title: Safety
layout: reference

# Safety in Siko

Siko uses a garbage collector but it also has an FFI layer so it can call C code or write/read memory using raw pointers. Siko uses the now traditional safe/unsafe split to mark section of code that break the usual safety guarantees. The unit of safety is a function. By default, a function is safe. You must explicitly mark it as `@unsafe` to use any unsafe features of the language. It is not possible to mark a function as  `@safe` or `@unsafe` if it does not use any of the unsafe operations.

We want to be able to talk about safety of a code pre monomorphization. Due to this design decision, there are some restrictions regarding where you can use or define unsafe functions. A trait instance method must be either fully safe or marked as `@safe`. Meaning it is not possible to "inject" an unsafe functionality into an otherwise safe code during monomorphization.
The same is true for effect handlers, an effect handler is either fully safe or marked as `@safe`.

A function is considered to be unsafe if it uses any of the following operations:
- has any expression that has pointer type
- calls an unsafe function
- calls an extern function
- dereferences a pointer
- takes the address of something
- calls sizeof
- creates a raw function pointer
- calls a raw function pointer
- transmutes a value into another type.

The design philosophy is that anything that reveals platform specific details and does not fit into a java/python esque feel of the language needs to be explicitly marked as unsafe.

The language must guarantee that literally everything you can do in safe mode is safe.

