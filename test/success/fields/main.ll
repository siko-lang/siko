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
   %b0i24 = alloca %struct.siko_Tuple_, align 4
   %b0i23 = alloca %struct.siko_Tuple_, align 4
   %b0i22 = alloca %struct.Bool_Bool, align 4
   %b0i18 = alloca %struct.Main_Person, align 8
   %b0i15 = alloca %struct.Main_Person, align 8
   %person2_3 = alloca %struct.Main_Person, align 8
   %b0i13 = alloca %struct.Main_Person, align 8
   %b0i12 = alloca %struct.Main_Address, align 8
   %b0i11 = alloca %struct.String_String, align 8
   %person1_2 = alloca %struct.Main_Person, align 8
   %b0i9 = alloca %struct.Main_Person, align 8
   %b0i8 = alloca %struct.Main_Address, align 8
   %b0i7 = alloca %struct.String_String, align 8
   %addr2_1 = alloca %struct.Main_Address, align 8
   %b0i5 = alloca %struct.Main_Address, align 8
   %b0i4 = alloca %struct.String_String, align 8
   %addr1_0 = alloca %struct.Main_Address, align 8
   %b0i2 = alloca %struct.Main_Address, align 8
   %b0i1 = alloca %struct.String_String, align 8
   %tmp_b0i1_1 = getelementptr inbounds %struct.String_String, ptr %b0i1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_b0i1_1, align 8
   %tmp_b0i1_2 = getelementptr inbounds %struct.String_String, ptr %b0i1, i32 0, i32 1
   store i64 4, ptr %tmp_b0i1_2, align 8
   call void @Main_Address(ptr %b0i1, ptr %b0i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %addr1_0, ptr align 8 %b0i2, i64 16, i1 false)
   %tmp_b0i4_1 = getelementptr inbounds %struct.String_String, ptr %b0i4, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_b0i4_1, align 8
   %tmp_b0i4_2 = getelementptr inbounds %struct.String_String, ptr %b0i4, i32 0, i32 1
   store i64 4, ptr %tmp_b0i4_2, align 8
   call void @Main_Address(ptr %b0i4, ptr %b0i5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %addr2_1, ptr align 8 %b0i5, i64 16, i1 false)
   %tmp_b0i7_1 = getelementptr inbounds %struct.String_String, ptr %b0i7, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_b0i7_1, align 8
   %tmp_b0i7_2 = getelementptr inbounds %struct.String_String, ptr %b0i7, i32 0, i32 1
   store i64 4, ptr %tmp_b0i7_2, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b0i8, ptr align 8 %addr1_0, i64 16, i1 false)
   call void @Main_Person(ptr %b0i7, ptr %b0i8, ptr %b0i9)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %person1_2, ptr align 8 %b0i9, i64 32, i1 false)
   %tmp_b0i11_1 = getelementptr inbounds %struct.String_String, ptr %b0i11, i32 0, i32 0
   store ptr @.str_2, ptr %tmp_b0i11_1, align 8
   %tmp_b0i11_2 = getelementptr inbounds %struct.String_String, ptr %b0i11, i32 0, i32 1
   store i64 5, ptr %tmp_b0i11_2, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b0i12, ptr align 8 %addr2_1, i64 16, i1 false)
   call void @Main_Person(ptr %b0i11, ptr %b0i12, ptr %b0i13)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %person2_3, ptr align 8 %b0i13, i64 32, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b0i15, ptr align 8 %person2_3, i64 32, i1 false)
   %b0i16 = getelementptr inbounds %struct.Main_Person, ptr %b0i15, i32 0, i32 1
   %b0i17 = getelementptr inbounds %struct.Main_Address, ptr %b0i16, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b0i18, ptr align 8 %person1_2, i64 32, i1 false)
   %b0i19 = getelementptr inbounds %struct.Main_Person, ptr %b0i18, i32 0, i32 1
   %b0i20 = getelementptr inbounds %struct.Main_Address, ptr %b0i19, i32 0, i32 0
   call void @String_String_eq(ptr %b0i20, ptr %b0i17, ptr %b0i22)
   call void @Std_Basic_Util_assert(ptr %b0i22, ptr %b0i23)
   call void @siko_Tuple_(ptr %b0i24)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i24, i64 0, i1 false)
   ret void
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %b6i1 = alloca %struct.siko_Tuple_, align 4
   %b4i2 = alloca %struct.siko_Tuple_, align 4
   %b4i1 = alloca %struct.siko_Tuple_, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i1, ptr align 4 %v, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i1, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b0i1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %b4i1)
   call void @siko_Tuple_(ptr %b4i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b4i2, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @siko_Tuple_(ptr %b6i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b6i1, i64 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

declare void @String_String_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


