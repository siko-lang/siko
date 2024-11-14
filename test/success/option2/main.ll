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
   %tuple_11 = alloca %struct.siko_Tuple_, align 4
   %a_10 = alloca %struct.Bool_Bool, align 4
   %call_5 = alloca %struct.siko_Tuple_, align 4
   %call_4 = alloca %struct.Bool_Bool, align 4
   %unit_16 = alloca %struct.siko_Tuple_, align 4
   %matchValue_15 = alloca %struct.siko_Tuple_, align 4
   %match_var_3 = alloca %struct.siko_Tuple_, align 4
   %call_2 = alloca %struct.Option_Option_Bool_Bool, align 4
   %call_1 = alloca %struct.Bool_Bool, align 4
   call void @Std_Basic_Util_getTrue(ptr %call_1)
   call void @Option_Option_Some_Bool_Bool(ptr %call_1, ptr %call_2)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_15, ptr align 4 %match_var_3, i64 0, i1 false)
   call void @siko_Tuple_(ptr %unit_16)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_16, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Option_Option_Bool_Bool, ptr %call_2, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Bool_Bool_False(ptr %call_4)
   call void @Std_Basic_Util_assert(ptr %call_4, ptr %call_5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_3, ptr align 4 %call_5, i64 0, i1 false)
   br label %block1
block5:
   %tmp_transform_8_3 = bitcast %struct.Option_Option_Bool_Bool* %call_2 to %struct.Option_Option_Some_Bool_Bool*
   %transform_8 = getelementptr inbounds %struct.Option_Option_Some_Bool_Bool, ptr %tmp_transform_8_3, i32 0, i32 1
   %tupleField_9 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_8, i32 0, i32 0
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_10, ptr align 4 %tupleField_9, i64 4, i1 false)
   call void @siko_Tuple_(ptr %tuple_11)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_3, ptr align 4 %tuple_11, i64 0, i1 false)
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

define private void @Std_Basic_Util_getTrue(ptr noundef %fn_result) {
block0:
   %call_1 = alloca %struct.Bool_Bool, align 4
   call void @Std_Basic_Util_siko_runtime_true(ptr %call_1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %call_1, i64 4, i1 false)
   ret void
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

declare void @Std_Basic_Util_siko_runtime_true(ptr noundef %fn_result)

define private void @Bool_Bool_False(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_False, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_False, ptr %this, i32 0, i32 0
   store volatile i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_False, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
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


