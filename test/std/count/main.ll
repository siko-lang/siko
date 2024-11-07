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
   %i_16_4 = alloca %struct.Int_Int, align 8
   %i_16_2 = alloca %struct.Int_Int, align 8
   %i_16_1 = alloca %struct.Int_Int, align 8
   %i_14_1 = alloca %struct.Int_Int, align 8
   %i_11_1 = alloca %struct.Int_Int, align 8
   %i_10_8 = alloca %struct.siko_Tuple_, align 4
   %i_10_7 = alloca %struct.siko_Tuple_, align 4
   %i_10_6 = alloca %struct.Bool_Bool, align 4
   %i_10_4 = alloca %struct.Int_Int, align 8
   %i_10_3 = alloca %struct.Int_Int, align 8
   %r_7 = alloca %struct.Int_Int, align 8
   %i_10_1 = alloca %struct.Int_Int, align 8
   %match_var_6 = alloca %struct.siko_Tuple_, align 4
   %i_9_6 = alloca %struct.Bool_Bool, align 4
   %i_9_4 = alloca %struct.Int_Int, align 8
   %i_9_3 = alloca %struct.Int_Int, align 8
   %a_5 = alloca %struct.Int_Int, align 8
   %i_9_1 = alloca %struct.Int_Int, align 8
   %i_8_4 = alloca %struct.Int_Int, align 8
   %i_8_2 = alloca %struct.Int_Int, align 8
   %i_8_1 = alloca %struct.Int_Int, align 8
   %i_6_1 = alloca %struct.Int_Int, align 8
   %loop_var_4 = alloca %struct.Int_Int, align 8
   %i_2_8 = alloca %struct.Int_Int, align 8
   %i_2_7 = alloca %struct.siko_Tuple_, align 4
   %i_2_6 = alloca %struct.Bool_Bool, align 4
   %i_2_4 = alloca %struct.Int_Int, align 8
   %i_2_3 = alloca %struct.Int_Int, align 8
   %r_3 = alloca %struct.Int_Int, align 8
   %i_2_1 = alloca %struct.Int_Int, align 8
   %match_var_2 = alloca %struct.siko_Tuple_, align 4
   %i_1_6 = alloca %struct.Bool_Bool, align 4
   %i_1_4 = alloca %struct.Int_Int, align 8
   %i_1_3 = alloca %struct.Int_Int, align 8
   %a_1 = alloca %struct.Int_Int, align 8
   %i_1_1 = alloca %struct.Int_Int, align 8
   %loop_var_0 = alloca %struct.Int_Int, align 8
   %i_0_1 = alloca %struct.Int_Int, align 8
   %tmp_i_0_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_0_1, i32 0, i32 0
   store i64 1, ptr %tmp_i_0_1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_0, ptr align 8 %i_0_1, i8 8, i1 false)
   br label %block1
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_1_1, ptr align 8 %loop_var_0, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %a_1, ptr align 8 %i_1_1, i8 8, i1 false)
   %tmp_i_1_3_1 = getelementptr inbounds %struct.Int_Int, ptr %i_1_3, i32 0, i32 0
   store i64 10, ptr %tmp_i_1_3_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_1_4, ptr align 8 %a_1, i8 8, i1 false)
   call void @Int_Int_lessThan(ptr %i_1_4, ptr %i_1_3, ptr %i_1_6)
   br label %block4
block2:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_2_1, ptr align 8 %loop_var_0, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %r_3, ptr align 8 %i_2_1, i8 8, i1 false)
   %tmp_i_2_3_1 = getelementptr inbounds %struct.Int_Int, ptr %i_2_3, i32 0, i32 0
   store i64 10, ptr %tmp_i_2_3_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_2_4, ptr align 8 %r_3, i8 8, i1 false)
   call void @Int_Int_eq(ptr %i_2_4, ptr %i_2_3, ptr %i_2_6)
   call void @Std_Basic_Util_assert(ptr %i_2_6, ptr %i_2_7)
   %tmp_i_2_8_1 = getelementptr inbounds %struct.Int_Int, ptr %i_2_8, i32 0, i32 0
   store i64 1, ptr %tmp_i_2_8_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_4, ptr align 8 %i_2_8, i8 8, i1 false)
   br label %block9
block4:
   %tmp_switch_var_block4_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_1_6, i32 0, i32 0
   %tmp_switch_var_block4_2 = load i32, ptr %tmp_switch_var_block4_1, align 4
   switch i32 %tmp_switch_var_block4_2, label %block5 [
i32 1, label %block7
]

block5:
   %i_5_1 = bitcast %struct.Bool_Bool* %i_1_6 to %struct.siko_Tuple_*
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_6_1, ptr align 8 %a_1, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_0, ptr align 8 %i_6_1, i8 8, i1 false)
   br label %block2
block7:
   %i_7_1 = bitcast %struct.Bool_Bool* %i_1_6 to %struct.siko_Tuple_*
   br label %block8
block8:
   %tmp_i_8_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_8_1, i32 0, i32 0
   store i64 1, ptr %tmp_i_8_1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_8_2, ptr align 8 %a_1, i8 8, i1 false)
   call void @Int_Int_add(ptr %i_8_2, ptr %i_8_1, ptr %i_8_4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_0, ptr align 8 %i_8_4, i8 8, i1 false)
   br label %block1
block9:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_9_1, ptr align 8 %loop_var_4, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %a_5, ptr align 8 %i_9_1, i8 8, i1 false)
   %tmp_i_9_3_1 = getelementptr inbounds %struct.Int_Int, ptr %i_9_3, i32 0, i32 0
   store i64 10, ptr %tmp_i_9_3_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_9_4, ptr align 8 %a_5, i8 8, i1 false)
   call void @Int_Int_lessThan(ptr %i_9_4, ptr %i_9_3, ptr %i_9_6)
   br label %block12
block10:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_10_1, ptr align 8 %loop_var_4, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %r_7, ptr align 8 %i_10_1, i8 8, i1 false)
   %tmp_i_10_3_1 = getelementptr inbounds %struct.Int_Int, ptr %i_10_3, i32 0, i32 0
   store i64 10, ptr %tmp_i_10_3_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_10_4, ptr align 8 %r_7, i8 8, i1 false)
   call void @Int_Int_eq(ptr %i_10_4, ptr %i_10_3, ptr %i_10_6)
   call void @Std_Basic_Util_assert(ptr %i_10_6, ptr %i_10_7)
   call void @siko_Tuple_(ptr %i_10_8)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %i_10_8, i8 0, i1 false)
   ret void
block11:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_11_1, ptr align 8 %match_var_6, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_4, ptr align 8 %i_11_1, i8 8, i1 false)
   br label %block9
block12:
   %tmp_switch_var_block12_1 = getelementptr inbounds %struct.Bool_Bool, ptr %i_9_6, i32 0, i32 0
   %tmp_switch_var_block12_2 = load i32, ptr %tmp_switch_var_block12_1, align 4
   switch i32 %tmp_switch_var_block12_2, label %block13 [
i32 1, label %block15
]

block13:
   %i_13_1 = bitcast %struct.Bool_Bool* %i_9_6 to %struct.siko_Tuple_*
   br label %block14
block14:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_14_1, ptr align 8 %a_5, i8 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loop_var_4, ptr align 8 %i_14_1, i8 8, i1 false)
   br label %block10
block15:
   %i_15_1 = bitcast %struct.Bool_Bool* %i_9_6 to %struct.siko_Tuple_*
   br label %block16
block16:
   %tmp_i_16_1_1 = getelementptr inbounds %struct.Int_Int, ptr %i_16_1, i32 0, i32 0
   store i64 1, ptr %tmp_i_16_1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %i_16_2, ptr align 8 %a_5, i8 8, i1 false)
   call void @Int_Int_add(ptr %i_16_2, ptr %i_16_1, ptr %i_16_4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_6, ptr align 8 %i_16_4, i8 8, i1 false)
   br label %block11
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

declare void @Int_Int_add(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_lessThan(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


