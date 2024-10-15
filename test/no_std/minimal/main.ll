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
   %tmp_i3_1 = alloca i32, align 4
   %i3 = load i32, ptr %tmp_i3_1, align 4
   store i32 0, ptr %i3, align 4
   ret i32 %i3
}

define i32 @Main_main() {
block0:
   %i1 = call i64 @Main_foo()
   %tmp_i2_1 = alloca i32, align 4
   %i2 = load i32, ptr %tmp_i2_1, align 4
   store i32 0, ptr %i2, align 4
block1:
   %tmp_i2_1 = alloca i32, align 4
   %i2 = load i32, ptr %tmp_i2_1, align 4
   store i32 0, ptr %i2, align 4
block2:
   %i2 = call i32 @Main_foo2()
   %tmp_i3_1 = alloca i32, align 4
   %i3 = load i32, ptr %tmp_i3_1, align 4
   store i32 0, ptr %i3, align 4
   ret i32 %i3
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


