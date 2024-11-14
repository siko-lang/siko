%struct.Bool_Bool = type { i32, [0 x i32] }

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
   %call_13 = alloca %struct.siko_Tuple_, align 4
   %call_12 = alloca %struct.Bool_Bool, align 4
   %call_10 = alloca %struct.siko_Tuple_, align 4
   %call_9 = alloca %struct.Bool_Bool, align 4
   %call_7 = alloca %struct.siko_Tuple_, align 4
   %call_6 = alloca %struct.Bool_Bool, align 4
   %call_4 = alloca %struct.siko_Tuple_, align 4
   %call_3 = alloca %struct.Bool_Bool, align 4
   %matchValue_16 = alloca %struct.siko_Tuple_, align 4
   %match_var_2 = alloca %struct.siko_Tuple_, align 4
   %lit_1 = alloca %struct.Int_Int, align 8
   %tmp_lit_1_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_1, i32 0, i32 0
   store volatile i64 5, ptr %tmp_lit_1_1, align 8
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_16, ptr align 4 %match_var_2, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %matchValue_16, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_2 = getelementptr inbounds %struct.Int_Int, ptr %lit_1, i32 0, i32 0
   %tmp_switch_var_block2_3 = load i64, ptr %tmp_switch_var_block2_2, align 8
   switch i64 %tmp_switch_var_block2_3, label %block6 [
i64 3, label %block3
i64 4, label %block4
i64 5, label %block5
]

block3:
   call void @Bool_Bool_False(ptr %call_3)
   call void @Std_Basic_Util_assert(ptr %call_3, ptr %call_4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_4, i64 0, i1 false)
   br label %block1
block4:
   call void @Bool_Bool_False(ptr %call_6)
   call void @Std_Basic_Util_assert(ptr %call_6, ptr %call_7)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_7, i64 0, i1 false)
   br label %block1
block5:
   call void @Bool_Bool_True(ptr %call_9)
   call void @Std_Basic_Util_assert(ptr %call_9, ptr %call_10)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_10, i64 0, i1 false)
   br label %block1
block6:
   call void @Bool_Bool_False(ptr %call_12)
   call void @Std_Basic_Util_assert(ptr %call_12, ptr %call_13)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_13, i64 0, i1 false)
   br label %block1
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %unit_9 = alloca %struct.siko_Tuple_, align 4
   %unit_5 = alloca %struct.siko_Tuple_, align 4
   %call_4 = alloca %struct.siko_Tuple_, align 4
   %matchValue_13 = alloca %struct.siko_Tuple_, align 4
   %match_var_2 = alloca %struct.siko_Tuple_, align 4
   %valueRef_1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_1, ptr align 4 %v, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_13, ptr align 4 %match_var_2, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %matchValue_13, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_4 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_5 = load i32, ptr %tmp_switch_var_block2_4, align 4
   switch i32 %tmp_switch_var_block2_5, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %call_4)
   call void @siko_Tuple_(ptr %unit_5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %unit_5, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @siko_Tuple_(ptr %unit_9)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %unit_9, i64 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

define private void @Bool_Bool_False(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_False, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_False, ptr %this, i32 0, i32 0
   store volatile i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_False, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store volatile i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


