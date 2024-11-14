%struct.Bool_Bool = type { i32, [0 x i32] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Option_Option_Bool_Bool = type { i32, [1 x i32] }

%struct.Option_Option_None_Bool_Bool = type { i32, %struct.siko_Tuple_ }

%struct.Option_Option_Some_Bool_Bool = type { i32, %struct.siko_Tuple_Bool_Bool }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Bool_Bool = type { %struct.Bool_Bool }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %call_62 = alloca %struct.siko_Tuple_, align 4
   %valueRef_61 = alloca %struct.Bool_Bool, align 4
   %a_60 = alloca %struct.Bool_Bool, align 4
   %call_55 = alloca %struct.siko_Tuple_, align 4
   %call_54 = alloca %struct.Bool_Bool, align 4
   %unit_67 = alloca %struct.siko_Tuple_, align 4
   %matchValue_66 = alloca %struct.siko_Tuple_, align 4
   %call_45 = alloca %struct.siko_Tuple_, align 4
   %valueRef_44 = alloca %struct.Bool_Bool, align 4
   %a_43 = alloca %struct.Bool_Bool, align 4
   %call_38 = alloca %struct.siko_Tuple_, align 4
   %call_37 = alloca %struct.Bool_Bool, align 4
   %match_var_53 = alloca %struct.siko_Tuple_, align 4
   %valueRef_52 = alloca %struct.Option_Option_Bool_Bool, align 4
   %a_51 = alloca %struct.Option_Option_Bool_Bool, align 4
   %call_50 = alloca %struct.Option_Option_Bool_Bool, align 4
   %matchValue_49 = alloca %struct.siko_Tuple_, align 4
   %valueRef_27 = alloca %struct.Bool_Bool, align 4
   %a_26 = alloca %struct.Bool_Bool, align 4
   %call_21 = alloca %struct.Bool_Bool, align 4
   %match_var_36 = alloca %struct.siko_Tuple_, align 4
   %valueRef_35 = alloca %struct.Option_Option_Bool_Bool, align 4
   %a_34 = alloca %struct.Option_Option_Bool_Bool, align 4
   %call_33 = alloca %struct.Option_Option_Bool_Bool, align 4
   %call_32 = alloca %struct.Bool_Bool, align 4
   %matchValue_31 = alloca %struct.Bool_Bool, align 4
   %valueRef_12 = alloca %struct.Bool_Bool, align 4
   %a_11 = alloca %struct.Bool_Bool, align 4
   %call_6 = alloca %struct.Bool_Bool, align 4
   %match_var_20 = alloca %struct.Bool_Bool, align 4
   %valueRef_19 = alloca %struct.Option_Option_Bool_Bool, align 4
   %a_18 = alloca %struct.Option_Option_Bool_Bool, align 4
   %call_17 = alloca %struct.Option_Option_Bool_Bool, align 4
   %matchValue_16 = alloca %struct.Bool_Bool, align 4
   %match_var_5 = alloca %struct.Bool_Bool, align 4
   %valueRef_4 = alloca %struct.Option_Option_Bool_Bool, align 4
   %a_3 = alloca %struct.Option_Option_Bool_Bool, align 4
   %call_2 = alloca %struct.Option_Option_Bool_Bool, align 4
   %call_1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %call_1)
   call void @Option_Option_Some_Bool_Bool(ptr %call_1, ptr %call_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_3, ptr align 4 %call_2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_4, ptr align 4 %a_3, i64 8, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_16, ptr align 4 %match_var_5, i64 4, i1 false)
   call void @Option_Option_None_Bool_Bool(ptr %call_17)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_18, ptr align 4 %call_17, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_19, ptr align 4 %a_18, i64 8, i1 false)
   br label %block8
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %valueRef_4, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Bool_Bool_False(ptr %call_6)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_5, ptr align 4 %call_6, i64 4, i1 false)
   br label %block1
block5:
   %tmp_transform_9_3 = bitcast %struct.Option_Option_Bool_Bool* %valueRef_4 to %struct.Option_Option_Some_Bool_Bool*
   %transform_9 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_transform_9_3, i32 0, i32 1
   %tupleField_10 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_9, i32 0, i32 0
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_11, ptr align 4 %tupleField_10, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_12, ptr align 4 %a_11, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_5, ptr align 4 %valueRef_12, i64 4, i1 false)
   br label %block1
block7:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_31, ptr align 4 %match_var_20, i64 4, i1 false)
   call void @Bool_Bool_True(ptr %call_32)
   call void @Option_Option_Some_Bool_Bool(ptr %call_32, ptr %call_33)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_34, ptr align 4 %call_33, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_35, ptr align 4 %a_34, i64 8, i1 false)
   br label %block14
block8:
   %tmp_switch_var_block8_4 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %valueRef_19, i32 0, i32 0
   %tmp_switch_var_block8_5 = load i32, ptr %tmp_switch_var_block8_4, align 4
   switch i32 %tmp_switch_var_block8_5, label %block9 [
i32 1, label %block11
]

block9:
   br label %block10
block10:
   call void @Bool_Bool_False(ptr %call_21)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_20, ptr align 4 %call_21, i64 4, i1 false)
   br label %block7
block11:
   %tmp_transform_24_6 = bitcast %struct.Option_Option_Bool_Bool* %valueRef_19 to %struct.Option_Option_Some_Bool_Bool*
   %transform_24 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_transform_24_6, i32 0, i32 1
   %tupleField_25 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_24, i32 0, i32 0
   br label %block12
block12:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_26, ptr align 4 %tupleField_25, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_27, ptr align 4 %a_26, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_20, ptr align 4 %valueRef_27, i64 4, i1 false)
   br label %block7
block13:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_49, ptr align 4 %match_var_36, i64 0, i1 false)
   call void @Option_Option_None_Bool_Bool(ptr %call_50)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_51, ptr align 4 %call_50, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_52, ptr align 4 %a_51, i64 8, i1 false)
   br label %block20
block14:
   %tmp_switch_var_block14_7 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %valueRef_35, i32 0, i32 0
   %tmp_switch_var_block14_8 = load i32, ptr %tmp_switch_var_block14_7, align 4
   switch i32 %tmp_switch_var_block14_8, label %block15 [
i32 1, label %block17
]

block15:
   br label %block16
block16:
   call void @Bool_Bool_False(ptr %call_37)
   call void @Std_Basic_Util_assert(ptr %call_37, ptr %call_38)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_36, ptr align 4 %call_38, i64 0, i1 false)
   br label %block13
block17:
   %tmp_transform_41_9 = bitcast %struct.Option_Option_Bool_Bool* %valueRef_35 to %struct.Option_Option_Some_Bool_Bool*
   %transform_41 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_transform_41_9, i32 0, i32 1
   %tupleField_42 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_41, i32 0, i32 0
   br label %block18
block18:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_43, ptr align 4 %tupleField_42, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_44, ptr align 4 %a_43, i64 4, i1 false)
   call void @Std_Basic_Util_assert(ptr %valueRef_44, ptr %call_45)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_36, ptr align 4 %call_45, i64 0, i1 false)
   br label %block13
block19:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_66, ptr align 4 %match_var_53, i64 0, i1 false)
   call void @siko_Tuple_(ptr %unit_67)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_67, i64 0, i1 false)
   ret void
block20:
   %tmp_switch_var_block20_10 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %valueRef_52, i32 0, i32 0
   %tmp_switch_var_block20_11 = load i32, ptr %tmp_switch_var_block20_10, align 4
   switch i32 %tmp_switch_var_block20_11, label %block21 [
i32 1, label %block23
]

block21:
   br label %block22
block22:
   call void @Bool_Bool_True(ptr %call_54)
   call void @Std_Basic_Util_assert(ptr %call_54, ptr %call_55)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_53, ptr align 4 %call_55, i64 0, i1 false)
   br label %block19
block23:
   %tmp_transform_58_12 = bitcast %struct.Option_Option_Bool_Bool* %valueRef_52 to %struct.Option_Option_Some_Bool_Bool*
   %transform_58 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_transform_58_12, i32 0, i32 1
   %tupleField_59 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_58, i32 0, i32 0
   br label %block24
block24:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_60, ptr align 4 %tupleField_59, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_61, ptr align 4 %a_60, i64 4, i1 false)
   call void @Std_Basic_Util_assert(ptr %valueRef_61, ptr %call_62)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_53, ptr align 4 %call_62, i64 0, i1 false)
   br label %block19
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

define private void @Option_Option_None_Bool_Bool(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Option_Option_None_Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Option_Option_None_Bool_Bool, ptr %this, i32 0, i32 0
   store volatile i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Option_Option_None_Bool_Bool, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define private void @Option_Option_Some_Bool_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Option_Option_Some_Bool_Bool, align 4
   %tag = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %this, i32 0, i32 0
   store volatile i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


