%struct.Int_Int = type { i64 }

%struct.Main_Struct1 = type { i64 }

define %struct.Main_Struct1 @Main_Struct1(i64 %num) {
block0:
   %this = alloca %struct.Main_Struct1, align 8
   %field0 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 0
   %tmp_this_1 = load %struct.Main_Struct1, ptr %this, align 8
   ret %struct.Main_Struct1 %tmp_this_1
}

define i64 @Main_foo() {
block0:
   %i1 = add i64 6, 0 
   ret i64 %i1
}

define i32 @Main_foo2() {
block0:
   %i1 = add i64 5, 0 
   %i2 = call %struct.Main_Struct1 @Main_Struct1(i64 %i1)
   %i3 = add i32 0, 0 
   ret i32 %i3
}

define i32 @Main_main() {
block0:
   %i1 = call i64 @Main_foo()
   %i2 = add i32 0, 0 
   %loop_var_0 = alloca i32, align 4
   store i32 %i2, ptr %loop_var_0, align 4
   br label %block1
block1:
   %i2 = add i32 0, 0 
   br label %block2
block2:
   %i2 = call i32 @Main_foo2()
   %i3 = add i32 0, 0 
   ret i32 %i3
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


