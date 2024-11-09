%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Int_Int = type { i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b6i2 = alloca %struct.siko_Tuple_, align 4
   %b6i1 = alloca %struct.Bool_Bool, align 4
   %b5i2 = alloca %struct.siko_Tuple_, align 4
   %b5i1 = alloca %struct.Bool_Bool, align 4
   %b4i2 = alloca %struct.siko_Tuple_, align 4
   %b4i1 = alloca %struct.Bool_Bool, align 4
   %b3i2 = alloca %struct.siko_Tuple_, align 4
   %b3i1 = alloca %struct.Bool_Bool, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.Int_Int, align 8
   %tmp_b0i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i1, i32 0, i32 0
   store i64 5, ptr %tmp_b0i1_1, align 8
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i1, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i64, ptr %tmp_switch_var_block2_1, align 8
   switch i64 %tmp_switch_var_block2_2, label %block6 [
i64 3, label %block3
i64 4, label %block4
i64 5, label %block5
]

block3:
   call void @Bool_Bool_False(ptr %b3i1)
   call void @Std_Basic_Util_assert(ptr %b3i1, ptr %b3i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b3i2, i64 0, i1 false)
   br label %block1
block4:
   call void @Bool_Bool_False(ptr %b4i1)
   call void @Std_Basic_Util_assert(ptr %b4i1, ptr %b4i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b4i2, i64 0, i1 false)
   br label %block1
block5:
   call void @Bool_Bool_True(ptr %b5i1)
   call void @Std_Basic_Util_assert(ptr %b5i1, ptr %b5i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b5i2, i64 0, i1 false)
   br label %block1
block6:
   call void @Bool_Bool_False(ptr %b6i1)
   call void @Std_Basic_Util_assert(ptr %b6i1, ptr %b6i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b6i2, i64 0, i1 false)
   br label %block1
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %b6i1 = alloca %struct.siko_Tuple_, align 4
   %b4i2 = alloca %struct.siko_Tuple_, align 4
   %b4i1 = alloca %struct.siko_Tuple_, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i1, ptr align 4 %v, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i1, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b0i1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %b4i1)
   call void @siko_Tuple_(ptr %b4i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b4i2, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @siko_Tuple_(ptr %b6i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b6i1, i64 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

define private void @Bool_Bool_False(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_False, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_False, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_False, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


