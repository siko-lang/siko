@.str_0 = private unnamed_addr constant [3 x i8] c"foo", align 1
@.str_1 = private unnamed_addr constant [6 x i8] c"notfoo", align 1
%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.String_String = type { ptr, i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private ptr @Main_foo(ptr noundef %s) {
block0:
   %valueRef_13 = alloca ptr, align 8
   %call_12 = alloca %struct.siko_Tuple_, align 4
   %call_11 = alloca %struct.Bool_Bool, align 4
   %call_9 = alloca %struct.Bool_Bool, align 4
   %valueRef_7 = alloca ptr, align 8
   %implicitRef1 = alloca ptr, align 8
   %literal_6 = alloca %struct.String_String, align 8
   %call_5 = alloca %struct.siko_Tuple_, align 4
   %call_4 = alloca %struct.Bool_Bool, align 4
   %valueRef_2 = alloca ptr, align 8
   %implicitRef0 = alloca ptr, align 8
   %literal_1 = alloca %struct.String_String, align 8
   %tmp_literal_1_1 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_literal_1_1, align 8
   %tmp_literal_1_2 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 1
   store i64 3, ptr %tmp_literal_1_2, align 8
   store ptr %literal_1, ptr %implicitRef0, align 8
   store ptr %s, ptr %valueRef_2, align 8
   %tmp_valueRef_2_3 = load ptr, ptr %valueRef_2, align 8
   %tmp_implicitRef0_4 = load ptr, ptr %implicitRef0, align 8
   call void @String_String_eq(ptr %tmp_valueRef_2_3, ptr %tmp_implicitRef0_4, ptr %call_4)
   call void @Std_Basic_Util_assert(ptr %call_4, ptr %call_5)
   %tmp_literal_6_5 = getelementptr inbounds %struct.String_String, ptr %literal_6, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_literal_6_5, align 8
   %tmp_literal_6_6 = getelementptr inbounds %struct.String_String, ptr %literal_6, i32 0, i32 1
   store i64 6, ptr %tmp_literal_6_6, align 8
   store ptr %literal_6, ptr %implicitRef1, align 8
   store ptr %s, ptr %valueRef_7, align 8
   %tmp_valueRef_7_7 = load ptr, ptr %valueRef_7, align 8
   %tmp_implicitRef1_8 = load ptr, ptr %implicitRef1, align 8
   call void @String_String_eq(ptr %tmp_valueRef_7_7, ptr %tmp_implicitRef1_8, ptr %call_9)
   call void @Bool_Bool_not(ptr %call_9, ptr %call_11)
   call void @Std_Basic_Util_assert(ptr %call_11, ptr %call_12)
   store ptr %s, ptr %valueRef_13, align 8
   %tmp_valueRef_13_9 = load ptr, ptr %valueRef_13, align 8
   ret ptr %tmp_valueRef_13_9
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %unit_31 = alloca %struct.siko_Tuple_, align 4
   %call_30 = alloca %struct.siko_Tuple_, align 4
   %call_29 = alloca %struct.Bool_Bool, align 4
   %implicitRef1 = alloca ptr, align 8
   %valueRef_27 = alloca %struct.String_String, align 8
   %valueRef_26 = alloca ptr, align 8
   %same_25 = alloca ptr, align 8
   %call_24 = alloca ptr, align 8
   %ref_20 = alloca ptr, align 8
   %valueRef_19 = alloca %struct.String_String, align 8
   %call_18 = alloca ptr, align 8
   %ref_17 = alloca ptr, align 8
   %valueRef_16 = alloca %struct.String_String, align 8
   %call_15 = alloca ptr, align 8
   %implicitRef0 = alloca ptr, align 8
   %valueRef_14 = alloca %struct.String_String, align 8
   %ref2_13 = alloca ptr, align 8
   %valueRef_11 = alloca ptr, align 8
   %ref_10 = alloca ptr, align 8
   %ref_4 = alloca ptr, align 8
   %valueRef_3 = alloca %struct.String_String, align 8
   %s_2 = alloca %struct.String_String, align 8
   %literal_1 = alloca %struct.String_String, align 8
   %tmp_literal_1_10 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_literal_1_10, align 8
   %tmp_literal_1_11 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 1
   store i64 3, ptr %tmp_literal_1_11, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %s_2, ptr align 8 %literal_1, i64 16, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_3, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_3, ptr %ref_4, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %ref_10, ptr align 8 %ref_4, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_11, ptr align 8 %ref_10, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %ref2_13, ptr align 8 %valueRef_11, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_14, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_14, ptr %implicitRef0, align 8
   %tmp_implicitRef0_12 = load ptr, ptr %implicitRef0, align 8
   %tmp_call_15_13 = call ptr @Main_foo(ptr %tmp_implicitRef0_12)
   store ptr %tmp_call_15_13, ptr %call_15, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_16, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_16, ptr %ref_17, align 8
   %tmp_ref_17_14 = load ptr, ptr %ref_17, align 8
   %tmp_call_18_15 = call ptr @Main_foo(ptr %tmp_ref_17_14)
   store ptr %tmp_call_18_15, ptr %call_18, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_19, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_19, ptr %ref_20, align 8
   %tmp_ref_20_16 = load ptr, ptr %ref_20, align 8
   %tmp_call_24_17 = call ptr @Main_foo(ptr %tmp_ref_20_16)
   store ptr %tmp_call_24_17, ptr %call_24, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %same_25, ptr align 8 %call_24, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_26, ptr align 8 %same_25, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_27, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_27, ptr %implicitRef1, align 8
   %tmp_implicitRef1_18 = load ptr, ptr %implicitRef1, align 8
   %tmp_valueRef_26_19 = load ptr, ptr %valueRef_26, align 8
   call void @String_String_eq(ptr %tmp_implicitRef1_18, ptr %tmp_valueRef_26_19, ptr %call_29)
   call void @Std_Basic_Util_assert(ptr %call_29, ptr %call_30)
   call void @siko_Tuple_(ptr %unit_31)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_31, i64 0, i1 false)
   ret void
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
   %tmp_switch_var_block2_20 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_21 = load i32, ptr %tmp_switch_var_block2_20, align 4
   switch i32 %tmp_switch_var_block2_21, label %block3 [
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

define private void @Bool_Bool_not(ptr noundef %self, ptr noundef %fn_result) {
block0:
   %call_6 = alloca %struct.Bool_Bool, align 4
   %call_3 = alloca %struct.Bool_Bool, align 4
   %matchValue_10 = alloca %struct.Bool_Bool, align 4
   %match_var_2 = alloca %struct.Bool_Bool, align 4
   %valueRef_1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_1, ptr align 4 %self, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_10, ptr align 4 %match_var_2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %matchValue_10, i64 4, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_22 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_23 = load i32, ptr %tmp_switch_var_block2_22, align 4
   switch i32 %tmp_switch_var_block2_23, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Bool_Bool_True(ptr %call_3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_3, i64 4, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @Bool_Bool_False(ptr %call_6)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_6, i64 4, i1 false)
   br label %block1
}

declare void @String_String_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


