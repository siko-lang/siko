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
   %b8i3 = alloca %struct.siko_Tuple_, align 4
   %b8i1 = alloca %struct.Bool_Bool, align 4
   %b7i3 = alloca %struct.siko_Tuple_, align 4
   %b7i1 = alloca %struct.Bool_Bool, align 4
   %b6i3 = alloca %struct.siko_Tuple_, align 4
   %b6i1 = alloca %struct.Bool_Bool, align 4
   %b5i3 = alloca %struct.siko_Tuple_, align 4
   %b5i1 = alloca %struct.Bool_Bool, align 4
   %b4i4 = alloca %struct.Bool_Bool, align 4
   %b4i1 = alloca %struct.String_String, align 8
   %b3i4 = alloca %struct.Bool_Bool, align 4
   %b3i1 = alloca %struct.String_String, align 8
   %b2i4 = alloca %struct.Bool_Bool, align 4
   %b2i1 = alloca %struct.String_String, align 8
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.String_String, align 8
   %tmp_b0i1_1 = getelementptr inbounds %struct.String_String, ptr %b0i1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_b0i1_1, align 8
   %tmp_b0i1_2 = getelementptr inbounds %struct.String_String, ptr %b0i1, i32 0, i32 1
   store i64 3, ptr %tmp_b0i1_2, align 8
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i1, i64 0, i1 false)
   ret void
block2:
   %tmp_b2i1_1 = getelementptr inbounds %struct.String_String, ptr %b2i1, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_b2i1_1, align 8
   %tmp_b2i1_2 = getelementptr inbounds %struct.String_String, ptr %b2i1, i32 0, i32 1
   store i64 3, ptr %tmp_b2i1_2, align 8
   call void @String_String_eq(ptr %b0i1, ptr %b2i1, ptr %b2i4)
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b2i4, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block6
]

block3:
   %tmp_b3i1_1 = getelementptr inbounds %struct.String_String, ptr %b3i1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_b3i1_1, align 8
   %tmp_b3i1_2 = getelementptr inbounds %struct.String_String, ptr %b3i1, i32 0, i32 1
   store i64 3, ptr %tmp_b3i1_2, align 8
   call void @String_String_eq(ptr %b0i1, ptr %b3i1, ptr %b3i4)
   %tmp_switch_var_block3_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b3i4, i32 0, i32 0
   %tmp_switch_var_block3_2 = load i32, ptr %tmp_switch_var_block3_1, align 4
   switch i32 %tmp_switch_var_block3_2, label %block4 [
i32 1, label %block7
]

block4:
   %tmp_b4i1_1 = getelementptr inbounds %struct.String_String, ptr %b4i1, i32 0, i32 0
   store ptr @.str_2, ptr %tmp_b4i1_1, align 8
   %tmp_b4i1_2 = getelementptr inbounds %struct.String_String, ptr %b4i1, i32 0, i32 1
   store i64 4, ptr %tmp_b4i1_2, align 8
   call void @String_String_eq(ptr %b0i1, ptr %b4i1, ptr %b4i4)
   %tmp_switch_var_block4_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b4i4, i32 0, i32 0
   %tmp_switch_var_block4_2 = load i32, ptr %tmp_switch_var_block4_1, align 4
   switch i32 %tmp_switch_var_block4_2, label %block5 [
i32 1, label %block8
]

block5:
   call void @Bool_Bool_False(ptr %b5i1)
   call void @Std_Basic_Util_assert(ptr %b5i1, ptr %b5i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b5i3, i64 0, i1 false)
   br label %block1
block6:
   call void @Bool_Bool_False(ptr %b6i1)
   call void @Std_Basic_Util_assert(ptr %b6i1, ptr %b6i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b6i3, i64 0, i1 false)
   br label %block1
block7:
   call void @Bool_Bool_True(ptr %b7i1)
   call void @Std_Basic_Util_assert(ptr %b7i1, ptr %b7i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b7i3, i64 0, i1 false)
   br label %block1
block8:
   call void @Bool_Bool_False(ptr %b8i1)
   call void @Std_Basic_Util_assert(ptr %b8i1, ptr %b8i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b8i3, i64 0, i1 false)
   br label %block1
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


