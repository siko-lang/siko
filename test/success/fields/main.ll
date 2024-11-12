@.str_1 = private unnamed_addr constant [4 x i8] c"John", align 1
@.str_2 = private unnamed_addr constant [5 x i8] c"Katie", align 1
@.str_0 = private unnamed_addr constant [4 x i8] c"Main", align 1
%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Main_Address = type { %struct.String_String }

%struct.Main_Person = type { %struct.String_String, %struct.Main_Address }

%struct.String_String = type { ptr, i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_Address(ptr noundef %street, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Address, align 8
   %field0 = getelementptr inbounds %struct.Main_Address, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field0, ptr align 8 %street, i64 16, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 16, i1 false)
   ret void
}

define private void @Main_Person(ptr noundef %name, ptr noundef %address, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Person, align 8
   %field0 = getelementptr inbounds %struct.Main_Person, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field0, ptr align 8 %name, i64 16, i1 false)
   %field1 = getelementptr inbounds %struct.Main_Person, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field1, ptr align 8 %address, i64 16, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 32, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %unit_24 = alloca %struct.siko_Tuple_, align 4
   %call_23 = alloca %struct.siko_Tuple_, align 4
   %call_22 = alloca %struct.Bool_Bool, align 4
   %implicitRef1 = alloca ptr, align 8
   %valueRef_18 = alloca %struct.Main_Person, align 8
   %implicitRef0 = alloca ptr, align 8
   %valueRef_15 = alloca %struct.Main_Person, align 8
   %person2_14 = alloca %struct.Main_Person, align 8
   %call_13 = alloca %struct.Main_Person, align 8
   %valueRef_12 = alloca %struct.Main_Address, align 8
   %literal_11 = alloca %struct.String_String, align 8
   %person1_10 = alloca %struct.Main_Person, align 8
   %call_9 = alloca %struct.Main_Person, align 8
   %valueRef_8 = alloca %struct.Main_Address, align 8
   %literal_7 = alloca %struct.String_String, align 8
   %addr2_6 = alloca %struct.Main_Address, align 8
   %call_5 = alloca %struct.Main_Address, align 8
   %literal_4 = alloca %struct.String_String, align 8
   %addr1_3 = alloca %struct.Main_Address, align 8
   %call_2 = alloca %struct.Main_Address, align 8
   %literal_1 = alloca %struct.String_String, align 8
   %tmp_literal_1_1 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_literal_1_1, align 8
   %tmp_literal_1_2 = getelementptr inbounds %struct.String_String, ptr %literal_1, i32 0, i32 1
   store i64 4, ptr %tmp_literal_1_2, align 8
   call void @Main_Address(ptr %literal_1, ptr %call_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %addr1_3, ptr align 8 %call_2, i64 16, i1 false)
   %tmp_literal_4_3 = getelementptr inbounds %struct.String_String, ptr %literal_4, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_literal_4_3, align 8
   %tmp_literal_4_4 = getelementptr inbounds %struct.String_String, ptr %literal_4, i32 0, i32 1
   store i64 4, ptr %tmp_literal_4_4, align 8
   call void @Main_Address(ptr %literal_4, ptr %call_5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %addr2_6, ptr align 8 %call_5, i64 16, i1 false)
   %tmp_literal_7_5 = getelementptr inbounds %struct.String_String, ptr %literal_7, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_literal_7_5, align 8
   %tmp_literal_7_6 = getelementptr inbounds %struct.String_String, ptr %literal_7, i32 0, i32 1
   store i64 4, ptr %tmp_literal_7_6, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_8, ptr align 8 %addr1_3, i64 16, i1 false)
   call void @Main_Person(ptr %literal_7, ptr %valueRef_8, ptr %call_9)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %person1_10, ptr align 8 %call_9, i64 32, i1 false)
   %tmp_literal_11_7 = getelementptr inbounds %struct.String_String, ptr %literal_11, i32 0, i32 0
   store ptr @.str_2, ptr %tmp_literal_11_7, align 8
   %tmp_literal_11_8 = getelementptr inbounds %struct.String_String, ptr %literal_11, i32 0, i32 1
   store i64 5, ptr %tmp_literal_11_8, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_12, ptr align 8 %addr2_6, i64 16, i1 false)
   call void @Main_Person(ptr %literal_11, ptr %valueRef_12, ptr %call_13)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %person2_14, ptr align 8 %call_13, i64 32, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_15, ptr align 8 %person2_14, i64 32, i1 false)
   %fieldRef_16 = getelementptr inbounds %struct.Main_Person, ptr %valueRef_15, i32 0, i32 1
   %fieldRef_17 = getelementptr inbounds %struct.Main_Address, ptr %fieldRef_16, i32 0, i32 0
   store ptr %fieldRef_17, ptr %implicitRef0, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_18, ptr align 8 %person1_10, i64 32, i1 false)
   %fieldRef_19 = getelementptr inbounds %struct.Main_Person, ptr %valueRef_18, i32 0, i32 1
   %fieldRef_20 = getelementptr inbounds %struct.Main_Address, ptr %fieldRef_19, i32 0, i32 0
   store ptr %fieldRef_20, ptr %implicitRef1, align 8
   %tmp_implicitRef1_9 = load ptr, ptr %implicitRef1, align 8
   %tmp_implicitRef0_10 = load ptr, ptr %implicitRef0, align 8
   call void @String_String_eq(ptr %tmp_implicitRef1_9, ptr %tmp_implicitRef0_10, ptr %call_22)
   call void @Std_Basic_Util_assert(ptr %call_22, ptr %call_23)
   call void @siko_Tuple_(ptr %unit_24)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_24, i64 0, i1 false)
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
   %tmp_switch_var_block2_11 = getelementptr inbounds %struct.Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   %tmp_switch_var_block2_12 = load i32, ptr %tmp_switch_var_block2_11, align 4
   switch i32 %tmp_switch_var_block2_12, label %block3 [
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

declare void @String_String_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


