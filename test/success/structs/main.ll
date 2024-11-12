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
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field0, ptr align 8 %s, i64 40, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 40, i1 false)
   ret void
}

define private void @Main_Struct1(ptr noundef %num, ptr noundef %num1, ptr noundef %num2, ptr noundef %num3, ptr noundef %num4, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Struct1, align 8
   %field0 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field0, ptr align 8 %num, i64 8, i1 false)
   %field1 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field1, ptr align 8 %num1, i64 8, i1 false)
   %field2 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 2
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field2, ptr align 8 %num2, i64 8, i1 false)
   %field3 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 3
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field3, ptr align 8 %num3, i64 8, i1 false)
   %field4 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field4, ptr align 8 %num4, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 40, i1 false)
   ret void
}

define private void @Main_foo(ptr noundef %fn_result) {
block0:
   %lit_1 = alloca %struct.Int_Int, align 8
   %tmp_lit_1_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_1, i32 0, i32 0
   store i64 6, ptr %tmp_lit_1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %lit_1, i64 8, i1 false)
   ret void
}

define private void @Main_foo2(ptr noundef %fn_result) {
block0:
   %unit_8 = alloca %struct.siko_Tuple_, align 4
   %call_7 = alloca %struct.Main_Large, align 8
   %call_6 = alloca %struct.Main_Struct1, align 8
   %lit_5 = alloca %struct.Int_Int, align 8
   %lit_4 = alloca %struct.Int_Int, align 8
   %lit_3 = alloca %struct.Int_Int, align 8
   %lit_2 = alloca %struct.Int_Int, align 8
   %lit_1 = alloca %struct.Int_Int, align 8
   %tmp_lit_1_2 = getelementptr inbounds %struct.Int_Int, ptr %lit_1, i32 0, i32 0
   store i64 1, ptr %tmp_lit_1_2, align 8
   %tmp_lit_2_3 = getelementptr inbounds %struct.Int_Int, ptr %lit_2, i32 0, i32 0
   store i64 2, ptr %tmp_lit_2_3, align 8
   %tmp_lit_3_4 = getelementptr inbounds %struct.Int_Int, ptr %lit_3, i32 0, i32 0
   store i64 3, ptr %tmp_lit_3_4, align 8
   %tmp_lit_4_5 = getelementptr inbounds %struct.Int_Int, ptr %lit_4, i32 0, i32 0
   store i64 4, ptr %tmp_lit_4_5, align 8
   %tmp_lit_5_6 = getelementptr inbounds %struct.Int_Int, ptr %lit_5, i32 0, i32 0
   store i64 5, ptr %tmp_lit_5_6, align 8
   call void @Main_Struct1(ptr %lit_1, ptr %lit_2, ptr %lit_3, ptr %lit_4, ptr %lit_5, ptr %call_6)
   call void @Main_Large(ptr %call_6, ptr %call_7)
   call void @siko_Tuple_(ptr %unit_8)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_8, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %unit_11 = alloca %struct.siko_Tuple_, align 4
   %call_10 = alloca %struct.siko_Tuple_, align 4
   %finalValueRef_4 = alloca %struct.siko_Tuple_, align 4
   %unit_7 = alloca %struct.siko_Tuple_, align 4
   %loopVar_3 = alloca %struct.siko_Tuple_, align 4
   %tuple_2 = alloca %struct.siko_Tuple_, align 4
   %call_1 = alloca %struct.Int_Int, align 8
   call void @Main_foo(ptr %call_1)
   call void @siko_Tuple_(ptr %tuple_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %loopVar_3, ptr align 4 %tuple_2, i64 0, i1 false)
   br label %block1
block1:
   call void @siko_Tuple_(ptr %unit_7)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %loopVar_3, ptr align 4 %unit_7, i64 0, i1 false)
   br label %block2
block2:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %finalValueRef_4, ptr align 4 %loopVar_3, i64 0, i1 false)
   call void @Main_foo2(ptr %call_10)
   call void @siko_Tuple_(ptr %unit_11)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_11, i64 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


