title: FAQ
layout: reference
priority: 50

# Frequently Asked Questions

## What is Siko?

Siko is a statically typed programming language with an effect system, implicits, and a powerful type system.
The project is about trying to implement a programming language that is general enough to really run anywhere without feeling like a low level language. The core idea is that the language hides all runtime details in safe mode and gives very friendly and more importantly expressive tools to describe algorithms in a very general runtime agnostic way.

## Is Siko self-hosting?

Yes. The Siko compiler compiles itself. It isn't particularly fast but now it is actually usable.
This is the first iteration of siko that is usable and self-hosting.

## What platforms does Siko target?

Siko currently compiles to (very unreadable) C, which is then compiled by Clang. Currently macOS and Linux are supported.
