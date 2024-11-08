%struct.Int_Int = type { i64 }

%struct.Main_Large = type { %struct.Main_Struct1 }

%struct.Main_Struct1 = type { %struct.Int_Int, %struct.Int_Int, %struct.Int_Int, %struct.Int_Int, %struct.Int_Int }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_Large(ptr noundef %s, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Large, align 8
   %field0 = getelementptr inbounds %struct.Main_Large, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %s, ptr align 8 %field0, i64 40, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 40, i1 false)
   ret void
}

define private void @Main_Struct1(ptr noundef %num, ptr noundef %num1, ptr noundef %num2, ptr noundef %num3, ptr noundef %num4, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Struct1, align 8
   %field0 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %num, ptr align 8 %field0, i64 8, i1 false)
   %field1 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %num1, ptr align 8 %field1, i64 8, i1 false)
   %field2 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 2
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %num2, ptr align 8 %field2, i64 8, i1 false)
   %field3 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 3
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %num3, ptr align 8 %field3, i64 8, i1 false)
   %field4 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %num4, ptr align 8 %field4, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 40, i1 false)
   ret void
}

define private void @Main_foo(ptr noundef %fn_result) {
block0:
   %i_0_1 = alloca %struct.Int_Int, align 8
   %tmp_i_0_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_1, i32 0, i32 0
   store i64 6, ptr %tmp_i_0_1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %i_0_1, i64 8, i1 false)
   ret void
}

define private void @Main_foo2(ptr noundef %fn_result) {
block0:
   %i_0_8 = alloca %struct.siko_Tuple_, align 4
   %i_0_7 = alloca %struct.Main_Large, align 8
   %i_0_6 = alloca %struct.Main_Struct1, align 8
   %i_0_5 = alloca %struct.Int_Int, align 8
   %i_0_4 = alloca %struct.Int_Int, align 8
   %i_0_3 = alloca %struct.Int_Int, align 8
   %i_0_2 = alloca %struct.Int_Int, align 8
   %i_0_1 = alloca %struct.Int_Int, align 8
   %tmp_i_0_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_1, i32 0, i32 0
   store i64 1, ptr %tmp_i_0_1_1, align 8
   %tmp_i_0_2_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_2, i32 0, i32 0
   store i64 2, ptr %tmp_i_0_2_1, align 8
   %tmp_i_0_3_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_3, i32 0, i32 0
   store i64 3, ptr %tmp_i_0_3_1, align 8
   %tmp_i_0_4_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_4, i32 0, i32 0
   store i64 4, ptr %tmp_i_0_4_1, align 8
   %tmp_i_0_5_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_5, i32 0, i32 0
   store i64 5, ptr %tmp_i_0_5_1, align 8
   call void @Main_Struct1(ptr %i_0_1, ptr %i_0_2, ptr %i_0_3, ptr %i_0_4, ptr %i_0_5, ptr %i_0_6)
   call void @Main_Large(ptr %i_0_6, ptr %i_0_7)
   call void @siko_Tuple_(ptr %i_0_8)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_0_8, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %i_2_3 = alloca %struct.siko_Tuple_, align 4
   %i_2_2 = alloca %struct.siko_Tuple_, align 4
   %i_2_1 = alloca %struct.siko_Tuple_, align 4
   %i_1_2 = alloca %struct.siko_Tuple_, align 4
   %i_1_1 = alloca %struct.siko_Tuple_, align 4
   %loop_var_0 = alloca %struct.siko_Tuple_, align 4
   %i_0_2 = alloca %struct.siko_Tuple_, align 4
   %i_0_1 = alloca %struct.Int_Int, align 8
   call void @Main_foo(ptr %i_0_1)
   call void @siko_Tuple_(ptr %i_0_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %loop_var_0, ptr align 4 %i_0_2, i64 0, i1 false)
   br label %block1
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %loop_var_0, i64 0, i1 false)
   call void @siko_Tuple_(ptr %i_1_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %loop_var_0, ptr align 4 %i_1_2, i64 0, i1 false)
   br label %block2
block2:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_2_1, ptr align 4 %loop_var_0, i64 0, i1 false)
   call void @Main_foo2(ptr %i_2_2)
   call void @siko_Tuple_(ptr %i_2_3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_2_3, i64 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


