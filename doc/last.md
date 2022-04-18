# The last programming language

Over time, the design of programming languages keeps evolving, they are better than ever. Older programming languages supported fewer abstractions so it was more complicated to express something that is now trivial in modern languages. These abstractions are found by recognizing patterns in everyday programs and eventually these patterns are integrated in newer languages.
This step is repeated over and over. It is probably not possible to tell how and when this process will end and how the last programming language will work but nevertheless, it is an interesting question to ponder.

If you write an algorithm in python, you cannot simply turn in into a program written in another
programming language like java or rust. Sometimes not even into an older version of the same programming language (turn python3 into python2).
Why is that? This issue is definitely not python specific, you cannot do this with any of currently available languages.
Due to this nature of programming, if you choose a language then you have to stick to it and accept its limitations.
Languages come with various range of libraries with various qualities. Unfortunately, we cannot just use a better
library from another language if it has more features or just better performance. Languages come with various
limitations/constraints regarding their runtimes, their memory model and object model are heavily entangled with the runtime.
When you write a program in a chosen language, the program explicitly or implicitly uses and depends on these constraints
and if these constraints do not match closely enough your "next programming language", moving over to another programming language
will be painful. Assuming that this hypothesis is true, let's try to design a language that tries to solve these issues.

> Our programming language will have to not depend on memory models or runtimes.

The memory model is explicitly or implicitly part of the program in case the program depends on the various constraints of the memory model.
In managed languages, the memory is managed by the garbage collector and programs build upon this. In such languages people do not
care about the details of memory management (unless they have to due to performance reasons) and just keep allocating objects, relying on the garbage collector. In such languages, references can be freely stored in any object. You do not have to think about lifetimes.
These programs are written in a style where the object reference graphs are usually complex. Usually everything is accessed and stored as a reference, value objects are usually not supported. The usual garbage collected language also supports mutation and these features allow designs
where they are also used heavily. The program will store many references to various objects and mutate those. Such a program heavily depends on these features so it would be extremely hard to rewrite it in a memory model independent fashion. Therefore in our hypothetical language we will not allow references and mutation.

Since we just banned references and mutation, we definitely don't need a garbage collector. Our memory management strategy will be values only
and the language will just copy everything, everywhere, all the time. It will be insanely slow but the resulting code hopefully will be truly memory management and runtime agnostic. But there are other runtime features beside memory management. What about those? For this thought experiment, we ban every runtime feature. Every feature of the programming language must be a compile time feature, meaning that the compiler has to be able completely remove said feature and convert it into something basic that does not require special handling. But what are those "basic features"?

## What can we do in this language then?

We don't want to cripple the language completely, so we must allow some things. Since our thought experiment is about semantics, we will not
argue much about syntax, in this exercise we'll use something derived from Haskell's syntax just because I happen to like it and it will contain arbitrary changes just because I happen to like other things as well:). If you squint hard enough, Haskell is already a value only language so it's syntax is probably a nice starting point anyway. In our language, we'll allow definiton of discrete values.

    data Bool = True | False

We'll also support record like things because naming fields are useful and does not require runtime support.

    data Job = { hard :: Bool }

We'll also support some builtin types because without them this language would be totally useless and we want it to be useful.

    data String = extern
    data Int = extern

    data Person = { name :: String, age :: Int }

But I think your String type probably allocates! Well, yes, but our program will not use that fact, it just handles the String as a value. Internally, i will be possible to allocate it with malloc() and free it with free() or simply rely on a garbage collector or any other fancy scheme you can think of. The program will not care.

We'll also support algebraic data types because in today's programming culture they are a must have and they don't need special runtime support.

    data Option a = Some a | None

Oops, we also added generics. Generics will be monomorphized so there won't be runtime support needed for them either.

But what about recursive data types?

    data Tree a = Leaf a | Node (Tree a) (Tree a)

How do we handle that? Well, obviously these require some kind of indirection, so they will be implicitly boxed under the hood. Other data types will not have to be boxed, because we only have values and we copy everything, everywhere, all the time.

For example, the Person above could be something like this:

    struct Person {
        name: String,
        age: i64
    }

How large is that Int anyway? What is the encoding of the String type? These questions are great but in my opinion, they do not alter the end result of this experiment so we'll just ignore them, for now. Int will be an i64 and String will be an UTF-8 string.

Now that we can define data structures we'll need some way to write behaviour as well because so far this is kind of useless.

We'll need functions.

    greet :: String -> ()
    greet name = println "Hello {}" % name

    main :: ()
    main = do
        println "Tell me your name"
        name <- getLine
        greet name

This is doing something already! We have blocks (marked by do), we can read stuff from command line and print stuff. We do not have higher kinded types and do notation, monads, this is a normal imperative, impure programming language. You could even say that so far this is just like a lame transpiler to rust, at best:

    fn greet(name: String) -> () {
        println!("Hello {}", name);
    }

    fn main() -> () {
        println!("Tell me your name:");
        let name: String = getLine();
        greet(name);
    }

Or a lame transpiler to python:

    def greet(name):
        print("Hello %s", name)

    def main():
        print("Tell me your name:")
        name = getLine()
        greet(name)

In this super simple example, we can convert the program into rust or python even though they have vastly different memory management strategies. We also completely ignored other differences.