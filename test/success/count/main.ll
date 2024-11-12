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
   %call_54 = alloca %struct.Int_Int, align 8
   %valueRef_52 = alloca %struct.Int_Int, align 8
   %lit_51 = alloca %struct.Int_Int, align 8
   %valueRef_46 = alloca %struct.Int_Int, align 8
   %matchValue_58 = alloca %struct.Int_Int, align 8
   %unit_66 = alloca %struct.siko_Tuple_, align 4
   %call_65 = alloca %struct.siko_Tuple_, align 4
   %call_64 = alloca %struct.Bool_Bool, align 4
   %valueRef_62 = alloca %struct.Int_Int, align 8
   %lit_61 = alloca %struct.Int_Int, align 8
   %r_60 = alloca %struct.Int_Int, align 8
   %finalValueRef_36 = alloca %struct.Int_Int, align 8
   %match_var_44 = alloca %struct.Int_Int, align 8
   %call_43 = alloca %struct.Bool_Bool, align 4
   %valueRef_41 = alloca %struct.Int_Int, align 8
   %lit_40 = alloca %struct.Int_Int, align 8
   %a_38 = alloca %struct.Int_Int, align 8
   %call_21 = alloca %struct.Int_Int, align 8
   %valueRef_19 = alloca %struct.Int_Int, align 8
   %lit_18 = alloca %struct.Int_Int, align 8
   %valueRef_13 = alloca %struct.Int_Int, align 8
   %loopVar_35 = alloca %struct.Int_Int, align 8
   %lit_34 = alloca %struct.Int_Int, align 8
   %call_33 = alloca %struct.siko_Tuple_, align 4
   %call_32 = alloca %struct.Bool_Bool, align 4
   %valueRef_30 = alloca %struct.Int_Int, align 8
   %lit_29 = alloca %struct.Int_Int, align 8
   %r_28 = alloca %struct.Int_Int, align 8
   %finalValueRef_3 = alloca %struct.Int_Int, align 8
   %match_var_11 = alloca %struct.Int_Int, align 8
   %call_10 = alloca %struct.Bool_Bool, align 4
   %valueRef_8 = alloca %struct.Int_Int, align 8
   %lit_7 = alloca %struct.Int_Int, align 8
   %a_5 = alloca %struct.Int_Int, align 8
   %loopVar_2 = alloca %struct.Int_Int, align 8
   %lit_1 = alloca %struct.Int_Int, align 8
   %tmp_lit_1_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_1, i32 0, i32 0
   store i64 1, ptr %tmp_lit_1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loopVar_2, ptr align 8 %lit_1, i64 8, i1 false)
   br label %block1
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %a_5, ptr align 8 %loopVar_2, i64 8, i1 false)
   %tmp_lit_7_2 = getelementptr inbounds %struct.Int_Int, ptr %lit_7, i32 0, i32 0
   store i64 10, ptr %tmp_lit_7_2, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_8, ptr align 8 %a_5, i64 8, i1 false)
   call void @Int_Int_lessThan(ptr %valueRef_8, ptr %lit_7, ptr %call_10)
   br label %block4
block2:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %finalValueRef_3, ptr align 8 %loopVar_2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %r_28, ptr align 8 %finalValueRef_3, i64 8, i1 false)
   %tmp_lit_29_3 = getelementptr inbounds %struct.Int_Int, ptr %lit_29, i32 0, i32 0
   store i64 10, ptr %tmp_lit_29_3, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_30, ptr align 8 %r_28, i64 8, i1 false)
   call void @Int_Int_eq(ptr %valueRef_30, ptr %lit_29, ptr %call_32)
   call void @Std_Basic_Util_assert(ptr %call_32, ptr %call_33)
   %tmp_lit_34_4 = getelementptr inbounds %struct.Int_Int, ptr %lit_34, i32 0, i32 0
   store i64 1, ptr %tmp_lit_34_4, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loopVar_35, ptr align 8 %lit_34, i64 8, i1 false)
   br label %block9
block4:
   %tmp_switch_var_block4_5 = getelementptr inbounds %struct.Bool_Bool, ptr %call_10, i32 0, i32 0
   %tmp_switch_var_block4_6 = load i32, ptr %tmp_switch_var_block4_5, align 4
   switch i32 %tmp_switch_var_block4_6, label %block5 [
i32 1, label %block7
]

block5:
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_13, ptr align 8 %a_5, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loopVar_2, ptr align 8 %valueRef_13, i64 8, i1 false)
   br label %block2
block7:
   br label %block8
block8:
   %tmp_lit_18_7 = getelementptr inbounds %struct.Int_Int, ptr %lit_18, i32 0, i32 0
   store i64 1, ptr %tmp_lit_18_7, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_19, ptr align 8 %a_5, i64 8, i1 false)
   call void @Int_Int_add(ptr %valueRef_19, ptr %lit_18, ptr %call_21)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loopVar_2, ptr align 8 %call_21, i64 8, i1 false)
   br label %block1
block9:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %a_38, ptr align 8 %loopVar_35, i64 8, i1 false)
   %tmp_lit_40_8 = getelementptr inbounds %struct.Int_Int, ptr %lit_40, i32 0, i32 0
   store i64 10, ptr %tmp_lit_40_8, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_41, ptr align 8 %a_38, i64 8, i1 false)
   call void @Int_Int_lessThan(ptr %valueRef_41, ptr %lit_40, ptr %call_43)
   br label %block12
block10:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %finalValueRef_36, ptr align 8 %loopVar_35, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %r_60, ptr align 8 %finalValueRef_36, i64 8, i1 false)
   %tmp_lit_61_9 = getelementptr inbounds %struct.Int_Int, ptr %lit_61, i32 0, i32 0
   store i64 10, ptr %tmp_lit_61_9, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_62, ptr align 8 %r_60, i64 8, i1 false)
   call void @Int_Int_eq(ptr %valueRef_62, ptr %lit_61, ptr %call_64)
   call void @Std_Basic_Util_assert(ptr %call_64, ptr %call_65)
   call void @siko_Tuple_(ptr %unit_66)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_66, i64 0, i1 false)
   ret void
block11:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %matchValue_58, ptr align 8 %match_var_44, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loopVar_35, ptr align 8 %matchValue_58, i64 8, i1 false)
   br label %block9
block12:
   %tmp_switch_var_block12_10 = getelementptr inbounds %struct.Bool_Bool, ptr %call_43, i32 0, i32 0
   %tmp_switch_var_block12_11 = load i32, ptr %tmp_switch_var_block12_10, align 4
   switch i32 %tmp_switch_var_block12_11, label %block13 [
i32 1, label %block15
]

block13:
   br label %block14
block14:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_46, ptr align 8 %a_38, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %loopVar_35, ptr align 8 %valueRef_46, i64 8, i1 false)
   br label %block10
block15:
   br label %block16
block16:
   %tmp_lit_51_12 = getelementptr inbounds %struct.Int_Int, ptr %lit_51, i32 0, i32 0
   store i64 1, ptr %tmp_lit_51_12, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_52, ptr align 8 %a_38, i64 8, i1 false)
   call void @Int_Int_add(ptr %valueRef_52, ptr %lit_51, ptr %call_54)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_44, ptr align 8 %call_54, i64 8, i1 false)
   br label %block11
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
   %tmp_switch_var_block2_13 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_14 = load i32, ptr %tmp_switch_var_block2_13, align 4
   switch i32 %tmp_switch_var_block2_14, label %block3 [
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

declare void @Int_Int_add(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_lessThan(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


