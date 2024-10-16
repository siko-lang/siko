define i32 @Main_main() {
block0:
   %i_0_1 = add i32 0, 0 
   ret i32 %i_0_1
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


