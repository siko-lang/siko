%struct.Int_Int = type { i64 }

%struct.Main_Large = type { %struct.Main_Struct1 }

%struct.Main_Struct1 = type { i64, i64, i64, i64, i64 }

define %struct.Main_Large @Main_Large(ptr noundef byval(%struct.Main_Struct1) align 8 %s) {
block0:
   %this = alloca %struct.Main_Large, align 8
   %field0 = getelementptr inbounds %struct.Main_Large, ptr %this, i32 0, i32 0
   store %struct.Main_Struct1 %s, ptr %field0, align 8
   %tmp_this_1 = load %struct.Main_Large, ptr %this, align 8
   ret %struct.Main_Large %tmp_this_1
}

define %struct.Main_Struct1 @Main_Struct1(i64 %num, i64 %num1, i64 %num2, i64 %num3, i64 %num4) {
block0:
   %this = alloca %struct.Main_Struct1, align 8
   %field0 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 0
   store i64 %num, ptr %field0, align 8
   %field1 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 1
   store i64 %num1, ptr %field1, align 8
   %field2 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 2
   store i64 %num2, ptr %field2, align 8
   %field3 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 3
   store i64 %num3, ptr %field3, align 8
   %field4 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 4
   store i64 %num4, ptr %field4, align 8
   %tmp_this_1 = load %struct.Main_Struct1, ptr %this, align 8
   ret %struct.Main_Struct1 %tmp_this_1
}

define i64 @Main_foo() {
block0:
   %i_0_1 = add i64 6, 0 
   ret i64 %i_0_1
}

define i32 @Main_foo2() {
block0:
   %i_0_1 = add i64 1, 0 
   %i_0_2 = add i64 2, 0 
   %i_0_3 = add i64 3, 0 
   %i_0_4 = add i64 4, 0 
   %i_0_5 = add i64 5, 0 
   %i_0_6 = call %struct.Main_Struct1 @Main_Struct1(i64 %i_0_1, i64 %i_0_2, i64 %i_0_3, i64 %i_0_4, i64 %i_0_5)
   %i_0_7 = call %struct.Main_Large @Main_Large(%struct.Main_Struct1 %i_0_6)
   %i_0_8 = add i32 0, 0 
   ret i32 %i_0_8
}

define i32 @Main_main() {
block0:
   %i_0_1 = call i64 @Main_foo()
   %i_0_2 = add i32 0, 0 
   %loop_var_0 = alloca i32, align 4
   store i32 %i_0_2, ptr %loop_var_0, align 4
   br label %block1
block1:
   %i_1_2 = add i32 0, 0 
   br label %block2
block2:
   %i_2_2 = call i32 @Main_foo2()
   %i_2_3 = add i32 0, 0 
   ret i32 %i_2_3
}

define i32 @main() {
   call void @Main_main()
   ret i32 0
}


