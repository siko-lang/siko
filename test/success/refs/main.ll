@.str_0 = private unnamed_addr constant [3 x i8] c"foo", align 1
@.str_1 = private unnamed_addr constant [6 x i8] c"notfoo", align 1
%struct.Bool_Bool = type { i32, [0 x i32] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Main_Object = type { ptr }

%struct.Option_Option_None_String_String = type { i32, %struct.siko_Tuple_ }

%struct.Option_Option_Some_String_String = type { i32, %struct.siko_Tuple_String_String }

%struct.Option_Option_String_String = type { i32, [2 x i64] }

%struct.String_String = type { ptr, i64 }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_String_String = type { %struct.String_String }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_String_String(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_String_String, align 8
   %field0 = getelementptr inbounds %struct.siko_Tuple_String_String, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field0, ptr align 8 %f0, i64 16, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 16, i1 false)
   ret void
}

define private void @Main_Object(ptr noundef %s, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Object, align 8
   %field0 = getelementptr inbounds %struct.Main_Object, ptr %this, i32 0, i32 0
   store ptr %s, ptr %field0, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 8, i1 false)
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
   store volatile i64 3, ptr %tmp_literal_1_2, align 8
   store ptr %literal_1, ptr %implicitRef0, align 8
   store ptr %s, ptr %valueRef_2, align 8
   %tmp_valueRef_2_3 = load ptr, ptr %valueRef_2, align 8
   %tmp_implicitRef0_4 = load ptr, ptr %implicitRef0, align 8
   call void @String_String_eq(ptr %tmp_valueRef_2_3, ptr %tmp_implicitRef0_4, ptr %call_4)
   call void @Std_Basic_Util_assert(ptr %call_4, ptr %call_5)
   %tmp_literal_6_5 = getelementptr inbounds %struct.String_String, ptr %literal_6, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_literal_6_5, align 8
   %tmp_literal_6_6 = getelementptr inbounds %struct.String_String, ptr %literal_6, i32 0, i32 1
   store volatile i64 6, ptr %tmp_literal_6_6, align 8
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
   %unit_59 = alloca %struct.siko_Tuple_, align 4
   %call_58 = alloca %struct.siko_Tuple_, align 4
   %call_57 = alloca %struct.Bool_Bool, align 4
   %implicitRef5 = alloca ptr, align 8
   %valueRef_55 = alloca %struct.String_String, align 8
   %implicitRef4 = alloca ptr, align 8
   %valueRef_54 = alloca %struct.String_String, align 8
   %s2_52 = alloca %struct.String_String, align 8
   %call_47 = alloca %struct.siko_Tuple_, align 4
   %call_46 = alloca %struct.Bool_Bool, align 4
   %unit_64 = alloca %struct.siko_Tuple_, align 4
   %matchValue_63 = alloca %struct.siko_Tuple_, align 4
   %match_var_44 = alloca %struct.siko_Tuple_, align 4
   %valueRef_43 = alloca %struct.Option_Option_String_String, align 8
   %o2_42 = alloca %struct.Option_Option_String_String, align 8
   %call_41 = alloca %struct.Option_Option_String_String, align 8
   %valueRef_40 = alloca %struct.String_String, align 8
   %call_39 = alloca %struct.siko_Tuple_, align 4
   %call_38 = alloca %struct.Bool_Bool, align 4
   %implicitRef3 = alloca ptr, align 8
   %valueRef_36 = alloca %struct.String_String, align 8
   %valueRef_34 = alloca %struct.Main_Object, align 8
   %o_33 = alloca %struct.Main_Object, align 8
   %call_32 = alloca %struct.Main_Object, align 8
   %implicitRef2 = alloca ptr, align 8
   %valueRef_31 = alloca %struct.String_String, align 8
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
   store volatile i64 3, ptr %tmp_literal_1_11, align 8
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
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_31, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_31, ptr %implicitRef2, align 8
   %tmp_implicitRef2_20 = load ptr, ptr %implicitRef2, align 8
   call void @Main_store(ptr %tmp_implicitRef2_20, ptr %call_32)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %o_33, ptr align 8 %call_32, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_34, ptr align 8 %o_33, i64 8, i1 false)
   %fieldRef_35 = getelementptr inbounds %struct.Main_Object, ptr %valueRef_34, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_36, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_36, ptr %implicitRef3, align 8
   %tmp_implicitRef3_21 = load ptr, ptr %implicitRef3, align 8
   %tmp_fieldRef_35_22 = load ptr, ptr %fieldRef_35, align 8
   call void @String_String_eq(ptr %tmp_implicitRef3_21, ptr %tmp_fieldRef_35_22, ptr %call_38)
   call void @Std_Basic_Util_assert(ptr %call_38, ptr %call_39)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_40, ptr align 8 %s_2, i64 16, i1 false)
   call void @Main_store2(ptr %valueRef_40, ptr %call_41)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %o2_42, ptr align 8 %call_41, i64 24, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_43, ptr align 8 %o2_42, i64 24, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %matchValue_63, ptr align 4 %match_var_44, i64 0, i1 false)
   call void @siko_Tuple_(ptr %unit_64)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_64, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_23 = getelementptr inbounds %struct.Option_Option_String_String, ptr %valueRef_43, i32 0, i32 0
   %tmp_switch_var_block2_24 = load i32, ptr %tmp_switch_var_block2_23, align 4
   switch i32 %tmp_switch_var_block2_24, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Bool_Bool_False(ptr %call_46)
   call void @Std_Basic_Util_assert(ptr %call_46, ptr %call_47)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_44, ptr align 4 %call_47, i64 0, i1 false)
   br label %block1
block5:
   %tmp_transform_50_25 = bitcast %struct.Option_Option_String_String* %valueRef_43 to %struct.Option_Option_Some_String_String*
   %transform_50 = getelementptr inbounds %struct.Option_Option_Some_String_String, ptr %tmp_transform_50_25, i32 0, i32 1
   %tupleField_51 = getelementptr inbounds %struct.siko_Tuple_String_String, ptr %transform_50, i32 0, i32 0
   br label %block6
block6:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %s2_52, ptr align 8 %tupleField_51, i64 16, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_54, ptr align 8 %s2_52, i64 16, i1 false)
   store ptr %valueRef_54, ptr %implicitRef4, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_55, ptr align 8 %s_2, i64 16, i1 false)
   store ptr %valueRef_55, ptr %implicitRef5, align 8
   %tmp_implicitRef5_26 = load ptr, ptr %implicitRef5, align 8
   %tmp_implicitRef4_27 = load ptr, ptr %implicitRef4, align 8
   call void @String_String_eq(ptr %tmp_implicitRef5_26, ptr %tmp_implicitRef4_27, ptr %call_57)
   call void @Std_Basic_Util_assert(ptr %call_57, ptr %call_58)
   call void @siko_Tuple_(ptr %unit_59)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_44, ptr align 4 %unit_59, i64 0, i1 false)
   br label %block1
}

define private void @Main_store(ptr noundef %s, ptr noundef %fn_result) {
block0:
   %valueRef_4 = alloca %struct.Main_Object, align 8
   %o_3 = alloca %struct.Main_Object, align 8
   %call_2 = alloca %struct.Main_Object, align 8
   %valueRef_1 = alloca ptr, align 8
   store ptr %s, ptr %valueRef_1, align 8
   %tmp_valueRef_1_28 = load ptr, ptr %valueRef_1, align 8
   call void @Main_Object(ptr %tmp_valueRef_1_28, ptr %call_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %o_3, ptr align 8 %call_2, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_4, ptr align 8 %o_3, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %valueRef_4, i64 8, i1 false)
   ret void
}

define private void @Main_store2(ptr noundef %s, ptr noundef %fn_result) {
block0:
   %valueRef_4 = alloca %struct.Option_Option_String_String, align 8
   %o_3 = alloca %struct.Option_Option_String_String, align 8
   %call_2 = alloca %struct.Option_Option_String_String, align 8
   %valueRef_1 = alloca %struct.String_String, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_1, ptr align 8 %s, i64 16, i1 false)
   call void @Option_Option_Some_String_String(ptr %valueRef_1, ptr %call_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %o_3, ptr align 8 %call_2, i64 24, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_4, ptr align 8 %o_3, i64 24, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %valueRef_4, i64 24, i1 false)
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
   %tmp_switch_var_block2_29 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_30 = load i32, ptr %tmp_switch_var_block2_29, align 4
   switch i32 %tmp_switch_var_block2_30, label %block3 [
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
   %tmp_switch_var_block2_31 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_32 = load i32, ptr %tmp_switch_var_block2_31, align 4
   switch i32 %tmp_switch_var_block2_32, label %block3 [
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

define private void @Option_Option_Some_String_String(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Option_Option_Some_String_String, align 8
   %tag = getelementptr inbounds %struct.Option_Option_Some_String_String, ptr %this, i32 0, i32 0
   store volatile i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Option_Option_Some_String_String, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_String_String, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field0, ptr align 8 %f0, i64 16, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 24, i1 false)
   ret void
}

declare void @String_String_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


