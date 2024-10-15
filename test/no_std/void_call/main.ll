define i32 @Main_foo() {
block0:
   %i1 = add i32 0, 0 
   ret i32 %i1
}

define i32 @Main_main() {
block0:
   %i1 = call i32 @Main_foo()
   %i2 = add i32 0, 0 
   ret i32 %i2
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


