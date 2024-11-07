%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type {  }

%struct.Bool_Bool_True = type {  }

%struct.Int_Int = type { i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %i_6_2 = alloca %struct.siko_Tuple_, align 4
   %i_6_1 = alloca %struct.Bool_Bool, align 4
   %i_5_2 = alloca %struct.siko_Tuple_, align 4
   %i_5_1 = alloca %struct.Bool_Bool, align 4
   %i_4_2 = alloca %struct.siko_Tuple_, align 4
   %i_4_1 = alloca %struct.Bool_Bool, align 4
   %i_3_2 = alloca %struct.siko_Tuple_, align 4
   %i_3_1 = alloca %struct.Bool_Bool, align 4
   %i_1_1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %i_0_1 = alloca %struct.Int_Int, align 8
   %tmp_i_0_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_1, i32 0, i32 0
   store i64 5, ptr %tmp_i_0_1_1, align 8
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %match_var_0, i8 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_1_1, i8 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i64, ptr %tmp_switch_var_block2_1, align 8
   switch i64 %tmp_switch_var_block2_2, label %block6 [
i64 3, label %block3
i64 4, label %block4
i64 5, label %block5
]

block3:
   call void @Bool_Bool_False(ptr %i_3_1)
   call void @Std_Basic_Util_assert(ptr %i_3_1, ptr %i_3_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_3_2, i8 0, i1 false)
   br label %block1
block4:
   call void @Bool_Bool_False(ptr %i_4_1)
   call void @Std_Basic_Util_assert(ptr %i_4_1, ptr %i_4_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_4_2, i8 0, i1 false)
   br label %block1
block5:
   call void @Bool_Bool_True(ptr %i_5_1)
   call void @Std_Basic_Util_assert(ptr %i_5_1, ptr %i_5_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_5_2, i8 0, i1 false)
   br label %block1
block6:
   call void @Bool_Bool_False(ptr %i_6_1)
   call void @Std_Basic_Util_assert(ptr %i_6_1, ptr %i_6_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_6_2, i8 0, i1 false)
   br label %block1
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %i_6_1 = alloca %struct.siko_Tuple_, align 4
   %i_4_2 = alloca %struct.siko_Tuple_, align 4
   %i_4_1 = alloca %struct.siko_Tuple_, align 4
   %i_1_1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %i_0_1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_0_1, ptr align 4 %v, i8 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %i_1_1, ptr align 4 %match_var_0, i8 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_1_1, i8 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_0_1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   %i_3_1 = bitcast %struct.Bool_Bool* %i_0_1 to %struct.siko_Tuple_*
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %i_4_1)
   call void @siko_Tuple_(ptr %i_4_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_4_2, i8 0, i1 false)
   br label %block1
block5:
   %i_5_1 = bitcast %struct.Bool_Bool* %i_0_1 to %struct.siko_Tuple_*
   br label %block6
block6:
   call void @siko_Tuple_(ptr %i_6_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %i_6_1, i8 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

define private void @Bool_Bool_False(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Bool_Bool_False*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool, ptr %this, i32 0, i32 1
   %payload2 = bitcast i8* %payload1 to %struct.Bool_Bool_True*
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i8 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


