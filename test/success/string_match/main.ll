@.str_1 = private unnamed_addr constant [3 x i8] c"bar", align 1
@.str_0 = private unnamed_addr constant [3 x i8] c"foo", align 1
@.str_2 = private unnamed_addr constant [4 x i8] c"zorp", align 1
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

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %call_19 = alloca %struct.siko_Tuple_, align 4
   %call_18 = alloca %struct.Bool_Bool, align 4
   %call_14 = alloca %struct.siko_Tuple_, align 4
   %call_13 = alloca %struct.Bool_Bool, align 4
   %call_9 = alloca %struct.siko_Tuple_, align 4
   %call_8 = alloca %struct.Bool_Bool, align 4
   %call_4 = alloca %struct.siko_Tuple_, align 4
   %call_3 = alloca %struct.Bool_Bool, align 4
   %eq_17 = alloca %struct.Bool_Bool, align 4
   %implicitRef3 = alloca ptr, align 8
   %lit_16 = alloca %struct.String_String, align 8
   %eq_12 = alloca %struct.Bool_Bool, align 4
   %implicitRef2 = alloca ptr, align 8
   %lit_11 = alloca %struct.String_String, align 8
   %eq_7 = alloca %struct.Bool_Bool, align 4
   %implicitRef1 = alloca ptr, align 8
   %lit_6 = alloca %struct.String_String, align 8
   %matchValue_22 = alloca %struct.siko_Tuple_, align 4
   %match_var_2 = alloca %struct.siko_Tuple_, align 4
   %implicitRef0 = alloca ptr, align 8
   %literal_1 = alloca %struct.String_String, align 8
   %tmp_literal_1_1 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_literal_1_1, align 8
   %tmp_literal_1_2 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 1
   store i64 3, ptr %tmp_literal_1_2, align 8
   store ptr %literal_1, ptr %implicitRef0, align 8
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_22, ptr align 4 %match_var_2, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %matchValue_22, i64 0, i1 false)
   ret void
block2:
   %tmp_lit_6_3 = getelementptr inbounds %struct.String_String, ptr %lit_6, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_lit_6_3, align 8
   %tmp_lit_6_4 = getelementptr inbounds %struct.String_String, ptr %lit_6, i32 0, i32 1
   store i64 3, ptr %tmp_lit_6_4, align 8
   store ptr %lit_6, ptr %implicitRef1, align 8
   %tmp_implicitRef0_5 = load ptr, ptr %implicitRef0, align 8
   %tmp_implicitRef1_6 = load ptr, ptr %implicitRef1, align 8
   call void @String_String_eq(ptr %tmp_implicitRef0_5, ptr %tmp_implicitRef1_6, ptr %eq_7)
   %tmp_switch_var_block2_7 = getelementptr inbounds %struct.Bool_Bool, ptr %eq_7, i32 0, i32 0
   %tmp_switch_var_block2_8 = load i32, ptr %tmp_switch_var_block2_7, align 4
   switch i32 %tmp_switch_var_block2_8, label %block3 [
i32 1, label %block6
]

block3:
   %tmp_lit_11_9 = getelementptr inbounds %struct.String_String, ptr %lit_11, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_lit_11_9, align 8
   %tmp_lit_11_10 = getelementptr inbounds %struct.String_String, ptr %lit_11, i32 0, i32 1
   store i64 3, ptr %tmp_lit_11_10, align 8
   store ptr %lit_11, ptr %implicitRef2, align 8
   %tmp_implicitRef0_11 = load ptr, ptr %implicitRef0, align 8
   %tmp_implicitRef2_12 = load ptr, ptr %implicitRef2, align 8
   call void @String_String_eq(ptr %tmp_implicitRef0_11, ptr %tmp_implicitRef2_12, ptr %eq_12)
   %tmp_switch_var_block3_13 = getelementptr inbounds %struct.Bool_Bool, ptr %eq_12, i32 0, i32 0
   %tmp_switch_var_block3_14 = load i32, ptr %tmp_switch_var_block3_13, align 4
   switch i32 %tmp_switch_var_block3_14, label %block4 [
i32 1, label %block7
]

block4:
   %tmp_lit_16_15 = getelementptr inbounds %struct.String_String, ptr %lit_16, i32 0, i32 0
   store ptr @.str_2, ptr %tmp_lit_16_15, align 8
   %tmp_lit_16_16 = getelementptr inbounds %struct.String_String, ptr %lit_16, i32 0, i32 1
   store i64 4, ptr %tmp_lit_16_16, align 8
   store ptr %lit_16, ptr %implicitRef3, align 8
   %tmp_implicitRef0_17 = load ptr, ptr %implicitRef0, align 8
   %tmp_implicitRef3_18 = load ptr, ptr %implicitRef3, align 8
   call void @String_String_eq(ptr %tmp_implicitRef0_17, ptr %tmp_implicitRef3_18, ptr %eq_17)
   %tmp_switch_var_block4_19 = getelementptr inbounds %struct.Bool_Bool, ptr %eq_17, i32 0, i32 0
   %tmp_switch_var_block4_20 = load i32, ptr %tmp_switch_var_block4_19, align 4
   switch i32 %tmp_switch_var_block4_20, label %block5 [
i32 1, label %block8
]

block5:
   call void @Bool_Bool_False(ptr %call_3)
   call void @Std_Basic_Util_assert(ptr %call_3, ptr %call_4)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_4, i64 0, i1 false)
   br label %block1
block6:
   call void @Bool_Bool_False(ptr %call_8)
   call void @Std_Basic_Util_assert(ptr %call_8, ptr %call_9)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_9, i64 0, i1 false)
   br label %block1
block7:
   call void @Bool_Bool_True(ptr %call_13)
   call void @Std_Basic_Util_assert(ptr %call_13, ptr %call_14)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_14, i64 0, i1 false)
   br label %block1
block8:
   call void @Bool_Bool_False(ptr %call_18)
   call void @Std_Basic_Util_assert(ptr %call_18, ptr %call_19)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_2, ptr align 4 %call_19, i64 0, i1 false)
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
   %tmp_switch_var_block2_21 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_22 = load i32, ptr %tmp_switch_var_block2_21, align 4
   switch i32 %tmp_switch_var_block2_22, label %block3 [
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

declare void @String_String_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


