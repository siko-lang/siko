define i32 @Main_main() {
block0:
   %tmp_i1_1 = alloca i32, align 4
   %i1 = load i32, ptr %tmp_i1_1, align 4
   store i32 0, ptr %i1, align 4
   ret i32 %i1
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


