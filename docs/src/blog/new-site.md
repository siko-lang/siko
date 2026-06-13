title: New Everything
date: 2026-06-13
layout: blog

This is, I think, my 5th attempt to write a Siko compiler. It has almost nothing to do with the original design, apart of the core idea that the language needs to be as runtime agnostic as possible. The current iteration is complex and more importantly advanced enough to compile itself (and several other programs) and it is usable in a practical sense so I feel like working on it again. The site has been rewritten as well. Funnily enough the static site generator is also written in Siko. I quite like the whole thing.
The new language completely abandoned the ownership model because it just eats into absolute everything. Forcing ownership model onto the language does not give you space to play with other ideas. Also, it does not give the user experience I wanted for Siko. The newest iteration just uses a GC with a twist. The user is unable to interact with object pointers in safe mode. So no java like object1 == object2 by pointers. Also there is no null. Structs are heap allocated reference types but == uses the eq trait so it compares values. Enums are stack allocated unless they participate in a cyclic graph of types.
Currently the language gives you unsafe FFI and it compiles into C so the unsafe layer works as a very strange dialect of C where you can also create value types.
